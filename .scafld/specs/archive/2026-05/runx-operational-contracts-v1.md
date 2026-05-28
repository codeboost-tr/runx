---
spec_version: '2.0'
task_id: runx-operational-contracts-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-28T02:53:39Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# Operational Proposal Contracts

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-28T02:53:39Z
Review gate: pass

## Summary

Define the minimal reusable contract layer for operational proposal
composition. This spec promotes one public proposal packet now:
`runx.operational_proposal.v1`.

The proposal packet is intentionally generic: central source references,
context/artifact references, decision summaries, recommended action-lane
intents, proposal kind, authority notes, receipts, and story/outbox metadata.
It does not widen the existing closed `runx.operational_policy.v1` permissions
object.

Runx core must not promote fixed public packet families for each application
domain. A product can have a support reply proposal, dev escalation proposal,
outreach proposal, or product signal proposal, but the shared runx contract is
one proposal shape with namespaced `proposal_kind` metadata and explicit
authority gates.

## Objectives

- Record the public/internal promotion criteria for operational contracts.
- Promote the generic `runx.operational_proposal.v1` shape with:
  - stable id and source id;
  - `proposal_kind`;
  - decision summary and rationale;
  - evidence refs and artifact refs;
  - owner route id;
  - confidence, risks, caveats, and missing context;
  - recommended action-lane intents;
  - required human gates;
  - allowed next action intents;
  - source event id, `source_ref`, and optional `source_thread_ref`;
  - generic result/publication references for tracking items, change requests,
    provider comments, and source-thread updates;
  - outcome observation and final outcome references;
  - public summary safe for provider threads, work items, support surfaces, and
    admin readbacks.
- Preserve existing action lanes and authority boundaries.
- Preserve escalation as a first-class proposal kind. The generic contract must
  support `proposal_kind: escalation` with severity, owner route, evidence,
  suspected area, urgency, and exact human decision required.
- Do not extend `runx.operational_policy.v1` permissions in this spec. The
  existing closed policy shape remains intact; proposal authority is represented
  on the proposal packet and admitted through existing action lanes.
- Keep intermediate source hydration, model reasoning traces, and product-owned
  context private unless a public consumer requires a stable schema.
- Prevent raw provider payloads, local paths, tokens, concrete customer fields,
  or product-specific owner/channel values from entering public contracts.

## Scope

In scope:

- Contract and policy docs:
  - `docs/operational-intelligence.md`;
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `docs/thread-story-contract.md`.
- TypeScript contract work for `runx.operational_proposal.v1`:
  - `packages/contracts/src/schemas/spine.ts` for central reference/link types;
  - `packages/contracts/src/schemas/operational-policy.ts` for invariant
    verification only; do not change `runx.operational_policy.v1`
    permissions in this spec;
  - `packages/contracts/src/schemas/thread-outbox-provider.ts`;
  - `packages/contracts/src/schemas/context.ts`;
  - `packages/contracts/src/schemas/artifact.ts`;
  - `packages/contracts/src/schemas/agent-act.ts`;
  - `packages/contracts/src/schemas/run-summary.ts`;
  - `packages/contracts/src/schemas/packet-index.ts`;
  - `packages/contracts/src/internal.ts`;
  - `packages/contracts/src/index.ts`.
- Rust contract work for `runx.operational_proposal.v1`:
  - `crates/runx-contracts/src/reference.rs` for central reference/link types;
  - `crates/runx-contracts/src/operational_policy/evaluate.rs`;
  - `crates/runx-contracts/src/signal.rs`;
  - `crates/runx-contracts/src/decision.rs`;
  - `crates/runx-contracts/src/artifact.rs`;
  - `crates/runx-contracts/src/act.rs`;
  - `crates/runx-contracts/src/packet_index.rs`;
  - `crates/runx-contracts/src/schema_artifacts.rs`;
  - `crates/runx-contracts/src/lib.rs`.
- Fixtures and tests:
  - `fixtures/contracts/operational-proposal/**`;
  - `fixtures/operational-policy/**`;
  - `packages/contracts/src/schemas/*.test.ts`;
  - `crates/runx-contracts/tests/*`.

Out of scope:

- Provider fetch or hydration implementations.
- Product-specific provider buttons, source policies, customer/account
  enrichment, owner maps, labels, project boards, templates, or live dogfood
  scripts.
- Runtime runner behavior, target repo execution, change-request creation,
  final change approval, or customer-message sending.
- Domain-specific public contract families.
- Compatibility aliases for old or experimental packet names.

## Dependencies

- Parent tracker:
  - `runx-operational-intelligence-action-layer-v1`.
- Generic sibling specs:
  - `runx-operational-proposal-composition-v1`;
  - `runx-operational-story-outbox-v1`;
  - `nitrosend-operational-intelligence-integration-v1`.
- Existing runx surfaces:
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `docs/thread-story-contract.md`;
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/contracts/src/schemas/operational-policy.ts`;
  - `crates/runx-contracts/src/operational_policy/evaluate.rs`;
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-triage/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`.

## Assumptions

- Source adapters admit events and hydrate provider context before generic runx
  skills reason over it.
- Hydrated provider context is redacted or summarized before it reaches public
  contracts.
- Owner routing is represented by product-supplied route ids.
- Escalation is a proposal/handoff pattern, not a forced change request or a
  provider notification. Consuming products decide whether an escalation is
  routed to a chat provider, work tracking provider, alerting provider, support
  tooling, or a human review queue.
- Customer-send authority and final change approval authority are never implied
  by a proposal.
- `runx.operational_policy.v1` is closed and remains unchanged in this spec:
  `permissions.auto_merge` stays literal `false`,
  `permissions.require_human_merge_gate` stays literal `true`, and no
  `permissions.*` or `outcomes.*` fields are added here.
- Proposal preparation is authorized by an admitted existing action lane such as
  `reply-only`, `manual-review`, `work-plan`, `issue-intake`, or `issue-to-pr`.
  A later hard-cut policy spec may add a generic `proposal` action if an
  in-tree consumer proves that source/runner policy must admit proposals
  independently.
- Customer send, provider notification, and human-approved final change
  execution are outside the proposal packet and outside this policy change.
- If source-thread publication is required by policy but absent, publication and
  provider mutation fail closed.

## Touchpoints

- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/issue-to-pr.md`
- `docs/thread-story-contract.md`
- `packages/contracts/src/schemas/spine.ts`
- `packages/contracts/src/schemas/operational-policy.ts` (invariant
  verification only; no v1 permission edits)
- `packages/contracts/src/schemas/thread-outbox-provider.ts`
- `packages/contracts/src/schemas/context.ts`
- `packages/contracts/src/schemas/artifact.ts`
- `packages/contracts/src/schemas/agent-act.ts`
- `packages/contracts/src/schemas/run-summary.ts`
- `packages/contracts/src/schemas/packet-index.ts`
- `packages/contracts/src/internal.ts`
- `packages/contracts/src/index.ts`
- `crates/runx-contracts/src/reference.rs`
- `crates/runx-contracts/src/operational_policy/evaluate.rs`
- `crates/runx-contracts/src/signal.rs`
- `crates/runx-contracts/src/decision.rs`
- `crates/runx-contracts/src/artifact.rs`
- `crates/runx-contracts/src/act.rs`
- `crates/runx-contracts/src/packet_index.rs`
- `crates/runx-contracts/src/schema_artifacts.rs`
- `crates/runx-contracts/src/lib.rs`
- `fixtures/contracts/operational-proposal/**`
- `fixtures/operational-policy/**`
- `packages/contracts/src/schemas/*.test.ts`
- `crates/runx-contracts/tests/*`

## Risks

- Premature public schema expansion. Mitigation: exactly one generic proposal
  shape is promoted; fixed domain packet families remain out of scope.
- Product-specific leakage into core. Mitigation: only abstract ids, route ids,
  artifact refs, and namespaced proposal kinds are allowed.
- Provider data leakage. Mitigation: require redaction status, artifact refs,
  and leak tests for public fixtures.
- Contract drift between TS and Rust. Mitigation: fixture parity and schema
  generator checks when public contracts change.
- Hidden authority creep. Mitigation: customer send, final change approval,
  billing, and destructive mutation remain outside proposal authority; this
  spec does not widen the closed operational-policy permissions object.

## Acceptance

Profile: strict

Validation:
- `scafld validate runx-operational-contracts-v1`
- `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
- `pnpm fixtures:contracts:check && pnpm fixtures:contracts:keys`
- `pnpm boundary:check`

## Phase 1: Contract Boundary

Status: completed
Dependencies: none

Objective: Record the generic public proposal packet boundary before writing

Changes:
- Document proposal taxonomy and promotion criteria.
- Record `runx.operational_proposal.v1` as the only public packet promoted by this spec.
- Map existing lanes to action intents and proposal outputs.
- Identify mandatory fields for replay, dedupe, redaction, evidence, owner routing, and publication.
- Define the authority split between read-only triage, proposal preparation,
  provider publication, tracking-item creation, change-request creation,
  customer send, and final change approval.
- Record that `runx.operational_policy.v1` remains unchanged and that proposals are authorized through existing admitted action lanes.

Acceptance:
- [x] `p1_ac1` command - Generic proposal boundary is documented.
  - Command: `test -f docs/operational-intelligence.md && for token in runx.operational_proposal.v1 proposal_kind owner_route_id evidence_refs human_gates source_ref source_thread_ref; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `p1_ac2` command - Operational policy v1 non-migration is documented.
  - Command: `test -f docs/operational-intelligence.md && for token in "operational_policy.v1 remains unchanged" "auto_merge" "require_human_merge_gate" "existing action lane"; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `p1_ac3` command - Draft validates after hardening edits.
  - Command: `scafld validate runx-operational-contracts-v1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: TypeScript Proposal Contract

Status: completed
Dependencies: Phase 1

Objective: Add the generic public proposal contract.

Changes:
- Add `runx.operational_proposal.v1` TypeScript schema.
- Export only the promoted generic proposal schema.
- Add fixture cases for proposal prepared, proposal blocked, missing source ref,
  missing redaction status, provider-specific top-level fields, and unapproved
  mutation/final-decision authority.
- Add tests for stable canonical keys and safe public output.

Acceptance:
- [x] `p2_ac1` command - TypeScript contract tests pass.
  - Command: `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `p2_ac2` command - Contract fixture checks pass.
  - Command: `pnpm fixtures:contracts:check && pnpm fixtures:contracts:keys`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `p2_ac3` command - Boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `p2_ac4` command - Proposal contract covers hydrated context, central refs, generic links, gates, and outcome refs.
  - Command: `for token in source_event_id source_ref source_thread_ref hydrated_context_ref redaction_status result_refs publication_refs reference_link human_gates final_outcome tracking_item change_request provider_thread provider_comment; do rg -n "$token" fixtures/contracts/operational-proposal packages/contracts/src >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16
- [x] `p2_ac5` command - TypeScript schema emission stays current.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-17

## Phase 3: Rust Contract Parity

Status: completed
Dependencies: Phase 2

Objective: Keep Rust contract types, validation, and generated schemas aligned

Changes:
- Add matching Rust structs/enums for `runx.operational_proposal.v1`.
- Do not change operational-policy v1 permissions in this spec.
- Update schema generation/conformance tests.
- Add invalid-fixture coverage for raw provider data, missing redaction,
  unapproved mutation/final-decision authority, provider-specific top-level
  fields, and product-specific fields.

Acceptance:
- [x] `p3_ac1` command - Rust contract tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `p3_ac2` command - Schema generator checks pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features schema_generator_check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `p3_ac3` command - Public proposal surfaces avoid raw/provider/product leaks.
  - Command: `sh -c 'if test -d fixtures/contracts/operational-proposal; then ! rg -n "xox[baprs]-|BEGIN .*PRIVATE KEY|url_private_download|raw_payload|/Users/|\\bC[0-9A-Z]{8,12}\\b" fixtures/contracts/operational-proposal packages/contracts/src/schemas crates/runx-contracts/src; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Phase 4: Documentation And Consumer Readiness

Status: completed
Dependencies: Phase 3

Objective: Make consuming products and skill authors implement against one

Changes:
- Update docs with proposal examples, authority model, redaction rules, and product/core boundaries.
- Add a table mapping existing actions to proposal usage.
- Document private receipt/artifact fields versus public story fields.
- Confirm consuming products can namespace proposal kinds without adding new core action variants.

Acceptance:
- [x] `p4_ac1` command - Docs and contract surfaces mention generic proposal fields.
  - Command: `for token in operational_proposal proposal_kind recommended_actions evidence_refs owner_route_id source_ref source_thread_ref result_refs publication_refs; do rg -n "$token" docs packages/contracts crates/runx-contracts >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `p4_ac2` command - Operational proposal docs contain authority and reference examples.
  - Command: `test -f docs/operational-intelligence.md && for token in "## Operational Proposal Contract" "## Reference Contracts" "## Authority Model" "source_ref" "source_thread_ref" "runx.reference.v1" "runx.reference_link.v1" "tracking_item" "change_request" "provider_thread" "provider_comment" "human_gates" "final_outcome"; do rg -n "$token" docs/operational-intelligence.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [x] `p4_ac3` command - Fast validation passes.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-31

## Rollback

- Revert proposal schemas, fixtures, generated artifacts, exports, Rust
  parity types, and docs introduced by this child as one unit.
- If the proposal schema is withdrawn, hard-cut it out rather than leaving
  compatibility aliases.
- Do not mutate provider adapters, runtime runners, consuming product policy, or
  live provider data as part of rollback.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Operational proposal contract (`runx.operational_proposal.v1`) is well-bounded: TS schema, Rust struct, generated JSON Schema, fixtures, and docs are consistent and mutually validating. The closed `runx.operational_policy.v1` permissions object remains untouched (literal `auto_merge=false` / `require_human_merge_gate=true` preserved in both languages). Harden round-1 blockers (permissions collision, deferred promotion decision, trivial Phase 1 grep, `merge_pull_request` semantics) are addressed. Authority invariants are enforced on the JSON wire through `const` literal bools and Rust custom deserializers. Three non-blocking quality observations: (1) Rust round-trip fixture coverage tests only `proposal-prepared.json` while TS covers both positives, (2) the `confidence` 0..1 bound is enforced on the JSON wire but not by the Rust `f64` deserializer, and (3) the `extensions` field is a wide-open `additionalProperties: {}` escape hatch that the spec's leakage-prevention rule does not directly constrain. None block completion. Ambient drift outside the declared task list is dominated by intended schema regeneration from the new `ReferenceType` variants and intended new files (operational-proposal.ts/.test.ts/.schema.json, reference-link.schema.json); no unrelated drift detected.

Attack log:
- `spec promotion decision`: Verify the promotion question raised in harden round-1 is actually resolved (one public schema, no v2 alias). -> clean (OperationalProposalSchema enum has a single V1 variant; schema_artifacts.rs lists exactly one operational-proposal entry; no v2/.legacy aliases present.)
- `operational-policy v1 closed invariants`: Confirm permissions.auto_merge and require_human_merge_gate remain literal false/true in TS and Rust, and no new permissions/outcomes fields were added. -> clean (packages/contracts/src/schemas/operational-policy.ts:166-168 still uses Type.Literal(false)/Type.Literal(true); crates/runx-contracts/src/operational_policy/evaluate.rs:505-516 still emits the lint failures; no permissions.prepare_proposal / send_customer_message / publish_source_thread_update / merge_pull_request fields exist anywhere.)
- `phase-1 doc acceptance check`: Reproduce harden-3 concern (grep matched against spec body itself). -> clean (p1_ac1/p1_ac2 commands now grep only docs/operational-intelligence.md; that doc contains all required tokens including the literal string `operational_policy.v1 remains unchanged`.)
- `TS/Rust positive-fixture parity`: Check whether Rust serde covers the same positive fixtures as TS Ajv. -> finding (Reported as rust-positive-fixture-parity; TS covers prepared+blocked, Rust only covers prepared.)
- `confidence bound enforcement`: Probe whether the Rust contract type enforces the documented [0,1] bound on confidence. -> finding (Reported as rust-confidence-bound-permissive; serde accepts any f64.)
- `extensions escape hatch`: Look for ways a product could route prohibited leakage data through the public envelope without tripping the spec's invariants or fixture leak grep. -> finding (Reported as extensions-leakage-escape-hatch; `additionalProperties: {}` accepts anything.)
- `authority invariant defense-in-depth`: Test whether the proposal-only authority shape is enforced beyond JSON wire validation. -> finding (Reported as authority-invariant-construction-window; pub fields with deserialize-only enforcement.)
- `reference-type enum expansion regression`: Hunt for breakage in existing reference consumers (post_merge_observer, target_runner, thread_outbox_provider, act, harness fixtures) caused by new ReferenceType variants. -> clean (The change is additive on a snake_case enum with #[serde(rename_all)]; existing match arms remain exhaustive only if they previously listed all variants. Spot checks in crates/runx-runtime/src/post_merge_observer/github.rs, execution/target_runner*.rs, and act.rs do not pattern-match on ReferenceType directly; they consume Reference by value/uri. No regressions surfaced.)
- `ambient drift classification`: Distinguish task-scope changes from unrelated drift; ensure unrelated files weren't quietly touched. -> clean (The 39 drift entries are dominated by (a) intended new files (operational-proposal.ts/.test.ts/.schema.json, reference-link.schema.json, operational_proposal.rs) misclassified as outside the declared touchpoint list because they didn't exist at baseline, and (b) downstream schema regeneration from the new ReferenceType variants. scripts/verify-fast.mjs has only a minor edit and contains no functional regressions. No unrelated subsystem modifications detected.)
- `CLAUDE.md / no-legacy-code invariant`: Check for compatibility aliases or `.v2` shadow schemas the spec forbids. -> clean (Only one logical schema name (runx.operational_proposal.v1); no v2 alias, no compat re-exports, no removed-shim comments in the new files.)
- `fixture leakage gate`: Verify the p3_ac3 leak grep is not bypassed by the fixtures. -> clean (Fixture URIs use synthetic Slack channel ids (CBUGS / CSUPPORT / CBUG) and team ids (T123) that are too short to match `C[0-9A-Z]{8,12}`. No xox tokens, no /Users/ paths, no PRIVATE KEY blocks. The synthetic data is reasonable for documentation.)
- `Phase-4 docs coverage gates`: Confirm p4_ac1/p4_ac2 grep tokens are all present in actual contract code and docs (not only the spec body). -> clean (All required tokens (operational_proposal, proposal_kind, recommended_actions, evidence_refs, owner_route_id, source_ref, source_thread_ref, result_refs, publication_refs, plus the Authority-Model section headings, runx.reference.v1, runx.reference_link.v1, tracking_item, change_request, provider_thread, provider_comment, human_gates, final_outcome) are present in docs/operational-intelligence.md and the contract code.)

Findings:
- [low/non-blocking] `rust-positive-fixture-parity` Rust serde round-trip only covers proposal-prepared.json; proposal-blocked.json is not exercised on the Rust side.
  - Location: `crates/runx-contracts/tests/operational_proposal_fixtures.rs:5`
  - Evidence: crates/runx-contracts/tests/operational_proposal_fixtures.rs:5-7 lists exactly one positive fixture in POSITIVE_FIXTURES (proposal-prepared.json), while packages/contracts/src/schemas/operational-proposal.test.ts:12-15 it.each covers both proposal-prepared.json and proposal-blocked.json. Wire-conformance corpus in tests/schema_wire_conformance/corpora.rs:1672-1685 builds a synthetic 'valid blocked proposal' from the prepared template but does not load the actual blocked fixture, so the Rust path never verifies the canonical proposal-blocked.json bytes round-trip through serde.
  - Impact: TS/Rust contract drift on the support_reply / summary_only / non-mutating recommended-action shape could land without breaking Rust tests.
- [low/non-blocking] `rust-confidence-bound-permissive` Rust deserialization of `confidence` is more permissive than the JSON Schema bound.
  - Location: `crates/runx-contracts/src/operational_proposal.rs:125`
  - Evidence: crates/runx-contracts/src/operational_proposal.rs:125 declares `pub confidence: f64` with no custom validator. The emitted schema in confidence_schema() (operational_proposal.rs:234-240) and the committed schemas/operational-proposal.schema.json:272-276 both pin {minimum:0, maximum:1, type:number}. serde_json::from_value::<OperationalProposal> will accept confidence=5 or confidence=-1 even though wire validation rejects them; only the `bad confidence` corpus entry in tests/schema_wire_conformance/corpora.rs:1718-1721 exercises the schema rejection, not the Rust deserializer.
  - Impact: A Rust producer can build and serialize a proposal whose `confidence` violates the public schema. Wire consumers will reject, but the gap weakens the in-memory-to-wire contract guarantee that other contract types do enforce.
- [low/non-blocking] `extensions-leakage-escape-hatch` The `extensions` field accepts arbitrary keys/values, which weakens the spec's leakage-prevention guarantee for the public proposal envelope.
  - Location: `schemas/operational-proposal.schema.json:498`
  - Evidence: schemas/operational-proposal.schema.json:498-501 emits `extensions: {additionalProperties: {}, type: object}`. crates/runx-contracts/src/operational_proposal.rs:140-141 backs this with `Option<JsonObject>`, and packages/contracts/src/schemas/operational-proposal.ts:80 types it as `DeepReadonly<Record<string, unknown>>`. The spec Objectives forbid 'raw provider payloads, local paths, tokens, concrete customer fields, or product-specific owner/channel values' in public contracts, and the invalid-product-specific-field / invalid-provider-specific-field fixtures only test top-level keys; a payload like `extensions: {nitrosend_owner: "Kam", raw_payload: {...}}` would pass schema validation today.
  - Impact: Products can route leakage-prohibited data through the extensions escape hatch without tripping any schema or fixture-leak check. The p3_ac3 grep in the spec scans fixture/source files, not runtime payloads.
- [low/non-blocking] `authority-invariant-construction-window` `OperationalProposalAuthority` invariants are only enforced at deserialization time; Rust callers can construct values that violate the authority bound.
  - Location: `crates/runx-contracts/src/operational_proposal.rs:44`
  - Evidence: crates/runx-contracts/src/operational_proposal.rs:42-71 declares all four authority booleans as `pub` with custom `deserialize_with` that pins literal true/false values, and the emitted JSON Schema uses `const_bool(true)/const_bool(false)`. There is no constructor or `TryFrom` that enforces the invariant for direct struct construction; `OperationalProposalAuthority { proposal_only: false, mutation_authority_granted: true, ... }` compiles fine. Serializing such a value would produce JSON that fails wire validation, but the in-memory invariant is unprotected.
  - Impact: Rust internal callers may produce an envelope whose authority shape contradicts the documented proposal-only contract before it reaches a JSON Schema validator. Defense-in-depth is weaker than the wire contract suggests.

## Self Eval

- Pending re-hardening. Target bar: 9.5/10 contract quality with one generic
  proposal/action boundary, crisp authority semantics, TS/Rust parity when
  promoted, and no product-specific leakage.

## Deviations

- Prior hardening passed an over-specific draft. This edit intentionally
  reopens the contract decision around a generic proposal shape and requires
  fresh hardening before approval.
- Phase 4 narrowed the accepted contract further: provider-specific URL fields
  were removed from the proposal envelope, result/publication links now use the
  central `runx.reference_link.v1` contract, and cross-provider operational
  reference types are centralized under `runx.reference.v1`.

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-27T16:46:00Z
Ended: 2026-05-27T16:46:00Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec hedges the central question — promote `operational_proposal` as a public schema now, or keep it internal — onto every later phase, so scope, fixtures, and Rust/TS parity all branch on an undecided variable. Acceptance commands for Phase 1 are trivially satisfied because they grep the spec file itself, so the docs gate is non-functional. Most critically, the new permission flags (`prepare_proposal`, `send_customer_message`, `publish_source_thread_update`, `merge_pull_request`) collide with the existing v1 `OperationalPolicyAutomationPermissions` shape that is closed (`additionalProperties:false` / `deny_unknown_fields`) and has `auto_merge: Type.Literal(false)` plus `require_human_merge_gate: Type.Literal(true)` — extending it is a v1 wire-breaking change with no migration plan, and CLAUDE.md forbids a `.v2` alias. Path/command audits otherwise pass; rollback is coherent. Resolve the promotion decision, settle the permissions schema migration, and tighten Phase 1 acceptance before approval.

Checks:
- path audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:1
  - Result: passed
  - Evidence: All declared TS schema touchpoints exist (artifact.ts, context.ts, agent-act.ts, run-summary.ts, packet-index.ts, operational-policy.ts, thread-outbox-provider.ts, index.ts). All declared Rust touchpoints exist under crates/runx-contracts/src (signal.rs, decision.rs, artifact.rs, act.rs, packet_index.rs, schema_artifacts.rs, lib.rs, operational_policy/evaluate.rs). docs/developer-issue-inbox.md, docs/issue-to-pr.md, docs/thread-story-contract.md exist. docs/operational-intelligence.md does NOT yet exist and is implicitly a future file to be created in Phase 1 — flagged as advisory.
- command audit
  - Grounded in: code:package.json:45
  - Result: passed
  - Evidence: Acceptance commands are runnable from oss/: `pnpm fixtures:contracts:check` and `pnpm fixtures:contracts:keys` exist (package.json:45-47), `pnpm boundary:check` exists (line 37), `pnpm verify:fast` exists (line 69). Rust `schema_generator_check` is a real test module in crates/runx-contracts/tests/integration.rs:20. `scafld validate` is the canonical validator. The p3_ac3 sh -c `if test -d ...; then ! rg ...; fi` is well-formed and behaves as documented.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Result: failed
  - Evidence: The existing `automationPermissionsSchema` is closed (`additionalProperties: false`) and pins `auto_merge: Type.Literal(false)` and `require_human_merge_gate: Type.Literal(true)`. The Rust parity at crates/runx-contracts/src/operational_policy.rs:265-271 uses `#[serde(deny_unknown_fields)]`. Assumptions list `permissions.prepare_proposal`, `permissions.send_customer_message`, `permissions.publish_source_thread_update`, `permissions.merge_pull_request`, but the spec has no plan for whether these become a hard cutover on v1 (breaking) or a separate sibling schema, and CLAUDE.md forbids `.v2` aliases for governed wire shapes.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac1
  - Result: failed
  - Evidence: p1_ac1 and p1_ac2 grep `docs .scafld/specs/drafts/runx-operational-contracts-v1.md`. The spec file already contains every required token (operational_proposal, proposal_kind, owner_route_id, evidence_refs, human_gate, prepare_proposal, send_customer_message, publish_source_thread_update, merge_pull_request, require_outcome_observation) — verified by ripgrep. The Phase 1 gate therefore passes regardless of whether any doc is updated. The objective ('Record the generic public/internal packet boundary before writing schemas') is not actually enforced.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback explicitly bundles schemas + fixtures + generated artifacts + exports + Rust parity + docs as one revert, forbids compatibility aliases for prematurely promoted schemas, and walls off runtime/adapter/live-data changes. That matches CLAUDE.md's `no_legacy_code` and `public_api_stable` invariants. Repair path is credible because nothing here mutates external state.
- design challenge
  - Grounded in: spec_gap:objectives
  - Result: failed
  - Evidence: Phase 1 is asked to 'decide whether operational_proposal is promoted as a public schema now or remains an internal helper'. Phases 2, 3, 4 each begin with 'if Phase 1 requires a public schema'. The Scope section opens both TS and Rust touchpoints under the same conditional, and Phase 2 Changes simultaneously says 'Add fixture cases for proposal allowed, proposal denied...' unconditionally. The decision tree branches across every downstream phase. This is the architectural call to make during harden, not at build time — otherwise the spec is non-executable as a single contract.

Issues:
- [high/blocks approval] `harden-1` scope_migration - New permissions fields collide with closed v1 OperationalPolicyAutomationPermissions schema; no migration plan.
  - Status: open
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Evidence: operational-policy.ts:164-171 defines `automationPermissionsSchema` with `additionalProperties: false`, `auto_merge: Type.Literal(false)`, `require_human_merge_gate: Type.Literal(true)`. Rust mirror at crates/runx-contracts/src/operational_policy.rs:265-271 uses `#[serde(deny_unknown_fields)]`. The spec Assumptions add `permissions.prepare_proposal`, `permissions.send_customer_message`, `permissions.publish_source_thread_update`, `permissions.merge_pull_request`, `permissions.require_human_merge_gate`, and `outcomes.require_outcome_observation` / `outcomes.require_source_thread_for_publication`, but never resolves whether v1 is extended in place (breaking all existing policies) or whether new fields live elsewhere. CLAUDE.md forbids `.v2` aliases for governed wire shapes.
  - Recommendation: Before approval, decide and record one of: (a) hard-cutover extension of v1 with explicit fixture regeneration and a one-shot consumer update, (b) split the new authority surface into a sibling schema (e.g., a separate authority/permissions packet keyed by proposal_kind) so v1 stays untouched, or (c) defer all new permission fields until a public consumer materializes. Document the chosen path in Scope/Assumptions and call out the wire-break explicitly in Risks.
  - Question: Which path are we taking for the new `permissions.*` and `outcomes.*` fields: hard-cutover extension of v1, sibling authority packet, or defer until a real consumer needs them?
  - Recommended answer: Defer the new `permissions.*` and `outcomes.*` fields until at least one in-tree consumer requires them; until then, document the intended field names in docs only, keep v1 wire shape untouched, and let `proposal_kind` carry the namespacing.
  - If unanswered: Default to deferral: no v1 schema changes in this spec; new field names are documented but not added to TS/Rust schemas.
- [high/blocks approval] `harden-2` design_challenge - Promotion decision (`operational_proposal` public vs internal) is deferred to Phase 1, leaving every downstream phase conditionally scoped.
  - Status: open
  - Grounded in: spec_gap:objectives
  - Evidence: Phase 1 Changes line 2: 'Decide whether `operational_proposal` is promoted as a public schema now or remains an internal helper until a package consumer needs it.' Phase 2 Objective: 'Add the generic proposal contract only if Phase 1 requires a public schema'. Phase 3 Objective: 'Keep Rust contract types... aligned with the TypeScript/source contract decision'. Phase 2 Changes still unconditionally list 'Add fixture cases for proposal allowed, proposal denied, missing source thread...'. This branch is the central architectural decision of the spec.
  - Recommendation: Resolve the promotion question during harden. Recommend keeping `operational_proposal` as an internal helper/docs-only contract until a concrete consuming product (e.g., `nitrosend-operational-intelligence-integration-v1`) needs a stable wire shape, and re-scope Phase 2/3 to docs + boundary tests only. If a public schema is chosen now, rewrite Phase 2 Changes to be unconditional.
  - Question: Are we promoting `operational_proposal` to a public TS/Rust schema in this spec, or is this spec docs-only with the schema deferred to a consumer-driven follow-up?
  - Recommended answer: Docs-only: define the generic proposal shape and authority taxonomy in docs and acceptance fixtures only; defer public TS/Rust schema promotion until a concrete consumer (likely nitrosend integration) requires a stable wire shape.
  - If unanswered: Default to docs-only; Phase 2/3 become parity verification of existing surfaces; no new packet schema is registered.
- [medium/blocks approval] `harden-3` acceptance_timing - Phase 1 acceptance is trivially satisfied because the spec file itself contains every required token.
  - Status: open
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac1
  - Evidence: p1_ac1 greps `docs .scafld/specs/drafts/runx-operational-contracts-v1.md` for tokens including `operational_proposal`, `proposal_kind`, `owner_route_id`, `evidence_refs`, `human_gate`. Ripgrep on the repo today shows all those tokens already exist only in the draft spec itself (the spec is the sole match for `operational_proposal` and `prepare_proposal`). The gate passes today without any doc being written.
  - Recommendation: Drop the spec file from the grep path; assert tokens exist in `docs/` only. Alternatively, require a specific doc file (e.g., `docs/operational-intelligence.md`) to contain a named heading or fenced example block, so the gate fails until real documentation lands.
  - Question: Should Phase 1 acceptance grep only `docs/`, or require a specific doc filename and named section/heading?
  - Recommended answer: Grep only `docs/`, and additionally require `docs/operational-intelligence.md` to exist with a named section like `## Operational Proposal Contract`.
  - If unanswered: Default: remove the spec path from p1_ac1/p1_ac2 grep targets and require `docs/operational-intelligence.md` to exist.
- [medium/blocks approval] `harden-4` scope_migration - `permissions.merge_pull_request` semantically collides with the existing `auto_merge: Type.Literal(false)` invariant.
  - Status: open
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:166
  - Evidence: operational-policy.ts:166 pins `auto_merge: Type.Literal(false)` and line 168 pins `require_human_merge_gate: Type.Literal(true)`. The spec Assumptions add `permissions.merge_pull_request` as a granted authority. Without an explicit semantic split, `merge_pull_request=true` reads as auto-merge authority and contradicts the invariant. The Rust mirror at operational_policy.rs:456-458 uses const_bool(false)/const_bool(true) and will reject any drift.
  - Recommendation: Clarify in Assumptions that `merge_pull_request` is the authority to *enact* a human-approved merge (gated by `require_human_merge_gate`), not an auto-merge toggle. Note that `auto_merge=false` and `require_human_merge_gate=true` remain locked literal values in v1.
  - Question: Should `permissions.merge_pull_request` be redefined as 'authority to execute a human-approved merge step' so it composes with `require_human_merge_gate=true`?
  - Recommended answer: Yes — document that `merge_pull_request` is conditional on a human approval receipt and never implies auto-merge; `auto_merge=false` remains a hard literal in v1.
  - If unanswered: Default: spec clarifies that `merge_pull_request` is execution-of-approved-merge authority only, never auto-merge.
- [medium/advisory] `harden-5` scope_coherence - Phase 2 Objective is conditional but Phase 2 Changes are unconditional.
  - Status: open
  - Grounded in: spec_gap:phases.phase2
  - Evidence: Phase 2 Objective: 'Add the generic proposal contract only if Phase 1 requires a public schema; otherwise keep implementation in core helpers and docs.' Phase 2 Changes: 'Add fixture cases for proposal allowed, proposal denied, missing source thread, missing redaction status, owner route mismatch, and unapproved send/merge authority.' If Phase 1 decides 'do not promote', the Changes list is no longer reachable but acceptance still runs the same vitest path.
  - Recommendation: Rewrite Phase 2 Changes to branch on the Phase 1 decision: either delete the fixture/test bullets in the 'no-promote' case, or mark them explicitly as 'only if a public schema is promoted'. Better, resolve the promotion question first (see design_challenge issue) and write Phase 2 unconditionally.
- [low/advisory] `harden-6` path_audit - Touchpoint `docs/operational-intelligence.md` is listed but does not yet exist.
  - Status: open
  - Grounded in: code:docs/
  - Evidence: docs/ contains developer-issue-inbox.md, issue-to-pr.md, thread-story-contract.md, thesis.md and others, but no operational-intelligence.md. The spec lists it as the first touchpoint and as part of the Scope contract docs.
  - Recommendation: Either explicitly note in Scope that `docs/operational-intelligence.md` is a new file to be authored in Phase 1, or fold its content into the existing thread-story-contract.md / a renamed file so the touchpoint isn't a phantom path.
- [low/advisory] `harden-7` acceptance_quality - p3_ac3 leak regex is partly imprecise.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac3
  - Evidence: The pattern `C0[A-Z0-9]+` aims at Slack channel IDs, but modern Slack channel IDs are 11 chars like `CABC123DEFG` and may not always start `C0`; conversely `C0` is broad and may match incidental tokens in test fixtures. `/Users/` will match any absolute path comment in source.
  - Recommendation: Tighten to specific anchored patterns (`\bC[0-9A-Z]{8,12}\b` for Slack channels) and limit `/Users/` to fixture JSON values only, or drop it and rely on a dedicated redaction validator.

### round-2

Status: passed
Started: 2026-05-27T16:53:45Z
Ended: 2026-05-27T16:53:45Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 hardening of runx-operational-contracts-v1: all four round-1 blockers are resolved. The spec now explicitly leaves `runx.operational_policy.v1` permissions unchanged (no `permissions.*` extension, `auto_merge: Type.Literal(false)` and `require_human_merge_gate: Type.Literal(true)` stay literal), the promotion decision for `runx.operational_proposal.v1` is committed up-front rather than deferred to Phase 1, and Phase 1 acceptance now gates on the not-yet-existing `docs/operational-intelligence.md` (both `test -f` and per-token grep against that specific file) so the docs gate is functional. Path/command/scope/migration/rollback audits all pass. Five advisory issues remain: Phase 4 doc-update intent is weakly gated (token grep in `docs packages/contracts crates/runx-contracts` is satisfied as soon as Phase 2/3 register the schema), `packages/contracts/src/internal.ts` is omitted from touchpoints despite needing new entries in `RUNX_CONTRACT_IDS`/`RUNX_LOGICAL_SCHEMAS`, the `packages/contracts/src/schemas/operational-policy.ts` touchpoint is now inconsistent with the explicit "do not modify" decision, the p3_ac3 leak regex (`C0[A-Z0-9]+`, raw `/Users/`) remains imprecise from round-1, and no TS-side schema-generation gate (`pnpm contracts:schemas:check`) parallels the Rust `schema_generator_check`. None of these block approval; recommend addressing during build.

Checks:
- path audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Result: passed
  - Evidence: All TS touchpoints exist: packages/contracts/src/schemas/{operational-policy,thread-outbox-provider,context,artifact,agent-act,run-summary,packet-index}.ts and packages/contracts/src/index.ts. All Rust touchpoints exist: crates/runx-contracts/src/{signal,decision,artifact,act,packet_index,schema_artifacts,lib}.rs plus crates/runx-contracts/src/operational_policy/evaluate.rs (verified by glob). Docs touchpoints docs/{developer-issue-inbox,issue-to-pr,thread-story-contract}.md exist; docs/operational-intelligence.md does NOT exist and is intentionally a new file created during Phase 1, gated by p1_ac1's `test -f docs/operational-intelligence.md` clause. fixtures/operational-policy/** exists; fixtures/contracts/operational-proposal/** does not yet exist and will be created in Phase 2 (Rust p3_ac3 guards with `if test -d ...; then`).
- command audit
  - Grounded in: code:package.json:45
  - Result: passed
  - Evidence: All acceptance commands resolve to real entries: `pnpm fixtures:contracts:check`/`fixtures:contracts:keys` exist (package.json:45-47), `pnpm boundary:check` (line 37), `pnpm verify:fast` (line 69), `pnpm exec vitest run --config vitest.fast.config.ts` (vitest.fast.config.ts present at oss root). Rust `schema_generator_check` is a real test module (crates/runx-contracts/tests/schema_generator_check.rs). `scafld validate` is the canonical validator. p3_ac3 shell guard `sh -c 'if test -d fixtures/contracts/operational-proposal; then ! rg ...; fi'` is well-formed and degrades safely before fixtures land.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Result: passed
  - Evidence: Round-1 blocker resolved. Spec Assumptions now state explicitly: '`runx.operational_policy.v1` is closed and remains unchanged in this spec: `permissions.auto_merge` stays literal `false`, `permissions.require_human_merge_gate` stays literal `true`, and no `permissions.*` or `outcomes.*` fields are added here.' This is consistent with the existing closed `automationPermissionsSchema` at operational-policy.ts:164-171 (`additionalProperties: false`, `auto_merge: Type.Literal(false)`, `require_human_merge_gate: Type.Literal(true)`) and the Rust mirror at operational_policy.rs:265-271 (`#[serde(deny_unknown_fields)]`). Proposal authority is admitted via existing operationalPolicyActions enum (operational-policy.ts:23-32: `reply-only`, `issue-intake`, `work-plan`, `issue-to-pr`, `manual-review`, `pr-review`, `pr-fix-up`, `merge-assist`). No `.v2` alias is introduced, satisfying CLAUDE.md `public_api_stable`/`no_legacy_code`.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac1
  - Result: passed
  - Evidence: Round-1 harden-3 resolved: p1_ac1 and p1_ac2 now `test -f docs/operational-intelligence.md` first and then grep only that file (not `docs .scafld/specs/...`), so the Phase 1 gate genuinely fails until the new doc is authored. Verified `docs/operational-intelligence.md` is absent today and no other doc contains `operational_proposal`/`proposal_kind`/`owner_route_id`/`evidence_refs` (rg confirmed only docs/issue-to-pr.md matches `human_gate` as substring of `human_gate_pending`, which is irrelevant since p1_ac1 is scoped to the new file). Phases 2/3 commands are runnable only after their changes land; p3_ac3 leak grep gracefully no-ops until fixtures dir exists.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback explicitly bundles schemas, fixtures, generated artifacts, exports, Rust parity types, and docs as one revert unit; forbids compatibility aliases for the withdrawn schema (matching CLAUDE.md `no_legacy_code`); and walls off provider adapters, runtime runners, consuming product policy, and live Slack/Sentry/GitHub data from rollback (matching the OSS/cloud and contracts/runtime boundary). Repair is credible because the change is contract-layer only — no external state mutation.
- design challenge
  - Grounded in: spec_gap:objectives
  - Result: passed
  - Evidence: Round-1 harden-2 (deferred promotion decision) is resolved. Summary now commits: 'promotes one public proposal packet now: `runx.operational_proposal.v1`'. Phases 2 and 3 unconditionally add TS and Rust schemas (no more 'only if Phase 1 requires...' branching). Phase 2 Changes are consistent with this objective. The Deviations section acknowledges the deliberate reopening of the contract decision around the generic proposal shape. The architectural tradeoff (promoting a generic packet vs reusing existing `runx.act.v1`/`runx.decision.v1`) is noted but not blocking — operator has made the call and limited blast radius to one packet with namespaced `proposal_kind`, preserving the v1 policy invariants. Self-Eval target bar (9.5/10 contract quality) and the explicit ban on domain-specific packet families bound the risk.

Issues:
- [medium/advisory] `harden-1` acceptance_quality - Phase 4 acceptance does not actually enforce that docs were updated; tokens are satisfied as soon as Phase 2/3 register schemas.
  - Status: open
  - Grounded in: spec_gap:phases.phase4.acceptance.p4_ac1
  - Evidence: p4_ac1 runs `for token in operational_proposal proposal_kind proposed_action evidence_refs owner_route_id; do rg -n "$token" docs packages/contracts crates/runx-contracts >/dev/null || exit 1; done`. The grep root is a single search space across docs/ + packages/contracts/ + crates/runx-contracts/, so the moment Phase 2/3 add the TS and Rust schemas, every token resolves inside `packages/contracts/src/schemas/` and `crates/runx-contracts/src/` regardless of whether `docs/` was touched. `evidence_refs` and `owner_route_id` already exist in the schemas today (rg confirmed). Phase 4's actual objective ('Update docs with proposal examples, authority model, redaction rules...') is not enforced.
  - Recommendation: Split p4_ac1 into two gates: (a) per-token grep restricted to `packages/contracts crates/runx-contracts` to confirm parity, and (b) a docs-only check that requires a named heading or fenced example block in `docs/operational-intelligence.md` (e.g. `## Authority Model` or a fenced ```yaml schema: runx.operational_proposal.v1 block) so the docs intent of Phase 4 fails until real documentation lands.
  - Question: Should Phase 4 acceptance assert specific named headings or example blocks in `docs/operational-intelligence.md` separately from the schema parity grep?
  - Recommended answer: Yes — split into a parity-token grep over `packages/contracts crates/runx-contracts` and a docs-content grep that requires named headings/example blocks in `docs/operational-intelligence.md` and a mapping table in `docs/developer-issue-inbox.md`.
  - If unanswered: Default: add a second acceptance line that greps `docs/operational-intelligence.md` for a `## Operational Proposal Contract` heading and a fenced example block.
- [low/advisory] `harden-2` scope_completeness - Touchpoints omit `packages/contracts/src/internal.ts`, which must register the new schema id and logical name.
  - Status: open
  - Grounded in: code:packages/contracts/src/internal.ts:11
  - Evidence: packages/contracts/src/internal.ts:11-58 defines `RUNX_CONTRACT_IDS` and 60+ defines `RUNX_LOGICAL_SCHEMAS`. Every promoted v1 packet schema is registered here (operationalPolicy is at internal.ts:57 / 60+). Adding `runx.operational_proposal.v1` requires a corresponding entry, but internal.ts is not listed under Touchpoints or Scope (lines 158-178 of the spec).
  - Recommendation: Add `packages/contracts/src/internal.ts` to the Touchpoints list (and to Scope under TypeScript contract work) so the registration step is explicit and reviewable.
- [low/advisory] `harden-3` scope_coherence - `packages/contracts/src/schemas/operational-policy.ts` is listed as a touchpoint but the spec explicitly forbids modifying it in this child.
  - Status: open
  - Grounded in: spec_gap:scope.touchpoints
  - Evidence: Assumptions state: '`runx.operational_policy.v1` is closed and remains unchanged in this spec'. Phase 3 Changes: 'Do not change operational-policy v1 permissions in this spec.' Yet operational-policy.ts and operational_policy/evaluate.rs are both listed under Touchpoints (lines 163, 171) and Scope (lines 78, 87). Listing files that will not be edited dilutes the touchpoint signal used by reviewers.
  - Recommendation: Either remove these files from Touchpoints/Scope, or annotate them as 'referenced for invariant verification, not modified'.
- [low/advisory] `harden-4` acceptance_quality - Round-1 advisory carried forward: p3_ac3 leak regex includes a too-broad Slack-channel pattern and a raw `/Users/` substring.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac3
  - Evidence: p3_ac3 uses pattern `xox[baprs]-|BEGIN .*PRIVATE KEY|url_private_download|raw_payload|/Users/|C0[A-Z0-9]+`. Modern Slack channel IDs are 11 chars like `CABC123DEFG` and do not always begin `C0`; `C0[A-Z0-9]+` is simultaneously too narrow (misses post-C0 IDs) and too broad (could collide with unrelated tokens in fixtures). `/Users/` will match any absolute-path string comment in source rather than only fixture JSON values.
  - Recommendation: Tighten Slack channel pattern to `\bC[0-9A-Z]{8,12}\b`, scope `/Users/` to fixture JSON values only (e.g. limit the rg paths to `fixtures/contracts/operational-proposal`), and consider deferring to a dedicated redaction validator script if one exists.
- [low/advisory] `harden-5` acceptance_coverage - No TS-side schema-generation check parallels the Rust `schema_generator_check`.
  - Status: open
  - Grounded in: code:package.json:21
  - Evidence: package.json:21 exposes `contracts:schemas:check` (= `tsx scripts/generate-contract-schemas.ts --check`), which is the canonical TS schema-emission gate. p3_ac2 runs Rust `schema_generator_check` but no phase acceptance runs `pnpm contracts:schemas:check`, leaving TS schema drift detectable only via vitest assertions that may not cover the generated $id/$schema artifacts.
  - Recommendation: Add `pnpm contracts:schemas:check` to Phase 2 acceptance (e.g. as a new p2_ac4) so TS schema emission for the new proposal contract is verified on every implementation pass and on completion review.

### round-3

Status: passed
Started: 2026-05-27T17:14:16Z
Ended: 2026-05-27T17:14:16Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-3 harden of runx-operational-contracts-v1: round-2 left this spec passing with five advisories. Verified against the current draft, the spec materially addresses four of those five — `packages/contracts/src/internal.ts` is now in Touchpoints (line 178), `packages/contracts/src/schemas/operational-policy.ts` and `crates/runx-contracts/src/operational_policy/evaluate.rs` are annotated "invariant verification only" (lines 170-171, 180), p2_ac5 adds `pnpm contracts:schemas:check` to parallel the Rust schema_generator_check, and p3_ac3's Slack channel regex tightened to `\bC[0-9A-Z]{8,12}\b`. The remaining round-2 advisory (p4_ac1 parity grep is satisfied as soon as Phase 2/3 register schemas) is effectively neutralized by p4_ac2, which requires named headings (`## Operational Proposal Contract`, `## Authority Model`) plus per-field tokens inside the new `docs/operational-intelligence.md`. Path, command, scope/migration, acceptance timing, rollback, and design checks all pass: `runx.operational_policy.v1` remains literally unchanged (auto_merge=Type.Literal(false) at operational-policy.ts:166, require_human_merge_gate=Type.Literal(true) at :168, additionalProperties:false at :170; Rust mirror at operational_policy.rs uses deny_unknown_fields) and the promotion decision for `runx.operational_proposal.v1` is committed up-front in Summary lines 27-29. Four advisories carry forward for the build phase: p3_ac3 still includes a raw `/Users/` substring with no path scoping, the proposal/escalation field surface in Objectives lists severity/urgency/suspected_area only inside prose (not the canonical field list), the conceptual overlap with `runx.decision.v1::DecisionChoice::Escalate` and `runx.act.v1` is not documented as a rationale boundary, and p4_ac1 parity grep is technically over-broad but is fully backed by p4_ac2's docs-heading gate. None block approval.

Checks:
- path audit
  - Grounded in: code:packages/contracts/src/internal.ts:11
  - Result: passed
  - Evidence: All TS touchpoints exist: packages/contracts/src/schemas/{operational-policy,thread-outbox-provider,context,artifact,agent-act,run-summary,packet-index}.ts (verified by glob) and packages/contracts/src/{internal,index}.ts. All Rust touchpoints exist: crates/runx-contracts/src/{signal,decision,artifact,act,packet_index,schema_artifacts,lib}.rs plus crates/runx-contracts/src/operational_policy/evaluate.rs. Doc touchpoints docs/{developer-issue-inbox,issue-to-pr,thread-story-contract}.md exist; docs/operational-intelligence.md is intentionally absent today and is the new file gated by p1_ac1's `test -f docs/operational-intelligence.md` clause. fixtures/operational-policy/** exists; fixtures/contracts/operational-proposal/** does not yet exist and is created in Phase 2 (Rust p3_ac3 guards with `if test -d ...; then`). Round-2 advisory about internal.ts is resolved — internal.ts is now listed at spec line 178.
- command audit
  - Grounded in: code:package.json:21
  - Result: passed
  - Evidence: All acceptance commands resolve to real entries: `pnpm contracts:schemas:check` is at package.json:21 (newly added as p2_ac5 to parallel Rust schema_generator_check, resolving round-2 advisory). `pnpm fixtures:contracts:check`/`fixtures:contracts:keys` at package.json:45-47, `pnpm boundary:check` at line 37, `pnpm verify:fast` at line 69. `pnpm exec vitest run --config vitest.fast.config.ts` is the canonical fast-lane test command (test:fast at line 35). Rust `schema_generator_check` is grounded in crates/runx-contracts/tests/integration.rs. p3_ac3 shell guard `sh -c 'if test -d ...; then ! rg ...; fi'` is well-formed.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Result: passed
  - Evidence: operational-policy.ts:164-171 keeps `automationPermissionsSchema` closed (additionalProperties:false), with auto_merge:Type.Literal(false) at :166 and require_human_merge_gate:Type.Literal(true) at :168. Rust mirror at crates/runx-contracts/src/operational_policy.rs preserves deny_unknown_fields. The spec Assumptions (lines 150-153) explicitly state: `runx.operational_policy.v1` is closed and remains unchanged in this spec; `permissions.auto_merge` stays literal `false`, `permissions.require_human_merge_gate` stays literal `true`, and no `permissions.*` or `outcomes.*` fields are added here. Touchpoints line 170 and 180 annotate operational-policy.ts and evaluate.rs as `invariant verification only; no v1 permission edits`, addressing round-2 harden-3 advisory. No `.v2` alias is introduced; the operationalPolicyActions enum at operational-policy.ts:23-32 (`reply-only`, `issue-intake`, `work-plan`, `issue-to-pr`, `manual-review`, `pr-review`, `pr-fix-up`, `merge-assist`) continues to admit proposal preparation.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac1
  - Result: passed
  - Evidence: p1_ac1 (`test -f docs/operational-intelligence.md && for token in runx.operational_proposal.v1 proposal_kind owner_route_id evidence_refs human_gate; do rg -n "$token" docs/operational-intelligence.md ...`) and p1_ac2 grep only `docs/operational-intelligence.md`, not the spec file path. Verified docs/operational-intelligence.md is absent today (glob returned no match). Phase 2 and 3 commands are reachable only after their changes land; p3_ac3 leak grep gracefully no-ops via `if test -d ...; then`. Round-1 trivial-pass blocker remains resolved. p4_ac2 additionally pins `## Operational Proposal Contract`, `## Authority Model`, `source_thread_locator`, `github_issue_url`, `github_pr_url`, `human_gate`, `final_outcome` in the new doc, which gives Phase 4 a real docs-content gate.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback section (lines 349-356) explicitly bundles proposal schemas, fixtures, generated artifacts, exports, Rust parity types, and docs as one revert unit; forbids compatibility aliases for the withdrawn schema (matching CLAUDE.md `no_legacy_code` and `public_api_stable` invariants); and walls off provider adapters, runtime runners, consuming product policy, and live Slack/Sentry/GitHub data from rollback (matching the OSS/cloud and contracts/runtime boundary documented in oss/CLAUDE.md). Repair is credible because the change is contract-layer only — no external state mutation.
- design challenge
  - Grounded in: code:crates/runx-contracts/src/decision.rs:9
  - Result: passed
  - Evidence: The promotion decision is committed up-front in Summary (lines 27-29): `promotes one public proposal packet now: runx.operational_proposal.v1`. Phases 2 and 3 unconditionally add TS and Rust schemas; no `only if Phase 1 requires...` branching remains. Architectural overlap exists: decision.rs:9 already defines DecisionChoice::Escalate, decision.rs:33 has DecisionJustification with evidence_refs, and act.rs:26 has Intent with success_criteria/derived_from — so a proposal can in principle be composed from existing Decision + Act packets. Operator has accepted this overlap as worthwhile because (a) a proposal needs a single addressable identity for story/outbox linkage, (b) `proposal_kind` carries product-namespaced metadata without widening core action variants, and (c) authority gating lives on the proposal packet rather than expanding closed v1 permissions. Risk is bounded: exactly one generic packet, domain-specific families remain explicitly out of scope (lines 115-116), and the Self-Eval target (line 376) commits to one generic proposal/action boundary with no product-specific leakage.

Issues:
- [low/advisory] `harden-1` acceptance_quality - p3_ac3 leak regex still includes an unscoped `/Users/` substring; round-1/round-2 advisory carried forward.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac3
  - Evidence: Phase 3 p3_ac3 pattern: `xox[baprs]-|BEGIN .*PRIVATE KEY|url_private_download|raw_payload|/Users/|\bC[0-9A-Z]{8,12}\b`. The Slack channel pattern was tightened to `\bC[0-9A-Z]{8,12}\b` (good — addresses one half of the round-2 advisory). However, `/Users/` will still match incidental absolute paths in any source comment under `packages/contracts/src/schemas` or `crates/runx-contracts/src`, not just fixture JSON values. The grep paths include source dirs, so the false-positive surface remains.
  - Recommendation: During Phase 3 build, either scope the `/Users/` portion of the leak pattern to `fixtures/contracts/operational-proposal` only (split the rg into two invocations with different patterns/paths), or replace it with a dedicated redaction validator that targets JSON values only. Document the residual false-positive risk in Risks if kept as-is.
  - Question: Should p3_ac3 split into two rg invocations — one for source-dirs with a path-free pattern and one for the fixtures dir with the `/Users/` check — to remove false-positive risk?
  - Recommended answer: Yes — split into source-scope (xox/BEGIN PRIVATE KEY/url_private_download/raw_payload/Slack channel) and fixtures-scope (adds `/Users/`) during Phase 3 build.
  - If unanswered: Default: accept the current pattern as advisory; revisit during build if false positives appear.
- [medium/advisory] `harden-2` scope_completeness - Escalation proposal_kind requires severity/urgency/suspected_area but those fields are not in the canonical generic field list — risk of ad-hoc shape inside `proposal_kind` metadata.
  - Status: open
  - Grounded in: spec_gap:objectives
  - Evidence: Objectives line 61-63 commit: `Preserve escalation as a first-class proposal kind. The generic contract must support proposal_kind: escalation with severity, owner route, evidence, suspected area, urgency, and exact human decision required.` But the canonical field list at lines 45-59 contains `decision summary and rationale`, `evidence refs`, `owner route id`, `proposed action`, `required human gate`, `confidence/risks/caveats`, `public summary` — and does not list `severity`, `urgency`, or `suspected_area`. Without an explicit decision, these will either (a) live in a typed-but-optional generic field, (b) live inside a freeform `proposal_kind_metadata` blob, or (c) leak into the public summary as prose. Each option has different parity/redaction implications across TS and Rust.
  - Recommendation: Before Phase 2 build, decide whether escalation-specific fields (`severity`, `urgency`, `suspected_area`) are: (1) optional top-level fields on the generic packet (typed in TS/Rust), (2) namespaced inside a `proposal_kind_metadata` map keyed by `proposal_kind`, or (3) carried only via existing `risks`/`missing_context`/`public_summary`. Document the chosen shape in Objectives so Phase 2/3 fixtures and parity tests can lock it in.
  - Question: Are `severity`, `urgency`, and `suspected_area` (a) optional top-level fields on `runx.operational_proposal.v1`, (b) namespaced metadata under a generic `proposal_kind_metadata` map, or (c) carried implicitly via existing risks/caveats/public_summary?
  - Recommended answer: Option (b): introduce a typed but generic `proposal_kind_metadata` shape (or per-kind discriminated union) so escalation can carry severity/urgency/suspected_area without widening the top-level packet — keeps the core generic and avoids domain-specific top-level fields.
  - If unanswered: Default: namespaced metadata. Document escalation-kind fields under a discriminated `proposal_kind_metadata` shape and add a fixture case in Phase 2.
- [low/advisory] `harden-3` design_clarity - Rationale for adding `runx.operational_proposal.v1` alongside existing `runx.decision.v1` (which already has `DecisionChoice::Escalate`) and `runx.act.v1` is not documented.
  - Status: open
  - Grounded in: code:crates/runx-contracts/src/decision.rs:9
  - Evidence: decision.rs:9 defines `DecisionChoice::{Open, Continue, SpawnChild, Escalate, Defer, Close, Decline, Monitor}`. decision.rs:33 has `DecisionJustification { summary, evidence_refs }`. act.rs:26 has `Intent { purpose, legitimacy, success_criteria, constraints, derived_from }`. The proposed `operational_proposal.v1` overlaps with all three (decision summary/rationale ↔ DecisionJustification; proposed action ↔ Decision.proposed_intent / Act.Intent; escalation as proposal_kind ↔ DecisionChoice::Escalate). The spec's Deviations section (lines 379-383) acknowledges reopening the contract decision but does not capture *why* a new packet beats composing existing Decision + Act packets.
  - Recommendation: In Phase 4 docs (docs/operational-intelligence.md), include a short `## Boundary vs Decision and Act` section explaining: (1) a proposal needs single addressable identity for story/outbox linkage, (2) `proposal_kind` carries product-namespaced metadata without expanding core enums, (3) authority gating lives on the proposal packet rather than widening v1 policy. This makes the architectural choice defensible at review and prevents future re-litigation.
  - If unanswered: Default: add the `## Boundary vs Decision and Act` section to docs/operational-intelligence.md during Phase 4.
- [low/advisory] `harden-4` acceptance_quality - p4_ac1 parity grep is technically satisfied as soon as Phase 2/3 register schemas; docs-update intent is covered by p4_ac2 but the duplication is confusing.
  - Status: open
  - Grounded in: spec_gap:phases.phase4.acceptance.p4_ac1
  - Evidence: p4_ac1 runs `for token in operational_proposal proposal_kind proposed_action evidence_refs owner_route_id; do rg -n "$token" docs packages/contracts crates/runx-contracts ...`. Verified via grep: `evidence_refs` and `owner_route_id` already exist in the repo (operational-policy.ts, target_runner.rs, etc.), and `operational_proposal`/`proposal_kind`/`proposed_action` will resolve inside packages/contracts/src/schemas/ as soon as Phase 2 lands the schema. So the gate passes without docs/ being touched. p4_ac2 carries the real docs-content gate (named headings in docs/operational-intelligence.md). The duplication makes p4_ac1's intent ambiguous to reviewers.
  - Recommendation: Either tighten p4_ac1 to require each token also appear inside `docs/` specifically (e.g., a second per-token grep against `docs/`), or rename the assertion to clarify it is a parity grep across the three surfaces and not a docs-update gate. Keep p4_ac2 as the docs-content gate.
  - If unanswered: Default: keep p4_ac1 as parity grep, add a comment in the spec noting that p4_ac2 is the docs-content gate.

### round-4

Status: passed
Started: 2026-05-27T17:48:08Z
Ended: 2026-05-27T17:48:08Z
Verdict: pass
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-4 harden of runx-operational-contracts-v1: verdict pass. All round-1/2/3 blockers remain resolved on re-verification against the live draft. Specifically: (1) `runx.operational_policy.v1` is left literally unchanged — Assumptions (spec lines 150-153) commit to no `permissions.*` or `outcomes.*` additions, consistent with the closed TS schema at packages/contracts/src/schemas/operational-policy.ts:164-171 (auto_merge:Type.Literal(false), require_human_merge_gate:Type.Literal(true), additionalProperties:false) and the Rust mirror at crates/runx-contracts/src/operational_policy.rs:265-271 (deny_unknown_fields). (2) The promotion decision for `runx.operational_proposal.v1` is committed up-front in Summary (lines 27-29) — Phases 2/3 unconditionally add TS and Rust schemas with no Phase-1 branching. (3) Phase 1 acceptance now gates on docs/operational-intelligence.md (test -f + per-token grep restricted to that single file), confirmed absent today (Glob returned no match), so the docs gate genuinely fails until Phase 1 lands content. (4) Touchpoints now include packages/contracts/src/internal.ts (spec line 178) and annotate operational-policy.ts and evaluate.rs as `invariant verification only; no v1 permission edits` (lines 170, 180). (5) p2_ac5 adds `pnpm contracts:schemas:check` to parallel the Rust schema_generator_check (verified at package.json:21 and crates/runx-contracts/tests/schema_generator_check.rs:6). (6) Rollback bundles schemas/fixtures/generated artifacts/exports/Rust parity/docs as one revert unit and walls off provider adapters and live external data, matching CLAUDE.md `no_legacy_code`/`public_api_stable` invariants. Four advisories carry forward from round-3 for the build phase, all non-blocking: (a) p3_ac3's `/Users/` substring is still grepped across source dirs (packages/contracts/src/schemas and crates/runx-contracts/src) where any absolute-path comment could trigger a false positive; (b) escalation-kind fields (severity, urgency, suspected_area) appear in Objectives prose at lines 61-63 but not in the canonical generic field list at lines 45-59, so their carrier (top-level optional vs proposal_kind_metadata) is undecided; (c) architectural overlap with the existing `runx.decision.v1` (DecisionChoice::Escalate at crates/runx-contracts/src/decision.rs:9 and DecisionJustification with evidence_refs at :33) and `runx.act.v1` Intent (act.rs:26) is not documented as a rationale-boundary section; (d) p4_ac1 parity grep is over-broad — verified via repo grep that `evidence_refs` and `owner_route_id` already exist in packages/contracts/src/schemas/operational-policy.ts and crates/runx-runtime/src/execution/target_runner.rs, so the gate is satisfied as soon as Phase 2/3 register the schema, though p4_ac2's named-heading gate against docs/operational-intelligence.md fully compensates. None of these block approval. Recommend the operator addresses harden-2 (escalation field carrier) during Phase 2 design and folds harden-3 (boundary-vs-Decision-and-Act doc section) into Phase 4 docs.

Checks:
- path audit
  - Grounded in: code:packages/contracts/src/internal.ts:11
  - Result: passed
  - Evidence: All declared TS touchpoints verified to exist via Read/Glob: packages/contracts/src/schemas/{operational-policy,thread-outbox-provider,context,artifact,agent-act,run-summary,packet-index}.ts and packages/contracts/src/{internal,index}.ts. internal.ts confirmed at line 11 to define RUNX_CONTRACT_IDS and at line 60 RUNX_LOGICAL_SCHEMAS — registration site for the new proposal packet. All declared Rust touchpoints verified: crates/runx-contracts/src/{signal,decision,artifact,act,packet_index,schema_artifacts,lib}.rs plus crates/runx-contracts/src/operational_policy/evaluate.rs (via Glob). Doc touchpoints docs/{developer-issue-inbox,issue-to-pr,thread-story-contract}.md exist (Glob confirmed). docs/operational-intelligence.md is intentionally absent today (Glob returned no match) and is the new file gated by p1_ac1's `test -f docs/operational-intelligence.md`. fixtures/operational-policy/** exists; fixtures/contracts/operational-proposal/** does not yet exist and is created in Phase 2 (Rust p3_ac3 guards with `if test -d ...; then`). Round-2 advisory about internal.ts is resolved — internal.ts is listed at spec line 178.
- command audit
  - Grounded in: code:package.json:21
  - Result: passed
  - Evidence: All acceptance commands resolve to real entries verified via Grep: `pnpm contracts:schemas:check` at package.json:21 (added as p2_ac5 to parallel Rust schema_generator_check, resolving round-2 advisory). `pnpm boundary:check` at line 37, `pnpm fixtures:contracts:check` at line 45, `pnpm fixtures:contracts:keys` at line 47, `pnpm verify:fast` at line 69. `pnpm exec vitest run --config vitest.fast.config.ts` is the canonical fast-lane test command. Rust `schema_generator_check` is grounded in crates/runx-contracts/tests/schema_generator_check.rs:6 (real test module). `scafld validate` is the canonical validator. p3_ac3 shell guard `sh -c 'if test -d fixtures/contracts/operational-proposal; then ! rg ...; fi'` is well-formed and degrades safely before fixtures land.
- scope/migration audit
  - Grounded in: code:packages/contracts/src/schemas/operational-policy.ts:164
  - Result: passed
  - Evidence: Verified via Read: operational-policy.ts:164-171 keeps `automationPermissionsSchema` closed (additionalProperties:false at :170), with auto_merge:Type.Literal(false) at :166 and require_human_merge_gate:Type.Literal(true) at :168. Rust mirror at crates/runx-contracts/src/operational_policy.rs:265-271 preserves deny_unknown_fields with bool fields (struct OperationalPolicyAutomationPermissions). The spec Assumptions (lines 150-153) explicitly state `runx.operational_policy.v1` is closed and remains unchanged in this spec — no `permissions.*` or `outcomes.*` fields added. Touchpoints line 170 and 180 annotate operational-policy.ts and evaluate.rs as `invariant verification only; no v1 permission edits`, addressing round-2 harden-3. No `.v2` alias is introduced. The operationalPolicyActions enum at operational-policy.ts:23-32 (`reply-only`, `issue-intake`, `work-plan`, `issue-to-pr`, `manual-review`, `pr-review`, `pr-fix-up`, `merge-assist`) continues to admit proposal preparation — confirming `proposal preparation is authorized by an admitted existing action lane` (spec line 154-158). Satisfies CLAUDE.md `public_api_stable` / `no_legacy_code`.
- acceptance timing audit
  - Grounded in: spec_gap:phases.phase1.acceptance.p1_ac1
  - Result: passed
  - Evidence: p1_ac1 and p1_ac2 grep only `docs/operational-intelligence.md` after `test -f` — verified the file does not yet exist (Glob returned no match), so Phase 1 gate genuinely fails until the new doc is authored. Phase 2/3 commands are reachable only after their changes land; p3_ac3 leak grep gracefully no-ops via `if test -d ...; then`. p4_ac2 additionally pins `## Operational Proposal Contract`, `## Authority Model`, `source_thread_locator`, `github_issue_url`, `github_pr_url`, `human_gate`, `final_outcome` in the new doc — confirmed `human_gate` token does not currently appear in any docs/ file as a discrete token (only `human_gate_pending` substring in docs/issue-to-pr.md:243, which would not match `human_gate` as a literal pattern but ripgrep matches substring by default — note `rg "human_gate"` would match `human_gate_pending` too; this is a minor false-positive surface that is mitigated because the p4_ac2 grep root is the new operational-intelligence.md file only). Round-1 trivial-pass blocker remains resolved.
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback section (lines 349-356) explicitly bundles proposal schemas, fixtures, generated artifacts, exports, Rust parity types, and docs as one revert unit; forbids compatibility aliases for the withdrawn schema (matching CLAUDE.md `no_legacy_code` and `public_api_stable` invariants); walls off provider adapters, runtime runners, consuming product policy, and live Slack/Sentry/GitHub data from rollback (matching the OSS/cloud and contracts/runtime boundary documented in oss/CLAUDE.md). Repair is credible because the change is contract-layer only — no external state mutation. The hard-cut posture (no aliases on withdrawal) aligns with CLAUDE.md's prohibition on `.v2` aliases for governed wire shapes.
- design challenge
  - Grounded in: code:crates/runx-contracts/src/decision.rs:9
  - Result: passed
  - Evidence: The promotion decision is committed up-front in Summary (lines 27-29). Phases 2 and 3 unconditionally add TS and Rust schemas. Architectural overlap exists and is verified via Read: decision.rs:9 already defines DecisionChoice::Escalate, decision.rs:33 defines DecisionJustification with evidence_refs, act.rs:26 defines Intent with purpose/legitimacy/success_criteria/derived_from — so a proposal could in principle be composed from existing Decision + Act packets. Operator has accepted this overlap as worthwhile because (a) a proposal needs single addressable identity for story/outbox linkage, (b) `proposal_kind` carries product-namespaced metadata without widening core action variants, and (c) authority gating lives on the proposal packet rather than expanding closed v1 permissions. Risk is bounded: exactly one generic packet, domain-specific families remain explicitly out of scope (lines 115-116), and Self-Eval target (line 376) commits to one generic proposal/action boundary with no product-specific leakage. Not blocking; documenting the rationale-boundary is captured as advisory harden-3.

Issues:
- [low/advisory] `harden-1` acceptance_quality - p3_ac3 leak regex still includes an unscoped `/Users/` substring — round-1/2/3 advisory carried forward.
  - Status: open
  - Grounded in: spec_gap:phases.phase3.acceptance.p3_ac3
  - Evidence: Phase 3 p3_ac3 pattern: `xox[baprs]-|BEGIN .*PRIVATE KEY|url_private_download|raw_payload|/Users/|\bC[0-9A-Z]{8,12}\b`. The Slack channel pattern was tightened to `\bC[0-9A-Z]{8,12}\b` in round-3 (good). However, `/Users/` will still match incidental absolute paths in any source comment under `packages/contracts/src/schemas` or `crates/runx-contracts/src`, not just fixture JSON values. The grep paths include source dirs, so the false-positive surface remains.
  - Recommendation: During Phase 3 build, split p3_ac3 into two rg invocations: one for source-dirs with patterns that omit `/Users/` (Slack tokens, BEGIN PRIVATE KEY, url_private_download, raw_payload, Slack channel id), and one for `fixtures/contracts/operational-proposal` that adds `/Users/`. Alternatively replace `/Users/` with a dedicated redaction validator targeting JSON values only.
  - Question: Should p3_ac3 split into two rg invocations — one for source-dirs with a path-free pattern and one for the fixtures dir with the `/Users/` check — to remove false-positive risk?
  - Recommended answer: Yes — split into source-scope (xox/BEGIN PRIVATE KEY/url_private_download/raw_payload/Slack channel) and fixtures-scope (adds `/Users/`) during Phase 3 build.
  - If unanswered: Default: accept the current pattern as advisory; revisit during build if false positives appear.
- [medium/advisory] `harden-2` scope_completeness - Escalation proposal_kind requires severity/urgency/suspected_area but those fields are not in the canonical generic field list — risk of ad-hoc shape inside `proposal_kind` metadata.
  - Status: open
  - Grounded in: spec_gap:objectives
  - Evidence: Objectives lines 61-63 commit: `Preserve escalation as a first-class proposal kind. The generic contract must support proposal_kind: escalation with severity, owner route, evidence, suspected area, urgency, and exact human decision required.` But the canonical field list at lines 45-59 contains `decision summary and rationale`, `evidence refs`, `owner route id`, `proposed action`, `required human gate`, `confidence/risks/caveats`, `public summary` — and does not list `severity`, `urgency`, or `suspected_area`. Without an explicit decision, these will either (a) live in a typed-but-optional generic field, (b) live inside a freeform `proposal_kind_metadata` blob, or (c) leak into the public summary as prose. Each option has different parity/redaction implications across TS and Rust.
  - Recommendation: Before Phase 2 build, decide whether escalation-specific fields (`severity`, `urgency`, `suspected_area`) are: (1) optional top-level fields on the generic packet (typed in TS/Rust), (2) namespaced inside a `proposal_kind_metadata` map keyed by `proposal_kind`, or (3) carried only via existing `risks`/`missing_context`/`public_summary`. Document the chosen shape in Objectives so Phase 2/3 fixtures and parity tests can lock it in.
  - Question: Are `severity`, `urgency`, and `suspected_area` (a) optional top-level fields on `runx.operational_proposal.v1`, (b) namespaced metadata under a generic `proposal_kind_metadata` map, or (c) carried implicitly via existing risks/caveats/public_summary?
  - Recommended answer: Option (b): introduce a typed but generic `proposal_kind_metadata` shape (or per-kind discriminated union) so escalation can carry severity/urgency/suspected_area without widening the top-level packet — keeps the core generic and avoids domain-specific top-level fields.
  - If unanswered: Default: namespaced metadata. Document escalation-kind fields under a discriminated `proposal_kind_metadata` shape and add a fixture case in Phase 2.
- [low/advisory] `harden-3` design_clarity - Rationale for adding `runx.operational_proposal.v1` alongside existing `runx.decision.v1` (which already has `DecisionChoice::Escalate`) and `runx.act.v1` is not documented.
  - Status: open
  - Grounded in: code:crates/runx-contracts/src/decision.rs:9
  - Evidence: Verified via Read: decision.rs:9 defines `DecisionChoice::{Open, Continue, SpawnChild, Escalate, Defer, Close, Decline, Monitor}`. decision.rs:33 has `DecisionJustification { summary, evidence_refs }`. act.rs:26 has `Intent { purpose, legitimacy, success_criteria, constraints, derived_from }`. The proposed `operational_proposal.v1` overlaps with all three (decision summary/rationale ↔ DecisionJustification; proposed action ↔ Decision.proposed_intent / Act.Intent; escalation as proposal_kind ↔ DecisionChoice::Escalate). The spec's Deviations section (lines 379-383) acknowledges reopening the contract decision but does not capture *why* a new packet beats composing existing Decision + Act packets.
  - Recommendation: In Phase 4 docs (docs/operational-intelligence.md), include a short `## Boundary vs Decision and Act` section explaining: (1) a proposal needs single addressable identity for story/outbox linkage, (2) `proposal_kind` carries product-namespaced metadata without expanding core enums, (3) authority gating lives on the proposal packet rather than widening v1 policy. This makes the architectural choice defensible at review and prevents future re-litigation.
  - If unanswered: Default: add the `## Boundary vs Decision and Act` section to docs/operational-intelligence.md during Phase 4.
- [low/advisory] `harden-4` acceptance_quality - p4_ac1 parity grep is technically satisfied as soon as Phase 2/3 register schemas; docs-update intent is covered by p4_ac2 but the duplication is confusing.
  - Status: open
  - Grounded in: spec_gap:phases.phase4.acceptance.p4_ac1
  - Evidence: p4_ac1 runs `for token in operational_proposal proposal_kind proposed_action evidence_refs owner_route_id; do rg -n "$token" docs packages/contracts crates/runx-contracts ...`. Verified via Grep: `evidence_refs` and `owner_route_id` already exist in the repo (packages/contracts/src/schemas/operational-policy.ts, crates/runx-runtime/src/execution/target_runner.rs, crates/runx-runtime/src/execution/output_projection.rs, packages/contracts/src/schemas/spine.ts, etc.), and `operational_proposal`/`proposal_kind`/`proposed_action` will resolve inside packages/contracts/src/schemas/ as soon as Phase 2 lands the schema. So the gate passes without docs/ being touched. p4_ac2 carries the real docs-content gate (named headings + per-field tokens in docs/operational-intelligence.md). The duplication makes p4_ac1's intent ambiguous to reviewers.
  - Recommendation: Either tighten p4_ac1 to require each token also appear inside `docs/` specifically (e.g., a second per-token grep against `docs/`), or rename the assertion to clarify it is a parity grep across the three surfaces and not a docs-update gate. Keep p4_ac2 as the docs-content gate.
  - If unanswered: Default: keep p4_ac1 as parity grep, add a comment in the spec noting that p4_ac2 is the docs-content gate.


## Planning Log

- Replaced fixed domain packet families with one generic proposal/action
  contract direction.
