---
spec_version: '2.0'
task_id: runx-hosted-scope-fail-closed-v1
created: '2026-06-11T09:01:56Z'
updated: '2026-06-11T10:22:53Z'
status: completed
harden_status: passed
size: small
risk_level: high
---

# Hosted API scope checks fail closed

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

The hosted API authorizes scopeless principals against every scope. `scopeAllows`
returns `true` for any principal whose `scopes` is absent and whose role is not
`service`, and self-serve principals are minted with no `scopes` field at all.
The result: any onboarded self-serve account passes `runs:write` and every other
scope gate. This is an authorization fail-open, and it gates any anonymous or
public exposure of the hosted API (gap 21, frantic-architecture.md §15.1).

This spec makes scope evaluation fail closed (no scopes means no authority),
mints self-serve principals with an explicit minimal scope set, gives legacy
operator bearer principals explicit configured scopes before the fail-closed
flip, and closes two adjacent hardening holes the same audit found: the in-memory rate limiter's
`prune()` is defined but never called, and `POST /v1/accounts` has no per-route
throttle. No OSS kernel semantics change; this is hosted-surface hardening only.

## Objectives

- `scopeAllows` fails closed: a principal with absent or empty `scopes` is denied
  every scope check. Remove the role-based allow for non-service principals.
- Self-serve principals are minted with an explicit, minimal scope set covering
  exactly the self-serve surface they legitimately use today and nothing wider,
  at `onboard`, `authenticateToken`, and `refresh` in `self-serve-identity.ts`.
- Legacy/operator bearer principals created from env fallback config carry
  explicit configured scopes; no principal relies on absent `scopes` for access.
- `POST /v1/accounts` gains per-principal/IP rate limiting consistent with the
  topup and credential-refresh routes.
- The rate limiter's `prune()` runs on a schedule or per-operation so bucket
  state cannot grow unbounded.
- No regression for legitimately scoped service principals or for the existing
  scoped self-serve surface.

## Scope

- Scope evaluation in `cloud/packages/auth`.
- Self-serve principal minting and its scope assignment.
- The run-control scope gate that consumes the evaluator.
- Rate-limiter pruning and the accounts-route throttle in `cloud/packages/api`.

Out of scope:
- The attribution model and board agent auth (gaps 3/4, `board-agent-auth-v1`).
- The broader hosted-ops launch lane (`runx-hosted-ops-v1`).
- Any OSS kernel authority or admission semantics.
- Widening the self-serve surface; this spec narrows authority, never grants new.

## Dependencies

- none

## Assumptions

- The cloud repo (`../cloud`) is a separate git repo with no `.scafld`; this
  kernel-side spec governs the sibling hosted auth surface, and acceptance
  commands run against the cloud workspace via `cd ../cloud`.
- The minimal self-serve scope set is exactly the surface self-serve users
  legitimately reach in the shipped product; any widening is a separate,
  explicitly approved change.
- Service-role principals keep their existing role-based path by design; only the
  scopeless non-service bypass is removed.

## Touchpoints

- `cloud/packages/auth/src/service-principals.ts`
- `cloud/packages/auth/src/principals.ts`
- `cloud/packages/auth/src/self-serve-identity.ts`
- `cloud/packages/api/src/security.ts`
- `cloud/packages/api/src/run-control-routes.ts`
- `cloud/packages/api/src/rate-limit.ts`
- `cloud/packages/api/src/billing-routes.ts`
- `cloud/packages/api/src/app/route-options.ts`
- `cloud/packages/api/src/app.ts`
- `cloud/packages/api/src/options/workflows.ts`
- `cloud/packages/api/src/server.ts`
- cloud test files alongside each touched module

## Risks

- **Fail-closed could lock out a caller that relied on the scopeless bypass.**
  Mitigation: enumerate every principal mint site, assign explicit minimal scopes
  before flipping `scopeAllows`, and add a regression test per legitimate caller
  path so the flip is proven safe, not assumed safe.
- **Public API authorization behavior change.** Mitigation: the change is confined
  to authorization; tests assert scoped principals still pass and scopeless are
  denied, so the surface contract is pinned by tests, not prose.

## Acceptance

Profile: strict

Validation:
- `cd ../cloud && pnpm vitest run packages/auth/src/service-principals.test.ts`
- `cd ../cloud && pnpm vitest run packages/auth/src/self-serve-identity.test.ts`
- `cd ../cloud && pnpm vitest run packages/api/src/rate-limit.test.ts packages/api/src/index.test.ts -t "account creation|scope"`

## Phase 1: Fail-closed scope evaluation

Status: completed
Dependencies: none

Objective: a principal with no scopes is denied every scope check.

Changes:
- In `service-principals.ts`, make `scopeAllows` return `false` when `scopes` is absent or empty for non-service principals; keep the service-role path as-is.
- Add regression tests: a scopeless non-service principal is denied a representative scope; an explicitly scoped principal is allowed only its scopes; a service principal is unaffected.

Acceptance:
- [x] `ac2` command - Scope evaluation fails closed
  - Command: `cd ../cloud && pnpm vitest run packages/auth/src/service-principals.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11

## Phase 2: Explicit minimal self-serve scopes

Status: completed
Dependencies: Phase 1

Objective: self-serve principals carry an explicit minimal scope set.

Changes:
- Add a single self-serve scope constant and apply it at `onboard`, `authenticateToken`, and `refresh` in `self-serve-identity.ts`, instead of leaving `scopes` unset.
- Ensure legacy/operator bearer principals created from env fallback config also carry explicit scopes before `scopeAllows` fails closed.
- Add a test proving a freshly self-served principal can do exactly the self-serve surface (e.g. submit a run) and nothing beyond it.

Acceptance:
- [x] `ac3` command - Self-serve principals are minimally scoped
  - Command: `cd ../cloud && pnpm vitest run packages/auth/src/self-serve-identity.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Phase 3: Rate-limiter pruning and accounts throttle

Status: completed
Dependencies: Phase 2

Objective: limiter state is bounded and account creation is throttled.

Changes:
- Invoke `prune()` on a schedule or per-operation so the bucket map cannot grow unbounded; add a test that bucket count stays bounded across many keys.
- Add per-principal/IP rate limiting to `POST /v1/accounts`, consistent with the topup and credential-refresh routes; add a test that the route throttles.

Acceptance:
- [x] `ac4` command - Limiter pruning and accounts throttle covered
  - Command: `cd ../cloud && pnpm vitest run packages/api/src/rate-limit.test.ts packages/api/src/index.test.ts -t "account creation|scope"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-26

## Rollback

- Each phase is an independent revert. The `scopeAllows` flip and the self-serve
  scope assignment must revert together (the assignment is the safety net for the
  flip); the limiter and accounts-throttle changes revert independently.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verified hosted scope fail-closed hardening. `scopeAllows` now returns false for non-admin principals with absent/empty scopes (service-principals.ts:232-240); admin retains its bypass. Self-serve principals are minted with explicit SELF_SERVE_SCOPES at onboard, authenticateToken, and refresh (self-serve-identity.ts:308-315, 351-360, 388-400, 442-452). Legacy/operator bearer principals always carry explicit scopes — string-valued config and the env fallback path both default to DEFAULT_LEGACY_BEARER_SCOPES `["*"]`; object-valued user/admin config falls back to the same default (principals.ts:11, 41, 57, 90-94). POST /v1/accounts is rate-limited per trusted client IP via createAccountCreationRateLimiter, wired through server.ts → app.ts → billing-routes.ts:36-44. `prune()` is invoked at the head of every `check`/`consume` (rate-limit.ts:36-50, 62-69), so bucket map cannot retain entries beyond `windowMs`. Acceptance suites (service-principals.test.ts:66-79, self-serve-identity.test.ts:16-32, rate-limit.test.ts:45-53, 116-120, index.test.ts:420-468) cover fail-closed scope evaluation, scoped self-serve principal minting, prune-on-traffic, and per-IP /v1/accounts throttling. No regressions to scoped service principals (broker exchange flow continues to populate explicit scopes) or authenticated read/write surfaces consumed by self-serve scopes. Ambient drift (25 files, all under oss/ CLI/runtime/receipts) is unrelated to this hosted-API spec.

Attack log:
- `cloud/packages/auth/src/service-principals.ts scopeAllows`: Confirm fail-closed for absent/empty scopes on non-service, non-admin roles; admin bypass preserved; service with empty scopes denied -> clean (Lines 232-240 return false unless principal.scopes is non-empty (admin role retains short-circuit). Test at service-principals.test.ts:66-79 covers user/service/admin matrix incl. wildcard.)
- `cloud/packages/auth/src/principals.ts normalizePrincipal + parseBearerPrincipals`: Search for paths that could mint a principal with absent scopes post-flip (env JSON object lacking scopes, string-valued entries, env fallback access token) -> clean (All bearer-mint paths now default to DEFAULT_LEGACY_BEARER_SCOPES `["*"]` for user/admin and rely on explicit ServicePrincipalConfig.scopes for service; nothing relies on absent scopes.)
- `cloud/packages/auth/src/self-serve-identity.ts onboard/authenticateToken/refresh`: Verify SELF_SERVE_SCOPES is applied at all three principal-emission points and matches the actual scope gates the self-serve surface enforces -> clean (SELF_SERVE_SCOPES = [runs:read, runs:write, receipts:read, receipts:write, billing:read, billing:write] is applied uniformly; matches denyMissingScope gates in run-control-routes.ts and receipt-notary-routes.ts. Notably excludes signals:write (harness-routes) and admin:* (governance/admin-routes), keeping authority narrow.)
- `cloud/packages/api/src/rate-limit.ts prune() and createAccountCreationRateLimiter wiring`: Confirm prune is invoked on every check/consume, accountCreationRateLimiter is created and routed through server → app → billing-routes for POST /v1/accounts, and the IP key uses the trusted-rightmost extractor -> clean (rate-limit.ts:36,42 call prune() unconditionally; server.ts:309 → app.ts:138 → route-options.ts:97 → billing-routes.ts:36-44 with key `ip:${extractClientIp}` which itself takes the rightmost-minus-trusted-hops entry of X-Forwarded-For (request-utils.ts:103-116). index.test.ts:420-468 asserts 429 from a second request sharing the trusted proxy hop.)
- `Ambient workspace drift outside task scope`: Verify the 25 modified/untracked paths sit entirely outside cloud/packages/{auth,api} and are not silently coupled to the hosted scope flip -> clean (All listed paths live under oss/ (runx CLI, runtime, receipts, contracts, fixtures, docs); none touch cloud/. Context only, not task-attributable.)

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: claude
- home_repo: cloud
- gap: frantic-architecture.md §15.1 gap 21

## Origin

Created by: Claude
Source: frantic-architecture.md §15.1 (2026-06-11 code re-audit), gap 21

## Harden Rounds

### round-1

Status: passed
Started: 2026-06-11T09:20:25Z
Ended: 2026-06-11T09:22:25Z

Observations:
- path
  - Result: clean
  - Anchor: spec_gap:touchpoints
  - Note: The fail-open behavior is isolated in `scopeAllows`, with callers already routed through cloud auth helpers.
- command
  - Result: clean
  - Anchor: spec_gap:acceptance
  - Note: Existing scope evaluator tests cover the old operator-compatible behavior and are the right first regression target.
- scope
  - Result: clean
  - Anchor: spec_gap:phase2
  - Note: Self-serve principal minting is centralized enough to assign explicit scopes before flipping the evaluator.
- timing
  - Result: advisory
  - Anchor: spec_gap:phase3
  - Note: Account creation is already invite-gated; add throttling in the same route without changing account semantics.
- rollback
  - Result: clean
  - Anchor: spec_gap:rollback
  - Note: Scope assignment and fail-closed evaluation must revert together; limiter pruning and account throttle can revert independently.
- design
  - Result: clean
  - Anchor: spec_gap:objectives
  - Note: The existing limiter already has `prune`; implementation should invoke it rather than add a second limiter abstraction.


## Planning Log

- none
