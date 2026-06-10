---
spec_version: '2.0'
task_id: runx-s-tier-cutover-v1
created: '2026-06-10T02:54:53Z'
updated: '2026-06-10T03:37:38Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# S-tier cutover: receipts prove governance

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-10T03:37:38Z
Review gate: pass

## Summary

Close the gap between "signed log" and "governance proof" for the local Rust
runtime. After this spec, every sealed privileged effect names the operator
grant that authorized it, offline `runx verify` checks that adherence and
fails closed when it is absent, the durable spend state cannot grow without
bound, and an operator can see runtime authority readiness in one diagnostic
view with errors that name the fix.

This is the S-tier cutover for the OSS kernel: the receipt stops being an
integrity artifact and becomes the proof that admitted authority bounded the
run. It builds directly on the completed Tier 0 capability-admission spine
(`runx-capability-admission-spine-v1`, archived 2026-06), the durable period
spend ledger, and the offline `runx verify` command, all already on main.

Hosted/cloud hardening is explicitly out of scope; it lives in a separate
repo and needs its own contract.

## Objectives

- Make sealed receipts carry per-effect grant evidence:
  - provider-permission effects record the operator grant id
    (`RUNX_PROVIDER_PERMISSION_GRANT_ID`) as a typed `Reference` in receipt
    authority evidence,
  - payment effects record the admitted spend capability/authority refs in
    the same shape,
  - reuse existing `Reference` and `ReceiptAuthority.grant_refs` wire shapes;
    do not add new public JSON schema fields.
- Make scope adherence a verification check, not a convention:
  - a pure check in `runx-receipts` flags any act carrying privileged effect
    evidence (`ProofKind::EffectEvidence`) on a receipt with no grant
    evidence,
  - `runx verify` surfaces that finding and exits non-zero,
  - tampering that strips grant refs after sealing is caught by the existing
    digest/signature checks; the new check catches receipts that were sealed
    without grant evidence in the first place.
- Keep durable effect state bounded:
  - closed period-spend windows are pruned on write once they fall outside a
    fixed retention horizon (keep the active window plus the previous one per
    ledger key),
  - pruning never touches idempotency entries, finality records, or the
    active window, and replay of a sealed step remains idempotent.
- Give operators one authority-readiness view:
  - `runx doctor authority` reports receipt signer configuration, verify-key
    configuration, effect-state path resolution, and provider-permission
    grant env presence (never values),
  - denial-path error messages name the exact env var, flag, or key that
    fixes them.
- Do not build a second policy engine, do not change public wire schemas,
  and do not touch hosted/cloud code.

## Scope

In scope:

- `crates/runx-runtime/src/effects/provider_permission.rs` receipt evidence
  emission for the admitted grant id.
- `crates/runx-pay/src/runtime.rs` receipt evidence emission for admitted
  spend capability/authority refs.
- `crates/runx-receipts/src/verify` pure scope-adherence finding.
- `crates/runx-cli/src/verify.rs` surfacing of the new finding.
- `crates/runx-pay/src/state.rs` period-window retention/pruning.
- `crates/runx-cli/src/doctor.rs` authority diagnostic section.
- Focused tests for each seam, including a sealed-without-grant negative
  fixture.
- `docs/security-authority-proof.md` notes for grant evidence, the verify
  scope pass, and state retention.

Out of scope:

- Hosted/cloud (`../cloud`) authz, billing, or deploy changes.
- New public JSON schema fields or `.v2` contract ids.
- Tier 2 agent-loop demotion and MCP-first entrypoint work.
- Registry resolver trust UX.
- Broad workspace cleanup or unrelated active specs.

## Dependencies

- Completed Tier 0 admission spine:
  [.scafld/specs/archive/2026-06/runx-capability-admission-spine-v1.md](/Users/kam/dev/runx/runx/oss/.scafld/specs/archive/2026-06/runx-capability-admission-spine-v1.md)
- Receipt contracts already carry grant evidence shapes:
  [crates/runx-contracts/src/receipt.rs](/Users/kam/dev/runx/runx/oss/crates/runx-contracts/src/receipt.rs)
- Offline verification command already on main:
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- Durable period spend ledger already on main:
  [crates/runx-pay/src/state.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/src/state.rs)
- Verification finding vocabulary:
  [crates/runx-receipts/src/verify/finding.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify/finding.rs)

## Grounding Evidence

- `ReceiptAuthority.grant_refs` already exists as `Vec<Reference>`, so grant
  evidence needs no schema change.
  [crates/runx-contracts/src/receipt.rs](/Users/kam/dev/runx/runx/oss/crates/runx-contracts/src/receipt.rs:187)
- Provider permission already requires and parses the operator grant id; the
  admission context holds `grant_id` but it is not yet emitted into sealed
  receipt evidence.
  [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs:12)
- Payment admission carries `spend_capability_ref` as a typed `Reference`
  through `StepPaymentAuthorityContext`.
  [crates/runx-pay/src/runtime.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/src/runtime.rs)
- Privileged effect evidence is already typed: payment rail proofs use
  `ProofKind::EffectEvidence` references, matchable without label text.
  [crates/runx-contracts/src/reference.rs](/Users/kam/dev/runx/runx/oss/crates/runx-contracts/src/reference.rs:105)
- The finding vocabulary already includes `AuthorityProofMissing`; the scope
  check can reuse it or add one narrowly-named code.
  [crates/runx-receipts/src/verify/finding.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify/finding.rs:40)
- The period spend ledger reserves into
  `EffectFamilyState.period_spend_ledger` keyed by window start and never
  removes closed windows.
  [crates/runx-pay/src/state.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/src/state.rs:251)
- `runx doctor` exists as a native command with its own plan/dispatch and can
  host an authority section.
  [crates/runx-cli/src/doctor.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/doctor.rs:12)
- `runx verify` groups receipt trees, verifies digests/signatures/lineage
  offline, and already exits non-zero on findings.
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)

## Assumptions

- The executor may be Codex or another coding agent; it must run the scafld
  build loop (`scafld build runx-s-tier-cutover-v1`) after implementing each
  phase so acceptance evidence is recorded, then `scafld review` with a real
  provider before `scafld complete`.
- Use `CARGO_TARGET_DIR=target/runx-s-tier-cutover` for all cargo commands to
  avoid contending with other agents' builds.
- Adding a `Reference` into `receipt.authority.grant_refs` (or act
  verification refs) changes receipt *content*, not the wire schema. Existing
  fixtures that re-seal provider-permission or payment receipts may need
  regeneration; regenerate only those fixtures and treat any other fixture
  drift as a defect in the change.
- The scope-adherence check is pure and lives in `runx-receipts`; the CLI
  only reports it. No filesystem, network, or env access in the pure crate.
- Retention pruning happens inside the existing locked-state transaction; no
  new background process, no clock reads inside pure helpers (window
  comparison uses the already-stored `window_start` strings, which order
  lexicographically as ISO dates).
- Grant evidence values are identifiers, never secret material; the existing
  receipt redactor still runs before sealing.

## Touchpoints

- [crates/runx-runtime/src/effects/provider_permission.rs](/Users/kam/dev/runx/runx/oss/crates/runx-runtime/src/effects/provider_permission.rs)
- [crates/runx-pay/src/runtime.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/src/runtime.rs)
- [crates/runx-pay/src/state.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/src/state.rs)
- [crates/runx-pay/tests/payment/state.rs](/Users/kam/dev/runx/runx/oss/crates/runx-pay/tests/payment/state.rs)
- [crates/runx-receipts/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify.rs)
- [crates/runx-receipts/src/verify/finding.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify/finding.rs)
- [crates/runx-receipts/tests/receipt_contracts.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/tests/receipt_contracts.rs)
- [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- [crates/runx-cli/src/doctor.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/doctor.rs)
- [crates/runx-cli/src/launcher.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/launcher.rs)
- [docs/security-authority-proof.md](/Users/kam/dev/runx/runx/oss/docs/security-authority-proof.md)

## Risks

- Receipt content changes can break sealed-fixture digests. Mitigation:
  regenerate only fixtures whose flows now legitimately carry grant evidence;
  any unrelated fixture drift blocks the phase.
- A scope-adherence check that fires on non-privileged acts would make every
  existing receipt invalid. Mitigation: gate the check on typed
  `ProofKind::EffectEvidence` references only, and prove a plain skill
  receipt with no privileged effects still verifies clean.
- Pruning could delete state that replay still needs. Mitigation: prune only
  `period_spend_ledger` entries strictly older than the retention horizon;
  idempotency entries and finality records are untouched; add a replay test
  against a pruned store.
- Doctor output could leak secret material. Mitigation: report presence and
  key ids only, never values; reuse the receipt redaction conventions.
- Over-reach into a second policy engine. Mitigation: the scope check is a
  single pure predicate over an already-sealed receipt; no new admission
  surfaces.

## Acceptance

Profile: strict

Validation:
- `cd crates && cargo fmt --check`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" provider_permission`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-pay`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-receipts`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli verify`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli doctor`
- `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli launcher`
- `! rg -n "operator-provider-grant|allow_explicit_manifest_path: true" crates/runx-runtime/src/effects/provider_permission.rs crates/runx-runtime/src/adapters/agent_tools.rs`
- `git diff --check`

## Phase 1: Grant Evidence In Sealed Receipts

Status: completed
Dependencies: none

Objective: Every sealed privileged effect names the operator authority that

Changes:
- Provider permission: when the effect admits via `RUNX_PROVIDER_PERMISSION_GRANT_ID`, emit a `Reference` carrying the grant id (e.g. uri `runx:grant:<id>`, `reference_type: Verification` or the existing grant reference shape used by `ReceiptAuthority.grant_refs`) into the sealed receipt's authority evidence for that step.
- Payment: emit the admitted `spend_capability_ref` and the child authority resource ref into the same receipt authority evidence during effect sealing, so the spend names its capability.
- Granted-scope lists stay in admission context/metadata as today; only identifiers go into grant evidence. Never secret material.
- Add runtime tests asserting the sealed receipt for a provider-permission step and a payment step contains the expected grant references, and that a step without privileged effects gains no grant refs.
- Regenerate only the receipt fixtures whose flows now legitimately carry grant evidence; record each regenerated fixture in Deviations.

Acceptance:
- [x] `ac1` command - Provider permission tests pass with grant evidence
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-runtime --features "http agent catalog mcp mcp-http-server" provider_permission`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - Payment runtime tests pass with grant evidence
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-pay`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7

## Phase 2: Scope-Adherence Verification

Status: completed
Dependencies: phase1

Objective: Verification proves admitted authority bounded the run, offline.

Changes:
- Add a pure check in `runx-receipts::verify`: for each act whose verification refs include a typed `ProofKind::EffectEvidence` reference, the receipt must carry grant evidence (non-empty `authority.grant_refs` or an act-level grant reference). Reuse `AuthorityProofMissing` or add one narrowly-named finding code such as `EffectGrantEvidenceMissing`; do not add a parallel verification entry point.
- Wire the check into the existing `verify_receipt` structural pass so every caller (CLI, tree verification, hosted readers) gets it without opt-in.
- `runx verify` requires no new flags: the finding flows through the existing report and non-zero exit.
- Tests: a receipt sealed with privileged evidence and grant refs verifies clean; the same receipt with grant refs absent fails with the new finding; a plain skill receipt with no privileged effects stays clean.

Acceptance:
- [x] `ac3` command - Receipt verification tests pass including scope pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-12
- [x] `ac4` command - CLI verify tests pass including scope finding surfacing
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli verify`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13

## Phase 3: Durable State Retention

Status: completed
Dependencies: phase2

Objective: The effect state file stays bounded under long-lived operation.

Changes:
- During period-spend reservation (inside the existing locked-state transaction), prune `period_spend_ledger` entries for the same family/authority/currency/period whose `window_start` is older than the previous window relative to the reservation being recorded. Lexicographic comparison of the stored ISO `window_start` strings is sufficient; no clock reads in pure code.
- Never prune the active window, run-spend ledgers, idempotency entries, finality records or events, or consumed capabilities.
- Tests: spend in window N prunes window N-2 but keeps N-1 and N; replay of an already-sealed step against a pruned store remains idempotent; a state file written before pruning existed still loads.

Acceptance:
- [x] `ac5` command - Payment state tests pass including retention
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-pay`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-18

## Phase 4: Operator Authority Diagnostics

Status: completed
Dependencies: phase3

Objective: One view answers "is this runtime ready to exercise authority,

Changes:
- Add an authority section to `runx doctor` (subcommand or section `runx doctor authority`): receipt signer configuration status (env names and key ids only), verify-key configuration status (`RUNX_RECEIPT_VERIFY_KID` / `RUNX_RECEIPT_VERIFY_ED25519_PUBLIC_KEY_BASE64`), resolved effect-state path (and the cross-run spend-cap consequence when it is unset), and provider-permission grant env presence.
- Every "not configured" line names the exact env var or flag that fixes it. No secret values, ever.
- Keep launcher help text in sync; extend launcher tests for the new subcommand path.
- Update `docs/security-authority-proof.md` with the grant-evidence shape, the verify scope pass, and the state retention policy.

Acceptance:
- [x] `ac6` command - Doctor tests pass with authority section
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli doctor`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `ac7` command - Launcher routing tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-s-tier-cutover cargo test -p runx-cli launcher`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Phase 5: Final Integration Gate

Status: completed
Dependencies: phase4

Objective: Prove the cutover is narrow, formatted, and free of authority

Changes:
- Run the security grep first and again as the final gate; it locks the Tier 0 invariants (no invented grant ids, no explicit manifest path resolution for model-selected tools) across this spec's edits.
- Run formatting, the full focused test list, and whitespace checks.
- Record any broader workspace failures as out-of-scope only with exact command output as evidence.

Acceptance:
- [x] `ac8` command - Rust formatting is clean
  - Command: `cd crates && cargo fmt --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `ac9` command - Security grep finds no authority regression
  - Command: `! rg -n "operator-provider-grant|allow_explicit_manifest_path: true" crates/runx-runtime/src/effects/provider_permission.rs crates/runx-runtime/src/adapters/agent_tools.rs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [x] `ac10` command - Diff has no whitespace errors
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-31

## Follow-Up Specs

- Hosted/cloud authz parity audit (separate repo `../cloud`): bring grant
  issuance, revocation, and billing surfaces under the same adversarial
  review rigor as this kernel; the 2026-06-10 session already fixed grant
  expiry and secret separation there.
- Tier 2: host-driven and authenticated-MCP execution as first-class
  entrypoints; agent-loop demotion to sample adapter.
- Receipt-store concurrency: move effect state beyond single-file JSON if
  hosted runners ever share a state path.

## Rollback

- Revert grant-evidence emission and the scope-adherence finding together;
  a verifier that requires evidence no emitter produces would brick all new
  receipts.
- Receipts sealed while this spec was live keep their grant references;
  they remain valid under the old verifier because the shapes already
  existed.
- Pruning rollback restores unbounded ledger growth but loses no enforcement
  correctness; never roll back by widening retention to "keep everything"
  silently — make it explicit in the diff.
- Do not loosen the ac9 security grep as a rollback shortcut.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: codex
Output: codex.output_file
Summary: No completion-blocking findings found. I treated recorded acceptance evidence as already executed per provider instruction and did not run build/test/mutation commands.

Attack log:
- `crates/runx-runtime/src/effects/provider_permission.rs; crates/runx-pay/src/runtime.rs; crates/runx-runtime/src/execution/runner/steps.rs; crates/runx-runtime/src/receipts/seal.rs`: Grant evidence emission trace -> clean (Inspected provider-permission and payment `RuntimeEffect::authority_grant_refs` implementations plus runner receipt sealing paths for normal, catalog, and replayed effect steps. Grant refs are passed into receipt authority before sealing.)
- `crates/runx-receipts/src/verify.rs; crates/runx-receipts/src/verify/finding.rs; crates/runx-cli/src/verify.rs`: Scope-adherence verification integration -> clean (Inspected `verify_receipt` structural pass and CLI report mapping. `EffectGrantEvidenceMissing` is wired into all receipt verification through the existing path and surfaces through `runx verify` findings without a new flag.)
- `crates/runx-pay/src/state.rs; crates/runx-pay/tests/payment/state.rs`: Durable period spend retention -> clean (Inspected period ledger pruning and focused tests. Retention is scoped to the same family/authority/currency/period tuple and preserves current/previous/newer windows while leaving idempotency/finality/run ledgers outside the prune path.)
- `crates/runx-cli/src/doctor.rs; crates/runx-cli/src/launcher.rs; crates/runx-cli/tests/doctor.rs; crates/runx-cli/tests/launcher.rs`: Operator authority diagnostics and redaction -> clean (Inspected `runx doctor authority` routing, diagnostic output, and tests. Missing configuration names the exact env vars, resolved state path is reported, key ids are allowed, and secret/grant/scope values are not emitted.)

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-06-10: Authored from the post-Tier-0 S-tier assessment: kernel
  admission (Tier 0), durable period ledger, and offline `runx verify` are
  on main; the remaining gap to "receipts prove governance" is per-effect
  grant evidence, a scope-adherence verification pass, state retention, and
  operator authority diagnostics.
- 2026-06-10: Grounded every phase in current code anchors
  (receipt.rs:187 grant_refs, provider_permission.rs grant id env,
  state.rs:251 period ledger, finding.rs:40 AuthorityProofMissing,
  doctor.rs:12 native doctor).
- 2026-06-10: Cloud-layer parity work intentionally split into a follow-up
  spec in the platform repo so this contract stays executable in one
  workspace.
