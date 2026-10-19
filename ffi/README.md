# Kubo FFI

This directory contains the C FFI layer for Kubo, exposing IPFS functionality
to C and other languages (notably Rust via `test/rust-ffi-test`).

## Building

Build the Go package:

```sh
make ffi
# or: cd ffi && go build ./...
```

Build a C shared library:

```sh
make ffi-shared
# or: cd ffi && go build -buildmode=c-shared -o libkubo.dylib .
```

Build a C static archive (for the Rust tests):

```sh
cd ffi && go build -buildmode=c-archive -o tmp/libkubo_ffi.a .
```

## Exports

The library exports the following C functions (see `kubo.go`, `libp2p.go`,
`git.go`, `nostr.go`, `error.go`):

- `kubo_version()` — Kubo version string.
- `kubo_ffi_last_error()` / `kubo_ffi_free_string()` / `kubo_ffi_free_buffer()` — error and memory management.
- `kubo_init_repo(repoPath)` — initialize a repository.
- `kubo_node_start(repoPath, online)` / `kubo_node_stop(handle)` — node lifecycle.
- `kubo_node_peer_id(handle)` / `kubo_node_id(handle)` — peer identity.
- `kubo_node_listening_addrs(handle)` / `kubo_node_api_addrs(handle)` / `kubo_swarm_peers(handle)` — addresses.
- `kubo_node_connect(handle, addr)` — connect to a peer.
- `kubo_unixfs_add_bytes(handle, data, length)` / `kubo_unixfs_cat(handle, cid, out, out_len)` — UnixFS.
- `kubo_block_put` / `kubo_block_get` / `kubo_block_stat` — raw blocks.
- `kubo_git_*` — git repository helpers.
- `kubo_nostr_relay_*` — Nostr relay helpers.
- `kubo_run_command(repoPath, argc, argv)` / `kubo_run_command_line(repoPath, line)` —
  generic runner that executes *any* Kubo command (full coverage of the
  CLI/CoreAPI surface) and returns its output.

## Testing

Run the C test runner:

```sh
cd ffi/cmd/testffi && cc -o testffi main.c -I ../../tmp ../../tmp/libkubo_ffi.a \
  -lpthread -ldl  # add macOS frameworks as needed
./testffi
```

Run the Rust test runner:

```sh
make test_rust_ffi
# or: cd test/rust-ffi-test && cargo run
```

## Notes

- The FFI is a `package main` with `//export` directives, so it builds as a
  cgo shared/static library.
- Callers are responsible for freeing strings via `kubo_ffi_free_string` and
  buffers via `kubo_ffi_free_buffer`.
- `kubo_ffi_last_error()` returns the last error message (if any) for the
  current thread.
