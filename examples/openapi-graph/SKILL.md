---
name: openapi-graph
description: OpenAPI front example; a graph whose step turns an OpenAPI operation into a governed tool call.
---
# OpenAPI graph

A single-step graph that drives the OpenAPI external-adapter sub-skill. The
runtime routes the graph step's `external-adapter` source through the
source-adapter registry to the external-adapter executor, which spawns the
adapter under the governed sandbox. The adapter resolves an OpenAPI operation
into a concrete HTTP request and the runtime seals it.

This is the concrete proof that the core runs from other specs, not just MCP.
Run the inline harness with `runx harness examples/openapi-graph`.
