---
spec_version: '2.0'
task_id: test-surface-build-consolidation
created: '2026-05-27T13:45:00Z'
updated: '2026-06-04T20:50:02Z'
status: review
harden_status: not_run
size: large
risk_level: medium
---

# Consolidate the test build surface

## Current State

Status: review
Current phase: final
Next: complete
Reason: review gate pass: 3 finding(s), 0 completion blocker(s)
Blockers: none
Allowed follow-up command: `scafld complete test-surface-build-consolidation`
Latest runner update: 2026-06-05T00:53:36Z
Review gate: pass

## Summary

The long runx CI job is build-bound, not test-bound. Test execution is already
fast: cloud vitest runs 527 tests in about 11s, and oss `verify:fast` is about
1.5min. The time goes into Rust compilation and linking, paid more than once per
run.

The dominant cause is that every top-level `tests/*.rs` file is compiled and
linked as its own crate. Cargo documents this and explicitly recommends a single
integration test split into modules when many integration tests make compile or
run time inefficient (Cargo Book, "Integration tests":
https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests).
For runx the case is stronger because `runx-runtime` links heavy adapter and
runtime deps (reqwest, tokio, rustls, rmcp) into every test binary.

This spec collapses the integration-test binaries to one per crate (the layout
several runx crates already imply), adds a guard so the `autotests = false`
layout cannot silently drop coverage, adopts cargo-nextest as the runner with
doctests kept as a separate `cargo test --doc` step, installs advisory tools as
prebuilt binaries, and tightens caching and job structure. No test assertions,
fixtures, or coverage are removed.

## Measured Baseline

Isolated worktree, clean target dir, deps and lib pre-warmed identically for both
runs, `runx-runtime --all-features`, `cargo test --no-run`:

- Current (42 separate test binaries): 244s to build, 382 tests.
- Merged (1 binary, same files as modules): 7s to build, same 382 tests.
- Result: about 35x on the test-build phase for one crate, identical tests.

Mechanism confirmed: test-source codegen is cheap; the cost is 42 separate linker
invocations, each statically linking `runx_runtime` plus reqwest/tokio/rustls/rmcp
into a ~51MB executable. `runx-runtime` is 42 of the workspace's integration
binaries, so the absolute saving is larger again.

Supporting signal: `oss/crates/target` was 183GB locally (cache bloat from
feature and toolchain permutations).

## Phase 1 Evidence (implemented)

Consolidated to one `tests/integration.rs` binary per crate, `autotests = false`,
each former test file kept intact as a module:

| crate | files merged | tests | suite result |
| --- | --- | --- | --- |
| runx-runtime | 42 | 383 | behavior-neutral vs baseline (see below) |
| runx-contracts | 17 | 80 | 80 passed |
| runx-cli | 13 + shared `support` | 74 | 74 passed |
| runx-core | 6 | 50 | 50 passed |
| runx-receipts | 3 | 26 | 26 passed |
| runx-parser | 5 | 20 | 20 passed |
| runx-sdk | 3 | 6 | 6 passed |

- Behavior-neutrality proof: under an identical local invocation, `runx-runtime`
  reports 372 pass / 11 fail both before consolidation (pristine HEAD worktree,
  42 separate binaries) and after (1 binary). The 11 failures are pre-existing,
  environment-dependent tests (receipt-signing issuer-key resolution) that only
  pass under the full verify-fast/CI orchestration; they are unrelated to and
  unaffected by consolidation.
- nextest validation: `cargo nextest run --workspace --all-features` runs 789
  tests, 778 pass, the same 11 runtime failures, nothing new. Process-per-test
  execution is green.
- `act.rs` (sdk) needed a structural fix because it owned submodules: it had a
  redundant inline `mod {}` wrapper that, once the file itself became a module,
  nested submodule paths one level too deep; the wrapper was removed so
  submodules resolve under `tests/<name>/`. `runx-pay/tests/payment.rs` remains
  unconsolidated because it is already a single integration-test file. `runx-cli`'s
  shared `tests/support/` is declared once in integration.rs and the five files
  that used it now reference `crate::support`.
- Every `--test <name>` invocation that targeted a now-removed standalone target
  was retargeted to `--test integration -- <module>` and verified to run the
  same subset (5 package.json scripts, the CI license-boundary guard, and the
  a2a/agent fixture-generator hint strings).

## Standard adopted

The forward standard for runx Rust testing:

- One integration binary per crate (`tests/integration.rs` + `autotests = false`).
- cargo-nextest as the normal runner (process-per-test isolation; see
  https://nexte.st/docs/design/why-process-per-test/), which removes the only
  regression introduced by sharing one binary: process-global state leaking
  across tests under threaded `cargo test`.
- Doctests run as a separate `cargo test --doc` step, because nextest does not
  execute doctests.
- A module-list and process-global-mutation guard
  (`scripts/check-integration-test-modules.mjs`, wired into verify-fast) so a new
  `tests/*.rs` cannot be silently un-compiled and so `env::set_var` /
  `set_current_dir` style mutations are banned in test code unless explicitly
  isolated and annotated.

## Scope

- In scope:
  - `crates/*/tests/` layout and the corresponding `[[test]]` / `autotests`
    manifest entries (done).
  - The module/process-global guard and its wiring (done).
  - `oss/.github/workflows/ci.yml` runner and advisory-tool steps (done).
  - Retargeting every `--test <name>` reference (done).
  - Follow-up CI structure: feature-surface review, job parallelization, cache.
- Out of scope:
  - Changing any test assertion, fixture, or expected value.
  - Removing or weakening any gate, including heavy graph and all-features tests.
  - JS/vitest test logic (already fast).
  - The stable/nightly toolchain arrangement: nightly exists specifically for
    `cargo-public-api` in the advisory parity step (see archived
    `rust-parity-ci-governance.md`); it is not removable here.

## Dependencies

- `heavy-test-suite-gating` (completed) established that CI must run
  `pnpm test:heavy:graph` and an all-features cargo test gate with a prebuilt
  eval binary. This spec keeps both: the heavy graph step is unchanged and the
  all-features gate is now `cargo nextest run --workspace --all-features` plus
  `cargo test --workspace --all-features --doc`.
- `runx-rust-95-release-readiness` (active) owns the mandatory Rust gates; the
  runner change here keeps those gates green.

## Risks

- Shared process state under threaded `cargo test`: addressed by the guard
  (bans process-global mutation; the scan found none) and by nextest's
  process-per-test execution.
- The 11 environment-dependent runtime tests must be confirmed green in CI under
  nextest; they are runner- and layout-independent (fail identically under
  cargo test and nextest, before and after consolidation), so this is a
  pre-existing orchestration concern, not a regression from this work.
- CI workflow edits cannot be fully validated locally; they take effect only on
  commit, so a dry-run on a branch is the gate before merge.

## Acceptance

- [x] `dod1` Each crate with tests builds exactly one integration binary; total
  test count unchanged. Evidence: 7 crates each emit one `tests/integration.rs`
  executable; per-crate counts recorded above (639 integration tests).
- [x] `dod2` Consolidated suites pass with no shared-state flakiness. Evidence:
  6/7 crates fully green; runtime behavior-neutral vs baseline; nextest green
  except the pre-existing 11.
- [x] `dod3` Heavy graph gate and an all-features cargo gate remain enforced in
  CI. Evidence: heavy graph step unchanged; `cargo nextest run --workspace
  --all-features` + `cargo test --workspace --all-features --doc` in Rust checks.
- [x] `dod4` Advisory tools installed prebuilt, not compiled from source.
  Evidence: `taiki-e/install-action` installs cargo-nextest, cargo-deny,
  cargo-public-api; the `cargo install` step is removed.
- [x] `dod5` Guard prevents silent coverage loss and bans process-global
  mutation. Evidence: `scripts/check-integration-test-modules.mjs` fails on an
  orphaned `tests/*.rs`, directory-style `tests/<name>/main.rs` targets under
  `autotests = false`, unresolved declared modules, and `env::set_var`; wired
  into verify-fast.
- [x] `dod6` Warm CI wall time materially reduced; before/after recorded from a
  real CI run. Evidence: branch `codex/readiness-ci-dry-run` ran `ci`
  workflow_dispatch successfully twice. First run
  `26987384653` (`995dd53b`, checks job
  <https://github.com/runxhq/runx/actions/runs/26987384653/job/79639879363>)
  completed in 8m30s. Warm rerun `26987685828` (`905290f7`, checks job
  <https://github.com/runxhq/runx/actions/runs/26987685828/job/79640782639>)
  completed in 7m34s. Final code run `26988331644` (`63d9350d`,
  checks job
  <https://github.com/runxhq/runx/actions/runs/26988331644/job/79642794879>)
  completed in 7m51s after the directory-target guard hardening.

## Phase 2: CI caching and structure

Status: completed

Implemented:

- Swapped the oss Cargo cache from `actions/cache` (keyed on `Cargo.lock`, target
  tree grew unbounded) to `Swatinem/rust-cache@v2` with `workspaces: crates`, the
  same Rust-aware cache the cloud workflow already uses. It keys on lockfile and
  rustc version and prunes stale artifacts.

Analyzed and deliberately NOT done:

- Job parallelization (split `checks` into parallel `verify` and `rust` jobs) was
  rejected after analysis: on a cold run each job restores its own cache and
  recompiles the dependency graph in its own target, so the expensive ~10min dep
  compile is paid twice and wall time (max of the jobs) does not improve; it only
  helps warm runs (already fast) by a couple of minutes. Low value, and it makes
  the cold case worse. Not pursued.
- Feature-surface "unification": clippy and the test build cannot share compiled
  artifacts (clippy wraps workspace crates), but heavy dependency artifacts ARE
  shared between them, and clippy/test already use the same `--all-features` set.
  There is no safe further consolidation here; closed.

Recommended next lever (needs a CI dry-run before merge):

- sccache as a persistent compilation cache (`RUSTC_WRAPPER=sccache`, GHA-cache
  backend, `CARGO_INCREMENTAL=0`). This is the real lever for the cold case that
  produced the original ~40min run: when `Cargo.lock` changes only some deps,
  sccache reuses cached object files for the unchanged ~297 deps instead of
  recompiling the whole graph. It coexists with clippy's `RUSTC_WORKSPACE_WRAPPER`.
  Left unimplemented because a misconfigured wrapper silently disables caching or
  breaks the build, and it cannot be validated locally.

Optional, lower priority:

- Gate `cargo package -p runx-cli` to pushes on main and tags rather than every
  PR (it does a full isolated rebuild). Mildly reduces per-PR publish safety, so
  decide explicitly.

- [x] `p2_ac1` command - oss Cargo cache uses rust-cache.
  - Command: `rg -n "Swatinem/rust-cache" .github/workflows/ci.yml`
  - Expected kind: `reviewed_output`
  - Status: pass
  - Evidence: `.github/workflows/ci.yml:72` uses `Swatinem/rust-cache@v2`.
- [x] `p2_ac2` manual - branch CI run green end to end under nextest; warm-run
  wall time recorded before/after.
  - Status: pass
  - Evidence: `ci` workflow_dispatch on branch `codex/readiness-ci-dry-run`
    passed at run `26987384653`, warm rerun `26987685828`, and final code
    run `26988331644`; each includes the Rust checks step with
    `cargo nextest run --workspace --all-features`.

## Rollback

- Phase 1 is the high-value change and the only one touching test layout. Each
  crate's consolidation is independent; if a crate ever cannot run cleanly in one
  binary, leave it unconsolidated rather than weakening tests.
- CI changes are revertible by restoring the prior `ci.yml`; no test content
  changes, so coverage is unaffected by any rollback.
- Keep each lever in its own commit so it can be reverted independently.

## Review

Status: completed
Verdict: pass
Mode: discover
Provider: claude:claude-opus-4-8
Output: claude.mcp_submit_review
Summary: Discover-mode re-review of the test-build consolidation. The only code change since the last passing review is commit 63d9350d (directory-target guard hardening), which closes the previously-flagged blind spot. Verified independently: all 7 crates the spec lists (runtime/contracts/cli/core/receipts/parser/sdk) set autotests=false plus a single [[test]] name="integration" target. runtime/tests/integration.rs declares exactly 42 sibling modules matching the 42 tests/*.rs files on disk; cli/tests/integration.rs declares 13 modules plus shared `support` (resolved via tests/support/mod.rs); sdk sets autotests=false with the integration target. The guard (scripts/check-integration-test-modules.mjs) now enforces three coverage axes — orphaned top-level tests/*.rs (95-106), directory-style tests/<name>/main.rs targets dropped under autotests=false (108-120, the new hardening), and unresolved declared modules (122-130) — plus the process-global mutation ban (33-39,133-154). A glob for crates/*/tests/*/main.rs returns nothing, so the new directory check neither false-fails CI nor masks current coverage loss. The guard runs in verify:fast's unconditional source-checks group (line 45), and CI runs pnpm verify:fast (ci.yml:52). CI gates are intact: nextest + cargo test --doc + heavy-graph (ci.yml:84-87,76-77), prebuilt advisory tools via taiki-e/install-action with no cargo install (62-66), Swatinem/rust-cache@v2 workspaces: crates (72-74), and the license-boundary retarget `--test integration -- license_boundary` (95) maps to runtime's declared license_boundary module. No new completion blockers found. The prior directory-blindspot finding is now fixed. One residual low/non-blocking item carries forward: dod6/p2_ac2 greenness and warm-time evidence rests on external GitHub Actions runs a read-only reviewer cannot fetch (cited SHAs 995dd53b/905290f7/63d9350d are real repo commits, lowering risk). One additional low note: the guard's mod-detection regex only matches `mod name;`, so a future `pub mod name;` declaration would be a latent false-positive — not present today. No test assertions, fixtures, or coverage removed; no regression introduced.

Attack log:
- `crates/*/Cargo.toml + tests/integration.rs (dod1)`: Confirm each consolidated crate emits exactly one integration binary (autotests=false + [[test]] name=integration) and integration.rs declares every sibling test file -> clean (Grep found autotests=false in all 7 spec-listed crates. runtime integration.rs declares 42 modules matching 42 tests/*.rs files on disk; cli declares 13 + support (support/mod.rs); sdk sets autotests=false with [[test]] name=integration path=tests/integration.rs.)
- `scripts/check-integration-test-modules.mjs (dod5 directory hardening, commit 63d9350d)`: Verify the new directory-style target check correctly closes the prior blind spot without false-failing CI -> clean (Lines 108-120 flag tests/<name>/main.rs under autotests=false. Glob crates/*/tests/*/main.rs returns none, so no false failure and no current coverage drop. Prior finding fixed.)
- `crates/*/tests (regression: directory-style targets dropped)`: Search for directory-style integration targets that would silently lose coverage or trip the new guard -> clean (Glob crates/*/tests/*/main.rs returned no files; cli tests/support is a directory with mod.rs (not main.rs), so it is correctly not flagged and resolves via moduleHasSource.)
- `.github/workflows/ci.yml + scripts/verify-fast.mjs (guard wiring)`: Confirm the coverage guard actually executes in CI and is not a dead script -> clean (verify-fast.mjs:45 runs node scripts/check-integration-test-modules.mjs in the unconditional parallel source-checks group; ci.yml:52 runs pnpm verify:fast. Guard cannot be bypassed silently.)
- `.github/workflows/ci.yml (dod3/dod4/p2_ac1)`: Confirm nextest + doctest + heavy-graph gates, prebuilt advisory tools with no cargo install, and Swatinem rust-cache -> clean (ci.yml:62-66 taiki-e/install-action installs cargo-nextest,cargo-deny,cargo-public-api; 72-74 rust-cache workspaces: crates; 84 nextest --all-features; 87 cargo test --doc; 76-77 heavy graph. No cargo install present.)
- `.github/workflows/ci.yml:95 (regression: --test retargeting)`: Confirm the license-boundary guard step still runs the same module subset after consolidation -> clean (cargo test ... -p runx-runtime --all-features --test integration -- license_boundary filters to the license_boundary:: module; runtime integration.rs declares mod license_boundary (line 28). Filter can only add, never drop, matching tests.)
- `scripts/check-integration-test-modules.mjs:53-61 (guard regex robustness)`: Probe the mod-detection regex for declaration forms that would misclassify a declared module -> finding (Regex matches only `mod name;`, not `pub mod name;`; a future pub-qualified declaration would false-fail the guard. Latent only — all current files use plain `mod name;`.)
- `dod6/p2_ac2 external CI evidence`: Cross-check cited CI run commit SHAs against git history and assess whether greenness is verifiable read-only -> finding (SHAs 995dd53b, 905290f7, 63d9350d are real recent repo commits, but Actions job greenness is not fetchable in read-only mode (low residual).)
- `workspace classification / spec mutation`: Confirm changes are limited to CI tooling + spec evidence with no out-of-scope drift or production test-logic -> clean (scafld reports baseline clean, task_changes none, ambient_drift none. Guard and ci.yml are CI tooling, not production code; test-logic-separation invariant preserved. Implementation landed in prior commits.)

Findings:
- [low/non-blocking] `guard-directory-target-blindspot` Prior finding: coverage guard ignored directory-style test targets. Now fixed in commit 63d9350d.
  - Location: `oss/scripts/check-integration-test-modules.mjs:108`
  - Evidence: The guard now iterates testsDir entries and fails when a directory contains main.rs (lines 108-120), telling the author such a target is dropped under autotests=false. A glob for crates/*/tests/*/main.rs returns no files, so the check neither false-fails CI nor masks current coverage loss.
  - Impact: Closes the dod5 'prevents silent coverage loss' gap for directory-style targets.
  - Validation: Read scripts/check-integration-test-modules.mjs:108-120; Glob crates/*/tests/*/main.rs returned no files.
- [low/non-blocking] `dod6-external-run-unverifiable` dod6/p2_ac2 green-CI and warm-time evidence rests on external GitHub Actions runs not fetchable in read-only review.
  - Location: `oss/.scafld/specs/active/test-surface-build-consolidation.md:177`
  - Evidence: dod6 cites workflow_dispatch runs 26987384653 (995dd53b), 26987685828 (905290f7), and 26988331644 (63d9350d). All three SHAs are real commits in this repo (63d9350d/905290f7 are recent HEAD-area commits), corroborating real referenced commits, but a read-only reviewer cannot open the Actions job URLs to confirm the runs were green.
  - Impact: The criteria depending on real CI behavior (suite green under nextest; warm wall time reduced) cannot be independently confirmed locally. Operator should open the cited job URLs and confirm green before complete.
  - Validation: Cross-checked cited SHAs against git history; external run greenness not fetchable in read-only mode.
- [low/non-blocking] `guard-mod-regex-pub-mod-brittleness` Guard mod-detection regex only matches `mod name;`, so a future `pub mod name;` declaration would be a latent false-positive CI failure.
  - Location: `oss/scripts/check-integration-test-modules.mjs:57`
  - Evidence: topLevelModNames uses /^mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/. A `pub mod name;` declaration is not captured, so the matching tests/name.rs would be reported as 'exists but not declared' (lines 99-106) and fail the guard even though the module is in fact declared. All current integration.rs files use plain `mod name;`, so this is latent, not active.
  - Impact: A reasonable future edit using `pub mod` would red the guard with a misleading message. Not present today; does not drop coverage.
  - Validation: Read lines 53-61 and 95-106; confirmed all current integration.rs files declare modules as `mod name;` at column 0.

Post-review follow-up:

- `guard-mod-regex-pub-mod-brittleness` fixed immediately after review by
  accepting plain `mod`, `pub mod`, and restricted `pub(...) mod` declarations at
  column 0 in `scripts/check-integration-test-modules.mjs`. Verified with
  `node scripts/check-integration-test-modules.mjs` and `pnpm verify:fast`.
