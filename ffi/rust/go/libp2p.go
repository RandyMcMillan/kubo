package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"context"
	"fmt"
	"io"
	"strings"
	"sync"
	"time"
	"unsafe"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/libp2p/go-libp2p/p2p/protocol/ping"
	ma "github.com/multiformats/go-multiaddr"
)

var (
	libp2pHostsMu    sync.RWMutex
	libp2pHosts             = make(map[uint64]*libp2pHostHandle)
	libp2pNextHandle uint64 = 1
)

type libp2pHostHandle struct {
	ctx    context.Context
	cancel context.CancelFunc
	host   host.Host

	streamsMu sync.Mutex
	streams   map[uint64]network.Stream
	incoming  chan network.Stream
}

//export kubo_libp2p_host_new
func kubo_libp2p_host_new() uint64 {
	ctx, cancel := context.WithCancel(context.Background())

	h, err := libp2p.New(libp2p.ListenAddrStrings("/ip4/127.0.0.1/tcp/0"))
	if err != nil {
		cancel()
		setError(fmt.Errorf("libp2p new: %w", err))
		return 0
	}

	handle := &libp2pHostHandle{
		ctx:      ctx,
		cancel:   cancel,
		host:     h,
		streams:  make(map[uint64]network.Stream),
		incoming: make(chan network.Stream, 16),
	}

	libp2pHostsMu.Lock()
	hid := libp2pNextHandle
	libp2pNextHandle++
	libp2pHosts[hid] = handle
	libp2pHostsMu.Unlock()

	setError(nil)
	return hid
}

//export kubo_libp2p_host_close
func kubo_libp2p_host_close(handle uint64) int64 {
	libp2pHostsMu.Lock()
	h, ok := libp2pHosts[handle]
	if ok {
		delete(libp2pHosts, handle)
	}
	libp2pHostsMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	h.cancel()
	if err := h.host.Close(); err != nil {
		setError(fmt.Errorf("close host: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_peer_id
func kubo_libp2p_host_peer_id(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	return C.CString(h.host.ID().String())
}

//export kubo_libp2p_host_listening_addrs
func kubo_libp2p_host_listening_addrs(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	var parts []string
	for _, a := range h.host.Addrs() {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_libp2p_host_connect
func kubo_libp2p_host_connect(handle uint64, addr *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
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

	if err := h.host.Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_ping
func kubo_libp2p_host_ping(handle uint64, peer_id *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	pid, err := peer.Decode(C.GoString(peer_id))
	if err != nil {
		setError(fmt.Errorf("decode peer id: %w", err))
		return -1
	}

	ctx, cancel := context.WithTimeout(h.ctx, 10*time.Second)
	defer cancel()

	result := <-ping.Ping(ctx, h.host, pid)
	if result.Error != nil {
		setError(fmt.Errorf("ping: %w", result.Error))
		return -1
	}

	setError(nil)
	return int64(result.RTT.Milliseconds())
}

//export kubo_libp2p_host_protocols
func kubo_libp2p_host_protocols(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	protos := h.host.Mux().Protocols()
	var names []string
	for _, p := range protos {
		names = append(names, string(p))
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

// ---------------------------------------------------------------------------
// Raw stream plumbing: register protocol handlers and move arbitrary bytes
// over libp2p streams — any protocol, not just the built-in ones.
// ---------------------------------------------------------------------------

var (
	libp2pStreamsMu sync.Mutex
	nextStreamID    uint64 = 1 // 0 is the FFI error sentinel; global so handles never collide across hosts
)

func (h *libp2pHostHandle) registerStream(s network.Stream) uint64 {
	libp2pStreamsMu.Lock()
	hid := nextStreamID
	nextStreamID++
	libp2pStreamsMu.Unlock()
	h.streamsMu.Lock()
	h.streams[hid] = s
	h.streamsMu.Unlock()
	return hid
}

//export kubo_libp2p_host_set_stream_handler
func kubo_libp2p_host_set_stream_handler(handle uint64, proto *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	pid := protocol.ID(C.GoString(proto))
	h.host.SetStreamHandler(pid, func(s network.Stream) {
		select {
		case h.incoming <- s:
		case <-h.ctx.Done():
			s.Reset()
		}
	})

	setError(nil)
	return 0
}

//export kubo_libp2p_stream_accept
func kubo_libp2p_stream_accept(handle uint64, timeoutMs C.int64_t) uint64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return 0
	}

	var timer <-chan time.Time
	if timeoutMs > 0 {
		t := time.NewTimer(time.Duration(timeoutMs) * time.Millisecond)
		defer t.Stop()
		timer = t.C
	}

	select {
	case s := <-h.incoming:
		setError(nil)
		return h.registerStream(s)
	case <-timer:
		setError(fmt.Errorf("accept timed out"))
		return 0
	case <-h.ctx.Done():
		setError(fmt.Errorf("host closed"))
		return 0
	}
}

//export kubo_libp2p_host_new_stream
func kubo_libp2p_host_new_stream(handle uint64, peerID *C.char, proto *C.char) uint64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return 0
	}

	pid, err := peer.Decode(C.GoString(peerID))
	if err != nil {
		setError(fmt.Errorf("decode peer id: %w", err))
		return 0
	}

	ctx, cancel := context.WithTimeout(h.ctx, 30*time.Second)
	defer cancel()
	s, err := h.host.NewStream(ctx, pid, protocol.ID(C.GoString(proto)))
	if err != nil {
		setError(fmt.Errorf("new stream: %w", err))
		return 0
	}

	setError(nil)
	return h.registerStream(s)
}

//export kubo_libp2p_stream_read
func kubo_libp2p_stream_read(streamHandle uint64, out **C.uint8_t, outLen *C.size_t) int64 {
	stream, found := lookupStream(streamHandle)
	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return -1
	}

	buf := make([]byte, 1<<20) // 1 MiB per read; loop for more
	n, err := stream.Read(buf)
	if err != nil {
		if err == io.EOF && n == 0 {
			*out = nil
			*outLen = 0
			setError(nil)
			return 1 // EOF
		}
		if n > 0 {
			*out = (*C.uint8_t)(C.CBytes(buf[:n]))
			*outLen = C.size_t(n)
			setError(nil)
			return 0
		}
		setError(fmt.Errorf("stream read: %w", err))
		return -1
	}

	*out = (*C.uint8_t)(C.CBytes(buf[:n]))
	*outLen = C.size_t(n)
	setError(nil)
	return 0
}

//export kubo_libp2p_stream_write
func kubo_libp2p_stream_write(streamHandle uint64, data *C.uint8_t, length C.size_t) int64 {
	var stream network.Stream
	found := false
	libp2pHostsMu.RLock()
	for _, hh := range libp2pHosts {
		hh.streamsMu.Lock()
		if s, ok := hh.streams[streamHandle]; ok {
			stream = s
			found = true
		}
		hh.streamsMu.Unlock()
		if found {
			break
		}
	}
	libp2pHostsMu.RUnlock()

	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return -1
	}

	n, err := stream.Write(C.GoBytes(unsafe.Pointer(data), C.int(length)))
	if err != nil {
		setError(fmt.Errorf("stream write: %w", err))
		return -1
	}

	setError(nil)
	return int64(n)
}

func lookupStream(streamHandle uint64) (network.Stream, bool) {
	libp2pHostsMu.RLock()
	defer libp2pHostsMu.RUnlock()
	for _, hh := range libp2pHosts {
		hh.streamsMu.Lock()
		s, ok := hh.streams[streamHandle]
		hh.streamsMu.Unlock()
		if ok {
			return s, true
		}
	}
	return nil, false
}

//export kubo_libp2p_stream_close
func kubo_libp2p_stream_close(streamHandle uint64) int64 {
	stream, found := lookupStream(streamHandle)
	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return -1
	}
	if err := stream.Close(); err != nil {
		setError(fmt.Errorf("stream close: %w", err))
		return -1
	}
	deregisterStream(streamHandle)
	setError(nil)
	return 0
}

//export kubo_libp2p_stream_reset
func kubo_libp2p_stream_reset(streamHandle uint64) int64 {
	stream, found := lookupStream(streamHandle)
	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return -1
	}
	if err := stream.Reset(); err != nil {
		setError(fmt.Errorf("stream reset: %w", err))
		return -1
	}
	deregisterStream(streamHandle)
	setError(nil)
	return 0
}

func deregisterStream(streamHandle uint64) {
	libp2pHostsMu.RLock()
	defer libp2pHostsMu.RUnlock()
	for _, hh := range libp2pHosts {
		hh.streamsMu.Lock()
		delete(hh.streams, streamHandle)
		hh.streamsMu.Unlock()
	}
}

//export kubo_libp2p_stream_protocol
func kubo_libp2p_stream_protocol(streamHandle uint64) *C.char {
	stream, found := lookupStream(streamHandle)
	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return nil
	}
	setError(nil)
	return C.CString(string(stream.Protocol()))
}

//export kubo_libp2p_stream_remote_peer
func kubo_libp2p_stream_remote_peer(streamHandle uint64) *C.char {
	stream, found := lookupStream(streamHandle)
	if !found {
		setError(fmt.Errorf("invalid stream handle %d", streamHandle))
		return nil
	}
	setError(nil)
	return C.CString(stream.Conn().RemotePeer().String())
}
