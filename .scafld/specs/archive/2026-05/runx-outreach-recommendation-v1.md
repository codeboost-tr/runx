---
spec_version: '2.0'
task_id: runx-outreach-recommendation-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-27T16:31:41Z'
status: cancelled
harden_status: in_progress
size: medium
risk_level: high
---

# Outreach Recommendation Lane

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

Create a generic runx lane that recommends customer/success outreach from
operational signals without sending messages. It should explain why outreach is
useful, who should own it by route id, when to act, what draft copy could say,
and when not to send.

This lane must not know Nitrosend customers, account names, Slack channels,
billing state, or owner names. Consuming products provide redacted customer
context and final send policy.

Suggested subagent ownership: Dalton.

## Objectives

- Define and implement a generic `outreach_recommendation` output over
  admitted, redacted operational signals.
- Support recommendation types such as activation help, deliverability setup,
  stuck onboarding, upgrade intent, failed payment followup, incident apology,
  feature discovery, and do-not-send.
- Require evidence refs, confidence, timing, owner route id, caveats, approval
  gate, and optional draft message.
- Make do-not-send cases first-class when context is risky, stale, private,
  low-confidence, or already handled.
- Keep all customer sends human-approved and product-owned.
- Provide story/outbox hints for internal review without emitting customer
  messages from runx core.

## Scope

In scope:

- New generic skill/lane:
  - `skills/outreach-recommendation/SKILL.md`;
  - `skills/outreach-recommendation/X.yaml`.
- Docs:
  - `docs/operational-intelligence.md`;
  - `docs/developer-issue-inbox.md`;
  - `docs/thread-story-contract.md` after story milestones exist.
- Contract/helper touchpoints only if promoted by
  `runx-operational-contracts-v1`:
  - `packages/contracts/src/schemas/*`;
  - `crates/runx-contracts/src/*`;
  - `packages/core/src/knowledge/thread-story.ts`.
- Fixtures:
  - `fixtures/operational-intelligence/outreach-recommendation/**`;
  - `fixtures/runtime/skills/outreach-recommendation/**`.
- Tests for recommendation taxonomy, do-not-send, approval gates, redaction,
  owner route abstraction, and story projection.

Out of scope:

- Sending customer emails, Slack DMs, SMS, in-app messages, or support replies.
- Live CRM, billing, support, product analytics, email, or Slack API calls.
- Product-specific templates, customer names, account ids, revenue numbers,
  owner names, GitHub users, or Slack channel ids.
- Automated discounts, refunds, billing changes, or contractual promises.
- Roadmap aggregation. Repeated outreach signals can feed
  `runx-roadmap-signal-v1`, but this lane outputs one recommendation at a
  time.

## Dependencies

- `runx-operational-contracts-v1` for recommendation and authority semantics.
  Phase 3 may not start until that spec has approved or otherwise recorded the
  stable `outreach_recommendation` and send-authority shapes.
- `runx-operational-story-outbox-v1` for outreach story rendering.
- `runx-roadmap-signal-v1` can consume aggregate outreach signals later.
- Existing source and issue docs:
  - `docs/developer-issue-inbox.md`;
  - `docs/thread-story-contract.md`;
  - `packages/core/src/source/index.ts`.
- Nitrosend integration child for product templates, approval UX, and live
  dogfood.

## Assumptions

- Consuming products supply redacted customer/account/product context and source
  policy.
- runx may draft internal suggestion copy, but it never marks a customer
  message as sent.
- Owner routing is abstract (`owner_route_id`), not a human name or GitHub
  login.
- Customer-sensitive fields remain in artifacts or product systems. Public runx
  output contains summaries and evidence refs only.
- If confidence is low or policy denies outreach, the correct result is
  do-not-send or manual review.
- Recommendation outputs must carry a `dedupe_ref` or `idempotency_key` so
  repeated evidence does not create duplicate outreach tasks.

## Decisions

- `skill_ownership`: create a sibling `outreach-recommendation` skill.
- The lane may draft internal suggestion copy but never customer-send payloads.
- Final customer copy, recipient selection, and send authority stay in the
  consuming product.

## Touchpoints

- `skills/outreach-recommendation/SKILL.md`
- `skills/outreach-recommendation/X.yaml`
- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/thread-story-contract.md`
- `packages/core/src/source/index.ts`
- `packages/core/src/knowledge/thread-story.ts`
- `packages/core/src/knowledge/index.ts`
- `fixtures/operational-intelligence/outreach-recommendation/**`
- `fixtures/runtime/skills/outreach-recommendation/**`
- contract files only when `runx-operational-contracts-v1` promotes public
  packet shapes

## Risks

- Customer trust and compliance risk. Mitigation: recommendations are unsent,
  carry approval gates, and include do-not-send caveats.
- Product-specific leakage. Mitigation: route ids and redacted summaries only;
  concrete templates and owners live in consuming repos.
- Low-signal outreach spam. Mitigation: require evidence refs, confidence,
  timing, caveats, and policy admission.
- Unsafe promises in draft copy. Mitigation: fixtures include blocked examples
  for refunds, guarantees, sensitive incidents, and private account data.
- Duplicate customer contact. Mitigation: recommendation includes existing
  outreach/context evidence when supplied and can return already-handled.

## Acceptance

Profile: standard

Validation:
- `scafld harden runx-outreach-recommendation-v1 --provider claude`
- `scafld validate runx-outreach-recommendation-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- If contracts change:
  `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- If runtime skill fixtures change:
  `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- outreach_recommendation`

## Phase 1: Harden Outreach Safety Boundary

Status: pending
Dependencies: none

Objective: Confirm recommendation semantics, approval gates, and do-not-send
rules before implementation.

Changes:
- Run Claude hardening.
- Define recommendation types, required evidence, confidence thresholds,
  caveats, owner route rules, and approval requirements.
- Define what copy can be drafted generically and what product templates own.

Acceptance:
- [ ] `p1_ac1` command - Ownership and no-send decision is recorded.
  - Command: `sh -c 'spec=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-outreach-recommendation-v1.md -print -quit); test -n "$spec" && rg -n "skill_ownership.*outreach-recommendation|never customer-send payloads|dedupe_ref|idempotency_key" "$spec"'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - Spec validates after hardening edits.
  - Command: `scafld validate runx-outreach-recommendation-v1`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Fixture Matrix

Status: pending
Dependencies: Phase 1

Objective: Prove useful and blocked recommendations with deterministic data.

Changes:
- Add fixture cases for:
  - activation help;
  - deliverability setup;
  - stuck onboarding;
  - upgrade intent;
  - failed payment followup;
  - incident apology/recovery;
  - already handled;
  - stale/low-confidence do-not-send;
  - sensitive/private manual review.
- Add expected outputs with evidence refs, owner route id, timing, caveats,
  approval gate, and optional draft copy.
- Include raw-looking customer/provider fields in private inputs and redacted
  public outputs.
- Split fixtures into `private/` source inputs and `public/` expected
  recommendation/story outputs; leak and no-send checks apply only to `public/`.
- Include repeated-source and already-handled private evidence to prove dedupe
  and do-not-send behavior.

Acceptance:
- [ ] `p2_ac1` command - Outreach fixtures cover send-worthy and do-not-send cases.
  - Command: `test -d fixtures/operational-intelligence/outreach-recommendation/public && for token in activation deliverability upgrade failed_payment do_not_send approval_gate already_handled manual_review dedupe_ref; do rg -n "$token" fixtures/operational-intelligence/outreach-recommendation >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - Public fixture output avoids obvious customer/provider leaks.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/outreach-recommendation/public && if rg -n "xox[baprs]-|BEGIN .*PRIVATE KEY|/Users/|customer_email|raw_payload|card_|stripe_|url_private_download" fixtures/operational-intelligence/outreach-recommendation/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac3` command - Recommendation fixtures cannot mark outreach as sent.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/outreach-recommendation/public && if rg -n "\"sent\"\\s*:\\s*true|delivery_status.*sent|sent_at|delivered_at" fixtures/operational-intelligence/outreach-recommendation/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Lane Implementation

Status: pending
Dependencies: Phase 2

Objective: Implement outreach recommendation generation without send authority.

Changes:
- Add the outreach-recommendation skill/lane.
- Emit recommendation type, evidence refs, confidence, owner route, suggested
  timing, caveats, approval gate, and optional draft.
- Enforce unsent semantics in output and tests.
- Enforce dedupe/idempotency semantics so repeated evidence updates or reuses the
  same recommendation instead of creating duplicate outreach tasks.
- Add tests for policy denies, do-not-send, missing customer context, owner
  route abstraction, dedupe, and redaction.

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
- [ ] `p3_ac4` command - Repeated outreach evidence is deduped.
  - Command: `rg -n "dedupe_ref|idempotency_key|already_handled" fixtures/operational-intelligence/outreach-recommendation/public skills/outreach-recommendation`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Story And Consumer Handoff

Status: pending
Dependencies: Phase 3

Objective: Make outreach recommendations reviewable in source threads and
consuming products.

Changes:
- Add story hints for outreach recommended, do-not-send, manual-review, and
  outcome.
- Add a non-mutating handoff packet for product adapters to convert into their
  own outreach queues or support tasks.
- Verify public story includes evidence, caveats, and exact next human action.

Acceptance:
- [ ] `p4_ac1` command - Docs mention outreach approval and no-send semantics.
  - Command: `for token in "outreach recommendation" "approval gate" "do-not-send" "unsent" owner_route_id; do rg -n "$token" skills/outreach-recommendation docs/developer-issue-inbox.md docs/thread-story-contract.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac2` command - Runtime skill fixtures pass when added.
  - Command: `test -d fixtures/runtime/skills/outreach-recommendation && cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- outreach_recommendation`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Remove outreach-recommendation skill files, fixtures, docs, and tests
  introduced by this child.
- Revert public packet changes only together with TS/Rust/schema fixtures if
  `runx-operational-contracts-v1` promoted them.
- No customer messages or provider mutations are part of this child, so
  rollback is source and fixture only.

## Review

Status: not_started
Verdict: none

Findings:
- none

Required gates:
- Draft hardening before approval:
  `scafld harden runx-outreach-recommendation-v1 --provider claude`
- Completion review after implementation:
  `scafld review runx-outreach-recommendation-v1 --provider claude`
- `--provider local` is not sufficient for completion.

## Self Eval

- Pending implementation. Target bar: recommendations are useful, evidence-led,
  safe, explicitly unsent, and reusable by non-Nitrosend products.

## Deviations

- none

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1
- suggested_subagent: Dalton

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-27T15:29:42Z
Ended: none

Checks:
- none

Issues:
- none


## Planning Log

- Split from the parent operational-intelligence program spec.
- This lane deliberately stops at recommendation and handoff. Product adapters
  own final copy, recipient selection, and send authority.
