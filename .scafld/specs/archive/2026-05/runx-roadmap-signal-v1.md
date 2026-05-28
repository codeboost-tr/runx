---
spec_version: '2.0'
task_id: runx-roadmap-signal-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-27T16:34:28Z'
status: cancelled
harden_status: in_progress
size: medium
risk_level: medium
---

# Roadmap Signal Aggregation Lane

## Current State

Status: cancelled
Current phase: planning
Next: done
Reason: cancel
Blockers: none
Allowed follow-up command: `none`
Latest runner update: none
Review gate: not_started

## Summary

Build the generic runx lane that aggregates repeated operational signals into
evidence-backed product and roadmap intelligence.

This lane consumes already-admitted, already-redacted operational packets from
support, bug, alert, outreach, manual, and issue-to-PR flows. It does not fetch
Slack, Sentry, support, customer, revenue, or analytics data itself. Provider
adapters and consuming products supply safe source artifacts; runx groups them
into ranked themes, explains the evidence, recommends the next product action,
and emits a compact story/outbox milestone.

The output is a ranked evidence packet, not a chat feed and not automatic
backlog mutation. A consuming product can decide whether to turn a theme into a
docs task, support macro, product issue, work plan, roadmap item, no-action
decision, or monitor state.

Suggested subagent ownership: Wegener.

## Objectives

- Define the minimal roadmap/product signal packet or internal helper needed to
  represent:
  - theme key and human title;
  - product area or `unknown_product_area`;
  - supporting signal ids and artifact refs;
  - dedupe keys, duplicate counts, affected counts, and trend;
  - segment/account/revenue summaries only when already redacted and supplied;
  - confidence, missing-context notes, and contraindications;
  - suggested owner route id, never a person name baked into runx core;
  - recommended action from a fixed taxonomy.
- Add a reusable `roadmap-signal` lane/skill that accepts portable operational
  signals and optional prior theme index state, then emits one deterministic
  `roadmap_signal_report` and one or more grouped themes.
- Group repeated signals by stable dedupe fingerprints, supplied product area,
  normalized theme key, and source evidence rather than opening parallel issues,
  PRs, or Slack root messages.
- Preserve source-thread provenance through artifact refs and story milestones
  without copying raw provider/customer payloads into public markdown.
- Distinguish docs, support/process, product/UX, code defect, work-plan,
  roadmap item, no-action, and monitor recommendations.
- Keep GitHub Projects/backlog export as a non-mutating handoff packet. Any
  actual GitHub Project, issue, or roadmap write belongs to a consuming adapter
  with explicit policy.
- Prove the lane with deterministic fixtures first, then with a sanitized
  Nitrosend dogfood export supplied by the Nitrosend integration spec.
- Keep active runtime/CLI implementation files out of this child unless a later
  approved spec explicitly widens scope.

## Scope

In scope:

- Product/roadmap signal contract decision:
  - use the packet decision from `runx-operational-contracts-v1`;
  - if that child has not landed, keep this implementation as internal
    skill/core helper output and do not promote a public packet from this child.
- New or updated docs:
  - `docs/roadmap-signal.md`;
  - `docs/developer-issue-inbox.md` only for queue/action vocabulary updates;
  - `docs/thread-story-contract.md` only after the story/outbox child has
    established roadmap milestones.
- New generic skill/lane artifacts:
  - `skills/roadmap-signal/SKILL.md`;
  - `skills/roadmap-signal/X.yaml`;
  - harness fixtures and oracle outputs under
    `fixtures/operational-intelligence/roadmap-signal/`.
- Contract and helper touchpoints when justified by the contract child:
  - `packages/contracts/src/schemas/*`;
  - `crates/runx-contracts/src/*`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/core/src/knowledge/index.ts`;
  - related schema generator and conformance tests.
- Tests for grouping, dedupe, redaction, recommendation taxonomy, idempotent
  replay, and story/export rendering.

Out of scope:

- Slack, Sentry, support, CRM, analytics, customer/account, or revenue provider
  hydration.
- Nitrosend channel ids, customer ids, account names, owner names, GitHub
  Projects configuration, or product-specific routing.
- Creating GitHub issues, GitHub Project items, Linear/Jira tickets, PRs, Slack
  messages, customer emails, or roadmap database rows.
- Auto-sending customer/outreach messages.
- Auto-merging PRs or deciding engineering priority without a human/product
  owner.
- Active runtime/CLI changes under `packages/cli/**` or
  `crates/runx-runtime/src/**`.
- Full analytics warehouse behavior, forecasting, scoring models, or dashboard
  UI.

## Dependencies

- Parent program spec:
  - `runx-operational-intelligence-action-layer-v1`.
- Required or closely-related child specs:
  - `runx-operational-contracts-v1` for packet/public-schema decisions;
  - `runx-operational-story-outbox-v1` for milestone/outbox vocabulary;
  - `runx-support-triage-response-v1`,
    `runx-alert-dev-escalation-v1`, and
    `runx-outreach-recommendation-v1` as eventual live signal producers.
- Existing docs and primitives:
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `docs/thread-story-contract.md`;
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-triage/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`;
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/contracts/src/schemas/operational-policy.ts`;
  - `crates/runx-contracts/src/operational_policy/evaluate.rs`.
- Implementation may proceed against static fixtures before all producer lanes
  are complete, but live dogfood must not claim end-to-end coverage until the
  relevant source lanes emit the operational packets defined by the contracts
  child.

## Assumptions

- Source admission and provider hydration happen before this lane. This lane
  sees portable signal packets and artifact refs, not raw Slack/Sentry/support
  payloads.
- Each input signal has a stable `signal_id`, source locator, thread locator
  when publication is allowed, dedupe fingerprint, and redaction status.
- Consuming adapters supply product-area taxonomy and owner-route ids. runx may
  classify `unknown_product_area`, but it must not invent Nitrosend owners,
  channels, customer names, or business priority.
- Customer/account/revenue/segment context is optional and must already be
  summarized/redacted before it enters runx.
- Repeated duplicates increase evidence weight and trend; they do not create
  parallel issues, PRs, source-thread posts, or roadmap rows.
- Public story text is a compact product summary with evidence refs. Full
  signal lists, raw artifacts, command logs, and provider payloads remain in
  receipts/artifacts.
- If a source thread is missing and policy requires reply-in-thread, publication
  fails closed.
- GitHub Projects/backlog export is a handoff artifact only in this child.

## Touchpoints

- Docs:
  - `docs/developer-issue-inbox.md`
  - `docs/thread-story-contract.md`
  - `docs/issue-to-pr.md`
  - new `docs/roadmap-signal.md`
- Skills:
  - `skills/roadmap-signal/SKILL.md`
  - `skills/roadmap-signal/X.yaml`
- Core/contracts when needed:
  - `packages/contracts/src/schemas/*`
  - `packages/contracts/src/index.ts`
  - `crates/runx-contracts/src/signal.rs`
  - `crates/runx-contracts/src/decision.rs`
  - `crates/runx-contracts/src/packet_index.rs`
  - `packages/core/src/knowledge/thread-story.ts`
  - `packages/core/src/knowledge/index.ts`
- Tests/fixtures:
  - `fixtures/operational-intelligence/roadmap-signal/**`
  - `packages/contracts/src/schemas/*.test.ts`
  - `crates/runx-contracts/tests/*`
  - focused core/skill tests for grouping, redaction, and story/export output
- Explicitly excluded from this child:
  - `packages/cli/**`
  - `crates/runx-runtime/src/**`
  - provider adapter credential surfaces
  - Nitrosend repo paths

## Risks

- Roadmap noise. Mitigation: aggregate only admitted signals, require evidence
  refs, and emit ranked themes rather than per-message updates.
- Product-policy leakage into runx. Mitigation: use abstract product areas,
  owner route ids, source ids, and action taxonomy; keep Nitrosend-specific
  mapping in the consuming repo.
- Provider/customer data leakage. Mitigation: fixtures include raw-looking
  support/Sentry/customer content and assert public output contains only safe
  summaries and artifact refs.
- Duplicate issue/PR creation by accident. Mitigation: this lane cannot mutate
  GitHub or Slack; export is a non-mutating handoff packet.
- Weak trend semantics. Mitigation: document an intentionally simple v1 trend
  model: count, recency bucket, duplicate count, and optional supplied affected
  counts; defer scoring sophistication.
- Contract churn before live dogfood. Mitigation: keep helper internal unless
  `runx-operational-contracts-v1` explicitly promotes a public schema.
- Missing producer lanes. Mitigation: deterministic fixtures unblock this lane,
  while live dogfood remains dependent on support/alert/outreach emitters that
  produce the contract-owned operational packets.
- Breaking active runtime work. Mitigation: do not touch active runtime/CLI
  files in this child.

## Owner And Source Policy

- runx owns the generic source and owner fields:
  - `source_id`;
  - `source_kind`;
  - `signal_id`;
  - `thread_locator` when safe publication is allowed;
  - `dedupe_fingerprint`;
  - `product_area`;
  - `owner_route_id`;
  - `recommended_action`.
- Consuming repos own the concrete policy:
  - Slack channel ids and names;
  - Sentry project ids;
  - support queue ids;
  - customer/account/revenue enrichment;
  - GitHub Projects fields;
  - human owner names or GitHub logins;
  - roadmap labels and backlog destinations.
- The lane must deny or mark incomplete any signal that lacks redaction status,
  evidence refs, stable source id, or dedupe fingerprint.
- Source-thread publication must use existing outbox policy:
  `publish_mode=reply`, source thread required when configured, and
  `missing_behavior=fail_closed`.
- Owner routing in public output is by route id and product area. A human or
  product adapter resolves the actual person/team.

## Fixtures And Live Dogfood

Deterministic fixtures must include:

- repeated support questions that roll into one docs/onboarding gap;
- repeated bugs with the same dedupe fingerprint that increase evidence but do
  not create parallel issue recommendations;
- Sentry/system alert noise that recommends monitor/no-action;
- outreach recommendations that roll into an activation or deliverability
  theme without marking a customer message as sent;
- mixed support/bug/alert signals for one product area with trend and suggested
  owner route;
- raw-looking provider/customer fields that must remain behind artifacts;
- missing product area and missing thread locator cases.
- private fixture inputs live under
  `fixtures/operational-intelligence/roadmap-signal/private/`; public expected
  reports live under `fixtures/operational-intelligence/roadmap-signal/public/`.

Live dogfood for this runx child is intentionally credential-free:

- consume a checked-in sanitized Nitrosend export produced by
  `nitrosend-operational-intelligence-integration-v1`;
- replay it through the generic `roadmap-signal` lane;
- verify the report contains ranked themes, safe evidence refs, dedupe counts,
  trend, and recommended actions;
- do not call Slack, Sentry, GitHub Projects, support tools, customer APIs, or
  revenue systems from runx OSS.

## Acceptance

Profile: standard

Validation:
- `scafld harden runx-roadmap-signal-v1 --provider claude`
- `scafld validate runx-roadmap-signal-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- If public TS contracts change:
  `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- If Rust contracts change:
  `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features`
- If contract fixtures change:
  `pnpm fixtures:contracts:check && pnpm fixtures:contracts:keys`
- If harness fixtures change:
  `pnpm fixtures:harness:check`

## Phase 1: Claude Hardening

Status: pending
Dependencies: none

Objective: Challenge the child boundary before implementation.

Changes:
- Run Claude hardening against this draft.
- Reconcile findings with the parent program contract and sibling child specs.
- Confirm that public packet promotion remains owned by
  `runx-operational-contracts-v1`.
- Keep runtime/CLI and provider adapter changes out of scope.

Acceptance:
- [ ] `p1_ac1` command - Contract-promotion boundary is recorded.
  - Command: `sh -c 'spec=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-roadmap-signal-v1.md -print -quit); test -n "$spec" && rg -n "do not promote a public packet|public packet promotion remains owned" "$spec"'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - Spec validates after hardening edits.
  - Command: `scafld validate runx-roadmap-signal-v1`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Contract, Docs, And Fixture Shape

Status: pending
Dependencies: Phase 1

Objective: Define the roadmap signal shape and fixture matrix before lane code.

Changes:
- Document the v1 product/roadmap signal packet or internal helper.
- Define the recommended-action taxonomy:
  - `docs`;
  - `support_macro`;
  - `process`;
  - `product_ux`;
  - `code_issue`;
  - `work_plan`;
  - `roadmap_item`;
  - `monitor`;
  - `no_action`;
  - `manual_review`.
- Add deterministic fixture inputs and expected report outputs.
- Add redaction and no-provider-payload rules.
- Add policy notes for abstract owner routes and source ids.

Acceptance:
- [ ] `p2_ac1` command - Docs mention the action taxonomy and source/owner boundary.
  - Command: `test -f docs/roadmap-signal.md && for token in support_macro roadmap_item owner_route_id source_id; do rg -n "$token" docs/roadmap-signal.md skills/roadmap-signal >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - Contract tests pass when public contracts are touched.
  - Command: `if git diff --name-only -- packages/contracts/src crates/runx-contracts/src | rg -q .; then pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src; else true; fi`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac3` command - Rust contract tests pass when Rust contracts are touched.
  - Command: `if git diff --name-only -- crates/runx-contracts/src | rg -q .; then cargo test --manifest-path crates/Cargo.toml -p runx-contracts --all-features; else true; fi`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac4` command - Contract fixture checks pass when fixtures are added.
  - Command: `if git diff --name-only -- fixtures/contracts packages/contracts crates/runx-contracts | rg -q .; then pnpm fixtures:contracts:check && pnpm fixtures:contracts:keys; else true; fi`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Roadmap Signal Lane

Status: pending
Dependencies: Phase 2

Objective: Implement deterministic grouping, ranking, and reporting.

Changes:
- Add the `roadmap-signal` skill/lane with explicit inputs and outputs.
- Implement grouping by dedupe fingerprint, normalized theme key, product area,
  and evidence refs.
- Implement idempotent replay against optional prior theme index state.
- Emit ranked themes with counts, duplicate count, trend, confidence,
  missing-context notes, suggested owner route, and recommended action.
- Ensure duplicates increase evidence/trend instead of creating parallel
  recommendations.
- Add tests for redaction, missing context, duplicate replay, trend, and
  recommendation taxonomy.

Acceptance:
- [ ] `p3_ac1` command - TypeScript compiles.
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac2` command - Fast tests pass.
  - Command: `pnpm test:fast`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac3` command - Boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac4` command - Public output is free of obvious raw provider/customer payload markers.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/roadmap-signal/public && if rg -n "xox[baprs]-|BEGIN .*PRIVATE KEY|slack\\.com/api|url_private_download|customer_email|raw_payload" fixtures/operational-intelligence/roadmap-signal/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Story, Export, And Dogfood

Status: pending
Dependencies: Phase 3

Objective: Produce safe reviewer-facing output and a non-mutating product handoff.

Changes:
- Add a compact story milestone for `roadmap_signal_recorded` after
  `runx-operational-story-outbox-v1` defines the shared milestone vocabulary.
- Add a non-mutating export/handoff packet for GitHub Projects or product
  backlog adapters.
- Replay a checked-in sanitized Nitrosend dogfood export fixture through the
  generic lane when that export fixture is supplied by the Nitrosend integration
  spec.
- Verify missing source-thread behavior fails closed when publication is
  configured as required.

Acceptance:
- [ ] `p4_ac1` command - Harness fixture checks pass when harness fixtures are added.
  - Command: `if git diff --name-only -- fixtures/runtime fixtures/operational-intelligence/roadmap-signal | rg -q .; then pnpm fixtures:harness:check; else true; fi`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac2` command - Story/export text includes evidence and next action without raw dumps.
  - Command: `test -d fixtures/operational-intelligence/roadmap-signal/public && for token in roadmap_signal_recorded recommended_action evidence_refs next_action; do rg -n "$token" docs/roadmap-signal.md fixtures/operational-intelligence/roadmap-signal/public >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac3` command - Full fast validation remains green.
  - Command: `pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Revert the `roadmap-signal` skill, docs, fixtures, helper code, and tests from
  this task.
- If a public TS/Rust packet was introduced, revert the schema, generated
  fixtures, exports, and Rust parity definitions together.
- Remove the `roadmap_signal_recorded` story milestone only if this child added
  it; do not disturb unrelated thread-story milestones.
- No data migrations, provider mutations, issue/PR creation, or customer sends
  are part of this child, so rollback is source-code and fixture only.
- If live dogfood output is noisy, disable only the consuming dogfood/export
  fixture and leave the generic deterministic fixtures in place until the
  contract is corrected.

## Review

Status: not_started
Verdict: none

Findings:
- none

Required gates:
- Draft hardening before approval:
  `scafld harden runx-roadmap-signal-v1 --provider claude`
- Completion review after implementation:
  `scafld review runx-roadmap-signal-v1 --provider claude`
- `--provider local` is not sufficient for completion.

## Self Eval

- Pending implementation. Target bar: fixtures prove grouping, redaction,
  idempotency, and recommendation taxonomy; live dogfood proves the lane can
  consume a sanitized Nitrosend export without provider credentials.

## Deviations

- none

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1
- suggested_subagent: Wegener

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-27T15:24:36Z
Ended: none

Checks:
- none

Issues:
- none


## Planning Log

- Read parent spec
  `.scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md`.
- Read runx docs:
  `docs/developer-issue-inbox.md`,
  `docs/issue-to-pr.md`, and
  `docs/thread-story-contract.md`.
- Preserved the parent boundary: runx owns generic packets, grouping, evidence,
  story, and outbox handoff; consuming products own source policy, enrichment,
  owners, channels, and backlog mutation.
- Scoped this draft away from active runtime/CLI implementation files and live
  provider credentials.
