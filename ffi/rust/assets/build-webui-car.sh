#!/usr/bin/env bash
# Build the IPFS WebUI from the `webui` submodule and produce
# ffi/rust/assets/ipfs-webui.car next to this script.
#
# NOTE (2026-11-14, measured): bit-exact reproduction of the official
# release CAR was attempted and FAILED — two consecutive builds of the
# same tree (d87d88be) with identical env produced different roots
# (bafybeig3xbjaz… vs bafybeihkcl66…). The webpack/terser pipeline is
# non-deterministic run-to-run on this machine. Therefore:
#   - the committed release CAR stays the canonical asset (it is what
#     upstream ships and what kubo's hardcoded WebUIPath expects)
#   - this script builds a FUNCTIONALLY IDENTICAL WebUI whose root CID
#     differs; use it for the gateway-direct URL, not for /webui redirect
#
# Alignment recipe used (all verified necessary, not sufficient):
#   - node version from webui/.tool-versions (nodejs 24.11.0, via nvm)
#   - build commit d87d88be7ed989cc0985f5f70aa0fa8e3315856b ("fix(files):
#     unpin option missing in batch delete (#2488)") — the v4.13.0 TAG
#     (c51d9809) is the later "[skip ci]" version bump; the release CAR's
#     index.html meta revision says d87d88b and its bundle embeds
#     package version 4.12.0 (pre-bump), matching this commit
#   - REACT_APP_GIT_REV=d87d88b — the rev string is baked into
#     public/index.html and the JS bundle
#   - CID recipe: `ipfs add --cid-version 1 -r build` (historical, do not
#     modernize — kubo hardcodes the release root in core/corehttp/webui.go)
#   - delete any build/.DS_Store before `ipfs add` (macOS noise changes the root)
#   - deps install must skip postinstall scripts: the `kubo` npm package
#     (e2e-test binary downloader) fails offline; the webpack build needs
#     none of them (esbuild/swc resolve via optional platform packages)
#
# Usage: bash ffi/rust/assets/build-webui-car.sh [--keep-build]
set -euo pipefail

EXPECTED_ROOT="bafybeiciqeyipumpmhxzlxnbqdbbv6u5uij4hy4wax64dmj7kvrhusiq6y"
UPSTREAM_REV="d87d88b" # git rev-parse --short of the actual release build commit
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WEBUI_DIR="$REPO_ROOT/webui"
OUT="$REPO_ROOT/ffi/rust/assets/ipfs-webui.car"

command -v ipfs >/dev/null || { echo "ipfs CLI required" >&2; exit 1; }
[ -d "$WEBUI_DIR/src" ] || { echo "webui submodule not initialized" >&2; exit 1; }

if [ ! -d "$WEBUI_DIR/node_modules" ]; then
  if [ -s "$HOME/.nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.nvm/nvm.sh"
    nvm use 24.11.0
  fi
  echo "== node $(node --version) (want v24.11.0) =="
  echo "== installing deps (postinstall scripts skipped) =="
  (cd "$WEBUI_DIR" && PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 \
    npm ci --no-audit --no-fund --ignore-scripts)
fi
echo "== webui HEAD: $(git -C "$WEBUI_DIR" rev-parse --short HEAD) =="
if [ "$(git -C "$WEBUI_DIR" rev-parse HEAD)" != "d87d88be7ed989cc0985f5f70aa0fa8e3315856b" ]; then
  echo "WARNING: webui checkout is not d87d88be — root CID will not match the release" >&2
fi

cd "$WEBUI_DIR"
npm run prebuild
REACT_APP_GIT_REV="$UPSTREAM_REV" NODE_ENV=production \
  npx react-app-rewired-esm build

ROOT="$(ipfs add --cid-version 1 -Q -r -n build)"
echo "built root: $ROOT"
echo "expected:   $EXPECTED_ROOT"
if [ "$ROOT" != "$EXPECTED_ROOT" ]; then
  echo "WARNING: CID mismatch — the /api /webui redirect will not match this build" >&2
fi

TMP_REPO="$(mktemp -d)"
trap 'rm -rf "$TMP_REPO"' EXIT
IPFS_PATH="$TMP_REPO" ipfs init --empty-repo >/dev/null
IPFS_PATH="$TMP_REPO" ipfs add --cid-version 1 -Q -r build >/dev/null
IPFS_PATH="$TMP_REPO" ipfs dag export "$ROOT" > "$OUT"
echo "wrote $OUT ($(wc -c < "$OUT") bytes)"

if [ "${1:-}" != "--keep-build" ]; then
  rm -rf "$WEBUI_DIR/build"
fi
