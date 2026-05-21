---
spec_version: '2.0'
task_id: rust-dev
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T14:24:18Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# Rust dev

## Current State

Status: draft
Current phase: safe closure candidate for the already-landed narrow dev
execution and presentation slice
Next: leave this lifecycle draft in place until an operator either archives it
as completed evidence or opens a separate watch-loop feature spec
Reason: the prior implementation evidence and current code inspection show the
narrow Rust runtime slice exists for dev fixture discovery, deterministic tool
fixture execution, executable fixture workspace files, polling watch debounce,
presentation, and dev-mode receipt metadata tagging.
`target.kind: skill` and `target.kind: graph` fixtures now execute through the
Rust harness replay path and validate against the dev fixture expectation
engine. Repo-integration skill fixtures bind workspace cwd through `RUNX_CWD`
instead of process-global cwd mutation. The Rust CLI dev JSON path now
pretty-prints like the TS CLI, and the native dev terminal presentation uses
the same no-color status glyphs as the TS presentation. The CLI watch decision
is to keep `runx dev --watch` fail-closed in Rust until a separate watch-loop
feature spec exists: TS parses `devWatch` in `packages/cli/src/args.ts` but
does not pass or use it in `packages/cli/src/commands/dev.ts`, and TS help does
not advertise `--watch`. Exposing a Rust loop now would be a new user-visible
feature with unspecified terminal, JSON, cancellation, and exit-code behavior.
This draft should not be treated as complete `runx dev` parity.
Blockers: no code blocker for the narrow done slice. Lifecycle closure remains
blocked by missing durable `rust-dev` session/review evidence: `scafld status
rust-dev --json` reports `status: draft`, `gate: harden`, and
`session_ok: false`, and there is no `.scafld/runs/rust-dev/session.json`.
The product blocker for full parity is the intentionally deferred long-running
CLI watch loop.
Allowed follow-up command: `scafld validate rust-dev --json`
Latest runner update: 2026-05-21T14:24:18Z
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

- None for the already-landed narrow execution and presentation slice.

## Future Blocker: CLI Watch Loop

`runx dev --watch` remains intentionally fail-closed in Rust. Current launcher
tests assert `runx dev --watch` returns `unknown dev flag --watch`, matching the
decision that Rust should not expose a long-running loop until a separate
feature spec defines:

- terminal rendering across repeated runs;
- JSON output shape for multi-run streams;
- cancellation and signal behavior;
- debounce semantics for real file-system watching;
- exit-code behavior after one or more failed runs.

This blocker is out of scope for the current closure slice and should be tracked
by a new spec if product behavior changes.

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
  passed with 7 tests, including deterministic native skill/graph fixtures and
  repo-integration workspace cwd binding.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --tests`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --all-targets`:
  passed.
- `git diff --check`: passed.
- `cargo fmt --manifest-path crates/Cargo.toml --package runx-cli --package runx-runtime`:
  passed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 5 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli dev_json_stdout_is_pretty_printed_like_ts_cli -- --nocapture`:
  passed with the focused CLI dev JSON unit test.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test dev -- --nocapture`:
  passed with 7 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli dev_ -- --nocapture`:
  passed with the dev JSON unit test plus the existing dev launcher routing
  tests.
- `cargo fmt --manifest-path crates/Cargo.toml --all -- --check`: passed.
- `git diff --check -- crates/runx-cli/src/dev.rs crates/runx-runtime/src/dev/presentation.rs crates/runx-runtime/tests/dev.rs .scafld/specs/drafts/rust-dev.md`:
  passed.
- Earlier broad filtered check `cargo test -p runx-runtime dev -- --nocapture`
  passed the new 5 dev tests and filtered the rest; initial invocation from repo
  root failed because the Cargo workspace lives under `crates/`.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 6 tests after adding executable workspace-file coverage.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test dev -- --nocapture`:
  passed with 8 tests after adding executable workspace-file coverage.
- `cargo fmt --manifest-path crates/Cargo.toml --package runx-runtime`:
  passed.
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime`: passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets`:
  passed.
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --all-targets`:
  passed.
- `git diff --check -- crates/runx-runtime/src/dev/loop.rs crates/runx-runtime/tests/dev.rs fixtures/dev/simple/tools/acme/executable/manifest.json fixtures/dev/simple/tools/acme/executable/fixtures/executable.yaml .scafld/specs/drafts/rust-dev.md`:
  passed.
- `scafld validate rust-dev`: passed.
- `scafld status rust-dev`: reported draft status with next step still the CLI
  watch decision.
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli dev_ -- --nocapture`:
  passed with the dev JSON unit test plus dev launcher routing tests, including
  `runx dev --watch` failing closed.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  blocked before dev tests by unrelated compile drift in
  `crates/runx-contracts/src/post_merge_observer.rs`; the error was unresolved
  import `plan::normalize_post_merge_observer_command`, and that file is
  outside this slice's ownership.
- `git diff --check -- .scafld/specs/drafts/rust-dev.md crates/runx-cli/src/launcher.rs crates/runx-cli/tests/launcher.rs crates/runx-runtime/src/dev`:
  passed.
- `scafld validate rust-dev`: passed.
- `scafld status rust-dev`: reported draft status with the next step to rerun
  focused runtime and CLI dev validation.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`:
  passed with 6 tests after the unrelated post-merge observer compile drift was
  resolved.
- `scafld validate rust-dev`: passed.
- `git diff --check`: passed.

Issues:
- Runtime slice implemented under `crates/runx-runtime/src/dev/**` with
  deterministic tool fixture execution only.
- Rust dev fixture workspace materialization now applies executable permissions
  to `workspace.executable_files`, matching the existing TS fixture-workspace
  contract.
- Deterministic native skill/graph dev fixture execution is implemented through
  the Rust harness replay path with stable fixture output projection.
- Native skill/graph repo-integration fixtures bind workspace cwd through
  `RUNX_CWD` without process-global cwd mutation.
- CLI dev routing keeps `--watch` fail-closed. Decision evidence: TS parses
  `devWatch` but `handleDevCommand` does not receive or use it, TS help does
  not advertise `--watch`, and the Rust launcher test already asserts
  `runx dev --watch` returns `unknown dev flag --watch`.
- CLI dev JSON and no-color terminal presentation parity tightened in the Rust
  CLI/runtime.
- Focused runtime validation passed after unrelated post-merge observer compile
  drift was resolved outside this slice's ownership.
