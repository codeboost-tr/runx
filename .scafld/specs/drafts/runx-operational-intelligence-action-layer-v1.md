---
spec_version: '2.0'
task_id: runx-operational-intelligence-action-layer-v1
created: '2026-05-27T14:41:04Z'
updated: '2026-05-27T15:24:26Z'
status: draft
harden_status: needs_revision
size: large
risk_level: high
---

# Runx Operational Proposal Composition

> **Tracking parent only.** Do not build this as one change. The old direction
> overfit operational intelligence into fixed support, alert, outreach, and
> roadmap lanes. The better shape is generic composition over the existing runx
> spine.

## Current State

Status: draft
Current phase: planning
Next: harden
Reason: reframed after subagent review found fixed-lane overfit
Blockers: child specs need hardening under the new proposal boundary
Allowed follow-up command: `scafld harden runx-operational-intelligence-action-layer-v1 --provider <provider>`
Latest runner update: none
Review gate: not_started

## Summary

Harden runx as a governed proposal/action composition layer:

`source event -> context artifact -> signal -> decision -> proposal -> governed action -> outcome story`

This is mostly composition on architecture runx already has: source
normalization, operational policy, skill graphs, receipts, issue intake,
tracking-to-change work, thread story, and provider outbox. The work is not to create a new
platform family for every operational use case. The work is to make the generic
spine clear, typed, safe, and easy for products to compose.

Runx owns portable execution and audit primitives. Consuming products own
business meaning. Nitrosend may have a support reply proposal, dev escalation
proposal, outreach proposal, or product signal proposal, but runx core should
see those as proposal kinds, not as separate platform subsystems.

Golden path invariant: originating source thread/event -> hydrated context
artifact -> optional read-only check/triage -> create/update a tracking item
when requested -> optional build-fix change request without requiring a prior
check -> governed change request with human final-change gate -> final outcome
posted back to the originating source thread and linked references.

## Objectives

- Define the reusable proposal/action composition boundary for runx.
- Preserve existing lanes such as `reply-only`, `manual-review`, `work-plan`,
  `issue-intake`, `issue-to-pr`, `pr-review`, `pr-fix-up`, and `merge-assist`.
- Add only generic proposal semantics where the current spine is missing a
  reviewable handoff artifact.
- Keep source hydration, customer/account enrichment, provider UX, owner maps,
  templates, and final send policy outside runx core.
- Keep story/outbox updates concise for humans while preserving rich evidence in
  receipts and artifacts.
- Prevent action enum sprawl. If policy needs one additional intent, it must be
  generic `proposal`, not a domain-specific action.
- Keep Aster/hosted admin as a control/readback surface over runx policy,
  runners, queues, approvals, and receipts, not a second workflow engine.

## Scope

In scope for this parent:

- Architecture boundary and child-spec tracking.
- Generic proposal/action/story terminology.
- Verification that child specs do not reintroduce fixed domain contracts.
- Documentation of Runx OSS, Runx hosted/control-plane, Aster, and consuming
  application boundaries.

Out of scope for this parent:

- Executing implementation directly.
- Adding support, alert, outreach, roadmap, or customer-success lanes as runx
  core products.
- Auto-sending customer messages.
- Auto-merging change requests.
- Mutating billing, account, SMS/email provider, or destructive provider state.
- Hardcoding Nitrosend channels, owners, account context, provider labels,
  project boards, or product semantics in runx.

## Dependencies

- Existing runx primitives:
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `docs/thread-story-contract.md`;
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/core/src/knowledge/outbox.ts`;
  - `packages/contracts/src/schemas/operational-policy.ts`;
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-triage/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`.
- Child specs that remain in the new direction:
  - `runx-operational-contracts-v1`;
  - `runx-operational-proposal-composition-v1`;
  - `runx-operational-story-outbox-v1`;
  - `nitrosend-operational-intelligence-integration-v1`.

## Assumptions

- Source adapters hydrate and redact provider context before generic runx skills
  reason over it.
- A proposal is reviewable state, not permission to mutate.
- Customer send, final change approval, billing mutation, and destructive
  provider actions remain separate human-approved authorities.
- A governed action may start from a hydrated source without a prior check
  receipt, but it must revalidate source context, policy, and authority at
  mutation time.
- Product-specific proposal kinds are namespaced by the consuming product or
  kept as skill metadata.
- Public provider/support messages should be useful summaries, not raw context
  dumps.
- Live dogfood should prove composition through a real consuming application,
  but the reusable runx layer remains product-neutral.

## Touchpoints

- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/issue-to-pr.md`
- `docs/thread-story-contract.md`
- `.scafld/specs/active/runx-operational-contracts-v1.md`
- `.scafld/specs/drafts/runx-operational-proposal-composition-v1.md`
- `.scafld/specs/drafts/runx-operational-story-outbox-v1.md`

## Risks

- Rebuilding product-specific workflows in runx. Mitigation: proposals are
  generic and app-specific kinds stay outside core.
- Adding too much contract surface. Mitigation: promote only the generic public
  shape required by consumers.
- Losing useful context by over-trimming. Mitigation: public stories summarize;
  receipts/artifacts retain reviewer-safe evidence refs.
- Provider noise. Mitigation: story/outbox remains source-thread-first and
  idempotent.
- Authority creep. Mitigation: proposal does not imply send, merge, billing, or
  destructive action permission.

## Acceptance

Profile: strict

Validation:
- `scafld validate runx-operational-intelligence-action-layer-v1`
- `scafld validate runx-operational-contracts-v1`
- `scafld validate runx-operational-proposal-composition-v1`
- `scafld validate runx-operational-story-outbox-v1`
- `git diff --check -- .scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md .scafld/specs/drafts/runx-operational-contracts-v1.md .scafld/specs/drafts/runx-operational-proposal-composition-v1.md .scafld/specs/drafts/runx-operational-story-outbox-v1.md`

## Phase 1: Boundary Reframe

Status: pending
Dependencies: none

Objective: Confirm the program is composition-first and not a fixed lane suite.

Changes:
- Record the generic spine.
- Record the Runx/Nitrosend/Aster boundaries.
- Confirm fixed domain child specs are not part of the active implementation
  set.

Acceptance:
- [ ] `p1_ac1` command - Parent validates.
  - Command: `scafld validate runx-operational-intelligence-action-layer-v1`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - Active child specs are the generic set.
  - Command: `for s in runx-operational-contracts-v1 runx-operational-proposal-composition-v1 runx-operational-story-outbox-v1; do scafld status "$s" --json >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac3` command - Golden path invariant is recorded.
  - Command: `for token in "originating source thread/event" "hydrated context artifact" "build-fix change request without requiring a prior check" "human final-change gate" "final outcome posted back"; do rg -n "$token" .scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Child Spec Readiness

Status: pending
Dependencies: Phase 1

Objective: Make each child own one clean part of the generic shape.

Changes:
- Contracts child owns proposal/action contract decisions.
- Proposal composition child owns skill graph and existing-lane composition.
- Story/outbox child owns human-readable projection and idempotent publication
  semantics.
- Nitrosend child owns application policy, provider UX, loaders, owner routing,
  and live dogfood.

Acceptance:
- [ ] `p2_ac1` command - Generic child specs validate.
  - Command: `for s in runx-operational-contracts-v1 runx-operational-proposal-composition-v1 runx-operational-story-outbox-v1; do scafld validate "$s" || exit $?; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - Child specs use proposal vocabulary.
  - Command: `for s in runx-operational-contracts-v1 runx-operational-proposal-composition-v1 runx-operational-story-outbox-v1; do rg -n "proposal|action intent|outcome story" ".scafld/specs/drafts/$s.md" >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Harden And Track Only

Status: pending
Dependencies: Phase 2

Objective: Keep this parent as a governance tracker while children do the work.

Changes:
- Harden this parent after child specs are coherent.
- Do not approve this parent for direct build.
- Use child reviews as implementation gates.

Acceptance:
- [ ] `p3_ac1` manual - Parent hardening passes after child updates.
  - Command: `scafld harden runx-operational-intelligence-action-layer-v1 --provider claude`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- This parent only describes work. Rollback is reverting the draft/spec edits.
- Do not change runtime, provider, customer, or external work-tracking state
  from this parent.

## Review

Status: not_started
Verdict: none

Findings:
- none

Required gates:
- Draft hardening before approval:
  `scafld harden runx-operational-intelligence-action-layer-v1 --provider claude`
- Completion review is not applicable unless this parent is later converted to
  an executable spec.

## Self Eval

- Pending hardening. Target bar: the active Runx work is a small, generic
  proposal/action composition hardening effort, not a domain-lane expansion.

## Deviations

- Fixed domain lane drafts were cancelled through scafld after subagent review
  found they overfit the architecture.

## Metadata

- created_by: scafld
- split_children: true

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- Previous hardening rounds applied to the fixed-lane version of this draft and
  are superseded by this proposal-composition reframe. Re-harden before
  approval.

## Planning Log

- Reframed from operational-intelligence product lanes to the generic proposal
  composition boundary.
- Subagent reviews agreed that runx already has most of the spine and should
  add only missing generic seams.
