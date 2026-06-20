import { describe, expect, it } from "vitest";

import {
  parseRunnerManifestYaml,
  parseToolManifestYaml,
  SkillParseError,
  SkillValidationError,
  validateRunnerManifest,
} from "./index.js";

describe("CLI runner manifest parser", () => {
  it("rejects execution profile YAML references and document markers", () => {
    for (const yaml of [
      "---\nskill: example",
      "runners:\n  one:\n    outputs: &shared\n      result: string\n",
      "runners:\n  one:\n    outputs: *shared\n",
      "runners:\n  one:\n    runx:\n      <<: *shared\n",
      "runners:\n  one:\n    type: !custom graph\n",
    ]) {
      expect(() => parseRunnerManifestYaml(yaml), yaml).toThrow(SkillParseError);
    }
  });

  it("rejects duplicate execution profile mapping keys", () => {
    expect(() => parseRunnerManifestYaml(`
runners:
  one:
    type: agent
    type: graph
`)).toThrow(/duplicate mapping key/);
  });

  it("rejects unknown top-level and runner fields", () => {
    expect(() => validateRunnerManifest(parseRunnerManifestYaml(`
skill: example
unexpected: true
runners:
  one:
    type: agent
`))).toThrow(SkillValidationError);

    expect(() => validateRunnerManifest(parseRunnerManifestYaml(`
runners:
  one:
    type: agent
    typo_field: true
`))).toThrow(SkillValidationError);
  });

  it("accepts the governed act effect source fields", () => {
    const manifest = validateRunnerManifest(parseRunnerManifestYaml(`
version: "1"
runners:
  observe:
    type: http
    url: https://example.test/observe
    method: POST
    act:
      effect_field_from: effect_field
      effect_from_input: thread_locator
      effect_prefix_from: effect_prefix
`));

    expect(manifest.version).toBe("1");
    expect(manifest.runners.observe?.source.act).toMatchObject({
      effect_field_from: "effect_field",
      effect_from_input: "thread_locator",
      effect_prefix_from: "effect_prefix",
    });
  });

  it("normalizes nested http source declarations", () => {
    const manifest = validateRunnerManifest(parseRunnerManifestYaml(`
runners:
  fetch:
    source:
      type: http
      http:
        url: https://example.test/api
        method: GET
        headers:
          accept: application/json
        allow_private_network: false
`));

    expect(manifest.runners.fetch?.source.http).toEqual({
      url: "https://example.test/api",
      method: "GET",
      headers: {
        accept: "application/json",
      },
      allowPrivateNetwork: false,
    });
  });
});

describe("CLI tool manifest parser", () => {
  it("keeps YAML parity checks for tool manifests", () => {
    expect(() => parseToolManifestYaml("name: tool\ndescription: one: two\n")).toThrow(SkillParseError);
  });
});
