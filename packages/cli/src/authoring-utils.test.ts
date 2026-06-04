import { execFile } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

import { afterEach, describe, expect, it } from "vitest";

import { resolveRunxBinary } from "../../../tests/runx-binary.js";
import { hashToolSource } from "./authoring-utils.js";

const execFileAsync = promisify(execFile);
const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((directory) => rm(directory, { recursive: true, force: true })));
});

describe("hashToolSource", () => {
  it("matches the native Rust tool builder source_hash", async () => {
    const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-tool-source-hash-"));
    tempDirs.push(tempDir);
    const toolDir = path.join(tempDir, "tools", "demo", "hash_parity");
    await mkdir(path.join(toolDir, "src", "nested"), { recursive: true });
    await writeFile(
      path.join(toolDir, "src", "helper.ts"),
      `export const helper = "helper";\n`,
    );
    await writeFile(
      path.join(toolDir, "src", "nested", "index.ts"),
      `export const nested = "nested";\n`,
    );
    await writeFile(
      path.join(toolDir, "src", "phantom.ts"),
      `export const phantom = "this file is not imported";\n`,
    );
    await writeFile(
      path.join(toolDir, "src", "index.ts"),
      [
        `import { helper } from "./helper.js";`,
        `export { nested } from "./nested/index.js?cache=1";`,
        `const ignoredDouble = "escaped quote before path: \\"./phantom.js\\"";`,
        `const ignoredSingle = 'escaped quote before path: \\'./phantom.js\\'';`,
        `export const result = helper + ignoredDouble + ignoredSingle;`,
        "",
      ].join("\n"),
    );
    await writeFile(
      path.join(toolDir, "run.mjs"),
      `process.stdout.write(JSON.stringify({ ok: true }));\n`,
    );
    await writeFile(
      path.join(toolDir, "manifest.json"),
      `${JSON.stringify({
        name: "demo.hash_parity",
        version: "0.1.0",
        description: "Source hash parity fixture.",
        source: {
          type: "cli-tool",
          command: "node",
          args: ["./run.mjs"],
        },
        runtime: {
          command: "node",
          args: ["./run.mjs"],
        },
        inputs: {},
        output: {},
        scopes: [],
      }, null, 2)}\n`,
    );

    const expected = await hashToolSource(toolDir);
    const { stdout } = await execFileAsync(resolveRunxBinary(), ["tool", "build", toolDir, "--json"], {
      cwd: tempDir,
      env: process.env,
    });
    const report = JSON.parse(stdout) as {
      readonly status: string;
      readonly built?: readonly { readonly source_hash?: string }[];
    };

    expect(report.status).toBe("success");
    expect(report.built?.[0]?.source_hash).toBe(expected);
  });
});
