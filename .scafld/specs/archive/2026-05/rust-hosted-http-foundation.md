---
spec_version: '2.0'
task_id: rust-hosted-http-foundation
created: '2026-05-19T09:37:36Z'
updated: '2026-05-19T09:43:06Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# Rust hosted HTTP foundation

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T09:43:06Z
Review gate: pass

## Summary

Add a tiny internal hosted HTTP crate for Rust clients. The crate owns only
shared request/response/transport primitives, base URL normalization, reqwest
blocking transport, and non-leaky header display/debug behavior. It does not
own registry parsing, connect grant/session models, OAuth polling, retries, or
receipt/admission behavior.

This keeps `rust-connect-client` from putting HTTP plumbing in `runx-cli` or
duplicating the registry client transport. The first consumer is
`runx-registry-client`, migrated without changing its public registry behavior.

## Context

CWD: `.`

Read-only references:
- `crates/runx-registry-client/src/http.rs`
- `crates/runx-registry-client/tests/client.rs`
- `crates/runx-sdk/src/client.rs`
- `.scafld/specs/drafts/rust-connect-client.md`

Files impacted:
- `crates/Cargo.toml`
- `crates/runx-hosted-http/Cargo.toml`
- `crates/runx-hosted-http/src/lib.rs`
- `crates/runx-registry-client/Cargo.toml`
- `crates/runx-registry-client/src/http.rs`
- `crates/runx-registry-client/src/lib.rs`
- `crates/runx-registry-client/tests/client.rs`

Do not modify:
- `crates/runx-cli/**`
- `crates/runx-runtime/src/connect/**`
- `fixtures/connect/**`
- TypeScript source files

## Invariants

- This crate is transport plumbing, not a cloud SDK.
- No domain model is added for connect, grants, sessions, registry skills, or
  receipts.
- Request headers are representable for connect, but debug/error output must
  not leak values. `authorization`, token-bearing headers, and future sensitive
  headers must display as `[redacted]`.
- Supported method enum includes `GET`, `POST`, and `DELETE`.
- The shared reqwest transport sends headers and JSON bodies exactly as given.
- Existing registry-client behavior and tests keep passing after migration.
- The SDK remains CLI-backed for connect v0; this spec must not add native SDK
  hosted HTTP behavior.

## Objectives

- Add `runx-hosted-http` with `HttpMethod`, `HostedHttpHeader`,
  `HostedHttpRequest`, `HostedHttpResponse`, `HostedTransport`,
  `ReqwestBlockingTransport`, `HostedHttpClient`, and `HostedHttpError`.
- Preserve a small mockable transport seam for registry and future connect
  tests.
- Redact header values in debug output.
- Migrate `runx-registry-client` to the shared request/response/transport
  primitives without changing registry API behavior.

## Scope

In scope:
- New internal crate and registry-client migration.
- Unit tests for base URL normalization, request routing, method/header/body
  transport shape, and header redaction.

Out of scope:
- Connect hosted HTTP implementation.
- OAuth opener/polling.
- Runtime admission integration.
- Retry/backoff policy.
- Async HTTP.
- Broader cloud API client.

## Dependencies

- `rust-registry-client`
- `rust-connect-cli-foundation`

## Sequencing

1. Add workspace crate and dependencies.
2. Implement hosted HTTP primitives and tests.
3. Migrate registry-client to use the shared transport types.
4. Run registry and SDK connect-list projection tests to prove no public
   registry/SDK regression.

## Acceptance

Profile: strict

Validation:
- [ ] `cmd_hosted_http_tests` - Shared hosted HTTP tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-hosted-http`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_registry_client_tests` - Registry client still passes on shared
  transport.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-registry-client`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_sdk_connect_projection` - SDK remains CLI-backed and connect-list
  projection still passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-sdk connect_list_reads_grant_projection`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_fmt` - Rust formatting passes.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_clippy` - New crate and migrated registry client lint cleanly.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-hosted-http -p runx-registry-client --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`

Definition of done:
- [ ] `dod1` `runx-hosted-http` exposes shared transport primitives with
  redacted header debug output.
- [ ] `dod2` Registry client uses the shared transport primitives and keeps its
  previous public behavior.
- [ ] `dod3` No connect domain behavior is implemented in this slice.
- [ ] `dod4` Downstream `rust-connect-client` can depend on the shared crate
  for bearer-header and DELETE-capable mockable transport.

## Rollback

- Remove `crates/runx-hosted-http`, remove it from the workspace, and restore
  registry-client's local request/response/transport definitions.

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: Implemented a transport-only runx-hosted-http crate with GET/POST/DELETE methods, request/response/header types, redacted header/body debug output, reqwest blocking transport, and base URL/route helpers. Migrated runx-registry-client to the shared transport without changing registry behavior. Validation passed: cargo test -p runx-hosted-http, cargo test -p runx-registry-client, cargo test -p runx-sdk connect_list_reads_grant_projection, cargo fmt --all --check, and cargo clippy -p runx-hosted-http -p runx-registry-client --all-targets -D warnings.

Attack log:
- `review gate`: manual human audit -> clean (Implemented a transport-only runx-hosted-http crate with GET/POST/DELETE methods, request/response/header types, redacted header/body debug output, reqwest blocking transport, and base URL/route helpers. Migrated runx-registry-client to the shared transport without changing registry behavior. Validation passed: cargo test -p runx-hosted-http, cargo test -p runx-registry-client, cargo test -p runx-sdk connect_list_reads_grant_projection, cargo fmt --all --check, and cargo clippy -p runx-hosted-http -p runx-registry-client --all-targets -D warnings.)

Findings:
- none

## Self Eval

- none

## Deviations

- Created as a prerequisite after connect CLI scoping showed the existing
  registry-local HTTP layer is too narrow for hosted connect.

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Scoped from Dalton/Volta findings. Keep as transport-only
  foundation; do not start connect behavior here.
