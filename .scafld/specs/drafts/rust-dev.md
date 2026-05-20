---
spec_version: '2.0'
task_id: rust-dev
created: '2026-05-18T00:00:00Z'
updated: '2026-05-20T10:34:14Z'
status: draft
harden_status: in_progress
size: medium
risk_level: medium
---

# Rust dev

## Current State

Status: draft
Current phase: ready_for_harden
Next: harden
Reason: hardening round in progress
Blockers: none
Allowed follow-up command: `scafld harden rust-dev --mark-passed`
Latest runner update: 2026-05-20T10:33:54Z
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
- none

Issues:
- none
