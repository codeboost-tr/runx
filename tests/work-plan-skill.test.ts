import path from "node:path";
import { readFile } from "node:fs/promises";

import { beforeAll, describe, expect, it } from "vitest";

import { runHarnessTarget } from "@runxhq/runtime-local/harness";
import { parseRunnerManifestYaml, validateRunnerManifest } from "@runxhq/core/parser";
import { ensureRunxBinary, kernelTestEnv } from "./host-protocol-test-utils.js";

describe("work-plan official skill", () => {
  beforeAll(() => {
    ensureRunxBinary();
  });

  it("ships as an explicit agent-step boundary with phased workspace-plan outputs", async () => {
    const manifest = validateRunnerManifest(
      parseRunnerManifestYaml(await readFile(path.resolve("skills/work-plan/X.yaml"), "utf8")),
    );
    const runner = manifest.runners["work-plan-agent"];

    expect(runner?.source.type).toBe("agent-step");
    if (!runner || runner.source.type !== "agent-step") {
      throw new Error("work-plan runner must declare an agent-step source.");
    }

    expect(runner.source.task).toBe("work-plan");
    expect(runner.source.outputs).toEqual({
      change_set: "object",
      harness_context: "object",
      objective_summary: "string",
      workspace_change_plan: "object",
      orchestration_steps: "array",
      required_skills: "array",
      open_questions: "array",
    });
    expect(runner.inputs.objective?.type).toBe("string");
    expect(runner.inputs.project_context?.type).toBe("string");
    expect(runner.inputs.change_set?.type).toBe("object");
  });

  it("passes the inline harness suite, including phased multi-repo decomposition", async () => {
    const result = await runHarnessTarget(path.resolve("skills/work-plan"), {
      env: kernelTestEnv(),
    });

    expect(result.source).toBe("inline");
    if (!("cases" in result)) {
      throw new Error("expected inline harness suite for work-plan");
    }
    expect(result.assertionErrors).toEqual([]);
    expect(result.cases.length).toBe(2);
    expect(result.cases.every((entry) => entry.status === "sealed")).toBe(true);
  });
});
