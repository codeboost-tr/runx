---
spec_version: '2.0'
task_id: runx-receipt-publish-cli-v1
created: '2026-06-11T09:01:50Z'
updated: '2026-06-11T09:58:42Z'
status: review
harden_status: passed
size: medium
risk_level: medium
---

# runx publish receipt-link CLI

## Current State

Status: review
Current phase: final
Next: complete
Reason: review gate pass: 1 finding(s), 0 completion blocker(s)
Blockers: none
Allowed follow-up command: `scafld complete runx-receipt-publish-cli-v1`
Latest runner update: 2026-06-11T10:29:19Z
Review gate: pass

## Summary

The hosted notary already publishes a receipt and serves a verified link:
`POST /v1/receipts/notarize` with `publish: true`, then
`GET /v1/receipts/notarizations/:hash` renders the public `/r` page. What is
missing is the client half. Nothing takes a locally sealed receipt and produces
that published link, so the north-star metric (time-to-first-receipt-link) cannot
be measured and the "verify it yourself" demo has no client (gap 22,
frantic-architecture.md §15.1).

This spec adds `runx publish <receipt>`: it posts a local sealed receipt to the
hosted notary publish endpoint as an authenticated full receipt upload and prints
the public `/r` link and the content hash. HTTP lives in `runx-cli`, reusing the
existing registry/url-add HTTP client pattern; per OSS conventions, no network
code enters `runx-receipts` or core.

## Objectives

- `runx publish <path-to-receipt>` posts the sealed receipt to the configured
  hosted notary publish endpoint and prints the public `/r` link and the hash.
- The endpoint base URL follows the existing public API convention:
  `RUNX_PUBLIC_API_BASE_URL`, defaulting to `https://runx.ai`, with an explicit
  `--api-base-url` override.
- Authentication uses `--token`, `RUNX_PUBLIC_API_TOKEN`, or the existing
  self-serve `RUNX_CONNECT_ACCESS_TOKEN`; the hosted endpoint enforces
  `receipts:write`.
- The command reuses the existing CLI HTTP client pattern (`registry.rs` /
  `url_add.rs`); no new HTTP stack, and no network code in `runx-receipts` or
  core.
- Fails closed with actionable errors: missing token, unreadable/invalid JSON
  receipt, and any non-2xx response from the notary.
- Output mirrors the notary response fields (`status`, `digest`, `public_hash`,
  `mode`, `published`, `public_url`, `receipt_id`, `verdict`) without inferring
  trust state locally.
- The TypeScript wrapper forwards `publish` to the native binary; no behavior is
  reimplemented in TypeScript (launcher-only discipline).

## Scope

- New `runx publish` command in `runx-cli`: module, launcher/main routing, help,
  and tests.
- TypeScript wrapper argument parsing and dispatch for the top-level `publish`.
- User-facing docs/hints that advertise the command.

Out of scope:
- The hosted notary endpoints (they exist) and their verification semantics.
- The receipt format or signing.
- Registry skill publishing (`runx registry` / GitHub indexing), which is a
  different `publish` and stays separate.

## Dependencies

- none (the hosted notarize-and-publish endpoints already ship in cloud)

## Assumptions

- `runx publish` is receipt-link publishing for operators and agents; registry
  administration stays under `runx registry`. Help text disambiguates the two.
- The native binary owns the HTTP call and all output shaping; the TypeScript
  package remains a launcher/wrapper.
- The notary response already distinguishes verified vs hash-only mode, so the
  command can mirror it without inferring trust state itself.

## Touchpoints

- `crates/runx-cli/src/main.rs`
- `crates/runx-cli/src/launcher.rs`
- `crates/runx-cli/src/publish.rs` (new)
- `crates/runx-cli/src/url_add.rs` / `runx_runtime::registry` (HTTP client pattern to reuse)
- `crates/runx-cli/tests/launcher.rs`
- `packages/cli/src/args.ts`
- `packages/cli/src/dispatch.ts`
- `packages/cli/src/help.ts`
- `packages/cli/src/index.test.ts`
- `README.md`

## Risks

- **Public CLI surface addition.** Mitigation: the command is additive, fails
  closed, and is pinned by tests; no existing command shape changes.
- **Confusion with registry publishing.** Mitigation: `runx publish` = receipt
  link only; skill publishing stays under `runx registry` / `runx add`; help and
  error text disambiguate.
- **Overclaiming verification.** Mitigation: output is derived from the notary
  response fields and does not invent a local trust verdict.
- **Duplicating HTTP plumbing.** Mitigation: reuse the registry/url-add client;
  do not add a second HTTP stack, and keep `runx-receipts`/core network-free.

## Acceptance

Profile: standard

Validation:
- `cargo fmt --all --manifest-path crates/Cargo.toml --check`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo clippy --manifest-path crates/Cargo.toml -p runx-cli --all-targets -- -D warnings`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo test --manifest-path crates/Cargo.toml -p runx-cli publish -- --nocapture`
- `pnpm vitest run packages/cli/src/index.test.ts`
- `git diff --check`
- `CARGO_TARGET_DIR=/tmp/runx-codex-target pnpm verify:fast`

## Phase 1: Native publish command

Status: completed
Dependencies: none

Objective: `runx publish <receipt>` posts to the notary and prints the link.

Changes:
- Add a `publish` module that reads a sealed receipt from a path, resolves the hosted notary base URL from `--api-base-url` / `RUNX_PUBLIC_API_BASE_URL` / default, resolves the bearer token from `--token` / `RUNX_PUBLIC_API_TOKEN` / `RUNX_CONNECT_ACCESS_TOKEN`, and POSTs to the publish endpoint using the existing CLI HTTP client pattern; do not introduce a second HTTP stack.
- Route `runx publish` through the launcher and `main.rs`; add help text.
- Fail closed on missing token, unreadable/invalid JSON receipt, or a non-2xx response, each with an actionable message.
- Shape output from the notary response: print the `/r` link, hash, mode, and replay status.
- Add tests (mocked HTTP) for: parsing, endpoint/token precedence, and the exact full-receipt publish request body.

Acceptance:
- [x] `ac1` command - Rust formatting
  - Command: `cargo fmt --all --manifest-path crates/Cargo.toml --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-6
- [x] `ac2` command - CLI clippy clean under workspace bar
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo clippy --manifest-path crates/Cargo.toml -p runx-cli --all-targets -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-7
- [x] `ac3` command - Publish command tests pass
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target cargo test --manifest-path crates/Cargo.toml -p runx-cli publish -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-8

## Phase 2: Wrapper forwarding, help, docs

Status: completed
Dependencies: Phase 1

Objective: the wrapper forwards `publish` and the surface is documented.

Changes:
- Parse and dispatch top-level `publish` in the TypeScript wrapper; forward to the native binary without reimplementing behavior.
- Update help and README to advertise `runx publish` and disambiguate it from registry/skill publishing.
- Add a wrapper test that `publish` dispatches to the native binary.

Acceptance:
- [x] `ac4` command - Wrapper dispatch tests pass
  - Command: `pnpm vitest run packages/cli/src/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-19
- [x] `ac5` command - Diff hygiene
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-20
- [x] `ac6` command - Fast repo verification
  - Command: `CARGO_TARGET_DIR=/tmp/runx-codex-target pnpm verify:fast`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21

## Rollback

- Revert the launcher/parser additions and the new module; restore prior help and
  README. The command is additive and stateless, so rollback needs no migration.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: runx publish receipt-link CLI satisfies the spec contract. Native publish module (crates/runx-cli/src/publish.rs) reuses DefaultRuntimeHttpTransport from runx_runtime::registry (no new HTTP stack; no network code in runx-receipts/core); posts {publish:true, receipt:<json>} to POST {base}/v1/receipts/notarize with Bearer auth; resolves base URL precedence (--api-base-url > RUNX_PUBLIC_API_BASE_URL > https://runx.ai) and token precedence (--token > RUNX_PUBLIC_API_TOKEN > RUNX_CONNECT_ACCESS_TOKEN); fails closed with actionable errors on missing token, unreadable / non-JSON receipt, and non-2xx (with ErrorEnvelope preferred). Launcher routes `publish` before skill/registry/add and exposes PrintPublishHelp for `--help`/`-h` (launcher.rs:186-192). Text output now surfaces all enumerated notary fields including verdict (publish.rs:346-351, asserted by publish_tests.rs:174), and empty `--api-base-url=""` now falls back to env / default (publish.rs:250-264, asserted by publish_tests.rs:100-108) — addressing both prior-review findings. TS wrapper is launcher-only: `publish` dispatches via streamNativeRunx and isSupportedCommand requires receiptPublishPath; help/README advertise the command and disambiguate it from `runx skill publish` / `runx registry publish`. Acceptance evidence (cargo fmt, clippy -D warnings, cargo test publish, pnpm vitest, git diff --check, verify:fast) all pass. One low non-blocking observation: empty `--token=""` short-circuits the env fallback (asymmetric with the now-fixed `--api-base-url=""` path).

Attack log:
- `task scope vs workspace diff`: Spec compliance: cross-check every objective against publish.rs, launcher.rs, main.rs, README.md, packages/cli/src/{args,dispatch,help,index.test}.ts -> clean (Native publish reuses DefaultRuntimeHttpTransport via runx_runtime::registry (no new HTTP stack; runx-receipts/core remain network-free). POST {publish:true,receipt:<full json>} to /v1/receipts/notarize with Bearer auth and content-type application/json. Base URL precedence (--api-base-url > RUNX_PUBLIC_API_BASE_URL > https://runx.ai) and token precedence (--token > RUNX_PUBLIC_API_TOKEN > RUNX_CONNECT_ACCESS_TOKEN). TS wrapper forwards only (no behavior reimplemented). README and help advertise the command and disambiguate it from skill/registry publish.)
- `prior review findings`: Verify the two prior low findings (publish-text-omits-verdict, publish-empty-api-base-url-flag-not-filtered) are addressed -> clean (publish.rs:346-351 now renders verdict in text mode and publish_tests.rs:174 asserts `verdict:     {"valid":true}` appears in human output. publish.rs:250-264 now filters trimmed plan.api_base_url for !is_empty before falling through to env/default; publish_tests.rs:100-108 asserts `  /  ` falls back to https://runx.ai. Both prior findings are resolved in code.)
- `token resolution dark patterns`: Empty/whitespace tokens, --token="" vs env precedence asymmetry, missing token, token starting with - -> finding (resolve_publish_token chains or_else BEFORE filtering: plan.token = Some("") yields Some("") through both or_else calls and only fails at the final filter — env tokens never consulted. Asymmetric with resolve_public_api_base_url. Recorded as publish-empty-token-flag-blocks-env-fallback (low, non-blocking). Native flag_value rejects values starting with --; token with single-dash value passes through (acceptable).)
- `endpoint resolution dark patterns`: Empty --api-base-url, trailing slash handling, asymmetric flag/env behavior -> clean (trim_end_matches('/') handles both single and repeated trailing slashes, applied in both resolve_public_api_base_url and publish_receipt URL composition. Empty/whitespace --api-base-url is now filtered before fall-through to env, then default — fix from prior review is verified by publish_tests.rs:100-108.)
- `request shape`: Wire body conforms to POST /v1/receipts/notarize with publish:true plus full receipt; auth/content-type headers -> clean (publish_receipt sends {"publish":true,"receipt":<full json>} to {base}/v1/receipts/notarize with Authorization: Bearer <token> and Content-Type: application/json. publish_tests.rs:111-147 asserts URL, method, auth header, and exact body against StubTransport. Receipt is round-tripped through serde_json::Value (BTreeMap key ordering) — acceptable because the notary re-parses, not byte-compares.)
- `error envelope handling`: Non-2xx with runx-api error envelope vs raw body vs invalid JSON -> clean (publish.rs:297-310 prefers ErrorEnvelope (code/detail/hint/retry_after_seconds) when present, falls back to HttpStatus { status, body }. Success path deserializes ReceiptPublishResponse with serde defaults on optional fields. PublishError::RunxApi Display drops hint/retry_after_seconds — minor presentation gap, not blocking and out of spec scope.)
- `launcher dispatch ordering`: Regression hunt: confirm `runx publish` does not collide with `runx skill publish` / `runx registry publish`, reachable before skill/registry routing -> clean (launcher.rs:186 first_arg_is(args, "publish") fires only when first token is exactly `publish`. `runx skill publish` flows through the skill branch at launcher.rs:282; `runx registry publish` through registry at launcher.rs:271. PrintPublishHelp routed via nested_help_requested before parse_publish_plan.)
- `TS wrapper forwarding`: Regression hunt: no behavior reimplemented in TS, argv ordering matches native parser, --token/--api-base-url precedence forwarded -> clean (dispatch.ts:220-230 builds ["publish", resolvePathFromUserInput(path), --api-base-url?, --token?, --json?] and streams to native. index.test.ts:1026-1070 asserts exact forwarded argv. isSupportedCommand (args.ts:369) gates publish on receiptPublishPath. parseArgs (args.ts:192-197) reads positionals[0] for path and inputs.apiBaseUrl|api-base-url + inputs.token for flags, with isReceiptPublish omitInputs strip (args.ts:215).)
- `ambient drift attribution`: Scope drift: separate task-required edits from unrelated workspace changes -> clean (lib.rs (publish module export) and publish_tests.rs are task-required despite drift labelling. fixtures/cli-parity/commands.json and cases/oracle.json coherently register the publish command id and validate case (commands.json:347-376, oracle.json:188-196). Other ambient drift (runtime.rs, journal.rs, graph.rs, skill/resolver.rs, identity.rs, conformance.rs, generate-cli-feature-parity.ts, check-rust-core-style.mjs) does not touch publish behavior.)
- `human output presentation`: Spec compliance: text mode mirrors all enumerated notary fields -> clean (render_publish_result at publish.rs:315-353 emits digest+mode header, then status, published, public_hash, optional receipt_id, public_url, replay_status, and verdict. publish_tests.rs:149-176 asserts each line including verdict. JSON mode (publish.rs:319-322) serializes the full ReceiptPublishResponse. Spec's enumerated fields are all surfaced in both modes.)
- `transport security and timeouts`: Dark patterns: SSRF / private-network access via --api-base-url, request timeout, header validation -> clean (DefaultRuntimeHttpTransport::new() blocks private networks via GuardedDnsResolver and applies 30s request / 10s connect timeouts (runtime_http.rs:127-159). Token resolution trims whitespace before constructing the Bearer header so trailing-newline pastes don't trip header-value validation. Token never appears in error Display output (token is in headers, not the body).)

Findings:
- [low/non-blocking] `publish-empty-token-flag-blocks-env-fallback` Empty --token="" short-circuits the env-token fallback chain, asymmetric with the now-fixed --api-base-url="" path.
  - Location: `crates/runx-cli/src/publish.rs:266`
  - Evidence: resolve_publish_token chains `plan.token.as_deref().or_else(env.get(PUBLIC_API_TOKEN)).or_else(env.get(CONNECT_ACCESS_TOKEN)).map(str::trim).filter(!is_empty)`. If plan.token = Some("") (user passed --token="" or --token "$UNSET_VAR"), the first .as_deref() yields Some(""), so neither or_else fires; only at the final .filter does the value get dropped, yielding None / MissingToken — bypassing the env tokens entirely. resolve_public_api_base_url at publish.rs:250-264 was restructured (after the prior review) so the plan branch independently filters !is_empty and falls through; the token resolver retains the older shape.
  - Impact: Operators who write `runx publish receipt.json --token "$RUNX_TOKEN"` where the shell expands $RUNX_TOKEN to empty get the MissingToken error instead of the documented env fallback. Mirrors the previously fixed --api-base-url asymmetry; minor UX/consistency gap, not a correctness or security issue.
  - Validation: Trace resolve_publish_token(plan_with_token=Some("".into()), env_with_RUNX_PUBLIC_API_TOKEN="abc") — returns None despite a usable env token. Contrast with resolve_public_api_base_url under the same empty-flag/env-populated scenario, which returns the env value.

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: claude
- gap: frantic-architecture.md §15.1 gap 22

## Origin

Created by: Claude
Source: frantic-architecture.md §15.1 (2026-06-11 code re-audit), gap 22

## Harden Rounds

### round-1

Status: passed
Started: 2026-06-11T09:20:25Z
Ended: 2026-06-11T09:21:44Z

Observations:
- path
  - Result: clean
  - Anchor: code:crates/runx-cli/src/main.rs:41
  - Note: Native command routing already owns network-capable CLI commands; `publish` belongs beside registry/url-add in runx-cli.
- command
  - Result: clean
  - Anchor: code:crates/runx-cli/src/launcher.rs:307
  - Note: The top-level help/launcher command table is the authoritative public CLI surface to update.
- scope
  - Result: clean
  - Anchor: code:crates/runx-cli/src/registry.rs:70
  - Note: Reuse the existing CLI HTTP/config pattern; do not move network code into contracts, receipts, core, or runtime.
- timing
  - Result: advisory
  - Anchor: code:packages/cli/src/dispatch.ts:64
  - Note: TypeScript should forward `publish` to the native binary only; no TS implementation of receipt publishing.
- rollback
  - Result: clean
  - Anchor: spec_gap:rollback
  - Note: The command is additive and stateless, so rollback is a simple route/module/help removal.
- design
  - Result: clean
  - Anchor: spec_gap:objectives
  - Note: Output must mirror notary mode and never call a hash-only publish verified.


## Planning Log

- none
