---
spec_version: '2.0'
task_id: rust-mcp-rmcp-cutover
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T12:06:52Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# rmcp client transport cutover for the MCP adapter

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T12:06:52Z
Review gate: pass

## Why this exists

The MCP adapter at
[`crates/runx-runtime/src/adapters/mcp/`](../../crates/runx-runtime/src/adapters/mcp/)
hand-rolls the MCP protocol: Content-Length framing, JSON-RPC request/response
correlation, the stdio client transport, and the stdio server loop. The
predecessor spec established that the upstream `rmcp` crate should own that
protocol surface, that the runx-specific surfaces stay, and that the cutover
must preserve the recorded byte-shape contract. `rmcp` started banned in
[`crates/deny.toml`](../../crates/deny.toml) precisely because this cutover had
not run; this slice converts that to a package-scoped `runx-runtime` exception
while the staged `mcp-rmcp` feature exists and moves the process client
transport to rmcp.

This spec does not re-decide design. Where this file and
`rust-mcp-rmcp-adoption` differ, the predecessor wins. This file is the
executable decomposition of that plan's "Follow-up cutover plan" section.

## Summary

Deliver the first two independently compiling stages of the MCP rmcp cutover:
add a disjoint `mcp-rmcp` feature with an exact pinned rmcp dependency, then run
`ProcessMcpTransport` tool listing and calls through rmcp over stdio. This
slice deliberately does not claim the server loop, rmcp-served wire parity, or
deletion gate. Those stages are owned by `rust-mcp-rmcp-server-cutover`.

The runx-specific surfaces (skill execution under MCP, argument templating,
sandbox metadata, the `runx:` host-result projection, receipt sealing) are not
touched.

## Runner note: 2026-05-21T10:58:27Z

Stage 1 and Stage 2 are represented by a compile-gated rmcp client path:
`mcp-rmcp` is a disjoint feature that enables the exact pinned `rmcp = "=1.7.0"`
dependency via `async-http`; `mcp` plus `mcp-rmcp` fails with the intentional
mutual-exclusion compile error. `ProcessMcpTransport::list_tools` and
`ProcessMcpTransport::call_tool` use rmcp behind `mcp-rmcp`, while
`FixtureMcpTransport`, templates, sandbox metadata, and receipt projection stay
unchanged. The scoped dependency-policy exception remains package-bound to
`runx-runtime`; full removal of the rmcp ban is still reserved for Stage 5
after wire parity and the deletion gate.

Validation reached both sides of the staged client cutover:
`cargo check -p runx-runtime --features mcp-rmcp`, `cargo test -p
runx-runtime --features mcp-rmcp --test mcp_adapter`, `cargo test -p
runx-runtime --features mcp --test mcp_server
mcp_server_matches_recorded_stdio_wire_contract`, `cargo test -p runx-runtime
--features mcp --test mcp_adapter`, `cargo clippy -p runx-runtime
--all-targets --features mcp-rmcp -- -D warnings`, `cargo deny check bans`, and
`cargo deny check licenses` pass. `cargo check -p runx-runtime --features mcp,mcp-rmcp` fails
with the expected mutual-exclusion compile error.

## Runner note: 2026-05-21T12:12:00Z

The review gate correctly caught two client-slice regressions in addition to
the over-broad Stage 3-5 claims. The client transport now records
receive-side Content-Length, size-limit, and JSON parse failures in the rmcp
transport error state before returning stream end to rmcp's `Transport`
interface, so downstream service errors can preserve the stable transport
message. The tokio child-process path now pipes and bounded-drains stderr like
the legacy client path instead of sending it to `/dev/null`.

New unit coverage under `--features mcp-rmcp --lib rmcp_transport_tests`
proves missing `Content-Length`, oversized body, and malformed JSON are
recorded as transport errors rather than clean EOF.

## Context

CWD: `.` (run cargo from `crates/`).

Packages:
- `crates/runx-runtime` (the `mcp` adapter modules and features)

Current sources (hand-rolled, still owned by the server/deletion follow-up):
- `crates/runx-runtime/src/adapters/mcp/framing.rs`
- `crates/runx-runtime/src/adapters/mcp/jsonrpc.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs` (`ProcessMcpTransport`)
- `crates/runx-runtime/src/adapters/mcp/server.rs` (`serve_mcp_json_rpc`)

Current sources (runx-specific, must stay unchanged):
- `crates/runx-runtime/src/adapters/mcp/server_skill.rs`
- `crates/runx-runtime/src/adapters/mcp/templates.rs`
- `crates/runx-runtime/src/adapters/mcp/sandbox_metadata.rs`
- `crates/runx-runtime/src/adapters/mcp/adapter.rs` (`McpAdapter` trait impl)
- `crates/runx-runtime/src/adapters/mcp/transport.rs` (`FixtureMcpTransport`)

Files impacted:
- `crates/runx-runtime/Cargo.toml` (features, optional `rmcp` dep)
- `crates/Cargo.lock` (committed with the dependency review)
- `crates/deny.toml` (scope the `rmcp` exception during this staged client
  path; remove the ban after `rust-mcp-rmcp-server-cutover` deletes the
  hand-rolled protocol path)
- `crates/runx-runtime/src/lib.rs` and `src/adapters.rs` (feature exposure)
- `crates/runx-runtime/src/adapters/mcp.rs` (mutual-exclusion `compile_error!`)
- `crates/runx-runtime/src/adapters/mcp/{transport,jsonrpc}.rs` (client
  transport gating during Stage 2)

Baseline already in repo (reuse, do not rewrite):
- `fixtures/runtime/adapters/mcp/wire-contract/basic-lifecycle.{requests,responses}.jsonl`
- `fixtures/runtime/adapters/mcp/wire-contract/error-paths.{requests,responses}.jsonl`
- test `mcp_server_matches_recorded_stdio_wire_contract`
  (`cargo test -p runx-runtime --features mcp --test mcp_server`)

Invariants:
- `mcp` (hand-rolled) and `mcp-rmcp` (rmcp-backed) are disjoint features.
  Enabling both is a build-time `compile_error!`.
- The runx-specific surfaces listed above are not modified by any stage.
- Every stage compiles and tests independently. No big-bang rewrite.
- The hand-rolled server/framing layer is not deleted by this spec. It is
  deleted only after the follow-up server cutover passes rmcp-served wire
  parity and its deletion gate.
- `cargo deny check licenses` stays clean; the rmcp tree (tokio, schemars,
  JSON-Schema helpers) must remain Apache-2.0 / MIT.

## Objectives

- Adopt `rmcp` for the process client transport behind a disjoint feature, with
  an exact pinned version.
- Preserve the existing `mcp` feature's recorded stdio wire contract while the
  staged `mcp-rmcp` client path is introduced.
- Preserve stable client error semantics for malformed JSON, missing
  `Content-Length`, oversized responses, timeout, and stderr draining.

## Scope

In scope:
- Stage 1 from `rust-mcp-rmcp-adoption`: feature flag, exact rmcp pin,
  dependency-policy exception, and mutual-exclusion build guard.
- Stage 2 from `rust-mcp-rmcp-adoption`: rmcp-backed process client transport
  for tool listing and calls; `FixtureMcpTransport` remains unchanged.
- Client-side parity repair for receive errors and stderr draining.

Out of scope:
- Server transport migration, rmcp-served wire parity, deletion of hand-rolled
  protocol modules, removal of the `mcp-rmcp` staging feature, and removal of
  the `deny.toml` rmcp ban. Owned by `rust-mcp-rmcp-server-cutover`.
- rmcp HTTP transports (SSE / streamable HTTP). Stdio only; follow up if a
  consumer needs HTTP (predecessor Open Questions).
- Publishing a public reusable rmcp `ServerHandler` type
  (deferred to `runx-mcp-public-server-trait`).
- Any change to runx skill execution, templating, sandbox metadata, the
  `runx:` projection, or receipt sealing.

## Stages

Stages and their per-stage acceptance gates are defined in
`rust-mcp-rmcp-adoption` and not restated here. Execution order:

1. Pull `rmcp = "=1.7.0"` behind `mcp-rmcp = ["dep:rmcp", "async-http",
   "tokio/process", "tokio/io-util"]` (no `"mcp"` in the list) with a
   `compile_error!` if both features are set.
   **Done.**
2. Behind `#[cfg(feature = "mcp-rmcp")]`, swap `ProcessMcpTransport` tool
   listing and calls to the rmcp client. `FixtureMcpTransport` unchanged.
   **Done for the process client path.**
3. Deferred to `rust-mcp-rmcp-server-cutover`: behind `mcp-rmcp`, swap the
   `serve_mcp_json_rpc` stdio loop for the rmcp server, wrapping
   `McpServerState` in an rmcp `ServerHandler`.
4. Deferred to `rust-mcp-rmcp-server-cutover`: diff the rmcp server's framed
   output against the recorded `*.responses.jsonl` baseline, holding to the
   predecessor's enumerated must-match / may-differ envelope.
5. Deferred to `rust-mcp-rmcp-server-cutover`: delete hand-rolled protocol
   code, make rmcp the only `mcp` path, and remove the `rmcp` ban from
   `deny.toml`.

## Dependencies

- `rust-mcp-rmcp-adoption` (archived, completed; design + baseline source of
  truth).
- `rust-async-http-layer` (archived, landed; supplies the adapter-tier
  tokio/reqwest exception this spec consumes).

## Dependency pin

`rmcp = "=1.7.0"` (exact pin). Verified as the latest stable release on
crates.io on 2026-05-21 (`max_stable_version` 1.7.0). The predecessor was
written against a pre-1.0 `rmcp` and listed "pre-1.0 churn" as a risk; rmcp has
since reached a stable 1.x line, which removes that risk. Use
`default-features = false` with the `client` feature for the Stage 2 client
path, keep the base tokio surface bounded to `rt`, `net`, and `time`, and add
`process`/`io-util` only through `mcp-rmcp`. Keep `cargo deny check licenses`
clean. At Stage 1, run `cargo update -p rmcp`,
confirm the resolved version is still 1.7.0 (or bump the pin to the
then-current latest and re-review), and commit the `Cargo.lock` diff with the
dependency review.

## Validation

- [x] `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp`
  passes.
- [x] `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features mcp,mcp-rmcp`
  fails with the intentional mutual-exclusion `compile_error!`.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test mcp_adapter --features mcp -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test mcp_adapter --features mcp-rmcp -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp-rmcp --lib rmcp_transport_tests -- --nocapture`
  passes.
- [x] `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server -- --nocapture`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool,mcp -- -D warnings`
  passes.
- [x] `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features mcp-rmcp -- -D warnings`
  passes.
- [x] From `crates/`, `cargo deny check bans` and `cargo deny check licenses`
  pass.

## Follow-up

- `rust-mcp-rmcp-server-cutover` owns the remaining server loop, rmcp-served
  wire parity, deletion-gate signal, and `deny.toml` ban removal.

## References

- [`rust-mcp-rmcp-adoption`](../archive/2026-05/rust-mcp-rmcp-adoption.md)
  (design, replacement map, wire-diff envelope, deletion gate)
- [`crates/deny.toml`](../../crates/deny.toml) (the `rmcp` ban to remove)
- [`plans/rust-takeover.md`](../../../plans/rust-takeover.md) §9 step 7
  ("MCP is last")
- [`oss/docs/rust-kernel-architecture.md`](../../docs/rust-kernel-architecture.md)
  §13 ("MCP is the hardest port")
- rmcp upstream: `https://github.com/modelcontextprotocol/rust-sdk`

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode review of rust-mcp-rmcp-cutover. The narrowed scope (Stages 1-2: feature flag + pinned rmcp client transport) is consistent across spec Summary, Stages, Out of scope, and Follow-up. All prior blocking and non-blocking findings are now repaired in code: F4 (receive errors recorded via RmcpTransportErrorState, surfaced through rmcp_service_error) at transport.rs:308-330,488-501 with rmcp_transport_tests covering missing Content-Length, oversized body, and malformed JSON; F6 (stderr piped + bounded drain via tokio::spawn) at transport.rs:418-424,433-446; and the prior R1 residual (init-time errors not propagated) is now closed: rmcp_initialization_error at transport.rs:504-512 consults error_state.take() before falling back to the fixed message, and rmcp_initialize_surfaces_recorded_transport_error at transport.rs:850-882 asserts the recorded io::Error message is propagated through serve_rmcp_client. deny.toml keeps the rmcp/tokio wrapper exceptions scoped to runx-runtime per the staged plan; full removal is reserved for rust-mcp-rmcp-server-cutover. Cargo.toml pins rmcp = "=1.7.0" with default-features=false and features=["client"], and mcp/mcp-rmcp emit compile_error when both are enabled (mcp.rs:12-13) with belt-and-braces runtime guards in transport.rs. Workspace did not mutate during this read-only verify pass. No new blockers found; ambient drift in unrelated paths (payment_ledger, kernel parity fixtures, credentials schema, runtime-local tests) is outside task scope and not attributable to this slice.

Attack log:
- `crates/runx-runtime/src/adapters/mcp/transport.rs RmcpContentLengthTransport::receive`: Verify F4 fix: malformed JSON / missing Content-Length / oversized body are recorded in error_state and propagated via rmcp_service_error -> clean (rmcp_transport_tests at transport.rs:819-848 cover all three cases and assert the recorded transport message. rmcp_service_error (lines 488-501) reads error_state before falling through to the generic 'MCP server request failed.' branch.)
- `crates/runx-runtime/src/adapters/mcp/transport.rs spawn_tokio_mcp_server + drain_tokio_stderr`: Verify F6 fix: stderr piped and bounded-drained instead of Stdio::null -> clean (Lines 418-424 use Stdio::piped(); lines 433-446 implement a bounded drain mirroring the legacy thread::spawn drain_stderr at lines 688-701; both list_tools_with_rmcp_async (169) and call_tool_with_rmcp_async (197) invoke drain_tokio_stderr immediately after spawn.)
- `crates/runx-runtime/src/adapters/mcp/transport.rs serve_rmcp_client + rmcp_initialization_error`: Verify R1 fix: initialize-time transport errors recorded in error_state are now surfaced on McpTransportError -> clean (serve_rmcp_transport (lines 242-256) maps the rmcp ClientInitializeError through rmcp_initialization_error (lines 504-512), which calls error_state.take() before the fixed fallback message. New test rmcp_initialize_surfaces_recorded_transport_error (lines 850-882) drives the path against a duplex stream and asserts the recorded message is preserved via message_for_test.)
- `.scafld/specs/active/rust-mcp-rmcp-cutover.md (scope vs prior F1/F2/F3)`: Confirm Stages 3-5 remain handed off and the spec text is internally consistent -> clean (Summary (line 45-53), Stages (lines 168-187), Out of scope (lines 156-160), and Follow-up (lines 230-233) consistently hand off server-loop migration, wire-parity diff, deletion gate, and deny.toml ban removal to rust-mcp-rmcp-server-cutover.)
- `crates/runx-runtime/Cargo.toml + crates/runx-runtime/src/adapters/mcp.rs`: Verify mutual-exclusion compile_error fires when both mcp and mcp-rmcp are enabled, and that the rmcp dep is exact-pinned with bounded features -> clean (Cargo.toml:23 declares mcp-rmcp = ['dep:rmcp','async-http','tokio/process','tokio/io-util']; Cargo.toml:36 pins rmcp = '=1.7.0' with default-features=false and features=['client']. mcp.rs:12-13 emits compile_error! when both features are enabled. Belt-and-braces runtime errors at transport.rs:108-114 and 130-136 cover the same combination.)
- `crates/deny.toml`: Verify the rmcp/tokio wrapper exceptions are scoped to runx-runtime per the narrowed spec -> clean (deny.toml:19 lists rmcp wrapper exception scoped to runx-runtime with the reason explicitly tied to the staged cutover; line 22 lists tokio with rmcp included as an approved wrapper. Removal of the rmcp ban is reserved for rust-mcp-rmcp-server-cutover.)
- `crates/runx-runtime/src/adapters/mcp/transport.rs next_rmcp_framed_message size guard`: Trace the outer size check (buffer.len() > MAX_CLIENT_RESPONSE_BYTES) against parse_next_rmcp_framed_message body_end for content_length = MAX exactly -> clean (For content_length = 1 MiB, body_end ≈ 1048603. With 8 KiB pipe reads, buffer crosses body_end in one chunk and parse succeeds before the outer size check fires. The inner content_length > MAX check at lines 388-393 rejects oversized advertised bodies before any further reads.)
- `crates/runx-runtime/src/adapters/mcp/transport.rs RmcpTransportErrorState`: Check that std::sync::Mutex usage inside async paths does not span awaits -> clean (record() and take() at lines 274-286 perform a single lock/store or lock/take with no await held under the guard. Safe under tokio current_thread runtime.)
- `crates/runx-runtime/src/adapters/mcp/transport.rs drain_tokio_stderr lifecycle`: Check the spawned drain task can be starved or leak under the current_thread runtime built per call -> clean (Drain is bounded to MAX_CLIENT_RESPONSE_BYTES. terminate_tokio_child (lines 427-430) awaits child.wait() which gives the drain task scheduling opportunities and closes stderr; runtime is dropped only after block_on returns, cancelling any residual drain. Matches legacy bounded-drain semantics.)
- `workspace mutation guard`: Compare pre-review and post-review workspace; ensure read-only verify pass did not mutate task-scoped files or this spec -> clean (Only Read/Grep tools were invoked. No edits to transport.rs, types.rs, Cargo.toml, deny.toml, mcp.rs, or the spec under review.)
- `ambient drift attribution`: Separate task changes from unrelated workspace drift to avoid mis-attribution -> clean (Ambient drift list (payment_ledger removal, kernel parity fixtures, credentials schema, runtime-local tests, contracts test, executor-control-schema-contract, runtime-local-auth-security, check-rust-crate-graph) is outside the declared task scope (Cargo.toml, deny.toml, src/adapters/mcp/*). Not attributable to this slice.)

Findings:
- [critical/non-blocking] `F1-server-loop-not-migrated` Stage 3 server loop migration explicitly handed off to rust-mcp-rmcp-server-cutover.
  - Location: `.scafld/specs/active/rust-mcp-rmcp-cutover.md:156`
  - Evidence: Spec Out of scope (lines 156-160), Stages (lines 168-187), Summary (lines 45-53), and Follow-up (lines 230-233) consistently scope this slice to Stages 1-2 only.
  - Impact: None; the work is owned by the named follow-up spec.
  - Validation: Spec text confirms handoff; no transport.rs change touches serve_mcp_json_rpc.
- [critical/non-blocking] `F2-stage4-wire-parity-not-checked` Stage 4 rmcp-served wire-parity diff handed off to rust-mcp-rmcp-server-cutover.
  - Location: `.scafld/specs/active/rust-mcp-rmcp-cutover.md:45`
  - Evidence: Summary (lines 45-49) explicitly states this slice does not claim rmcp-served wire parity.
  - Impact: None; the follow-up spec owns the rmcp-served baseline diff.
  - Validation: Existing mcp_server_matches_recorded_stdio_wire_contract continues to guard the legacy server under the `mcp` feature.
- [critical/non-blocking] `F3-stage5-deletion-and-ban-not-removed` Deletion of hand-rolled framing/jsonrpc/server modules and removal of the deny.toml rmcp ban remain owned by rust-mcp-rmcp-server-cutover.
  - Location: `crates/deny.toml:19`
  - Evidence: deny.toml:19 keeps the rmcp wrapper exception scoped to runx-runtime with an explicit removal note tied to the follow-up. framing/jsonrpc/server modules retained while the `mcp` feature still exists.
  - Impact: None for this slice; deletion is gated on follow-up wire parity.
  - Validation: deny.toml exception retained per narrowed spec; full removal scheduled in follow-up.
- [high/non-blocking] `F4-rmcp-receive-swallows-errors` RmcpContentLengthTransport::receive records receive-side io::Error in RmcpTransportErrorState and rmcp_service_error re-surfaces it before falling through to the generic message.
  - Location: `crates/runx-runtime/src/adapters/mcp/transport.rs:308`
  - Evidence: transport.rs:323-332 records errors into self.error_state before returning None to rmcp. next_rmcp_framed_message (lines 341-365) and parse_next_rmcp_framed_message (lines 368-404) emit distinct io::Error variants for missing Content-Length, oversized body, and serde_json failure. rmcp_service_error (lines 488-501) takes the recorded message and constructs McpTransportError::failed(message). rmcp_transport_tests (lines 819-848) cover all three cases.
  - Impact: Transport error semantics now match the legacy client path for malformed JSON, missing Content-Length, and oversized responses.
  - Validation: cargo test -p runx-runtime --features mcp-rmcp --lib rmcp_transport_tests recorded as passing in the runner note.
- [medium/non-blocking] `F5-mcp-rmcp-still-uses-handrolled-framing` rmcp client transport still imports super::framing helpers; deletion of the framing module is owned by the follow-up server cutover.
  - Location: `crates/runx-runtime/src/adapters/mcp/transport.rs:32`
  - Evidence: transport.rs:31-32 imports find_header_end and content_length under both feature gates. The narrowed spec defers deletion of hand-rolled protocol modules to rust-mcp-rmcp-server-cutover (Out of scope, lines 157-160).
  - Impact: No behavioral regression; framing helpers stay until the legacy `mcp` feature is removed.
  - Validation: Accepted per narrowed scope; rust-mcp-rmcp-server-cutover owns removal.
- [medium/non-blocking] `F6-stderr-dropped-under-mcp-rmcp` spawn_tokio_mcp_server now uses Stdio::piped() and drain_tokio_stderr bounded-drains stderr up to MAX_CLIENT_RESPONSE_BYTES, matching the legacy spawn_mcp_server semantics.
  - Location: `crates/runx-runtime/src/adapters/mcp/transport.rs:408`
  - Evidence: transport.rs:418-424 pipes stderr (Stdio::piped()). list_tools_with_rmcp_async (line 169) and call_tool_with_rmcp_async (line 197) call drain_tokio_stderr immediately after spawn. drain_tokio_stderr (lines 433-446) spawns a tokio task that reads up to MAX_CLIENT_RESPONSE_BYTES, mirroring drain_stderr (lines 688-701) used by the legacy client.
  - Impact: Servers that log heavily to stderr no longer risk blocking on a full stderr pipe; bounded drain envelope matches legacy.
  - Validation: mcp_adapter tests pass under --features mcp-rmcp per the runner note.
- [low/non-blocking] `R1-rmcp-init-error-state-not-surfaced` serve_rmcp_client now propagates initialize-time transport errors via rmcp_initialization_error, which consults RmcpTransportErrorState before falling back to the fixed initialization message.
  - Location: `crates/runx-runtime/src/adapters/mcp/transport.rs:504`
  - Evidence: transport.rs:242-256 wraps rmcp::serve_client behind serve_rmcp_transport, mapping ClientInitializeError through rmcp_initialization_error (lines 504-512), which calls error_state.take() and falls back to 'MCP client initialization failed.' only when no transport error was recorded. New test rmcp_initialize_surfaces_recorded_transport_error (transport.rs:850-882) constructs a duplex stream with a malformed initialize body and asserts the McpTransportError message contains the recorded serde_json error text via message_for_test (types.rs:190-194).
  - Impact: Internal observability gap from the prior verify pass is closed; initialize-time transport diagnostics are now preserved on the internal McpTransportError.message before sanitization.
  - Validation: rmcp_transport_tests now includes rmcp_initialize_surfaces_recorded_transport_error covering the initialize handshake error path.

