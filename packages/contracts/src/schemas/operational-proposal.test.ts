import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import {
  validateOperationalProposalContract,
} from "./operational-proposal.js";

const fixtureRoot = new URL("../../../../fixtures/contracts/operational-proposal/", import.meta.url);

describe("operational proposal schema", () => {
  it.each([
    "proposal-prepared.json",
    "proposal-blocked.json",
  ])("accepts positive fixture %s", (fixtureName) => {
    const proposal = readExpected(fixtureName);

    expect(validateOperationalProposalContract(proposal)).toMatchObject({
      schema: "runx.operational_proposal.v1",
      redaction_status: expect.any(String),
      source_ref: expect.objectContaining({
        type: expect.any(String),
        uri: expect.any(String),
      }),
      authority: {
        proposal_only: true,
        mutation_authority_granted: false,
        publication_authority_granted: false,
        final_decision_authority_granted: false,
      },
    });
  });

  it.each([
    "invalid-authority-claim.json",
    "invalid-missing-redaction.json",
    "invalid-missing-source-ref.json",
    "invalid-provider-specific-field.json",
    "invalid-product-specific-field.json",
  ])("rejects invalid fixture %s", (fixtureName) => {
    expect(() => validateOperationalProposalContract(readExpected(fixtureName))).toThrow();
  });
});

function readExpected(fixtureName: string): unknown {
  const fixture = JSON.parse(readFileSync(new URL(fixtureName, fixtureRoot), "utf8")) as {
    readonly expected: unknown;
  };
  return fixture.expected;
}
