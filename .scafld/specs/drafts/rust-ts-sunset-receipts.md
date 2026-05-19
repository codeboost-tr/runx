---
spec_version: '2.0'
task_id: rust-ts-sunset-receipts
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T02:35:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# TS sunset: receipts

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created under `plans/rust-takeover.md`. Fifth TS sunset.
Blockers: `rust-ts-sunset-executor` complete, `runx-contract-spine-hard-cutover`
approved, `rust-receipts-parity` consumed by every receipt producer after it
targets post-cutover harness receipts, and the proof/tree/path receipt hardening
specs completed.
Allowed follow-up command: `scafld harden rust-ts-sunset-receipts`
Latest runner update: none
Review gate: not_started

## Summary

Delete `packages/core/src/receipts/`. Receipts are produced by the Rust
runtime as sealed harness receipts; verification runs from `runx-receipts`.
The TS implementation is no longer reached by any live caller, and the Rust
receipt stack has proof-strength verification rather than structural parity
only.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-receipts`
- `cloud/packages/receipts-store` (may still consume TS receipts types
  pre-cutover; verify in Phase 1)

Current TypeScript sources:
- `packages/core/src/receipts/**` (to be deleted)

Files impacted:
- `packages/core/src/receipts/` (deleted)
- `packages/core/src/index.ts`

Invariants:
- Existing pre-cutover receipts on disk are either migrated, archived, or read
  through an explicit offline archival verifier. Live governed paths use
  post-cutover harness receipts only.
- Cloud receipts-store has migrated to consume `runx-contracts` types or is
  on its own sunset path.
- TypeScript receipt deletion cannot proceed while any live caller depends on
  old receipt field names, old digest semantics, suffix child lookup, or
  repo-local receipt path discovery.
- The final fixture catalogue covers at least denied approval, approval
  round-trip, graph fanout, replay, child harness, verification form, abnormal
  seal, digest tamper, signature tamper, redaction mismatch, and external
  proof-missing cases.

## Objectives

- Enumerate importers; verify migration.
- Prove Rust receipt verification covers proof, tree, and runtime path
  discovery responsibilities through their dedicated specs.
- Preserve enough fixture history that TS deletion does not erase the oracle for
  critical security and reviewer-story behavior.
- Delete TS receipts implementation.

## Scope

In scope:
- TS receipts deletion.

Out of scope:
- Cloud receipts-store internal changes.

## Dependencies

- `rust-ts-sunset-executor`.
- `runx-contract-spine-hard-cutover`.
- `rust-receipts-parity` targeting post-cutover harness receipts.
- `rust-receipt-proof-verification`.
- `rust-receipt-tree-resolution`.
- `rust-runtime-receipt-path-discovery`.
- `rust-approval-gate-parity` for approval receipt round-trip evidence.

## Open Questions

- Whether cloud receipts-store gets its own Rust port before this sunset.
  If yes, that's an additional cloud spec; if no, the cloud-side keeps a
  contract-typed view via `runx-contracts`.
