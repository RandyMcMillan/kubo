# WebUI asset

`include_bytes!`d by `examples/web.rs` so the example is self-contained:
no network fetch on first `/webui` load.

## Files

- `ipfs-webui-v4.13.0.car` — release asset from
  <https://github.com/ipfs/ipfs-webui/releases/download/v4.13.0/ipfs-webui%40v4.13.0.car>,
  sha256 `0f26ce282643f26fc699451f3b71bcac6bacdd96c052612406337d69e8a5e55b`.
  Root CID `bafybeiciqeyipumpmhxzlxnbqdbbv6u5uij4hy4wax64dmj7kvrhusiq6y`
  matches this Kubo's `core/corehttp.WebUIPath` (v4.13.0). Per upstream
  policy this CAR is the canonical copy of the WebUI.
- The CAR is gitignored (29MB). `build.rs` fetches it automatically on
  first build (sha256-verified); `KUBO_FFI_SKIP_WEBUI_ASSET=1` skips that.

## Build from source

`build-webui-car.sh` builds the WebUI from the `webui` submodule with the
upstream CI recipe (node 24.11.0 per `.tool-versions`, build commit
`d87d88be` — the v4.13.0 tag is the later `[skip ci]` version bump,
`REACT_APP_GIT_REV=d87d88b`, `ipfs add --cid-version 1 -r build`).

Measured 2026-11-14: the build is NOT bit-reproducible run-to-run
(same tree, same env, two builds → two different roots:
`bafybeig3xbjaz…` vs `bafybeihkcl66…`). The committed release CAR stays
the canonical asset; script-built CARs are functionally identical but
have a different root, so the API `/webui` redirect (hardcoded to the
release root) will not resolve them — use the gateway URL in that case.

Run after updating the webui submodule:

```bash
bash ffi/rust/assets/build-webui-car.sh
```
