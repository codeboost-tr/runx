import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { validateExternalAdapterResponseContract } from "./external-adapter.js";

// The example adapters in examples/ are authored against the shared adapter kit
// (examples/adapter-kit/adapter.mjs), which hand-builds the response frame. These
// tests spawn the real adapters the way the runtime does and validate the frame
// they emit against the same external-adapter response contract the Rust runtime
// enforces, so the kit cannot silently drift from the protocol.
const examplesRoot = new URL("../../../../examples/", import.meta.url);

function runExampleAdapter(relativePath: string, invocation: unknown): unknown {
  const adapter = fileURLToPath(new URL(relativePath, examplesRoot));
  const result = spawnSync(process.execPath, [adapter], {
    input: JSON.stringify(invocation),
    encoding: "utf8",
  });
  expect(result.status, result.stderr || result.error?.message).toBe(0);
  return JSON.parse(result.stdout) as unknown;
}

describe("example external adapters emit contract-conformant response frames", () => {
  it("the echo adapter (via the shared kit) emits a valid response frame", () => {
    const frame = runExampleAdapter("external-adapter-tool/adapter.mjs", {
      schema: "runx.external_adapter.invocation.v1",
      protocol_version: "runx.external_adapter.v1",
      invocation_id: "test-echo",
      adapter_id: "adapter.example.echo",
      source_type: "external-adapter",
      inputs: { message: "hi" },
    });
    const validated = validateExternalAdapterResponseContract(frame);
    expect(validated.schema).toBe("runx.external_adapter.response.v1");
    expect(validated.invocation_id).toBe("test-echo");
    expect(validated.adapter_id).toBe("adapter.example.echo");
  });

  it("the openapi adapter emits a valid response frame offline (dry-resolve fallback)", () => {
    const frame = runExampleAdapter("openapi-tool/openapi-adapter.mjs", {
      schema: "runx.external_adapter.invocation.v1",
      protocol_version: "runx.external_adapter.v1",
      invocation_id: "test-openapi",
      adapter_id: "adapter.example.openapi",
      source_type: "external-adapter",
      inputs: { operation_id: "getPet", petId: "p-7" },
    });
    const validated = validateExternalAdapterResponseContract(frame);
    expect(validated.schema).toBe("runx.external_adapter.response.v1");
    expect(validated.invocation_id).toBe("test-openapi");
  });

  it("a failing adapter still emits a contract-conformant failed frame", () => {
    const frame = runExampleAdapter("openapi-tool/openapi-adapter.mjs", {
      schema: "runx.external_adapter.invocation.v1",
      protocol_version: "runx.external_adapter.v1",
      invocation_id: "test-openapi-fail",
      adapter_id: "adapter.example.openapi",
      source_type: "external-adapter",
      inputs: { operation_id: "doesNotExist" },
    });
    const validated = validateExternalAdapterResponseContract(frame);
    expect(validated.schema).toBe("runx.external_adapter.response.v1");
    expect(validated.status).toBe("failed");
  });
});
