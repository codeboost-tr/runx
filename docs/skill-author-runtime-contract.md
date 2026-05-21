# Skill Author Runtime Contract

This document defines the author-visible v1 subprocess ABI for `cli-tool`
skills. It is shared by the TypeScript adapter while it survives and the Rust
runtime cutover. Internal receipt IDs, artifact IDs, sandbox metadata internals,
and temporary paths are not part of this contract unless named here.

## Process

The runtime starts the declared command with `shell: false` semantics. Arguments
are resolved before spawn. The skill process runs with piped stdin, stdout, and
stderr. Stdout and stderr are drained while the process runs and each stream is
captured up to 1 MiB without emitting broken UTF-8.

## Environment

The child environment is deny-by-default. The sandbox allowlist admits only
declared host variables plus runtime-authored `RUNX_*` variables.

Guaranteed variables:

- `RUNX_CWD`: the workspace root, resolved as `RUNX_CWD ?? INIT_CWD ?? current_dir`.
- `RUNX_INPUTS_JSON`: serialized inputs when the full input payload is at most 48 KiB.
- `RUNX_INPUTS_PATH`: path to a UTF-8 JSON file when the full input payload is larger than 48 KiB.
- `RUNX_INPUT_<NAME>`: per-input scalar/stringified value when the serialized value is at most 8 KiB.

Input env names are normalized by replacing non-alphanumeric runs with `_`,
trimming separators, and uppercasing. For example, `thread.title` becomes
`RUNX_INPUT_THREAD_TITLE`.

Large per-input values are omitted from `RUNX_INPUT_*`; authors must read
`RUNX_INPUTS_JSON` or `RUNX_INPUTS_PATH` for the full payload.

## Stdin

When `inputMode` is `stdin`, stdin receives the full input object as JSON and
then closes. Otherwise stdin closes without input.

## Cwd Policy

Relative source cwd values resolve from the skill directory. Non-unrestricted
profiles fail closed when cwd escapes the declared policy boundary:

- `skill-directory`: cwd must stay within the skill directory.
- `workspace`: cwd must stay within `RUNX_CWD ?? INIT_CWD ?? current_dir`.
- `custom`: cwd must stay within the skill directory or workspace.

`unrestricted-local-dev` may escape after explicit approval metadata, but the
runtime must not claim approval when no runner approval was supplied.

## Timeout

Timeout is terminal. On Unix, the runtime starts the skill in a new process
group, sends `SIGTERM` to the group, then sends `SIGKILL` after a short grace
period. Non-Unix runtimes must at least terminate the direct child and report
the platform limitation in tests or docs.

## Output

A zero exit code without timeout or abort maps to a sealed/success status.
Timeout, abort, spawn failure, or non-zero exit maps to failure. Structured JSON
stdout remains author output; graph runners may parse object stdout into step
outputs, but raw stdout and stderr remain visible.

## Fixture Gate

`pnpm fixtures:skill-author-runtime:check` runs the same fixture entrypoint
through the TypeScript adapter and Rust runtime. The gate compares only
author-visible behavior: status, stdout/stderr, exit code where relevant,
parsed stdout JSON, cwd relation, input delivery mode, output truncation, and
descendant timeout cleanup.
