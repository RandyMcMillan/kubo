package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"sync"
	"unsafe"

	iface "github.com/ipfs/kubo/core/coreiface"
	"github.com/ipfs/kubo/core/coreiface/options"
	peer "github.com/libp2p/go-libp2p/core/peer"
)

// ---------------------------------------------------------------------------
// PubSub (Batch E)
// ---------------------------------------------------------------------------

var (
	pubsubSubsMu  sync.RWMutex
	pubsubSubs    = make(map[uint64]iface.PubSubSubscription)
	pubsubSubNext uint64 = 1
)

//export kubo_pubsub_publish
func kubo_pubsub_publish(handle uint64, topic *C.char, data *C.uint8_t, length C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	payload := C.GoBytes(unsafe.Pointer(data), C.int(length))
	if err := h.api.PubSub().Publish(h.ctx, C.GoString(topic), payload); err != nil {
		setError(fmt.Errorf("pubsub publish: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_pubsub_ls
func kubo_pubsub_ls(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	topics, err := h.api.PubSub().Ls(h.ctx)
	if err != nil {
		setError(fmt.Errorf("pubsub ls: %w", err))
		return nil
	}
	data, err := json.Marshal(topics)
	if err != nil {
		setError(fmt.Errorf("pubsub ls marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_pubsub_peers
func kubo_pubsub_peers(handle uint64, topic *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	topicFilter := C.GoString(topic)
	peers, err := h.api.PubSub().Peers(h.ctx, func(s *options.PubSubPeersSettings) error {
		s.Topic = topicFilter
		return nil
	})
	if err != nil {
		setError(fmt.Errorf("pubsub peers: %w", err))
		return nil
	}
	out := make([]string, 0, len(peers))
	for _, p := range peers {
		out = append(out, p.String())
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("pubsub peers marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_pubsub_subscribe
func kubo_pubsub_subscribe(handle uint64, topic *C.char) uint64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return 0
	}
	sub, err := h.api.PubSub().Subscribe(h.ctx, C.GoString(topic))
	if err != nil {
		setError(fmt.Errorf("pubsub subscribe: %w", err))
		return 0
	}
	pubsubSubsMu.Lock()
	id := pubsubSubNext
	pubsubSubNext++
	pubsubSubs[id] = sub
	pubsubSubsMu.Unlock()
	setError(nil)
	return id
}

//export kubo_pubsub_next
func kubo_pubsub_next(subHandle uint64) *C.char {
	pubsubSubsMu.RLock()
	sub, ok := pubsubSubs[subHandle]
	pubsubSubsMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid subscription handle %d", subHandle))
		return nil
	}
	msg, err := sub.Next(context.Background())
	if err != nil {
		setError(fmt.Errorf("pubsub next: %w", err))
		return nil
	}
	data, err := json.Marshal(map[string]string{
		"from": string(msg.From()),
		"data": base64.StdEncoding.EncodeToString(msg.Data()),
	})
	if err != nil {
		setError(fmt.Errorf("pubsub next marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_pubsub_unsubscribe
func kubo_pubsub_unsubscribe(subHandle uint64) int64 {
	pubsubSubsMu.Lock()
	sub, ok := pubsubSubs[subHandle]
	if ok {
		delete(pubsubSubs, subHandle)
	}
	pubsubSubsMu.Unlock()
	if !ok {
		setError(fmt.Errorf("invalid subscription handle %d", subHandle))
		return -1
	}
	if err := sub.Close(); err != nil {
		setError(fmt.Errorf("pubsub unsubscribe: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Routing (Batch E)
// ---------------------------------------------------------------------------

//export kubo_routing_get
func kubo_routing_get(handle uint64, key *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	value, err := h.api.Routing().Get(h.ctx, C.GoString(key))
	if err != nil {
		setError(fmt.Errorf("routing get: %w", err))
		return -1
	}
	if len(value) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}
	cBuf := C.malloc(C.size_t(len(value)))
	copy((*[1 << 30]byte)(cBuf)[:len(value):len(value)], value)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(value))
	setError(nil)
	return 0
}

//export kubo_routing_put
func kubo_routing_put(handle uint64, key *C.char, value *C.uint8_t, length C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	val := C.GoBytes(unsafe.Pointer(value), C.int(length))
	if err := h.api.Routing().Put(h.ctx, C.GoString(key), val); err != nil {
		setError(fmt.Errorf("routing put: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_routing_find_peer
func kubo_routing_find_peer(handle uint64, peerID *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	pid, err := peer.Decode(C.GoString(peerID))
	if err != nil {
		setError(fmt.Errorf("parse peer id: %w", err))
		return nil
	}
	info, err := h.api.Routing().FindPeer(h.ctx, pid)
	if err != nil {
		setError(fmt.Errorf("routing find peer: %w", err))
		return nil
	}
	addrs := make([]string, 0, len(info.Addrs))
	for _, a := range info.Addrs {
		addrs = append(addrs, a.String())
	}
	data, err := json.Marshal(map[string]any{"id": info.ID.String(), "addrs": addrs})
	if err != nil {
		setError(fmt.Errorf("routing find peer marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_routing_find_providers
func kubo_routing_find_providers(handle uint64, pathStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	p, err := resolvePathArg(C.GoString(pathStr))
	if err != nil {
		setError(err)
		return nil
	}
	ch, err := h.api.Routing().FindProviders(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("routing find providers: %w", err))
		return nil
	}
	out := []map[string]any{}
	for info := range ch {
		addrs := make([]string, 0, len(info.Addrs))
		for _, a := range info.Addrs {
			addrs = append(addrs, a.String())
		}
		out = append(out, map[string]any{"id": info.ID.String(), "addrs": addrs})
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("routing find providers marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_routing_provide
func kubo_routing_provide(handle uint64, pathStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	p, err := resolvePathArg(C.GoString(pathStr))
	if err != nil {
		setError(err)
		return -1
	}
	if err := h.api.Routing().Provide(h.ctx, p); err != nil {
		setError(fmt.Errorf("routing provide: %w", err))
		return -1
	}
	setError(nil)
	return 0
}
