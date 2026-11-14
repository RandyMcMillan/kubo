package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"

	iface "github.com/ipfs/kubo/core/coreiface"
)

//export kubo_name_publish
func kubo_name_publish(handle uint64, pathStr *C.char) *C.char {
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
	name, err := h.api.Name().Publish(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("name publish: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(name.String())
}

//export kubo_name_resolve
func kubo_name_resolve(handle uint64, name *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	p, err := h.api.Name().Resolve(h.ctx, C.GoString(name))
	if err != nil {
		setError(fmt.Errorf("name resolve: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(p.String())
}

//export kubo_name_search
func kubo_name_search(handle uint64, name *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	ch, err := h.api.Name().Search(h.ctx, C.GoString(name))
	if err != nil {
		setError(fmt.Errorf("name search: %w", err))
		return nil
	}
	type result struct {
		Path string `json:"path,omitempty"`
		Err  string `json:"err,omitempty"`
	}
	out := []result{}
	for r := range ch {
		entry := result{Path: r.Path.String()}
		if r.Err != nil {
			entry.Err = r.Err.Error()
		}
		out = append(out, entry)
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("name search marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_key_generate
func kubo_key_generate(handle uint64, name *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	key, err := h.api.Key().Generate(h.ctx, C.GoString(name))
	if err != nil {
		setError(fmt.Errorf("key generate: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(keyJSON(key))
}

//export kubo_key_list
func kubo_key_list(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	keys, err := h.api.Key().List(h.ctx)
	if err != nil {
		setError(fmt.Errorf("key list: %w", err))
		return nil
	}
	out := make([]json.RawMessage, 0, len(keys))
	for _, k := range keys {
		out = append(out, json.RawMessage(keyJSON(k)))
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("key list marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_key_self
func kubo_key_self(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	key, err := h.api.Key().Self(h.ctx)
	if err != nil {
		setError(fmt.Errorf("key self: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(keyJSON(key))
}

//export kubo_key_rename
func kubo_key_rename(handle uint64, oldName *C.char, newName *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	key, _, err := h.api.Key().Rename(h.ctx, C.GoString(oldName), C.GoString(newName))
	if err != nil {
		setError(fmt.Errorf("key rename: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(keyJSON(key))
}

//export kubo_key_remove
func kubo_key_remove(handle uint64, name *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	key, err := h.api.Key().Remove(h.ctx, C.GoString(name))
	if err != nil {
		setError(fmt.Errorf("key remove: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(keyJSON(key))
}

func keyJSON(key iface.Key) string {
	data, err := json.Marshal(map[string]string{
		"name": key.Name(),
		"path": key.Path().String(),
	})
	if err != nil {
		return "{}"
	}
	return string(data)
}
