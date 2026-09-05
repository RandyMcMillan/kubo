package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"bytes"
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"
	"sync"
	"unsafe"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/go-cid"
	ipfs "github.com/ipfs/kubo"
	"github.com/ipfs/kubo/commands"
	"github.com/ipfs/kubo/config"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/core/coreapi"
	"github.com/ipfs/kubo/core/corehttp"
	coreiface "github.com/ipfs/kubo/core/coreiface"
	"github.com/ipfs/kubo/core/coreiface/options"
	"github.com/ipfs/kubo/core/node/libp2p"
	"github.com/ipfs/kubo/plugin/loader"
	"github.com/ipfs/kubo/repo/fsrepo"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	manet "github.com/multiformats/go-multiaddr/net"
)

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

//export kubo_version
func kubo_version() *C.char {
	return C.CString(ipfs.CurrentVersionNumber)
}

// ---------------------------------------------------------------------------
// Plugin loading (once)
// ---------------------------------------------------------------------------

var (
	pluginsOnce sync.Once
	pluginsErr  error
)

func ensurePlugins() error {
	pluginsOnce.Do(func() {
		plugins, err := loader.NewPluginLoader("")
		if err != nil {
			pluginsErr = fmt.Errorf("error loading plugins: %w", err)
			return
		}
		if err := plugins.Initialize(); err != nil {
			pluginsErr = fmt.Errorf("error initializing plugins: %w", err)
			return
		}
		if err := plugins.Inject(); err != nil {
			pluginsErr = fmt.Errorf("error injecting plugins: %w", err)
			return
		}
	})
	return pluginsErr
}

// ---------------------------------------------------------------------------
// Repo initialization
// ---------------------------------------------------------------------------

//export kubo_init_repo
func kubo_init_repo(repoPath *C.char) int64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return -1
	}

	path := C.GoString(repoPath)

	identity, err := config.CreateIdentity(io.Discard, []options.KeyGenerateOption{
		options.Key.Type(options.Ed25519Key),
	})
	if err != nil {
		setError(fmt.Errorf("create identity: %w", err))
		return -1
	}

	cfg, err := config.InitWithIdentity(identity)
	if err != nil {
		setError(fmt.Errorf("init config: %w", err))
		return -1
	}

	cfg.Addresses.Swarm = []string{
		"/ip4/127.0.0.1/tcp/0",
	}
	cfg.Swarm.Transports.Network.QUIC = config.False
	cfg.Swarm.Transports.Network.Relay = config.False
	cfg.Swarm.Transports.Network.WebTransport = config.False
	cfg.Swarm.Transports.Network.WebRTCDirect = config.False
	cfg.Swarm.Transports.Network.Websocket = config.False
	cfg.AutoTLS.Enabled = config.False
	cfg.Routing.Type = config.NewOptionalString("none")
	cfg.Bootstrap = []string{}
	cfg.Discovery.MDNS.Enabled = false

	if err := fsrepo.Init(path, cfg); err != nil {
		setError(fmt.Errorf("init repo: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Node registry
// ---------------------------------------------------------------------------

type nodeHandle struct {
	ctx            context.Context
	cancel         context.CancelFunc
	node           *core.IpfsNode
	api            coreiface.CoreAPI
	apiListener    manet.Listener
	apiNetListener net.Listener
	apiServer      *http.Server
}

var (
	nodesMu    sync.RWMutex
	nodes      = make(map[uint64]*nodeHandle)
	nextHandle uint64 = 1
)

//export kubo_node_start
func kubo_node_start(repoPath *C.char, online C.uint8_t) uint64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return 0
	}

	path := C.GoString(repoPath)

	repo, err := fsrepo.Open(path)
	if err != nil {
		setError(fmt.Errorf("open repo: %w", err))
		return 0
	}

	ctx, cancel := context.WithCancel(context.Background())

	cfg := &core.BuildCfg{
		Online: online != 0,
		Repo:   repo,
	}
	if !cfg.Online {
		cfg.Routing = libp2p.NilRouterOption
	}

	n, err := core.NewNode(ctx, cfg)
	if err != nil {
		cancel()
		setError(fmt.Errorf("new node: %w", err))
		return 0
	}

	api, err := coreapi.NewCoreAPI(n)
	if err != nil {
		n.Close()
		cancel()
		setError(fmt.Errorf("core api: %w", err))
		return 0
	}

	h := &nodeHandle{
		ctx:    ctx,
		cancel: cancel,
		node:   n,
		api:    api,
	}

	nodesMu.Lock()
	handle := nextHandle
	nextHandle++
	nodes[handle] = h
	nodesMu.Unlock()

	setError(nil)
	return handle
}

//export kubo_node_stop
func kubo_node_stop(handle uint64) int64 {
	nodesMu.Lock()
	h, ok := nodes[handle]
	if ok {
		delete(nodes, handle)
	}
	nodesMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	if h.apiListener != nil {
		h.apiListener.Close()
	}
	h.cancel()
	if err := h.node.Close(); err != nil {
		setError(fmt.Errorf("close node: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_node_start_api
func kubo_node_start_api(handle uint64, multiaddr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrStr := C.GoString(multiaddr)
	addr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return nil
	}

	listener, err := manet.Listen(addr)
	if err != nil {
		setError(fmt.Errorf("listen: %w", err))
		return nil
	}

	cctx := commands.Context{
		ConstructNode: func() (*core.IpfsNode, error) {
			return h.node, nil
		},
		ReqLog: &commands.ReqLog{},
	}

	opts := []corehttp.ServeOption{
		corehttp.CheckVersionOption(),
		corehttp.CommandsOption(cctx),
		corehttp.WebUIOption,
		corehttp.VersionOption(),
	}

	h.apiListener = listener
	h.apiNetListener = manet.NetListener(listener)
	h.apiServer = &http.Server{}

	go func() {
		handler, err := corehttp.MakeHandler(h.node, h.apiNetListener, opts...)
		if err != nil {
			setError(fmt.Errorf("make handler: %w", err))
			return
		}
		h.apiServer.Handler = handler
		h.apiServer.Serve(h.apiNetListener)
	}()

	setError(nil)
	return C.CString(listener.Multiaddr().String())
}

//export kubo_node_api_addrs
func kubo_node_api_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	if h.apiListener == nil {
		setError(fmt.Errorf("api not started"))
		return nil
	}

	setError(nil)
	return C.CString(h.apiListener.Multiaddr().String())
}

//export kubo_node_peer_id
func kubo_node_peer_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	return C.CString(h.node.Identity.String())
}

//export kubo_node_listening_addrs
func kubo_node_listening_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrs, err := h.api.Swarm().LocalAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("local addrs: %w", err))
		return nil
	}

	var parts []string
	for _, a := range addrs {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_connect
func kubo_node_connect(handle uint64, addr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	addrStr := C.GoString(addr)
	maddr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return -1
	}

	info, err := peer.AddrInfoFromP2pAddr(maddr)
	if err != nil {
		setError(fmt.Errorf("extract peer info: %w", err))
		return -1
	}

	if err := h.api.Swarm().Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_swarm_peers
func kubo_swarm_peers(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	peers, err := h.api.Swarm().Peers(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm peers: %w", err))
		return nil
	}

	var parts []string
	for _, p := range peers {
		addr := ""
		if p.Address() != nil {
			addr = p.Address().String()
		}
		parts = append(parts, p.ID().String()+"\t"+addr)
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_id
func kubo_node_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	id := h.node.Identity.String()
	pk, err := h.node.Identity.ExtractPublicKey()
	if err != nil {
		setError(fmt.Errorf("extract public key: %w", err))
		return nil
	}
	pkBytes, err := pk.Raw()
	if err != nil {
		setError(fmt.Errorf("raw public key: %w", err))
		return nil
	}

	info := fmt.Sprintf(
		`{"id":"%s","public_key":"%s"}`,
		id,
		base64.StdEncoding.EncodeToString(pkBytes),
	)
	setError(nil)
	return C.CString(info)
}

// ---------------------------------------------------------------------------
// UnixFS helpers
// ---------------------------------------------------------------------------

//export kubo_unixfs_add_bytes
func kubo_unixfs_add_bytes(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	file := files.NewBytesFile(goData)

	p, err := h.api.Unixfs().Add(h.ctx, file)
	if err != nil {
		setError(fmt.Errorf("unixfs add: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(p.RootCid().String())
}

//export kubo_unixfs_cat
func kubo_unixfs_cat(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	node, err := h.api.Unixfs().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("unixfs get: %w", err))
		return -1
	}
	defer node.Close()

	file, ok := node.(files.File)
	if !ok {
		setError(fmt.Errorf("node is not a file"))
		return -1
	}

	buf, err := io.ReadAll(file)
	if err != nil {
		setError(fmt.Errorf("read file: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Block API
// ---------------------------------------------------------------------------

//export kubo_block_put
func kubo_block_put(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	stat, err := h.api.Block().Put(h.ctx, bytes.NewReader(goData))
	if err != nil {
		setError(fmt.Errorf("block put: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(stat.Path().RootCid().String())
}

//export kubo_block_get
func kubo_block_get(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	reader, err := h.api.Block().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block get: %w", err))
		return -1
	}

	buf, err := io.ReadAll(reader)
	if err != nil {
		setError(fmt.Errorf("read block: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

//export kubo_block_stat
func kubo_block_stat(handle uint64, cidStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	stat, err := h.api.Block().Stat(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block stat: %w", err))
		return -1
	}

	setError(nil)
	return int64(stat.Size())
}
