package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"

	ma "github.com/multiformats/go-multiaddr"
)

//export kubo_swarm_disconnect
func kubo_swarm_disconnect(handle uint64, addr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	maddr, err := ma.NewMultiaddr(C.GoString(addr))
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return -1
	}
	if err := h.api.Swarm().Disconnect(h.ctx, maddr); err != nil {
		setError(fmt.Errorf("swarm disconnect: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_swarm_local_addrs
func kubo_swarm_local_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	addrs, err := h.api.Swarm().LocalAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm local addrs: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(multiaddrsJSON(addrs))
}

//export kubo_swarm_listen_addrs
func kubo_swarm_listen_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	addrs, err := h.api.Swarm().ListenAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm listen addrs: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(multiaddrsJSON(addrs))
}

//export kubo_swarm_known_addrs
func kubo_swarm_known_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	known, err := h.api.Swarm().KnownAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm known addrs: %w", err))
		return nil
	}
	out := map[string][]string{}
	for p, addrs := range known {
		out[p.String()] = multiaddrStrings(addrs)
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("swarm known addrs marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

func multiaddrStrings(addrs []ma.Multiaddr) []string {
	out := make([]string, 0, len(addrs))
	for _, a := range addrs {
		out = append(out, a.String())
	}
	return out
}

func multiaddrsJSON(addrs []ma.Multiaddr) string {
	data, err := json.Marshal(multiaddrStrings(addrs))
	if err != nil {
		return "[]"
	}
	return string(data)
}
