---
spec_version: '2.0'
task_id: rust-receipt-tree-resolution
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Rust receipt tree resolution

## Current State

Status: draft
Current phase: none
Next: harden
Reason: parent receipts already reference child receipts, but robust tree
verification needs an explicit resolver contract, bounded traversal, cycle
detection, and durable fixture coverage before receipt proof results can be
trusted across multi-step runs.
Blockers: `rust-receipts-parity`; resolver contract must stay compatible with
`rust-receipt-proof-verification`.
Allowed follow-up command: `scafld harden rust-receipt-tree-resolution`
Latest runner update: none
Review gate: not_started

## Summary

Make receipt tree verification first-class in Rust. The verifier should resolve
child receipt references through a typed resolver, reject ambiguous references,
detect duplicate children, cycles, missing children, orphan children, wrong
parent links, and traversal-limit breaches, and return reviewer-safe findings.

This spec owns tree semantics only. Cryptographic proof checks belong to
`rust-receipt-proof-verification`; disk/path discovery belongs to
`rust-runtime-receipt-path-discovery`.

## Context

CWD: `.` (runx OSS workspace)

Relevant code and fixtures:
- `crates/runx-receipts/src/tree.rs`
- `crates/runx-receipts/src/verify/**`
- `crates/runx-receipts/tests/harness_receipts.rs`
- `crates/runx-runtime/src/receipts.rs`
- `fixtures/contracts/harness-spine/**`
- `fixtures/runtime/**`

Invariants:
- Receipt references are typed. Suffix matching is not acceptable for governed
  receipts.
- The tree verifier must be deterministic, bounded, and safe against hostile
  receipt graphs.
- A parent receipt cannot claim successful child execution if a child receipt is
  absent, duplicated, cyclic, orphaned, or linked to a different parent.
- Public findings never include operator-local absolute paths.

## Objectives

- Define the `ReceiptResolver` boundary used by verifiers and runtime callers.
- Reject ambiguous or malformed child receipt references.
- Detect duplicate child ids, missing child ids, cycles, orphan child receipts,
  parent/child mismatch, depth limit, and breadth limit failures.
- Provide positive and negative tree fixtures.
- Keep the verifier pure and IO-free; local filesystem resolution is injected by
  runtime code.

## Scope

In scope:
- `runx-receipts` resolver trait or equivalent callback.
- Tree verification findings and summary aggregation.
- Fixture-backed tests for normal, fanout, nested, duplicate, missing, cycle,
  orphan, and wrong-parent cases.
- Runtime adapter glue only where needed to call the resolver boundary.

Out of scope:
- Signature/seal proof verification.
- Receipt directory discovery, manifest IO, or cloud storage.
- Slack/GitHub/Nitrosend presentation details.

## Dependencies

- `rust-receipts-parity`.
- Coordinates with `rust-receipt-proof-verification`.
- Feeds `rust-runtime-skill-execution`, `rust-nitrosend-dogfood`, and
  `rust-ts-sunset-receipts`.

## Assumptions

- Child references use a stable runx URI or exact receipt id; partial id and
  suffix lookup remain forbidden.
- Runtime may maintain a local receipt index, but the receipt crate receives an
  already-scoped resolver.

## Touchpoints

- `runx-receipts` tree verifier and finding codes.
- Runtime receipt store interface.
- Contract fixtures for graph and harness receipts.
- CLI and Aster receipt summaries.

## Risks

- Unbounded traversal can make receipt verification a denial-of-service vector.
- Ambiguous child lookup can verify the wrong receipt.
- Overlapping tree and proof concerns can create duplicated verifier logic.

## Acceptance

Profile: strict

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime`
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts --all-targets --all-features -- -D warnings`
- `git diff --check`

Required behavior:
- [ ] Exact child id or typed runx URI resolves; suffix-only references fail.
- [ ] Duplicate child receipt ids produce a blocker finding.
- [ ] Missing child references produce a blocker finding.
- [ ] Cyclic child references produce a blocker finding without recursion panic.
- [ ] Orphan child receipts in a supplied set produce a finding.
- [ ] Child receipt with mismatched parent id produces a blocker finding.
- [ ] Configured depth and breadth limits are enforced.
- [ ] Positive nested/fanout fixture verifies cleanly.
- [ ] Tree verification summary is deterministic across repeated runs.

## Phase 1: Resolver Contract

Status: pending
Dependencies: none

Objective: Define the pure resolver boundary.

Changes:
- Add resolver trait/callback API.
- Define child reference normalization rules.
- Define traversal bounds and finding codes.

Acceptance:
- [ ] Resolver tests cover exact id, typed URI, malformed URI, and ambiguous id.

## Phase 2: Tree Verifier

Status: pending
Dependencies: Phase 1

Objective: Verify complete receipt trees.

Changes:
- Implement bounded traversal.
- Track reached, visiting, duplicate, orphan, and parent-link state.
- Aggregate child findings into parent summary honestly.

Acceptance:
- [ ] Negative fixtures fail with stable finding codes.
- [ ] Positive fixtures pass without warnings.

## Phase 3: Runtime Wiring

Status: pending
Dependencies: Phase 2

Objective: Let runtime callers use tree verification without putting IO in the
receipt crate.

Changes:
- Wire runtime receipt indexes into the resolver contract.
- Keep path discovery in `rust-runtime-receipt-path-discovery`.

Acceptance:
- [ ] Runtime integration test verifies a graph receipt with children.

## Rollback

- Keep the existing structural tree checks until bounded resolver tests pass,
  then remove redundant code in the same reviewed change.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means multi-step receipt verification cannot be
  fooled by missing, ambiguous, or hostile child receipts.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: isolate receipt tree trust semantics from proof and IO work

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into tree-resolution contract after receipt
  verifier review.
