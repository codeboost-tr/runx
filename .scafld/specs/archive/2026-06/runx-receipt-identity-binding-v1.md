---
spec_version: '2.0'
task_id: runx-receipt-identity-binding-v1
created: '2026-06-11T09:01:56Z'
updated: '2026-06-11T10:22:53Z'
status: completed
harden_status: passed
size: medium
risk_level: medium
---

# Receipt display identity from signed fields only

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-11T10:22:53Z
Review gate: pass

## Summary

A sealed receipt's signed body already carries its identity: Rust exposes
`receipt.subject.reference`, the wire shape exposes `subject.ref`, and together
with `subject.kind` they name the skill; `issuer`/`authority`/`acts` carry the rest.
`metadata` (skill name, source type, actor labels) is a runtime-local read aid
that the canonicalizer strips from the signed body. Yet display surfaces render
identity from `metadata.skill_name`, so a sealed receipt's displayed skill/actor
identity can be rewritten without breaking the signature (gap 23,
frantic-architecture.md §15.1).

This spec makes displayed identity derive only from signed fields and treats
`metadata` as non-authoritative: it is never the source of a trust-bearing label.
There is no receipt-format change and no new contract id (the signed body already
holds the identity), so the no-`.v2` boundary rule is preserved. The fix is a
signed-identity accessor plus a conformance test in OSS, and a display-sourcing
correction in the cloud projection and the `/r` page.

## Objectives

- A documented, enforced invariant: receipt `metadata` is non-authoritative, and
  no trust-bearing identity label is sourced from it.
- A signed-identity accessor in OSS that derives a display identity (skill name
  from Rust `receipt.subject.reference` / wire `subject.ref` plus
  `subject.kind`, actor/issuer labels from signed fields) so every consumer has
  one correct source.
- A conformance test proving a receipt's displayed identity is fully
  reconstructable from signed fields alone, and that rewriting `metadata` does not
  change the derived identity (complementing the existing canonical body-digest
  test).
- Cloud display surfaces (history projection and the `/r` page) read identity from
  the signed fields, not from `metadata.skill_name`.
- No new contract id, no change to the canonical signed body, no removal of
  `metadata` (it remains a cosmetic read aid).

## Scope

- OSS: a signed-identity accessor and conformance test in `runx-receipts`, and the
  `metadata` non-authoritative invariant documented in `runx-contracts`.
- Cloud: the receipt history projection and the `/r` page identity sourcing.

Out of scope:
- Changing the canonical signed body or adding signed fields (not needed; identity
  is already signed).
- Removing or repurposing `metadata`.
- The verification algorithm and trust root.

## Dependencies

- none

## Assumptions

- Rust `receipt.subject.reference` and wire `subject.ref`, with `subject.kind`,
  already carry the authoritative skill identity on every sealed receipt, so
  identity binding is a sourcing correction, not a body change.
- The cloud repo (`../cloud`) is a separate git repo with no `.scafld`; this
  kernel-side spec governs the OSS accessor/conformance and the sibling cloud
  display surfaces, with cloud acceptance run via `cd ../cloud`.
- A friendly display label can be derived deterministically from signed subject
  reference fields; the previous `metadata.skill_name` was a convenience, not new
  information.

## Touchpoints

- `crates/runx-contracts/src/receipt.rs`
- `crates/runx-receipts/src/canonical.rs`
- `crates/runx-receipts/tests/conformance.rs`
- `crates/runx-runtime/src/journal.rs`
- `cloud/packages/api/src/summaries.ts`
- `cloud/packages/worker/src/hosted-run-projection.ts`
- `cloud/packages/db/src/platform/run-summary.ts`
- `cloud/packages/api/src/receipt-notary-projection.ts`
- `cloud/apps/web/src/pages/r/[hash].astro`

## Risks

- **Display regression if `subject.ref` formats differently from the old
  `metadata.skill_name`.** Mitigation: the accessor produces a friendly label from
  `subject.ref`; snapshot/display tests pin the rendered identity.
- **Two repos in one change.** Mitigation: land the OSS accessor and conformance
  test first; the cloud surfaces then consume the signed source, gated by the
  conformance guarantee.
- **A consumer still reaching for `metadata` identity.** Mitigation: the
  conformance test and the documented invariant make metadata-sourced identity a
  reviewable violation, not a silent one.

## Acceptance

Profile: standard

Validation:
- `cargo fmt --all --manifest-path crates/Cargo.toml --check`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts -p runx-contracts --all-targets -- -D warnings`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo test --manifest-path crates/Cargo.toml -p runx-receipts --test integration conformance -- --nocapture`
- `cd ../cloud && pnpm vitest run packages/api/src/receipt-projection-feed.test.ts packages/db/src/receipt-projections.test.ts packages/api/src/run-control-service.test.ts packages/worker/src/hosted-run-projection.test.ts`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target pnpm verify:fast`

## Phase 1: Signed-identity accessor and conformance

Status: completed
Dependencies: none

Objective: one signed source of display identity, proven by conformance.

Changes:
- Add a signed-identity accessor that derives a display identity from signed fields (Rust `receipt.subject.reference`, wire `subject.ref`, `subject.kind`, issuer/authority/acts).
- Document in `receipt.rs` that `metadata` is non-authoritative and never the source of a trust-bearing identity label.
- Add a conformance test: a receipt's displayed identity is reconstructable from signed fields alone, and rewriting `metadata` does not change the derived identity.

Acceptance:
- [x] `ac1` command - Rust formatting
  - Command: `cargo fmt --all --manifest-path crates/Cargo.toml --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - Receipts/contracts clippy clean
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts -p runx-contracts --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac3` command - Identity-from-signed conformance passes
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo test --manifest-path crates/Cargo.toml -p runx-receipts conformance -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Cloud display sources signed identity

Status: completed
Dependencies: Phase 1

Objective: cloud surfaces render identity from signed fields, not metadata.

Changes:
- Change local history, hosted run projections/summaries, and the `/r` page to derive displayed skill/actor identity from signed fields, not from `metadata.skill_name` or metadata actor labels.
- Extend the notarization projection with signed subject URI/display data so `/r` can render signed skill identity rather than only subject kind/ref type.
- Add/extend tests pinning that a metadata rewrite does not change the rendered identity.

Acceptance:
- [x] `ac5` command - Cloud projection tests pass
  - Command: `cd ../cloud && pnpm vitest run packages/api/src/receipt-projection-feed.test.ts packages/db/src/receipt-projections.test.ts packages/api/src/run-control-service.test.ts packages/worker/src/hosted-run-projection.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18
- [x] `ac6` command - OSS fast verification
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19

## Rollback

- Revert the cloud display sourcing first, then the OSS accessor/conformance.
  Nothing touches the signed body, so there is no receipt migration.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify pass: all five prior findings are repaired. (1) `crates/runx-runtime/src/journal.rs:352` now derives `name`, `harness_id`, `source_type`, and `actors` from `signed_display_identity(receipt)`; no `metadata_string`/`metadata_values` calls remain in OSS for trust-bearing fields. (2) `crates/runx-receipts/src/identity.rs` provides `SignedDisplayIdentity` + `signed_display_identity`, re-exported from `crates/runx-receipts/src/lib.rs:15`. (3) `crates/runx-receipts/tests/conformance.rs:133` adds `conformance_display_identity_ignores_unsigned_metadata` proving a metadata rewrite leaves the derived identity unchanged. (4) `crates/runx-contracts/src/receipt.rs:322` now documents that `metadata` "is non-authoritative and must never be the source of a trust-bearing identity label" and lists the signed display sources. (5) The `runx-runtime` integration test `journal_history.rs` was updated to assert the new signed-field values (`SIGNED_RUNTIME_SUBJECT`/`SIGNED_LOCAL_ACTOR`), and a new `history_display_identity_ignores_unsigned_metadata` test pins the metadata-invariance at the projection layer. The trusted-kernel boundary holds: `runx-receipts/identity.rs` only depends on `runx-contracts` types and stays pure. Cloud deliverables sit in the sibling repo and were attested by ac5; not re-verified here per scope. One low-severity observation: the displayed `name` is now the raw subject URI rather than a humanized label, mildly contradicting the spec's "friendly label" mitigation language, but the rendered identity is pinned by tests and acceptance accepts it.

Attack log:
- `prior finding-1 (metadata-sourced actors/source_type)`: Re-read history_row_with_policy in journal.rs:352 and confirm no metadata_string/metadata_values calls remain anywhere in OSS for identity-bearing fields. -> clean (Journal projection now consumes signed_display_identity exclusively. Grep across crates finds no remaining metadata-sourced identity reads.)
- `prior finding-2 (missing accessor)`: Verify identity.rs exists, is re-exported, and only depends on signed-body fields (no metadata access). -> clean (identity.rs reads subject.kind, subject.reference.uri, issuer.issuer_type, authority.actor_ref, acts[].by; canonical.rs strip confirms these survive into the signed body.)
- `prior finding-3 (missing conformance test)`: Open conformance.rs and confirm a test asserts metadata-rewrite invariance. -> clean (conformance_display_identity_ignores_unsigned_metadata mutates receipt.metadata and asserts signed_display_identity is unchanged.)
- `prior finding-4 (receipt.rs doc)`: Grep for 'non-authoritative' / 'trust-bearing' in contracts crate. -> clean (Receipt doc comment lines 322-326 carry the spec's invariant language and enumerate signed identity sources.)
- `prior finding-5 (hidden runtime regression)`: Audit journal_history.rs to confirm assertions match the new signed projection. -> clean (Test fixtures still call receipt_with_metadata but assertions now compare against SIGNED_RUNTIME_SUBJECT and SIGNED_LOCAL_ACTOR; the new dedicated test pins metadata invariance.)
- `regression hunt: other display surfaces`: Grep OSS for any remaining receipt.metadata reads that could rebuild identity (skill_name, actor, runner, source_type). -> clean (Only pay/supervisor and CLI runtime use `metadata.get` against HarnessReplayOutput, not Receipt.metadata. No display-side reads from Receipt.metadata remain.)
- `convention/no-legacy`: Check identity.rs against pure-crate rules and CONVENTIONS no-legacy/no-fallback boundaries. -> clean (Pure module; no fs/network/subprocess; no dual-source fallback to metadata; no compatibility aliases.)
- `dark patterns: subtle bugs in signed_actor_labels`: Probe push_unique for empty strings, duplicates, and ordering invariants when acts[].by provider/model overlap with authority.actor_ref. -> clean (push_unique filters empty strings and dedupes by exact match; iteration order is deterministic (authority first, then acts in order).)
- `accessor design vs spec language`: Compare accessor output (raw URI) against spec mitigation that promised a 'friendly label'. -> finding (Tracked as finding-6 (low). Display label is the raw subject URI, not a humanized name; consumers must humanize themselves.)
- `ambient drift overlap`: Check whether ambient drift files (publish.rs, CLI parity fixtures, README, etc.) re-introduce metadata-sourced identity for receipts. -> clean (Drift is owned by sibling specs (receipt-publish-cli-v1, hosted-scope-fail-closed-v1); no overlap with identity sourcing path.)

Findings:
- [high/non-blocking] `finding-1-actors-source-still-metadata` Local-history projection now sources actor/source identity from signed fields
  - Location: `crates/runx-runtime/src/journal.rs:352`
  - Evidence: history_row_with_policy now calls signed_display_identity(receipt) and assigns name=identity.subject_ref, harness_id=identity.subject_ref, source_type=identity.source_type, actors=identity.actors. No metadata_string/metadata_values calls remain in OSS for identity-bearing fields (grep clean).
  - Validation: journal_history.rs assertions now compare against SIGNED_LOCAL_ACTOR / SIGNED_RUNTIME_SUBJECT; the new history_display_identity_ignores_unsigned_metadata test exercises the metadata-rewrite invariant end-to-end.
- [high/non-blocking] `finding-2-no-signed-identity-accessor` Signed-identity accessor added to runx-receipts
  - Location: `crates/runx-receipts/src/identity.rs:16`
  - Evidence: New module crates/runx-receipts/src/identity.rs defines SignedDisplayIdentity and signed_display_identity; re-exported by crates/runx-receipts/src/lib.rs:15. Accessor reads only subject.kind, subject.reference.uri, issuer.issuer_type, authority.actor_ref.uri, and acts[].by — all part of the signed body per canonical strip in canonical.rs:62.
  - Validation: runtime/journal.rs consumes the accessor and journal_history.rs verifies the produced identity; runx-receipts compiles and the conformance test calls it.
- [high/non-blocking] `finding-3-no-identity-conformance-test` Identity conformance test added
  - Location: `crates/runx-receipts/tests/conformance.rs:133`
  - Evidence: conformance_display_identity_ignores_unsigned_metadata loads the success fixture, captures signed_display_identity, then mutates receipt.metadata with skill_name/source_type/runner/actor labels and asserts the derived identity is unchanged.
  - Validation: ac3 (cargo test -p runx-receipts conformance) reports pass.
- [medium/non-blocking] `finding-4-receipt-invariant-doc-missing` receipt.rs documents the metadata non-authoritative invariant
  - Location: `crates/runx-contracts/src/receipt.rs:322`
  - Evidence: Receipt doc comment now states metadata 'is non-authoritative and must never be the source of a trust-bearing identity label' and enumerates the signed display sources (subject.kind, subject.ref, issuer, authority.actor_ref, acts[].by).
  - Validation: Grep for 'non-authoritative' / 'trust-bearing' in crates returns the new doc.
- [medium/non-blocking] `finding-5-hidden-runtime-test-regression` journal_history integration test updated to the signed-field projection
  - Location: `crates/runx-runtime/tests/journal_history.rs:108`
  - Evidence: Test now asserts history.receipts[0].name == SIGNED_RUNTIME_SUBJECT ('hrn_journal-history_strict-proof'), actors == vec![SIGNED_LOCAL_ACTOR] ('runx:principal:local_runtime'), and source_type == 'local'. A new history_display_identity_ignores_unsigned_metadata test confirms the projection ignores metadata edits.
  - Validation: Although verify:fast still does not run cargo test -p runx-runtime, the assertions are now consistent with the signed-field code path; ac6 (verify:fast) passes and the receipts conformance test independently pins the accessor.
- [low/non-blocking] `finding-6-friendly-label-mitigation-partial` Accessor exposes raw subject URI; spec's 'friendly label' mitigation is unimplemented in OSS
  - Location: `crates/runx-receipts/src/identity.rs:19`
  - Evidence: signed_display_identity sets subject_ref = receipt.subject.reference.uri.to_string(). The spec risk mitigation (line 102-104) said 'the accessor produces a friendly label from subject.ref'; the implementation passes the URI through unchanged. Consumers like journal.rs assign this raw URI to LocalHistoryReceipt.name, so the rendered history row now shows e.g. 'hrn_journal-history_strict-proof' where it used to show 'Deploy Skill'.
  - Impact: The history list/JSON display becomes less human-readable for unsigned skills. Tests pin the new behavior, so this is an accepted-risk UX regression rather than a correctness bug.

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: claude
- home_repo: oss+cloud
- gap: frantic-architecture.md §15.1 gap 23

## Origin

Created by: Claude
Source: frantic-architecture.md §15.1 (2026-06-11 code re-audit), gap 23

## Harden Rounds

### round-1

Status: passed
Started: 2026-06-11T09:20:25Z
Ended: 2026-06-11T09:22:09Z

Observations:
- path
  - Result: clean
  - Anchor: code:crates/runx-contracts/src/receipt.rs:313
  - Note: The contract already documents metadata as a runtime-local read aid; the fix should change display sourcing, not the signed body.
- command
  - Result: advisory
  - Anchor: code:crates/runx-receipts/src/canonical.rs:59
  - Note: Acceptance should target receipt identity conformance and display tests, not a broad cloud package sweep.
- scope
  - Result: clean
  - Anchor: code:crates/runx-runtime/src/journal.rs:355
  - Note: Local history currently prefers unsigned `metadata.skill_name`; this is an in-scope OSS display source.
- timing
  - Result: clean
  - Anchor: spec_gap:phase2
  - Note: Cloud run projections derive actor/source labels from metadata today and should be corrected after the OSS accessor exists.
- rollback
  - Result: clean
  - Anchor: spec_gap:rollback
  - Note: No signed-body or schema change means rollback is display/accessor-only with no receipt migration.
- design
  - Result: clean
  - Anchor: spec_gap:objectives
  - Note: Metadata remains cosmetic; trust-bearing labels must derive from signed subject/issuer/authority fields.


## Planning Log

- none
