---
spec_version: '2.0'
task_id: rust-runtime-receipt-path-discovery
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# Rust runtime receipt path discovery

## Current State

Status: draft
Current phase: none
Next: harden
Reason: receipt verification should be pure, but the runtime still needs a
stable way to discover, write, index, and read receipt stores for graph runs,
issue-to-PR runs, and Aster-managed runners.
Blockers: `rust-runtime-skeleton`; coordinates with receipt tree/proof specs.
Allowed follow-up command: `scafld harden rust-runtime-receipt-path-discovery`
Latest runner update: none
Review gate: not_started

## Summary

Add runtime-owned receipt path discovery and local receipt store handling. This
keeps filesystem concerns out of `runx-receipts` while giving the Rust runtime,
CLI, and Aster a deterministic contract for where receipts live, how indexes are
loaded, how writes are made atomic, and how public projections avoid leaking
machine-local paths.

## Context

CWD: `.` (runx OSS workspace)

Relevant code and fixtures:
- `crates/runx-runtime/src/**`
- `crates/runx-receipts/src/**`
- `fixtures/runtime/**`
- CLI wiring that passes receipt store paths into runtime runs

Expected discovery inputs:
- Explicit CLI option.
- Policy/runtime config value.
- Environment override for CI and local dogfood.
- Workspace default under the run state directory.

Invariants:
- The receipt crate remains IO-free.
- Runtime may canonicalize and use local paths internally, but public GitHub,
  Slack, Aster, and reviewer outputs use relative/safe labels.
- Receipt writes are atomic enough that a failed run does not leave a successful
  looking receipt index.
- Unknown or unreadable receipt stores fail closed for governed verification.

## Objectives

- Define precedence for receipt store path discovery.
- Add a runtime local receipt store interface for read, write, list, and index
  operations.
- Support manifest/index loading for receipt tree verification.
- Redact or relativize path details in all public projections.
- Add tests for env/config/default precedence, atomic writes, malformed index,
  and missing store errors.

## Scope

In scope:
- Runtime receipt store path resolution.
- Local filesystem receipt store implementation.
- Public projection redaction/relativization for receipt paths.
- Integration tests using `fixtures/runtime/**`.

Out of scope:
- Receipt proof semantics.
- Receipt tree traversal semantics.
- Cloud receipts store implementation.
- Nitrosend Slack/GitHub copy.

## Dependencies

- `rust-runtime-skeleton`.
- `rust-receipt-tree-resolution` for final tree-verification integration.
- `rust-receipt-proof-verification` for final proof-verification integration.

## Assumptions

- CI and dogfood runners can provide explicit receipt paths when needed.
- Runtime defaults may live under scafld/run state, but the exact root is a
  runtime concern and must be documented.

## Touchpoints

- `runx-runtime` receipt store modules.
- CLI options/env parsing.
- Runtime summary and receipt projections.
- Aster runner surfaces that need receipt links or summaries.

## Risks

- Leaking absolute paths in GitHub or Slack comments can expose operator machine
  structure.
- Inconsistent precedence between CLI and Aster can make production behavior
  hard to debug.
- Non-atomic writes can make observers consume partial receipt data.

## Acceptance

Profile: standard

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `git diff --check`

Required behavior:
- [ ] CLI option wins over policy config.
- [ ] Policy config wins over environment default.
- [ ] Environment override wins over workspace default.
- [ ] Missing governed receipt store fails closed with actionable error.
- [ ] Malformed receipt index fails with a typed error.
- [ ] Receipt writes are atomic or temp-file-backed.
- [ ] Public run summaries never include absolute local filesystem paths.
- [ ] Runtime can feed discovered receipt children into the tree resolver.

## Phase 1: Discovery Contract

Status: pending
Dependencies: none

Objective: Define how a run discovers its receipt store.

Changes:
- Add precedence docs and tests.
- Define config/env/CLI names.

Acceptance:
- [ ] Discovery unit tests cover all precedence cases.

## Phase 2: Local Store

Status: pending
Dependencies: Phase 1

Objective: Implement local receipt store IO in runtime.

Changes:
- Add store read/write/list/index APIs.
- Use atomic writes for receipts and indexes.

Acceptance:
- [ ] Runtime integration test writes and reloads receipts deterministically.

## Phase 3: Safe Projection

Status: pending
Dependencies: Phase 2

Objective: Ensure public outputs are reviewer-safe.

Changes:
- Redact absolute paths in summary/projection code.
- Add regression fixture with an absolute path input.

Acceptance:
- [ ] Test proves public projection contains no absolute local path.

## Rollback

- Keep existing runtime summary behavior until the local receipt store tests are
  green, then remove duplicated path handling in the same reviewed change.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means receipt storage is predictable for operators
  without contaminating proof code or public comments.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: isolate runtime IO from receipt proof/tree verification

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into runtime receipt store discovery
  contract.
