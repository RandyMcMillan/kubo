# Kubo FFI

C FFI layer for Kubo, exposing IPFS functionality to C and other languages
(notably Rust via `ffi/rust` and Swift via `ffi/swift`).

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

Build a C static archive (used by the Rust/Swift bindings):

```sh
cd ffi && go build -buildmode=c-archive -o tmp/libkubo_ffi.a .
```

## Exports

Typed exports (see `kubo.go`, `libp2p.go`, `git.go`, `nostr.go`, `error.go`,
`command.go`):

- `kubo_version()` — Kubo version string.
- `kubo_ffi_last_error()` / `kubo_ffi_free_string()` / `kubo_ffi_free_buffer()` — error and memory management.
- `kubo_get_logs()` — accumulated Go/network log output since the last call.
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
  generic runner that executes **any** Kubo command in-process (full coverage
  of the CLI/CoreAPI surface, no shelling out).

## Full coverage

`kubo_run_command` / `kubo_run_command_line` parse and execute any Kubo
command through Kubo's own in-process command framework (`cli.Parse` +
`cmds.NewExecutor` + `BuildEnv`). This gives complete protocol/CLI coverage
without spawning a subprocess or a separate binary.

## Testing

C test runner:

```sh
cd ffi/cmd/testffi
# build the archive first
cd ../.. && go build -buildmode=c-archive -o tmp/libkubo_ffi.a . && cd cmd/testffi
cc -o testffi main.c -I ../../tmp ../../tmp/libkubo_ffi.a \
  -lpthread -ldl   # macOS: add -framework Security -framework CoreFoundation -lresolv
./testffi
```

Rust binding and tests:

```sh
make rust-ffi          # build the Rust binding (ffi/rust)
make rust-ffi-test     # build + run the Rust integration tests (test/rust-ffi-test)
# or: cd test/rust-ffi-test && cargo run
```

Swift binding and tests:

```sh
make swift-ffi         # build the Swift binding (ffi/swift)
make test_swift_ffi    # run Swift tests
# or: cd ffi/swift && make test
```

Xcode sample app:

```sh
make xcode-ffi         # build the Xcode/SwiftPM app
# or: cd ffi/xcode && make build
make app               # build a signed .app bundle (cd ffi/xcode && make app)
```

## Memory management

- Strings returned by `kubo_*` functions must be freed with `kubo_ffi_free_string`.
- Buffers returned by `kubo_*_get` / `kubo_*_cat` must be freed with
  `kubo_ffi_free_buffer`.
- `kubo_ffi_last_error()` returns the last error message for the current
  thread (if any).

## Language bindings

- `ffi/rust` — Safe Rust crate (`kubo-ffi`) wrapping the C FFI.
- `ffi/swift` — Swift package (`Kubo`) wrapping the C FFI.
- `ffi/xcode` — Xcode/SwiftPM sample app demonstrating the FFI with a GUI.
- `test/rust-ffi-test` — Rust integration test runner.
