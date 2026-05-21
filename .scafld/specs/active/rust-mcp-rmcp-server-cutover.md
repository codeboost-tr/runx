---
spec_version: '2.0'
task_id: rust-mcp-rmcp-server-cutover
created: '2026-05-21T12:12:00Z'
updated: '2026-05-21T16:42:00Z'
status: active
harden_status: not_run
size: large
risk_level: high
---

# rmcp server and deletion cutover for MCP

## Current State

Status: active
Current phase: staged rmcp server proof landed; deletion gate still blocked
Next: decide whether to approve a byte-level lifecycle fixture change or keep
the hand-rolled server as the canonical Content-Length fixture oracle
Reason: `rust-mcp-rmcp-cutover` now owns only the completed Stage 1-2 client
transport slice. This draft owns the remaining server-loop migration,
rmcp-served wire parity, and deletion gate. It must not be executed blindly:
the current tree still depends on hand-rolled Content-Length framing for
`serve_mcp_json_rpc`, and rmcp 1.7.0's default async read/write transport is
newline-delimited rather than runx's recorded Content-Length stdio wire shape.
Blockers: rmcp 1.7.0 rejects the current recorded `basic-lifecycle` fixture's
`initialize` request because it uses `params: {}` rather than the MCP-required
`protocolVersion`, `capabilities`, and `clientInfo` fields. The staged rmcp
path now proves a valid rmcp lifecycle over the shared Content-Length
transport, but it does not justify deleting `framing.rs`, `jsonrpc.rs`, or the
canonical `mcp` fixture oracle yet.
Allowed follow-up command: `scafld harden rust-mcp-rmcp-server-cutover`
Latest runner update: 2026-05-21T16:42:00Z
Review gate: not_started

## Summary

Complete the remaining MCP rmcp cutover after the client transport slice.
The first executable slice adds an rmcp-backed server loop for `runx mcp
serve` behind the staged `mcp-rmcp` feature and proves it can serve the runx
tool surface over Content-Length framing. The deletion slice remains blocked
until the recorded wire fixtures are either updated through an approved
byte-diff envelope or the owner narrows the cutover to keep the existing
hand-rolled server fixture oracle.

This is a clean cutover target, not a compatibility shim. Until the server
transport is proven byte-compatible, the existing `mcp` path remains the
authoritative server path and the staged `mcp-rmcp` feature remains a client
transport plus server-lifecycle proof.

## Context

CWD: `.` from the OSS repo root.

Completed prerequisite:
- `rust-mcp-rmcp-cutover` Stage 1-2: disjoint `mcp-rmcp` feature, exact
  `rmcp = "=1.7.0"` pin, rmcp-backed `ProcessMcpTransport`, client error
  preservation, stderr draining, and deny/license gates.

Completed in this slice:
- Shared `RmcpContentLengthTransport` is used by the rmcp client and staged
  rmcp server path.
- `serve_mcp_json_rpc` has a staged `mcp-rmcp` server implementation that
  serves a finite fixture input containing initialization, tool listing, and
  tool calls through rmcp without changing the canonical `mcp` path.
- Existing `mcp` tests remain the canonical recorded wire oracle for
  `basic-lifecycle`, `error-paths`, skill execution, and sealed harness
  receipt projection.

Current hand-rolled server/protocol files:
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `crates/runx-runtime/src/adapters/mcp/framing.rs`
- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs`
- mcp-only client path in `crates/runx-runtime/src/adapters/mcp/transport.rs`

Runx-specific surfaces that must stay:
- `server_skill.rs`
- `templates.rs`
- `sandbox_metadata.rs`
- `adapter.rs`
- `McpServerTool`, `McpHostRunResult`, and sealed harness receipt projection

## Objectives

- Add an rmcp-backed server loop for `serve_mcp_json_rpc` behind the staged
  cutover path without changing runx tool behavior.
- Preserve the recorded Content-Length stdio wire contract for
  `basic-lifecycle` and `error-paths` fixtures, or explicitly record the
  predecessor-approved diff envelope with byte-level evidence.
- Keep malformed request, invalid header, oversized request, unknown method,
  tool error, needs-agent, denied, escalated, failed, and receipt-sealing
  behavior stable.
- Once rmcp-served wire parity passes, remove the hand-rolled protocol path,
  collapse `mcp-rmcp` into the canonical `mcp` feature, and remove the scoped
  `rmcp`/tokio wrapper exception from `crates/deny.toml`.

## Non-Goals

- No SSE or streamable HTTP MCP transport.
- No public reusable rmcp server trait unless the server cutover requires it.
- No compatibility alias between old and new feature names after the deletion
  gate. The end state is one `mcp` path.
- No change to harness receipts, skill execution, sandbox metadata, or
  argument templating.

## Design Constraints

- rmcp's built-in `AsyncRwTransport` is newline-delimited JSON. Runx's recorded
  MCP stdio wire contract is Content-Length framed. The server design must
  either use an rmcp transport that preserves Content-Length framing or provide
  a small, reviewed transport adapter with explicit wire-contract tests.
- The server cutover must not repeat the client-slice review defects:
  receive-side framing errors must be observable, and child stderr must be
  bounded-drained when a child process is used.
- Feature flags are temporary execution scaffolding only. The final code must
  not keep a permanent legacy path.

## Validation

- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --test mcp_server -- --nocapture`
  runs the staged rmcp server path and passes for a valid rmcp lifecycle.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --test mcp_adapter -- --nocapture`
  passes for the rmcp client path.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --lib rmcp_transport_tests -- --nocapture`
  passes for shared Content-Length transport error capture.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_adapter --test mcp_server -- --nocapture`
  passes for the canonical legacy `mcp` wire fixture path while deletion is
  blocked.
- A wire-contract test compares rmcp-served raw stdout bytes against
  `fixtures/runtime/adapters/mcp/wire-contract/basic-lifecycle.responses.jsonl`
  and `error-paths.responses.jsonl`. This is still blocked by the fixture
  lifecycle mismatch above; do not delete the hand-rolled server until this
  check is resolved by an approved fixture diff or a narrowed objective.
- A production `runx mcp serve` deletion gate must prove the rmcp server loop
  streams responses without waiting for stdin EOF. The current `mcp-rmcp`
  server slice is finite-input proof only.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp -- -D warnings`
  passes for the canonical `mcp` path.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp-rmcp -- -D warnings`
  is required before completion; as of this slice it is blocked by concurrent
  payment-state dead-code work outside the MCP write set, not by MCP code.
- Deletion gate only: `cargo deny --manifest-path crates/Cargo.toml check bans licenses`
  passes with no scoped `rmcp` ban exception after the hand-rolled server path
  is removed and `mcp-rmcp` collapses into `mcp`.
- `rg "^mod (framing|jsonrpc)" crates/runx-runtime/src/adapters/mcp.rs`
  returns no matches after deletion.

## Acceptance

- `runx mcp serve` is backed by rmcp for protocol dispatch.
- There is exactly one MCP feature path in `runx-runtime`.
- No hand-rolled JSON-RPC/framing modules remain unless the harden pass records
  a specific rmcp limitation and the owner explicitly narrows the deletion
  objective before build.
- The public wire-contract fixtures remain the source of truth for MCP stdio.

## References

- `.scafld/specs/active/rust-mcp-rmcp-cutover.md`
- `.scafld/specs/archive/2026-05/rust-mcp-rmcp-adoption.md`
- `crates/runx-runtime/src/adapters/mcp/server.rs`
- `fixtures/runtime/adapters/mcp/wire-contract/`
