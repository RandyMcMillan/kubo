package nostrrelay

import (
	"context"
	"fmt"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/go-cid"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/core/coreapi"
	"github.com/ipfs/kubo/plugin"
)

// Plugins is the exported list of plugins that will be loaded.
var Plugins = []plugin.Plugin{
	&relayPlugin{},
}

type relayPlugin struct {
	enabled bool
	addr    string
	relay   *Relay
}

var _ plugin.PluginDaemonInternal = (*relayPlugin)(nil)

func (*relayPlugin) Name() string {
	return "nostr-relay"
}

func (*relayPlugin) Version() string {
	return "0.1.0"
}

func (p *relayPlugin) Init(env *plugin.Environment) error {
	p.enabled = false
	if env.Config == nil {
		return nil
	}
	cfg, ok := env.Config.(map[string]any)
	if !ok {
		return nil
	}
	if v, ok := cfg["Enabled"].(bool); ok {
		p.enabled = v
	}
	p.addr = "127.0.0.1:7777"
	if v, ok := cfg["Addr"].(string); ok && v != "" {
		p.addr = v
	}
	return nil
}

func (p *relayPlugin) Start(node *core.IpfsNode) error {
	if !p.enabled {
		return nil
	}

	ipfs := &ipfsNodeAdder{node: node}
	p.relay = NewRelay(p.addr, ipfs)

	go func() {
		if err := p.relay.Start(node.Context()); err != nil {
			// log error
		}
	}()

	return nil
}

func (p *relayPlugin) Close() error {
	if p.relay != nil && p.relay.httpSrv != nil {
		return p.relay.httpSrv.Close()
	}
	return nil
}

// ipfsNodeAdder implements IPFSAdder using the in-process IpfsNode.
type ipfsNodeAdder struct {
	node *core.IpfsNode
}

func (a *ipfsNodeAdder) Add(ctx context.Context, data []byte) (cid.Cid, error) {
	file := files.NewBytesFile(data)
	defer file.Close()

	api, err := coreapi.NewCoreAPI(a.node)
	if err != nil {
		return cid.Undef, err
	}

	p, err := api.Unixfs().Add(ctx, file)
	if err != nil {
		return cid.Undef, err
	}

	resolved, _, err := api.ResolvePath(ctx, p)
	if err != nil {
		return cid.Undef, err
	}
	return resolved.RootCid(), nil
}

func (a *ipfsNodeAdder) Cat(ctx context.Context, c cid.Cid) ([]byte, error) {
	api, err := coreapi.NewCoreAPI(a.node)
	if err != nil {
		return nil, err
	}

	node, err := api.Unixfs().Get(ctx, path.FromCid(c))
	if err != nil {
		return nil, err
	}
	defer node.Close()

	f, ok := node.(files.File)
	if !ok {
		return nil, fmt.Errorf("not a file")
	}

	var data []byte
	buf := make([]byte, 1024)
	for {
		n, err := f.Read(buf)
		if n > 0 {
			data = append(data, buf[:n]...)
		}
		if err != nil {
			break
		}
	}
	return data, nil
}
