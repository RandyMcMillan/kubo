#!/usr/bin/env bash
# Build the IPFS WebUI from the `webui` submodule and produce
# ffi/rust/assets/ipfs-webui.car next to this script.
#
# Reproducibility notes (upstream recipe, .github/workflows/ci.yml):
#   - node version from webui/.tool-versions (nodejs 24.11.0)
#   - REACT_APP_GIT_REV must equal `git rev-parse --short` of the upstream
#     v4.13.0 tag commit (c51d9809), because it is baked into
#     public/index.html and the JS bundle
#   - CID recipe: `ipfs add --cid-version 1 -r build` (historical, do not
#     modernize — kubo hardcodes the resulting root in core/corehttp/webui.go)
#
# Usage: bash ffi/rust/assets/build-webui-car.sh [--keep-build]
set -euo pipefail

EXPECTED_ROOT="bafybeiciqeyipumpmhxzlxnbqdbbv6u5uij4hy4wax64dmj7kvrhusiq6y"
UPSTREAM_REV="c51d9809" # git rev-parse --short of upstream v4.13.0 tag
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WEBUI_DIR="$REPO_ROOT/webui"
OUT="$REPO_ROOT/ffi/rust/assets/ipfs-webui.car"

command -v ipfs >/dev/null || { echo "ipfs CLI required" >&2; exit 1; }
[ -d "$WEBUI_DIR/src" ] || { echo "webui submodule not initialized" >&2; exit 1; }

echo "== node $(node --version) (want v24.11.0) =="
echo "== webui HEAD: $(git -C "$WEBUI_DIR" rev-parse --short HEAD) =="

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
