---
spec_version: '2.0'
task_id: runx-core-authority-proof-split-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-27T00:00:00Z'
status: completed
harden_status: passed
size: small
risk_level: medium
---

# runx core authority proof split v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Review gate: pass

## Summary

`crates/runx-core/src/policy/authority_proof.rs` combined local connected-auth
scope admission, credential binding validation, authority-proof projection, and
sandbox summary extraction in one waived file. This spec decomposed those
responsibilities into focused modules while preserving the public policy facade:

- `build_local_scope_admission`
- `validate_credential_binding`
- `build_authority_proof`
- `build_authority_proof_metadata`

No fixture input names, serialized policy output shapes, decision strings, or
public `runx_core::policy` exports were intentionally changed.

## Scope

- Keep the public `policy` exports stable.
- Move local scope-admission logic to `authority_proof/admission.rs`.
- Move credential binding validation to `authority_proof/binding.rs`.
- Move authority-proof metadata projection to `authority_proof/projection.rs`.
- Move sandbox metadata/declaration summarization to
  `authority_proof/sandbox_summary.rs`.
- Keep only small normalization helpers in `authority_proof/util.rs`.
- Remove the stale `large-file` waiver from `authority_proof.rs`.
- Do not touch runtime, CLI, MCP, payment, or active release-readiness work.

## Evidence

Commands run after implementation:

```sh
cargo fmt --manifest-path crates/Cargo.toml --all
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo test --manifest-path crates/Cargo.toml -p runx-core --lib policy
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo test --manifest-path crates/Cargo.toml -p runx-core --test policy_fixtures --no-run
CARGO_TARGET_DIR=/tmp/runx-authority-proof-split-target cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings
rg -n "rust-style-allow: large-file" crates/runx-core/src/policy/authority_proof.rs crates/runx-core/src/policy/authority_proof
git diff --check -- .scafld/specs/archive/2026-05/runx-core-authority-proof-split-v1.md crates/runx-core/src/policy/authority_proof.rs crates/runx-core/src/policy/authority_proof
```

All commands passed. A direct execution attempt for
`cargo test -p runx-core --test policy_fixtures` compiled successfully, then
the integration-test binary stalled before Rust test startup at the macOS
loader. Other concurrent integration-test binaries in the workspace showed the
same loader symptom, so this slice used the policy library tests, the
policy-fixture compile gate, and full `runx-core --all-targets` clippy as its
review evidence.

## Review Notes

- This is an internal decomposition only; policy fixture dispatch still calls
  the same public functions through `runx_core::policy`.
- Existing dirty files were present in CLI, runtime MCP, and adapter fixtures.
  This spec did not touch them.
