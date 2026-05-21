import { readFileSync } from "node:fs";

import { Value } from "@sinclair/typebox/value";
import { describe, expect, it } from "vitest";

import {
  externalAdapterCancellationFrameV1Schema,
  externalAdapterCredentialRequestV1Schema,
  externalAdapterHostResolutionFrameV1Schema,
  externalAdapterInvocationV1Schema,
  externalAdapterManifestV1Schema,
  externalAdapterResponseV1Schema,
  validateExternalAdapterCancellationFrameContract,
  validateExternalAdapterCredentialRequestContract,
  validateExternalAdapterHostResolutionFrameContract,
  validateExternalAdapterInvocationContract,
  validateExternalAdapterManifestContract,
  validateExternalAdapterResponseContract,
} from "./external-adapter.js";

const fixtureRoot = new URL("../../../../fixtures/contracts/external-adapter/", import.meta.url);

describe("external adapter protocol schemas", () => {
  it("validates manifest, invocation, response, host resolution, cancellation, and credential frames", () => {
    expect(validateExternalAdapterManifestContract(readExpected("manifest.json")).schema)
      .toBe("runx.external_adapter.manifest.v1");
    expect(validateExternalAdapterInvocationContract(readExpected("invocation.json")).schema)
      .toBe("runx.external_adapter.invocation.v1");
    expect(validateExternalAdapterResponseContract(readExpected("response.json")).schema)
      .toBe("runx.external_adapter.response.v1");
    expect(validateExternalAdapterHostResolutionFrameContract(readExpected("host-resolution-frame.json")).schema)
      .toBe("runx.external_adapter.host_resolution.v1");
    expect(validateExternalAdapterCancellationFrameContract(readExpected("cancellation-frame.json")).schema)
      .toBe("runx.external_adapter.cancellation.v1");
    expect(validateExternalAdapterCredentialRequestContract(readExpected("credential-request.json")).schema)
      .toBe("runx.external_adapter.credential_request.v1");
  });

  it("keeps external adapter responses as observations, not runtime-local result envelopes", () => {
    const response = {
      ...(readExpected("response.json") as Record<string, unknown>),
      status: "sealed",
      receipt_id: "receipt_should_not_cross_adapter_boundary",
    };

    expect(Value.Check(externalAdapterResponseV1Schema, response)).toBe(false);
    expect(() => validateExternalAdapterResponseContract(response)).toThrow();
  });

  it("rejects secret material in credential request frames", () => {
    const request = {
      ...(readExpected("credential-request.json") as Record<string, unknown>),
      secret_material: "ghp_do_not_cross_boundary",
    };

    expect(Value.Check(externalAdapterCredentialRequestV1Schema, request)).toBe(false);
    expect(() => validateExternalAdapterCredentialRequestContract(request)).toThrow();
  });

  it("rejects unknown fields on all top-level frame shapes", () => {
    expect(Value.Check(externalAdapterManifestV1Schema, withExtra("manifest.json"))).toBe(false);
    expect(Value.Check(externalAdapterInvocationV1Schema, withExtra("invocation.json"))).toBe(false);
    expect(Value.Check(externalAdapterResponseV1Schema, withExtra("response.json"))).toBe(false);
    expect(Value.Check(externalAdapterHostResolutionFrameV1Schema, withExtra("host-resolution-frame.json"))).toBe(false);
    expect(Value.Check(externalAdapterCancellationFrameV1Schema, withExtra("cancellation-frame.json"))).toBe(false);
    expect(Value.Check(externalAdapterCredentialRequestV1Schema, withExtra("credential-request.json"))).toBe(false);
  });
});

function readExpected(fixtureName: string): unknown {
  const fixture = JSON.parse(readFileSync(new URL(fixtureName, fixtureRoot), "utf8")) as {
    readonly expected: unknown;
  };
  return fixture.expected;
}

function withExtra(fixtureName: string): unknown {
  return {
    ...(readExpected(fixtureName) as Record<string, unknown>),
    unexpected: true,
  };
}
