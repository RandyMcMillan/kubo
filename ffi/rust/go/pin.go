package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"bytes"
	"encoding/json"
	"fmt"
	"unsafe"

	path "github.com/ipfs/boxo/path"
	cid "github.com/ipfs/go-cid"
	iface "github.com/ipfs/kubo/core/coreiface"
	gocarv2 "github.com/ipld/go-car/v2"
)

// resolvePathArg accepts "/ipfs/..." / "/ipns/..." strings or bare CIDs.
func resolvePathArg(arg string) (path.Path, error) {
	if bytes.HasPrefix([]byte(arg), []byte("/ipfs/")) || bytes.HasPrefix([]byte(arg), []byte("/ipns/")) {
		return path.NewPath(arg)
	}
	c, err := cid.Decode(arg)
	if err != nil {
		return nil, fmt.Errorf("parse %q: %w", arg, err)
	}
	return path.FromCid(c), nil
}

//export kubo_pin_add
func kubo_pin_add(handle uint64, pathStr *C.char) int64 {
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
	if err := h.api.Pin().Add(h.ctx, p); err != nil {
		setError(fmt.Errorf("pin add: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_pin_rm
func kubo_pin_rm(handle uint64, pathStr *C.char) int64 {
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
	if err := h.api.Pin().Rm(h.ctx, p); err != nil {
		setError(fmt.Errorf("pin rm: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_pin_ls
func kubo_pin_ls(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	ch := make(chan iface.Pin, 64)
	if err := h.api.Pin().Ls(h.ctx, ch); err != nil {
		setError(fmt.Errorf("pin ls: %w", err))
		return nil
	}
	type entry struct {
		Path string `json:"path"`
		Name string `json:"name,omitempty"`
		Type string `json:"type"`
	}
	out := []entry{}
	for p := range ch {
		out = append(out, entry{Path: p.Path().String(), Name: p.Name(), Type: p.Type()})
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("pin ls marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_pin_is_pinned
func kubo_pin_is_pinned(handle uint64, pathStr *C.char) int64 {
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
	_, pinned, err := h.api.Pin().IsPinned(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("pin is_pinned: %w", err))
		return -1
	}
	setError(nil)
	if pinned {
		return 1
	}
	return 0
}

//export kubo_pin_update
func kubo_pin_update(handle uint64, fromStr *C.char, toStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	from, err := resolvePathArg(C.GoString(fromStr))
	if err != nil {
		setError(err)
		return -1
	}
	to, err := resolvePathArg(C.GoString(toStr))
	if err != nil {
		setError(err)
		return -1
	}
	if err := h.api.Pin().Update(h.ctx, from, to); err != nil {
		setError(fmt.Errorf("pin update: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_pin_verify
func kubo_pin_verify(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	ch, err := h.api.Pin().Verify(h.ctx)
	if err != nil {
		setError(fmt.Errorf("pin verify: %w", err))
		return nil
	}
	type issue struct {
		Ok       bool   `json:"ok"`
		BadNodes int    `json:"bad_nodes"`
		Err      string `json:"err,omitempty"`
	}
	out := []issue{}
	for st := range ch {
		entry := issue{Ok: st.Ok(), BadNodes: len(st.BadNodes())}
		if st.Err() != nil {
			entry.Err = st.Err().Error()
		}
		out = append(out, entry)
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("pin verify marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}

//export kubo_dag_import
func kubo_dag_import(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}
	reader, err := gocarv2.NewBlockReader(bytes.NewReader(C.GoBytes(unsafe.Pointer(data), C.int(length))))
	if err != nil {
		setError(fmt.Errorf("dag import: %w", err))
		return nil
	}
	bs := h.node.Blockstore
	for {
		blk, err := reader.Next()
		if err != nil {
			break // io.EOF or truncation; blocks read so far are kept
		}
		if err := bs.Put(h.ctx, blk); err != nil {
			setError(fmt.Errorf("dag import put: %w", err))
			return nil
		}
	}
	roots := make([]string, 0, len(reader.Roots))
	for _, root := range reader.Roots {
		if err := h.api.Pin().Add(h.ctx, path.FromCid(root)); err != nil {
			setError(fmt.Errorf("dag import pin root %s: %w", root, err))
			return nil
		}
		roots = append(roots, root.String())
	}
	data2, err := json.Marshal(map[string]any{"roots": roots})
	if err != nil {
		setError(fmt.Errorf("dag import marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data2))
}

//export kubo_unixfs_ls
func kubo_unixfs_ls(handle uint64, pathStr *C.char) *C.char {
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
	ch := make(chan iface.DirEntry, 64)
	if err := h.api.Unixfs().Ls(h.ctx, p, ch); err != nil {
		setError(fmt.Errorf("unixfs ls: %w", err))
		return nil
	}
	type entry struct {
		Name string `json:"name"`
		Cid  string `json:"cid"`
		Size uint64 `json:"size"`
		Type uint64 `json:"type"`
	}
	out := []entry{}
	for e := range ch {
		out = append(out, entry{
			Name: e.Name,
			Cid:  e.Cid.String(),
			Size: e.Size,
			Type: uint64(e.Type),
		})
	}
	data, err := json.Marshal(out)
	if err != nil {
		setError(fmt.Errorf("unixfs ls marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(data))
}
