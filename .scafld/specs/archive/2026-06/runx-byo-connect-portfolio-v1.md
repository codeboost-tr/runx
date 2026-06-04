---
spec_version: '2.0'
task_id: runx-byo-connect-portfolio-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T20:40:15Z'
status: completed
harden_status: not_run
size: medium
risk_level: medium
---

# runx-byo-connect-portfolio-v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-04T20:40:15Z
Review gate: pass

## Summary

Close the OSS side of the BYO provider gap: a per-run local credential descriptor
must reach a graph-step `http` source as a scoped secret header, seal a receipt,
and record only non-secret credential-delivery evidence. Hosted OAuth brokerage,
grant issuance, and credential custody remain cloud/private per
`docs/licensing-boundary.md`; OSS proves consumption over the already-shipped HTTP
front and then uses that path to build the demand-shaped non-GitHub skill
portfolio (search / mail / calendar / db / browser).

## Objectives

- A locally supplied credential descriptor reaches a non-GitHub graph HTTP step
  via `${secret:NAME}`, with no argv secret and no raw secret in outputs, graph
  state, or sealed receipt metadata.
- A governed non-GitHub provider read seals a receipt and records a
  receipt-safe `CredentialDeliveryObservation`.
- The first portfolio skills over the http/external-adapter fronts (sql-analyst,
  inbox-and-calendar-exec, knowledge-router, deep-research-brief, lead-enrichment),
  each maturity-tiered with a harness case.

## Scope

In scope:
- Local `--credential` + `--secret-env` consumption through graph HTTP steps.
- A runnable non-GitHub HTTP provider example proving the descriptor -> header ->
  receipt path.
- The first ~5 non-GitHub skills over the shipped http front (and the OpenAPI front
  for spec-backed APIs), using local/fixture descriptors in OSS; harness +
  maturity tiering.

Out of scope:
- GitHub (already wired).
- Hosted OAuth brokerage, hosted connect-session UX, credential custody, grant
  issuance, and grant revocation (cloud/private).
- Deep per-provider polish / the full ~351-provider sprawl (start with high-demand).

## Dependencies

- SHIPPED: the HTTP front, credential delivery contracts, and local per-run
  credential descriptors.
- The OpenAPI front (Wave 2) for spec-backed providers.

## Assumptions

- The HTTP front already governs any REST provider once a credential is delivered
  (verified shipped: method+URL+headers, SSRF/private-net opt-in, `${secret:NAME}`
  headers). This spec proves graph-step delivery; hosted OAuth remains a separate
  cloud dependency, not an OSS runtime prerequisite.

## Touchpoints

- The HTTP front (`adapters/http.rs`), graph skill execution
  (`execution/skill_run.rs`), local credential provision tests, the BYO HTTP
  example, and the new portfolio skills + official lock + maturity tiers.
- `skills/sql-analyst`, `skills/inbox-and-calendar-exec`,
  `skills/knowledge-router`, `skills/lead-enrichment`, and the existing
  `skills/deep-research-brief`.
- `packages/cli/src/official-skills.lock.json` and `scripts/harness-sweep.mjs`
  for first-party skill maturity/lock coverage.

## Risks

- **Provider sprawl.** Mitigation: start with a few high-demand providers; the front
  generalizes, the demand does not.
- **Auth-scope correctness.** Mitigation: local descriptors carry explicit scopes;
  hosted OAuth scope negotiation stays cloud/private.

## Acceptance

Profile: strict

Validation:
- A credentialed local fixture read runs through the graph HTTP front; the response
  seals and the graph state/receipt metadata carry a non-secret credential
  observation.
- The first portfolio skills run under local/fixture descriptors, seal receipts,
  and are maturity-tiered + locked.
- `pnpm verify:fast` + the new harness cases green.

## Phase 1: Local credential descriptor + graph HTTP demo

Status: completed
Dependencies: HTTP front (shipped), local credential descriptors (shipped)

Objective: a locally supplied credential descriptor reaches a non-GitHub HTTP
graph step as a scoped secret header, without leaking the raw secret.

Changes:
- Thread `--credential` + `--secret-env` delivery through graph execution options.
- Verify `examples/byo-http-graph` + `examples/byo-http-tool` using `${secret:RUNX_EXAMPLE_CRM_TOKEN}` against a local non-GitHub HTTP fixture.

Acceptance:
- [x] `ac1` command - non-GitHub local credential read seals
  - Command: `sh examples/byo-http-graph/run.sh`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6

## Phase 2: The first non-GitHub portfolio skills

Status: completed
Dependencies: Phase 1

Objective: the demand-shaped seeds run over the http/external-adapter fronts using local/fixture descriptors in OSS.

Changes:
- Build sql-analyst, inbox-and-calendar-exec, knowledge-router, deep-research-brief, lead-enrichment; harness + maturity. Keep live hosted OAuth provider brokerage as a cloud/private dependency.

Acceptance:
- [x] `ac2` command - portfolio skills run + are tiered
  - Command: `export RUNX_RECEIPT_SIGN_KID=runx-demo-key RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI= RUNX_RECEIPT_SIGN_ISSUER_TYPE=hosted; for s in sql-analyst inbox-and-calendar-exec knowledge-router lead-enrichment deep-research-brief; do rdir="$(mktemp -d)"; crates/target/debug/runx harness "skills/$s" --json --receipt-dir "$rdir" >/tmp/runx-$s-harness.json || exit $?; node -e "const fs=require('fs'); const r=JSON.parse(fs.readFileSync('/tmp/runx-' + process.argv[1] + '-harness.json','utf8')); if(r.status !== 'passed') { console.error(JSON.stringify(r,null,2)); process.exit(1); }" "$s"; done`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-11

## Rollback

- Phase 1 is additive runtime/example work; revert the graph credential-delivery
  patch and example files if it regresses. Portfolio skills are additive +
  maturity-gated (alpha first). Hosted OAuth/connect changes are out of OSS scope.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: command
Output: command.stdout
Summary: Command-provider review passed. Verified scafld review scope includes the new portfolio skill paths, official lock, and sweep script; the BYO active spec validates; the five portfolio harnesses passed; the official skill lock contains 56 entries including the four new seeds; the focused harness sweep passed with only pre-existing MCP fixture failures allowed; and no hosted OAuth/connect/custody identifiers appear in the new skill/code paths.

Attack log:
- `scafld review scope`: print review context and verify new skill directories, official lock, and harness sweep are task-scoped rather than ambient -> clean
- `portfolio harness acceptance`: run sql-analyst, inbox-and-calendar-exec, knowledge-router, lead-enrichment, and deep-research-brief inline harnesses with isolated receipt dirs -> clean
- `official skill maturity lock`: regenerate official-skills.lock.json and verify lock length is 56 with new seed skill ids present -> clean
- `full official skill sweep`: run harness-sweep with expected-count 56 and allow only pre-existing MCP fixture failures issue-triage and pr-review-note -> clean
- `hosted credential boundary`: scan new skill/code paths for OAuth, connect-session, Nango, hosted connect, custody, and RUNX_CONNECT identifiers -> clean
- `manifest and spec validity`: run scafld validate and inspect new X.yaml harness/artifact markers -> clean

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

- none
