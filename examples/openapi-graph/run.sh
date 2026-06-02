#!/usr/bin/env sh
# OpenAPI front demo: a governed OpenAPI call against a local fixture endpoint.
#
# Starts the fixture pets server, runs the graph (whose step resolves the getPet
# operation and calls it), and shows the real response sealed into the receipt.
# No external network; override the binary with RUNX_BIN=/path/to/runx ./run.sh
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
OSS="$(cd "$HERE/../.." && pwd)"
RUNX="${RUNX_BIN:-$OSS/crates/target/debug/runx}"
[ -x "$RUNX" ] || RUNX="$(command -v runx || true)"
[ -n "$RUNX" ] || { echo "runx binary not found; set RUNX_BIN." >&2; exit 1; }

# A demo-only receipt-signing identity (runx mandates signed receipts).
export RUNX_RECEIPT_SIGN_KID="${RUNX_RECEIPT_SIGN_KID:-runx-demo-key}"
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64="${RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64:-QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=}"
export RUNX_RECEIPT_SIGN_ISSUER_TYPE="${RUNX_RECEIPT_SIGN_ISSUER_TYPE:-hosted}"

node "$HERE/server.mjs" &
SERVER=$!
trap 'kill $SERVER 2>/dev/null || true' EXIT
sleep 1

RDIR="$(mktemp -d 2>/dev/null || echo /tmp/runx-openapi-demo)"
"$RUNX" harness "$OSS/examples/openapi-graph" --receipt-dir "$RDIR" --json

echo "------------------------------------------------------------"
echo "the governed OpenAPI call executed against the fixture endpoint:"
grep -rhoE '"executed": *true|"status_code": *200|pet-p-42' "$RDIR" 2>/dev/null | sort -u
