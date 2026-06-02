---
name: openapi-adapter
description: External-adapter sub-skill; turns an OpenAPI operation into a governed tool call.
source:
  type: external-adapter
  external_adapter:
    manifest_path: manifest.json
inputs:
  operation_id:
    type: string
    required: true
    description: The OpenAPI operationId to invoke.
  petId:
    type: string
    required: false
    description: Path parameter for the getPet operation.
  fields:
    type: string
    required: false
    description: Optional query parameter.
---
An OpenAPI front, expressed as an external adapter. The adapter reads a
checked-in OpenAPI spec (`openapi.json`), resolves the requested operation,
validates parameters against the spec, and returns the resolved HTTP request as
the sealed result. This proves the governed core runs from an external spec, not
only from MCP: the same `external-adapter` lane carries any protocol.

Run it as a step in a graph; external-adapter is a graph-step front, not a
top-level runner.
