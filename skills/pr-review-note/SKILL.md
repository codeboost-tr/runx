---
name: pr-review-note
description: Govern a GitHub PR review-note lane over MCP; comment scope is admitted, merge scope is refused.
---
# PR Review Note

This skill models the safe GitHub review-note lane: an operator may grant a
bounded PR comment scope without implicitly granting push or merge authority.
The harness proves both sides through the deterministic MCP fixture.
