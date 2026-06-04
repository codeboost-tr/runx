#!/usr/bin/env sh
# GitHub MCP hero demo: governed read succeeds; out-of-scope write is refused
# before the MCP mutation tool runs. No external network is used.
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
OSS="$(cd "$HERE/../.." && pwd)"
RUNX="${RUNX_BIN:-$OSS/crates/target/debug/runx}"
[ -x "$RUNX" ] || RUNX="$(command -v runx || true)"
[ -n "$RUNX" ] || { echo "runx binary not found; set RUNX_BIN." >&2; exit 1; }

export RUNX_RECEIPT_SIGN_KID="${RUNX_RECEIPT_SIGN_KID:-runx-demo-key}"
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64="${RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64:-QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=}"
export RUNX_RECEIPT_SIGN_ISSUER_TYPE="${RUNX_RECEIPT_SIGN_ISSUER_TYPE:-hosted}"

RDIR="$(mktemp -d 2>/dev/null || echo /tmp/runx-github-mcp-demo)"
"$RUNX" harness "$HERE" --receipt-dir "$RDIR" --json

echo "------------------------------------------------------------"
echo "receipts: $RDIR"
grep -rhoE '"reason_code": *"authority_denied"|"disposition": *"blocked"' "$RDIR" 2>/dev/null | sort -u
