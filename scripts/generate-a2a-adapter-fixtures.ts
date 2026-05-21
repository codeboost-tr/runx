import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const fixtureRoot = path.join(workspaceRoot, "fixtures", "runtime", "adapters", "a2a");
const oracleRoot = path.join(fixtureRoot, "oracles");
const check = process.argv.includes("--check");

process.chdir(workspaceRoot);

type JsonRecord = Record<string, unknown>;

interface OracleCase {
  readonly name: string;
  readonly expectedStatus: "sealed" | "failure";
}

const cases: readonly OracleCase[] = [
  { name: "fixture-success", expectedStatus: "sealed" },
  { name: "fixture-failure-sanitized", expectedStatus: "failure" },
  { name: "missing-metadata", expectedStatus: "failure" },
  { name: "embedded-template", expectedStatus: "sealed" },
  { name: "exact-template", expectedStatus: "sealed" },
  { name: "resolved-inputs", expectedStatus: "sealed" },
  { name: "unsupported-agent-card", expectedStatus: "failure" },
];

const owner = {
  spec: ".scafld/specs/archive/2026-05/rust-runtime-adapters-a2a.md",
  rustTest: "crates/runx-runtime/tests/a2a_parity.rs",
  cargo: "cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features a2a,agent --test a2a_parity",
} as const;

if (!check) {
  throw new Error(
    "A2A adapter oracle generation is retired; checked-in fixtures are Rust-owned. "
      + "Run this script with --check and refresh behavior through the Rust owner if needed.",
  );
}

await assertCompletedRustOwner();

for (const oracleCase of cases) {
  await assertCaseFixture(oracleCase);
}
await checkNoStaleOracleFiles();

console.log(`checked ${cases.length} A2A adapter oracle cases (retired TS generator; Rust owner: ${owner.rustTest})`);

async function assertCompletedRustOwner(): Promise<void> {
  const spec = await readFile(path.join(workspaceRoot, owner.spec), "utf8");
  if (!/^status:\s*completed$/mu.test(spec) || !/^Review gate:\s*pass$/mu.test(spec)) {
    throw new Error(`${owner.spec} does not declare completed Rust ownership with a passing review gate.`);
  }
  const rustTest = await readFile(path.join(workspaceRoot, owner.rustTest), "utf8");
  for (const required of ["A2aAdapter", "FixtureA2aTransport", "run_harness_fixture_with_adapter"]) {
    if (!rustTest.includes(required)) {
      throw new Error(`${owner.rustTest} is missing Rust A2A ownership marker ${required}.`);
    }
  }
}

async function assertCaseFixture(oracleCase: OracleCase): Promise<void> {
  const requestPath = path.join(casePath(oracleCase.name), "request.json");
  const request = await readJson(requestPath);
  assertEqual(request.case, oracleCase.name, `${relative(requestPath)} case`);
  assertEqual(request.mode, "a2a-adapter", `${relative(requestPath)} mode`);
  assertEqual(recordField(request, "source").type, "a2a", `${relative(requestPath)} source.type`);
  assertNoPackageBoundary(requestPath, JSON.stringify(request));

  for (const extension of ["stdout", "stderr", "json"] as const) {
    const oraclePath = path.join(oracleRoot, `${oracleCase.name}.${extension}`);
    const contents = await readFile(oraclePath, "utf8");
    assertCleanOracle(oracleCase.name, oraclePath, contents);
    if (extension === "json") {
      const receipt = parseJson(contents, oraclePath);
      assertEqual(receipt.status, oracleCase.expectedStatus, `${relative(oraclePath)} status`);
    }
  }

  const statusPath = path.join(oracleRoot, `${oracleCase.name}.status`);
  const status = await readFile(statusPath, "utf8");
  assertCleanOracle(oracleCase.name, statusPath, status);
  assertEqual(status, `${oracleCase.expectedStatus}\n`, `${relative(statusPath)} contents`);
}

async function checkNoStaleOracleFiles(): Promise<void> {
  const expectedOracleFiles = new Set<string>();
  for (const oracleCase of cases) {
    for (const extension of ["stdout", "stderr", "status", "json"] as const) {
      expectedOracleFiles.add(path.join(oracleRoot, `${oracleCase.name}.${extension}`));
    }
  }
  for (const filePath of await collectFiles(oracleRoot)) {
    if (!expectedOracleFiles.has(filePath)) {
      throw new Error(`stale A2A adapter oracle file: ${relative(filePath)}`);
    }
  }
}

async function readJson(filePath: string): Promise<JsonRecord> {
  return parseJson(await readFile(filePath, "utf8"), filePath);
}

function parseJson(contents: string, filePath: string): JsonRecord {
  const value = JSON.parse(contents) as unknown;
  if (!isRecord(value)) {
    throw new Error(`${relative(filePath)} must contain a JSON object.`);
  }
  return value;
}

function recordField(record: JsonRecord, key: string): JsonRecord {
  const value = record[key];
  if (!isRecord(value)) {
    throw new Error(`expected ${key} to be an object`);
  }
  return value;
}

function isRecord(value: unknown): value is JsonRecord {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function assertEqual(actual: unknown, expected: unknown, label: string): void {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertNoPackageBoundary(filePath: string, contents: string): void {
  for (const value of ["@runxhq/runtime-local", "@runxhq/adapters", "packages/runtime-local", "packages/adapters"]) {
    if (contents.includes(value)) {
      throw new Error(`${relative(filePath)} still references retired package boundary ${value}.`);
    }
  }
}

function assertCleanOracle(name: string, filePath: string, contents: string): void {
  assertNoPackageBoundary(filePath, contents);
  const forbidden = [
    workspaceRoot,
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GITHUB_TOKEN",
    "RUNX_AGENT_API_KEY",
    "sk-fixture-redacted",
    "super-secret-value",
  ];
  for (const value of forbidden) {
    if (value && contents.includes(value)) {
      throw new Error(`${name}: ${relative(filePath)} contains forbidden value '${value}'`);
    }
  }
  if (/\b(?:sk-[A-Za-z0-9_-]+|ghp_[A-Za-z0-9_]+)\b/.test(contents)) {
    throw new Error(`${name}: ${relative(filePath)} appears to contain a secret token`);
  }
  if (/\b20\d{2}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\b/.test(contents)) {
    throw new Error(`${name}: ${relative(filePath)} contains a wall-clock timestamp`);
  }
}

async function collectFiles(directory: string): Promise<readonly string[]> {
  try {
    const directoryStat = await stat(directory);
    if (!directoryStat.isDirectory()) {
      return [];
    }
  } catch {
    return [];
  }

  const files: string[] = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectFiles(entryPath));
    } else if (entry.isFile()) {
      files.push(entryPath);
    }
  }
  return files.sort();
}

function casePath(name: string): string {
  return path.join(fixtureRoot, name);
}

function relative(filePath: string): string {
  return path.relative(workspaceRoot, filePath).split(path.sep).join("/");
}
