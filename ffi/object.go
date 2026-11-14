package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"

	"github.com/ipfs/boxo/path"
)

//export kubo_object_add_link
func kubo_object_add_link(handle uint64, base *C.char, name *C.char, child *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	basePath, err := resolvePathArg(C.GoString(base))
	if err != nil {
		setError(err)
		return nil
	}
	childPath, err := resolvePathArg(C.GoString(child))
	if err != nil {
		setError(err)
		return nil
	}
	updated, err := h.api.Object().AddLink(h.ctx, basePath, C.GoString(name), childPath)
	if err != nil {
		setError(fmt.Errorf("object add link: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(updated.String())
}

//export kubo_object_rm_link
func kubo_object_rm_link(handle uint64, base *C.char, link *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	basePath, err := resolvePathArg(C.GoString(base))
	if err != nil {
		setError(err)
		return nil
	}
	updated, err := h.api.Object().RmLink(h.ctx, basePath, C.GoString(link))
	if err != nil {
		setError(fmt.Errorf("object rm link: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(updated.String())
}

// immutablePathString renders a boxo path.ImmutablePath, mapping the zero
// value (present in object diff add/remove entries) to "" instead of
// panicking on its nil inner path.
func immutablePathString(p path.ImmutablePath) (s string) {
	defer func() {
		if recover() != nil {
			s = ""
		}
	}()
	return p.String()
}

//export kubo_object_diff
func kubo_object_diff(handle uint64, base *C.char, target *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	basePath, err := resolvePathArg(C.GoString(base))
	if err != nil {
		setError(err)
		return nil
	}
	targetPath, err := resolvePathArg(C.GoString(target))
	if err != nil {
		setError(err)
		return nil
	}
	changes, err := h.api.Object().Diff(h.ctx, basePath, targetPath)
	if err != nil {
		setError(fmt.Errorf("object diff: %w", err))
		return nil
	}
	type change struct {
		Type   int    `json:"type"`
		Path   string `json:"path"`
		Before string `json:"before,omitempty"`
		After  string `json:"after,omitempty"`
	}
	out := make([]change, 0, len(changes))
	for _, c := range changes {
		entry := change{Type: int(c.Type), Path: c.Path}
		if before := immutablePathString(c.Before); before != "" && before != "/" {
			entry.Before = before
		}
		if after := immutablePathString(c.After); after != "" && after != "/" {
			entry.After = after
		}
		out = append(out, entry)
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("object diff marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}
