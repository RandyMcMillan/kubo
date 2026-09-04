// Package main exports C symbols for use by Rust via CGO.
package main

/*
#include <stdlib.h>
*/
import "C"

import (
	"context"
	"fmt"
	"io"
	"strings"
	"sync"
	"unsafe"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/go-cid"
	ipfs "github.com/ipfs/kubo"
	"github.com/ipfs/kubo/config"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/core/coreapi"
	coreiface "github.com/ipfs/kubo/core/coreiface"
	"github.com/ipfs/kubo/core/coreiface/options"
	"github.com/ipfs/kubo/core/node/libp2p"
	"github.com/ipfs/kubo/plugin/loader"
	"github.com/ipfs/kubo/repo/fsrepo"
)

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

var (
	lastErrorMu sync.Mutex
	lastError   string
)

func setError(err error) {
	lastErrorMu.Lock()
	defer lastErrorMu.Unlock()
	if err != nil {
		lastError = err.Error()
	} else {
		lastError = ""
	}
}

//export kubo_last_error
func kubo_last_error() *C.char {
	lastErrorMu.Lock()
	defer lastErrorMu.Unlock()
	if lastError == "" {
		return nil
	}
	return C.CString(lastError)
}

//export kubo_free_string
func kubo_free_string(s *C.char) {
	C.free(unsafe.Pointer(s))
}

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
	ctx    context.Context
	cancel context.CancelFunc
	node   *core.IpfsNode
	api    coreiface.CoreAPI
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

	h.cancel()
	if err := h.node.Close(); err != nil {
		setError(fmt.Errorf("close node: %w", err))
		return -1
	}

	setError(nil)
	return 0
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

	// Accept both raw CIDs and /ipfs/... paths.
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

//export kubo_free_buffer
func kubo_free_buffer(buf *C.uint8_t) {
	C.free(unsafe.Pointer(buf))
}

// ---------------------------------------------------------------------------
// Main (required for c-archive buildmode but never called)
// ---------------------------------------------------------------------------

func main() {}
