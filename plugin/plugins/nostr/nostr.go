package nostr

import (
	"context"
	"encoding/hex"
	"fmt"
	"sync"
	"time"

	logging "github.com/ipfs/go-log/v2"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/plugin"
	"github.com/libp2p/go-libp2p/core/crypto"
	"github.com/libp2p/go-libp2p/core/event"
	"github.com/libp2p/go-libp2p/core/network"
	gnostr "github.com/nbd-wtf/go-nostr"
	"github.com/nbd-wtf/go-nostr/keyer"
)

var log = logging.Logger("plugin/nostr")

// Plugins is the exported list of plugins that will be loaded.
var Plugins = []plugin.Plugin{
	&nostrPlugin{},
}

type nostrPlugin struct {
	enabled bool
	config  Config

	signer keyer.KeySigner
	pool   *gnostr.SimplePool

	events   chan nostrEvent
	cancel   context.CancelFunc
	wg       sync.WaitGroup
}

var _ plugin.PluginDaemonInternal = (*nostrPlugin)(nil)

func (*nostrPlugin) Name() string {
	return "nostr"
}

func (*nostrPlugin) Version() string {
	return "0.1.0"
}

func (np *nostrPlugin) Init(env *plugin.Environment) error {
	cfg, err := parseConfig(env.Config)
	if err != nil {
		return fmt.Errorf("nostr plugin config error: %w", err)
	}
	np.config = cfg
	np.enabled = cfg.Enabled
	if !np.enabled {
		return nil
	}

	np.events = make(chan nostrEvent, 1024)
	return nil
}

func (np *nostrPlugin) Start(node *core.IpfsNode) error {
	if !np.enabled {
		return nil
	}

	ctx, cancel := context.WithCancel(node.Context())
	np.cancel = cancel

	// derive or load signing key
	if err := np.initSigner(node); err != nil {
		return fmt.Errorf("nostr plugin failed to initialize signer: %w", err)
	}

	// initialize relay pool
	np.pool = gnostr.NewSimplePool(ctx)

	// subscribe to libp2p events
	if err := np.subscribeEvents(node); err != nil {
		return err
	}

	// start publisher goroutine
	np.wg.Add(1)
	go np.publisher(ctx)

	// publish initial repository announcement
	if np.config.PublishRepoEvents {
		np.emit(np.buildRepoAnnouncement(node))
	}

	return nil
}

func (np *nostrPlugin) Close() error {
	if np.cancel != nil {
		np.cancel()
	}
	np.wg.Wait()
	if np.pool != nil {
		np.pool.Close("nostr plugin shutdown")
	}
	return nil
}

func (np *nostrPlugin) initSigner(node *core.IpfsNode) error {
	if np.config.PrivateKey != "" {
		ks, err := keyer.NewPlainKeySigner(np.config.PrivateKey)
		if err != nil {
			return err
		}
		np.signer = ks
		return nil
	}

	// try to derive from libp2p key if it is secp256k1
	if node.PrivateKey != nil {
		if sk, ok := node.PrivateKey.(*crypto.Secp256k1PrivateKey); ok {
			raw, err := sk.Raw()
			if err != nil {
				return fmt.Errorf("failed to extract raw secp256k1 key: %w", err)
			}
			ks, err := keyer.NewPlainKeySigner(hex.EncodeToString(raw))
			if err != nil {
				return err
			}
			np.signer = ks
			log.Info("derived nostr key from libp2p secp256k1 key")
			return nil
		}
	}

	return fmt.Errorf("no nostr private key configured and libp2p key is not secp256k1; please set Plugins.nostr.Config.PrivateKey")
}

func (np *nostrPlugin) subscribeEvents(node *core.IpfsNode) error {
	if !np.config.PublishPeerEvents {
		return nil
	}

	sub, err := node.PeerHost.EventBus().Subscribe(new(event.EvtPeerConnectednessChanged))
	if err != nil {
		return fmt.Errorf("failed to subscribe to peer connectedness events: %w", err)
	}

	np.wg.Add(1)
	go func() {
		defer sub.Close()
		defer np.wg.Done()
		for {
			select {
			case <-node.Context().Done():
				return
			case e, ok := <-sub.Out():
				if !ok {
					return
				}
				ev := e.(event.EvtPeerConnectednessChanged)
				np.emit(np.buildPeerEvent(ev))
			}
		}
	}()

	idSub, err := node.PeerHost.EventBus().Subscribe(new(event.EvtPeerIdentificationCompleted))
	if err != nil {
		return fmt.Errorf("failed to subscribe to peer identification events: %w", err)
	}

	np.wg.Add(1)
	go func() {
		defer idSub.Close()
		defer np.wg.Done()
		for {
			select {
			case <-node.Context().Done():
				return
			case e, ok := <-idSub.Out():
				if !ok {
					return
				}
				ev := e.(event.EvtPeerIdentificationCompleted)
				np.emit(np.buildIdentifyEvent(node, ev))
			}
		}
	}()

	return nil
}

func (np *nostrPlugin) emit(evt *gnostr.Event) {
	if evt == nil {
		return
	}
	select {
	case np.events <- nostrEvent{event: evt, createdAt: time.Now()}:
	default:
		log.Warn("nostr event queue full, dropping event")
	}
}

func (np *nostrPlugin) publisher(ctx context.Context) {
	defer np.wg.Done()
	for {
		select {
		case <-ctx.Done():
			return
		case e := <-np.events:
			if e.event == nil {
				continue
			}
			// ensure event is signed
			if e.event.Sig == "" {
				if err := np.signer.SignEvent(ctx, e.event); err != nil {
					log.Errorw("failed to sign nostr event", "error", err)
					continue
				}
			}
			np.publishToRelays(ctx, *e.event)
		}
	}
}

func (np *nostrPlugin) publishToRelays(ctx context.Context, evt gnostr.Event) {
	if len(np.config.Relays) == 0 {
		return
	}
	pubCtx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()

	for res := range np.pool.PublishMany(pubCtx, np.config.Relays, evt) {
		if res.Error != nil {
			log.Warnw("failed to publish to relay", "relay", res.RelayURL, "error", res.Error)
		} else {
			log.Debugw("published to relay", "relay", res.RelayURL, "event_id", evt.ID)
		}
	}
}

type nostrEvent struct {
	event     *gnostr.Event
	createdAt time.Time
}

func (np *nostrPlugin) buildRepoAnnouncement(node *core.IpfsNode) *gnostr.Event {
	peerID := node.Identity.String()
	addrs := node.PeerHost.Addrs()
	cloneURLs := make([]string, 0, len(addrs))
	for _, a := range addrs {
		cloneURLs = append(cloneURLs, fmt.Sprintf("/p2p/%s%s", peerID, a.String()))
	}

	repo := gnostr.Tags{
		gnostr.Tag{"d", peerID},
		gnostr.Tag{"name", fmt.Sprintf("kubo-%s", peerID[:16])},
		gnostr.Tag{"description", fmt.Sprintf("Kubo IPFS node %s", node.Identity.ShortString())},
	}
	if len(cloneURLs) > 0 {
		tag := make(gnostr.Tag, 1, 1+len(cloneURLs))
		tag[0] = "clone"
		tag = append(tag, cloneURLs...)
		repo = append(repo, tag)
	}

	return &gnostr.Event{
		Kind:      gnostr.KindRepositoryAnnouncement,
		Tags:      repo,
		Content:   "",
		CreatedAt: gnostr.Now(),
	}
}

func (np *nostrPlugin) buildPeerEvent(ev event.EvtPeerConnectednessChanged) *gnostr.Event {
	var action string
	switch ev.Connectedness {
	case network.Connected:
		action = "connected"
	case network.NotConnected:
		action = "disconnected"
	default:
		action = fmt.Sprintf("connectedness=%d", ev.Connectedness)
	}

	return &gnostr.Event{
		Kind:    gnostr.KindTextNote,
		Content: fmt.Sprintf("peer %s %s", ev.Peer.ShortString(), action),
		Tags: gnostr.Tags{
			gnostr.Tag{"p", ev.Peer.String()},
			gnostr.Tag{"kubo", "peer-event"},
		},
		CreatedAt: gnostr.Now(),
	}
}

func (np *nostrPlugin) buildIdentifyEvent(node *core.IpfsNode, ev event.EvtPeerIdentificationCompleted) *gnostr.Event {
	agent, _ := node.Peerstore.Get(ev.Peer, "AgentVersion")
	agentStr, _ := agent.(string)
	if agentStr == "" {
		agentStr = "unknown"
	}

	return &gnostr.Event{
		Kind:    gnostr.KindTextNote,
		Content: fmt.Sprintf("identified peer %s running %s", ev.Peer.ShortString(), agentStr),
		Tags: gnostr.Tags{
			gnostr.Tag{"p", ev.Peer.String()},
			gnostr.Tag{"kubo", "identify"},
		},
		CreatedAt: gnostr.Now(),
	}
}
