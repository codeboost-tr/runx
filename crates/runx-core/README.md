# runx-core

Pure Rust parity kernel for runx state-machine and policy decisions.

This crate currently implements state-machine parity against the TypeScript
oracle fixtures under `fixtures/kernel/state-machine/` and policy parity against
the checked-in policy fixture set under `fixtures/kernel/policy/`. The policy
surface includes local admission, sandbox normalization/admission, retry,
graph-scope, authority proof, credential binding, scope admission, and public
work helpers.

TypeScript remains the source of truth until a separate cutover spec changes
consumers.

`runx-core` must stay free of filesystem, network, subprocess, MCP, adapter,
and CLI presentation behavior.
