---
spec_version: '2.0'
task_id: rust-dev
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T00:45:45Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# Rust dev

## Current State

Status: draft
Current phase: deterministic native skill/graph fixture execution implemented
Next: wire repo-integration cwd semantics for native skill/graph fixtures and
CLI watch/presentation cutover
Reason: a narrow Rust runtime slice now exists for dev fixture discovery,
deterministic tool fixture execution, polling watch debounce, presentation, and
dev-mode receipt metadata tagging. Deterministic `target.kind: skill` and
`target.kind: graph` fixtures now execute through the Rust harness replay path
and validate against the dev fixture expectation engine. This is not complete
`runx dev` parity yet.
Blockers: repo-integration skill/graph fixtures are intentionally rejected until
the Rust sandbox/runtime can bind workspace cwd without process-global cwd
mutation; the Rust CLI dev command is owned by the CLI cutover worker; the TS
command currently parses `--watch` but does not run a watch loop, so CLI-level
watch parity still needs an owning cutover decision.
Allowed follow-up command: implement native repo-integration cwd plumbing for
skill/graph fixtures, then rerun runtime dev validation; do not mark passed
until the remaining blockers are closed.
Latest runner update: 2026-05-21T00:45:45Z
Review gate: not_started

## Summary

Port `runx dev` to Rust. Dev mode runs a skill or chain in an iterative
loop with file watch, fast-feedback receipts, and harness wiring. Today
this lives in `packages/cli/src/commands/dev/` and consumes runner-local
plus harness primitives.

## Context

CWD: `.`

Packages:
- `@runxhq/cli` (dev command tree)
- `@runxhq/runtime-local` (runner-local, harness)
- `crates/runx-runtime`

Current TypeScript sources:
- `packages/cli/src/commands/dev/**`
- `packages/cli/src/commands/dev.ts`
- `packages/runtime-local/src/harness/runner.ts`

Files impacted:
- `crates/runx-runtime/src/dev/watch.rs`
- `crates/runx-runtime/src/dev/loop.rs`
- `crates/runx-runtime/src/dev/presentation.rs`
- `fixtures/dev/**`

Invariants:
- File watch debounce and ignore patterns match TS.
- Dev mode never silently consumes secrets; reuses connect grants.
- Receipts emitted in dev are clearly tagged as dev-mode in metadata.

## Objectives

- Port dev mode loop with file watch.
- Match presentation (terminal output) to TS via snapshot tests.

## Scope

In scope:
- Dev loop, file watch, presentation.

Out of scope:
- New dev features beyond TS.

## Dependencies

- `rust-runtime-skeleton` (archived completed; review gate pass).
- `rust-harness` (archived completed; harden passed and review gate pass).

## Open Questions

- File watch library choice (notify, watchexec). Defer to Phase 1.

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-20T10:34:14Z
Ended: none

Checks:
- `cargo fmt --package runx-runtime` from `crates`: passed.
- `cargo test -p runx-runtime --test dev -- --nocapture` from `crates`: passed
  with 5 tests.
- `cargo check -p runx-runtime` from `crates`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --package runx-runtime`: passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests.
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests in the default feature set.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test dev -- --nocapture`:
  passed with 6 tests, including deterministic native skill and graph fixtures.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --tests`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --all-targets`:
  passed.
- `git diff --check`: passed.
- Earlier broad filtered check `cargo test -p runx-runtime dev -- --nocapture`
  passed the new 5 dev tests and filtered the rest; initial invocation from repo
  root failed because the Cargo workspace lives under `crates/`.

Issues:
- Runtime slice implemented under `crates/runx-runtime/src/dev/**` with
  deterministic tool fixture execution only.
- Deterministic native skill/graph dev fixture execution is implemented through
  the Rust harness replay path with stable fixture output projection.
- Native skill/graph repo-integration fixtures remain explicit failure metadata
  until sandbox cwd semantics are wired without process-global cwd mutation.
- CLI dev routing and release presentation cutover intentionally untouched.
