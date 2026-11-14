package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"

	"github.com/ipfs/kubo/core/corerepo"
)

//export kubo_repo_stat
func kubo_repo_stat(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	usage, err := h.node.Repo.GetStorageUsage(h.ctx)
	if err != nil {
		setError(fmt.Errorf("repo stat: %w", err))
		return nil
	}
	data, err := json.Marshal(map[string]any{"storage_usage_bytes": usage})
	if err != nil {
		setError(fmt.Errorf("repo stat marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_repo_gc
func kubo_repo_gc(handle uint64) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	if err := corerepo.GarbageCollect(h.node, h.ctx); err != nil {
		setError(fmt.Errorf("repo gc: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_bootstrap_list
func kubo_bootstrap_list(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	cfg, err := h.node.Repo.Config()
	if err != nil {
		setError(fmt.Errorf("bootstrap list: %w", err))
		return nil
	}
	data, err := json.Marshal(cfg.Bootstrap)
	if err != nil {
		setError(fmt.Errorf("bootstrap list marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_bootstrap_add
func kubo_bootstrap_add(handle uint64, multiaddr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	cfg, err := h.node.Repo.Config()
	if err != nil {
		setError(fmt.Errorf("bootstrap add: %w", err))
		return -1
	}
	addr := C.GoString(multiaddr)
	for _, existing := range cfg.Bootstrap {
		if existing == addr {
			setError(nil)
			return 0
		}
	}
	cfg.Bootstrap = append(cfg.Bootstrap, addr)
	if err := h.node.Repo.SetConfig(cfg); err != nil {
		setError(fmt.Errorf("bootstrap add persist: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_bootstrap_rm
func kubo_bootstrap_rm(handle uint64, multiaddr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	cfg, err := h.node.Repo.Config()
	if err != nil {
		setError(fmt.Errorf("bootstrap rm: %w", err))
		return -1
	}
	addr := C.GoString(multiaddr)
	kept := cfg.Bootstrap[:0]
	for _, existing := range cfg.Bootstrap {
		if existing != addr {
			kept = append(kept, existing)
		}
	}
	cfg.Bootstrap = kept
	if err := h.node.Repo.SetConfig(cfg); err != nil {
		setError(fmt.Errorf("bootstrap rm persist: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_stats_bw
func kubo_stats_bw(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	if h.node.Reporter == nil {
		setError(fmt.Errorf("bandwidth reporter unavailable (offline node?)"))
		return nil
	}
	totals := h.node.Reporter.GetBandwidthTotals()
	data, err := json.Marshal(map[string]any{
		"total_in":  totals.TotalIn,
		"total_out": totals.TotalOut,
		"rate_in":   totals.RateIn,
		"rate_out":  totals.RateOut,
	})
	if err != nil {
		setError(fmt.Errorf("stats bw marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_config_get
func kubo_config_get(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	cfg, err := h.node.Repo.Config()
	if err != nil {
		setError(fmt.Errorf("config get: %w", err))
		return nil
	}
	data, err := json.Marshal(cfg)
	if err != nil {
		setError(fmt.Errorf("config get marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_config_set
func kubo_config_set(handle uint64, key *C.char, valueJSON *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	var value any
	if err := json.Unmarshal([]byte(C.GoString(valueJSON)), &value); err != nil {
		setError(fmt.Errorf("config set: value must be JSON: %w", err))
		return -1
	}
	if err := h.node.Repo.SetConfigKey(C.GoString(key), value); err != nil {
		setError(fmt.Errorf("config set: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_node_shutdown
func kubo_node_shutdown(handle uint64) int64 {
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
	if err := h.node.Close(); err != nil {
		setError(fmt.Errorf("node shutdown: %w", err))
		return -1
	}
	setError(nil)
	return 0
}
