---
spec_version: '2.0'
task_id: x402-pay-authority-cli-admission-v1
created: '2026-05-21T00:00:00Z'
updated: '2026-05-21T00:00:00Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# x402-pay authority CLI admission v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: implemented with fixture-only CLI coverage
Blockers: P1.14 still needs a dedicated quote-drift fixture
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T00:00:00Z
Review gate: not_run

## Summary

Close P1.8 by adding a native CLI harness fixture that routes a crafted x402
payment reservation through Rust runtime authority admission and rejects a child
`AuthorityTerm` broader than its parent before mock rail fulfillment can run.

## Scope And Touchpoints

In scope:

- `fixtures/graphs/payment/x402-pay-negative-authority-broader-child.yaml`
- `fixtures/harness/x402-pay-negative-authority-broader-child.yaml`
- `fixtures/skills/x402-pay-negative-authority-broader-child-reserve/SKILL.md`
- `fixtures/skills/x402-pay-negative-authority-broader-child-reserve/run.sh`
- `crates/runx-cli/tests/x402_native_dogfood.rs`
- `tests/x402-pay-dogfood-mock.test.ts`
- `.scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md`

Out of scope:

- Rust runtime or contracts source changes.
- Live x402 rails, Stripe, refunds, disputes, or money movement.
- Closing P1.14 quote drift without a dedicated drifted spend fixture.

## Acceptance

Profile: strict

Validation:
- [x] `v1` scafld - Spec validates.
  - Command: `scafld validate x402-pay-authority-cli-admission-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
- [x] `v2` native dogfood - Native x402 negative fixture test passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test x402_native_dogfood native_x402_negative_fixtures_refuse_without_settlement`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
- [x] `v3` dogfood sentinel - Mock scenario coverage accounting recognizes P1.8 closure while P1.14 remains punch-listed.
  - Command: `cargo build --quiet --manifest-path crates/Cargo.toml -p runx-cli --bin runx && RUNX_KERNEL_EVAL_BIN=crates/target/debug/runx pnpm exec vitest run tests/x402-pay-dogfood-mock.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0

## Evidence

The crafted reservation fixture keeps the spend capability binding internally
valid for `amount_minor: 125`, `rail: mock`, and the `act_fulfill` child harness
reference. It independently widens the child payment authority by setting
`child_authority.bounds.payment.max_per_call_minor` to `20000` while the parent
allows only `10000`. Native runtime admission therefore reaches
`admit_step_authority`, fails the payment subset comparator, and returns
`child payment authority is not a subset of parent authority` before the fulfill
skill emits any mock credential or rail session material.

## Rollback

Strategy: per_file

Commands:
- `git checkout HEAD -- crates/runx-cli/tests/x402_native_dogfood.rs tests/x402-pay-dogfood-mock.test.ts .scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-punchlist.md`
- `rm -f fixtures/graphs/payment/x402-pay-negative-authority-broader-child.yaml fixtures/harness/x402-pay-negative-authority-broader-child.yaml .scafld/specs/archive/2026-05/x402-pay-authority-cli-admission-v1.md`
- `rm -rf fixtures/skills/x402-pay-negative-authority-broader-child-reserve`

## Follow-up

P1.14 is now unblocked because the CLI fixture path demonstrably reaches native
authority admission. A separate quote-drift lane should craft a valid child
authority but drift the spend binding above reserved bounds, then assert the
same pre-rail stop.
