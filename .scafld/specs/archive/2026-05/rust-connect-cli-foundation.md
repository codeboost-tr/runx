---
spec_version: '2.0'
task_id: rust-connect-cli-foundation
created: '2026-05-19T09:31:00Z'
updated: '2026-05-19T09:36:43Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# Rust connect CLI foundation

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-19T09:36:43Z
Review gate: pass

## Summary

Add the Rust CLI foundation for `runx connect` without implementing the hosted
HTTP client yet. The native binary owner is `crates/runx-cli`, not
`crates/runx-runtime`. This slice captures the TypeScript argument shape in a
typed Rust plan, wires a clearly gated launcher action for future client work,
and leaves default production `runx connect` behavior delegated to the current
TypeScript CLI until the approved hard cutover spec supplies the Rust hosted
client.

This is a hard-cutover prerequisite, not a compatibility layer: it does not add
alternate field names, legacy readers, local credential storage, or a TypeScript
fallback inside a Rust connect implementation. It only prevents the next spec
from building on the wrong crate boundary.

## Context

CWD: `.`

TypeScript references:
- `packages/cli/src/args.ts`
- `packages/cli/src/dispatch.ts`
- `packages/cli/src/commands/connect.ts`
- `packages/cli/src/connect-http.ts`

Rust references:
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/src/main.rs`
- `crates/runx-cli/tests/launcher.rs`
- `crates/runx-cli/tests/tool.rs`
- `crates/runx-runtime/src/config.rs`
- `crates/runx-registry-client/src/http.rs`

Files impacted:
- `crates/runx-cli/src/connect.rs`
- `crates/runx-cli/src/lib.rs`
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/src/main.rs`
- `crates/runx-cli/tests/connect.rs`
- `crates/runx-cli/tests/launcher.rs`
- `fixtures/connect/README.md` only if the CLI foundation needs a placeholder
  explaining that hosted HTTP fixtures belong to `rust-connect-client`

Do not modify in this spec:
- `packages/cli/src/**`
- `packages/core/src/**`
- `crates/runx-runtime/src/connect/**`
- `fixtures/connect/contracts/**`
- `fixtures/connect/oracles/**`

## Invariants

- `runx connect` is authority intake, not skill execution.
- Native command ownership is `runx-cli`; `runx-runtime` remains a library.
- Default `runx connect` behavior remains delegated until the hosted Rust
  client lands. This spec may add an explicit native-connect opt-in signal for
  routing tests, but that signal must fail closed with a stable "not available"
  diagnostic rather than silently invoking TypeScript.
- The final cutover spec must make `runx connect` a single canonical Rust path;
  no Rust-to-TypeScript connect fallback is allowed after that point.
- The Rust parser accepts the current TypeScript command surface only:
  `connect list`, `connect revoke <grant-id>`, and
  `connect <provider> [--scope ...] [--scope-family ...]
  [--authority-kind read_only|constructive|destructive] [--target-repo ...]
  [--target-locator ...] [--json]`.
- Flag spelling parity includes kebab, snake, and camel inputs that TS already
  normalizes for `scope_family`, `target_repo`, `target_locator`, and
  `authority_kind`.
- Unknown flags, missing values, invalid authority kinds, missing providers, and
  malformed revoke invocations fail before any network attempt.
- No connect token, authorization URL, OAuth code, device code, credential
  body, bearer header, or raw cloud response is printed by this foundation.

## Objectives

- Add typed Rust `ConnectPlan`, `ConnectAction`, and
  `ConnectAuthorityKind` models in `runx-cli`.
- Add parser coverage for the exact TS `runx connect` argument surface.
- Add a gated launcher route proving the native binary can own `connect`
  without changing default delegation.
- Update `runx --shim-help` to name the native-connect gate as preparatory, not
  as a shipped hosted client.
- Add tests that prove default `connect` still delegates and gated `connect`
  returns a typed native plan or stable not-available diagnostic.
- Amend the downstream `rust-connect-client` draft acceptance commands in this
  repo if needed so future execution targets `runx-cli`, not `runx-runtime`.

## Scope

In scope:
- Rust CLI parsing and launcher action.
- Stable not-available error path for a gated native connect action.
- Documentation comments or fixture placeholder explaining that hosted HTTP
  fixtures are owned by `rust-connect-client`.
- Downstream spec correction if current acceptance commands point at the wrong
  crate.

Out of scope:
- Hosted HTTP client implementation.
- Mock auth server.
- Browser opener and polling.
- Grant list/revoke/preprovision execution.
- Policy admission integration.
- Deleting TypeScript connect dispatch.
- Shared HTTP crate extraction. That may be designed in parallel, but this
  slice should not introduce a broad HTTP abstraction before the hosted client
  needs it.

## Dependencies

- `rust-runx-cli-placeholder`
- `rust-tool-catalogs`
- `rust-local-config`

## Sequencing

1. Add `runx_cli::connect` parser/types and export them.
2. Add launcher recognition for `connect` only when an explicit native connect
   opt-in is set.
3. Add main handling for the gated native action with a stable "not available
   until rust-connect-client" diagnostic.
4. Add launcher and connect parser tests.
5. Correct the `rust-connect-client` draft's CLI command target from
   `runx-runtime` to `runx-cli` if it still points at the library crate.

## Acceptance

Profile: strict

Validation:
- [ ] `cmd_cli_connect_tests` - Native connect parser and launcher tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli connect`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_cli_launcher_tests` - Existing launcher tests still pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli launcher`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_cli_all_tests` - Full Rust CLI suite passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_fmt` - Rust formatting passes.
  - Command: `cargo fmt --manifest-path crates/Cargo.toml --all --check`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_clippy` - Rust CLI linting passes.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-cli --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
- [ ] `cmd_connect_client_target` - Downstream connect draft no longer targets
  `runx-runtime` for CLI execution.
  - Command: `! rg -n "cargo run --manifest-path crates/Cargo.toml -p runx-runtime -- connect" .scafld/specs/drafts/rust-connect-client.md`
  - Expected kind: `exit_code_zero`

Definition of done:
- [ ] `dod1` Default `runx connect ...` launcher planning still delegates to
  the current JavaScript CLI until the hosted Rust client cutover lands.
- [ ] `dod2` Explicit native-connect routing produces a typed `ConnectPlan`
  for valid list/revoke/preprovision inputs.
- [ ] `dod3` Invalid connect inputs fail closed with stable diagnostics before
  a network/client object can exist.
- [ ] `dod4` The native-connect gated main path does not call TypeScript and
  does not attempt network.
- [ ] `dod5` `rust-connect-client` is left buildable against `runx-cli` as the
  command binary owner.

## Rollback

- Remove `crates/runx-cli/src/connect.rs`, connect exports, launcher action,
  launcher/main routing, tests, and downstream draft command correction.
- Default production connect behavior returns to the pre-spec launcher
  delegation because this slice does not delete TypeScript dispatch.

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: Implemented the runx-cli connect foundation as a typed, gated native route without hosted HTTP execution. Default connect still delegates to JS; RUNX_RUST_CONNECT plans list/revoke/preprovision and fails closed until rust-connect-client lands. Corrected rust-connect-client draft CLI commands to target runx-cli. Validation passed: cargo test -p runx-cli connect, cargo test -p runx-cli launcher, cargo test -p runx-cli, cargo fmt --all --check, cargo clippy -p runx-cli --all-targets -D warnings, and no runx-runtime connect CLI command target remains in the downstream spec.

Attack log:
- `review gate`: manual human audit -> clean (Implemented the runx-cli connect foundation as a typed, gated native route without hosted HTTP execution. Default connect still delegates to JS; RUNX_RUST_CONNECT plans list/revoke/preprovision and fails closed until rust-connect-client lands. Corrected rust-connect-client draft CLI commands to target runx-cli. Validation passed: cargo test -p runx-cli connect, cargo test -p runx-cli launcher, cargo test -p runx-cli, cargo fmt --all --check, cargo clippy -p runx-cli --all-targets -D warnings, and no runx-runtime connect CLI command target remains in the downstream spec.)

Findings:
- none

## Self Eval

- none

## Deviations

- This spec was created after `rust-connect-client` because that draft was found
  to be pointed at the wrong Rust crate boundary.

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Split from `rust-connect-client` after review showed the native
  CLI command owner is `runx-cli`, while the connect-client draft acceptance
  commands invoked `runx-runtime`, which is library-only.
