# @runxhq/langchain

Optional LangChain bridge for `runx`.

`runx` remains the kernel for policy, receipts, and execution. This package is an ecosystem bridge, not a second runtime.

## Rust takeover boundary

`@runxhq/langchain` remains an optional bridge after the Rust takeover. It
continues to invoke governed runx workflows through the `runx` CLI boundary
rather than becoming a runtime.

The old in-process LangChain tool-catalog adapter was sunset because the Rust
CLI has no stable boundary for registering arbitrary JavaScript tool instances.
Publish runx tool manifests and inspect/search them through `runx tool ... --json`
instead.

See the [TypeScript interop boundary](../../docs/ts-interop-boundary.md) for
the package disposition and ownership rules.

## APIs

- `createRunxCliSkillRunner(...)`
  Build a small runner over `runx skill <skill> --json`. It uses `RUNX_BIN` or
  `runx` by default and accepts CLI-scoped `env`, `cwd`, and `command` options.
- `createRunxLangChainTool(...)`
  Wrap a governed runx workflow as a LangChain tool without moving execution,
  approvals, or receipts into LangChain.
- `createLangChainToolCatalogAdapter(...)`
  Sunset API. Calling it throws with migration guidance.
