# Kubo FFI

C FFI layer for Kubo, exposing the IPFS stack to C and other languages —
notably Rust via `ffi/rust` (the primary consumer) and Swift via
`ffi/swift`.

One dependency does everything: a full Kubo node (unixfs adds, blocks,
dag/CAR, pins, objects, IPNS, keys, swarm, pubsub, DHT routing, gateway +
API serving, MFS via the command runner), a standalone libp2p host with
raw stream plumbing for arbitrary protocols, git-over-IPFS, and nostr
keys/relays.

## Layout

- `ffi/*.go` — the FFI module (`package main`, built as c-archive/c-shared).
  One file per subsystem: `kubo.go` (node lifecycle, content, serving),
  `pin.go`, `swarm.go`, `ipns.go`, `object.go`, `pubsub.go`, `ops.go`,
  `libp2p.go`, `git.go`, `nostr.go`, `command.go`, `error.go`.
- `ffi/rust/` — the `kubo-ffi` crate: safe Rust over the c-archive.
  - `src/lib.rs` — all 136 wrappers + `exposure_matrix()` compile-time test
    + `ffi_full_coverage` runtime suite.
  - `src/web.rs` — `web` feature: self-contained WebUI server.
  - `src/main.rs` — `kubo-ffi` binary (WebUI server with `--features web`,
    FFI demo without).
  - `go/` — self-contained copy of the Go module for crates.io tarballs
    (requires upstream `kubo` from the proxy; `build.rs` prefers the
    canonical `../` module in this repo).
  - `examples/` — `web.rs` (self-contained WebUI demo), `smoke.rs`,
    `tui.rs`.
- `ffi/EXPOSURE.md` — the authoritative accounting: 136/136 exports
  wrapped, 40/40 CoreAPI first-class, phase log with commits and tags.
- `webui/` — submodule (ipfs-webui source; used by
  `ffi/rust/assets/build-webui-car.sh` to rebuild the WebUI CAR).

## Building

Go side:

```sh
make ffi
cd ffi && go build -buildmode=c-archive -o tmp/libkubo_ffi.a .
```

Rust side (build.rs compiles the Go module automatically):

```sh
cd ffi/rust
cargo build                          # lib
cargo build --features web           # lib + WebUI server module
cargo run --example web              # self-contained WebUI demo
cargo test --lib                     # 5 tests incl. the runtime suite
cargo test --lib ffi_full_coverage -- --nocapture --test-threads=1
```

`build.rs` fetches the WebUI CAR (29MB, sha256-verified) into `OUT_DIR`
on first build; `KUBO_FFI_SKIP_WEBUI_ASSET=1` opts out (offline/CI).

## Hard rules (enforced in CI-by-convention)

1. Every new `//export` gets a safe Rust wrapper AND an
   `exposure_matrix()` entry — the crate fails to compile otherwise.
2. After changing `ffi/*.go`, sync `ffi/rust/go/` (same files, plus the
   standalone `go.mod`/`go.sum` that require upstream kubo from the
   proxy).
3. Runtime-proof new surface in `ffi_full_coverage` (bannered sections).
4. One topic per commit; tag each batch (`ffi-batch-*`, `ffi-exposure-*`).

## Safety notes for FFI callers

- Error strings come from process-global Go state (`kubo_ffi_last_error`);
  under concurrent calls they can race — diagnostics only.
- Fetching non-local content blocks on provider discovery; use the
  `_timeout` variants (`kubo_unixfs_cat_timeout`, `kubo_block_get_timeout`).
- Stream/peer/node handles are u64 tokens owned by Go registries; closing
  twice is tolerated, using after close is not.

The original export-by-export reference below predates the batch work;
`EXPOSURE.md` is the maintained record.

---

## Exports (historical reference, see EXPOSURE.md)

Typed exports (see `kubo.go`, `libp2p.go`, `git.go`, `nostr.go`,
`error.go`, `command.go`):

- `kubo_version()` — Kubo version string.
- `kubo_ffi_last_error()` / `kubo_ffi_free_string()` / `kubo_ffi_free_buffer()` — error and memory management.
- `kubo_get_logs()` — accumulated Go/network log output since the last call.
- `kubo_init_repo(repoPath)` — initialize an isolated (loopback) repository.
- `kubo_init_repo_networked(repoPath)` — initialize a repository configured for
  real network participation (0.0.0.0 listen, mDNS, QUIC, relay, AutoTLS, auto
  routing, default bootstrap peers).
- `kubo_init_repo_with_config(repoPath, configJSON)` — initialize a repository
  from a caller-supplied Kubo config JSON blob (full control).
- `kubo_node_start(repoPath, online)` / `kubo_node_stop(handle)` — node lifecycle.
- `kubo_node_peer_id(handle)` / `kubo_node_id(handle)` — peer identity.
- `kubo_node_listening_addrs(handle)` / `kubo_node_api_addrs(handle)` / `kubo_swarm_peers(handle)` — addresses.
- `kubo_node_connect(handle, addr)` — connect to a peer.
- `kubo_node_start_api(handle, multiaddr)` — serve HTTP API + WebUI + gateway.
  Exposes the writable CoreAPI; do not bind to a public address.
- `kubo_node_start_gateway(handle, multiaddr)` — serve ONLY the IPFS gateway
  (/ipfs, /ipns) and registered static dirs. Safe for public registry servers.
- `kubo_node_add_static(handle, prefix, dir)` — register a static directory to
  be served. Must be called BEFORE `kubo_node_start_api`/`kubo_node_start_gateway`.
- `kubo_node_add_crate_redirect(handle, registryDir)` — register a redirect
  handler: `/crates/{prefix}/{crate}/{version}.crate` requests are 302-redirected
  to `/ipfs/<cid>` when the index entry has an `ipfs_cid`. Falls back to
  serving from disk. Must be called BEFORE `kubo_node_start_gateway`.
- `kubo_unixfs_add_bytes(handle, data, length)` — add bytes; content is PINNED.
- `kubo_unixfs_cat(handle, cid, out, out_len)` — fetch bytes (binary-safe).
  Accepts full paths like `/ipfs/<cid>/config.json` for directory traversal.
- `kubo_block_put` / `kubo_block_get` / `kubo_block_stat` — raw blocks.
- `kubo_git_*` — git repository helpers (clone, open, head, free, init,
  is_bare, branches, remotes, create_branch, commit_lookup, tree_entries,
  blob_read, status, diff_trees, fetch_all, blame, log, tags, push).
- `kubo_nostr_*` — Nostr key/event/relay helpers (generate_key,
  get_public_key, event_sign, event_sign_with_tags, event_verify, nip19
  encode/decode, nip05 verify/query, relay connect/close/subscribe/drain/
  unsubscribe/publish, nsec_to_swarm_key).
- `kubo_swarm_key_generate()` — generate a random IPFS swarm key.
- `kubo_watch_path(repoPath, watchPath)` / `kubo_watch_stop(handle)` — directory watcher (ipfswatch).
- `kubo_run_command(repoPath, argc, argv)` / `kubo_run_command_line(repoPath, line)` —
  generic runner that executes **any** Kubo command in-process (full coverage
  of the CLI/CoreAPI surface, no shelling out).

## Full coverage

`kubo_run_command` / `kubo_run_command_line` parse and execute any Kubo
command through Kubo's own in-process command framework (`cli.Parse` +
`cmds.NewExecutor` + `BuildEnv`). This gives complete protocol/CLI coverage
without spawning a subprocess or a separate binary.

Caveats for the command runner:
- Command lines must NOT include the leading `ipfs` token (e.g. `"version"`,
  not `"ipfs version"`).
- The returned string is the command's emitted value, which is frequently a
  Go struct debug dump rather than plain text. Do not use it for binary
  content; use `kubo_unixfs_cat` / `kubo_block_get` for bytes.
- `kubo_run_command_line` spawns its own node; prefer `kubo_run_command_on_node`
  (and the Rust `Node::run_on_node`) when a node handle is already running.

## Registry / public serving

For a Cargo-style static registry served over IPFS, the recommended sequence
is:

1. `kubo_init_repo` (isolated) or `kubo_init_repo_networked` (public).
2. `kubo_node_start(repoPath, true)`.
3. `kubo_node_add_static(handle, "/", registryDir)` — register the registry dir.
4. `kubo_unixfs_add_bytes(handle, crateBytes, len)` — publish a crate; content
   is pinned automatically and a CID is returned.
5. `kubo_node_start_gateway(handle, "/ip4/0.0.0.0/tcp/0")` — serve the
   gateway (`/ipfs/<cid>`) and the static registry (`/config.json`,
   `/de/mo/<crate>`) on one public listener. The writable CoreAPI is NOT
   exposed.

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

## Cross-compilation

The build script (`ffi/rust/build.rs`) maps the Rust `TARGET` triple to
Go `GOOS`/`GOARCH` and sets `GOTOOLCHAIN` from the `go` directive in
`go.mod`. Supported targets:

- **macOS**: x86_64, aarch64 (native + same-family cross)
- **Linux**: x86_64, aarch64
- **Windows**: x86_64 (MSVC via MinGW)
- **iOS**: device (aarch64), simulator (x86_64/aarch64), Mac Catalyst

Apple cross-compilation uses `xcrun` to select the correct SDK and compiler.
Unsupported host/target pairs bail out early with a clear error message.

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
- `ffi/rust/examples/tui.rs` — ratatui TUI demo (peers, logs).
- `ffi/rust/examples/web.rs` — WebUI service demo.
- `test/rust-ffi-test` — Rust integration test runner.
