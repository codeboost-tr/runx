---
spec_version: '2.0'
task_id: runx-core-kernel-eval-split-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-27T00:00:00Z'
status: completed
harden_status: passed
size: small
risk_level: medium
---

# runx core kernel eval split v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Review gate: pass

## Summary

`crates/runx-core/src/kernel_eval.rs` carried the public kernel eval API,
JSON size/shape limits, wire input definitions, supported-kind registry,
policy dispatch, and state-machine dispatch in one waived file. This spec
split those responsibilities while preserving the externally callable
`evaluate_kernel_document_str` API and error codes.

## Scope

- Keep `runx_core::kernel_eval::{evaluate_kernel_document_str,
  KernelEvalError, KernelEvalOutput}` stable.
- Move JSON size and structural limits to `kernel_eval/limits.rs`.
- Move wire input enums and supported-kind registry to `kernel_eval/input.rs`.
- Move policy and state-machine dispatch to `kernel_eval/dispatch.rs`.
- Remove the stale large-file waiver from `kernel_eval.rs`.
- Do not change error classification, error messages, serialized output, or
  kernel fixture behavior.

## Evidence

Commands run after implementation:

```sh
cargo fmt --manifest-path crates/Cargo.toml --all
cargo test --manifest-path crates/Cargo.toml -p runx-core kernel_eval
cargo test --manifest-path crates/Cargo.toml -p runx-core --test kernel_eval
cargo clippy --manifest-path crates/Cargo.toml -p runx-core --all-targets -- -D warnings
```

All commands passed.

## Review Notes

- The split is internal to `runx-core`; runtime and CLI callers keep using the
  same public module path.
- The only dirty files observed outside this spec were CLI test and runtime
  MCP fixture files owned by concurrent work. This spec did not touch them.
