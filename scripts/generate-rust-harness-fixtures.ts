import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import { canonicalJsonStringify, sha256Prefixed } from "@runxhq/contracts";

const repoRoot = path.resolve(import.meta.dirname, "..");
const oracleDir = path.join(repoRoot, "fixtures/harness/oracle");
const harnessDir = path.join(repoRoot, "fixtures/harness");

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

interface OracleReceipt {
  /** Harness fixture basename, e.g. `echo-skill`. */
  readonly fixture: string;
  /** The Rust-true body digest (the receipt's top-level `digest`). */
  readonly bodyDigest: string;
  /** The Rust-true full-receipt digest (sha256 over the canonical receipt). */
  readonly receiptDigest: string;
  /** The canonical receipt JSON exactly as the Rust binary emits it. */
  readonly canonicalJson: string;
}

const write = process.argv.includes("--write") || process.argv.includes("--generate");
const check = process.argv.includes("--check") || !write;

// The committed top-level receipts each map 1:1 to `runx harness <fixture> --json`.
// The graph step/child receipts (`*.first.json`, `*.fulfill.json`, ...) are not
// emitted by the CLI; they are produced and validated authoritatively by the Rust
// runtime test `replay_receipts_match_checked_in_canonical_oracles` in
// crates/runx-runtime/tests/harness_fixtures.rs, so this generator does not
// recompute them in TypeScript.
const fixtures = ["echo-skill", "sequential-graph", "payment-approval-graph"] as const;

const receipts = fixtures.map((fixture) => harnessReceiptFromRust(fixture));

if (write) {
  mkdirSync(oracleDir, { recursive: true });
  for (const receipt of receipts) {
    writeFileSync(oraclePath(receipt), `${receipt.canonicalJson}\n`);
  }
}

if (check) {
  let failed = false;
  for (const receipt of receipts) {
    let expected = "";
    try {
      expected = readFileSync(oraclePath(receipt), "utf8");
    } catch {
      console.error(`missing harness oracle ${relative(oraclePath(receipt))}`);
      failed = true;
      continue;
    }
    if (expected !== `${receipt.canonicalJson}\n`) {
      console.error(`stale harness oracle ${relative(oraclePath(receipt))}`);
      failed = true;
    }
    const contents = readFileSync(path.join(harnessDir, `${receipt.fixture}.yaml`), "utf8");
    if (!contents.includes(`body_digest: ${receipt.bodyDigest}`)) {
      console.error(`stale body_digest in fixtures/harness/${receipt.fixture}.yaml`);
      failed = true;
    }
    if (!contents.includes(`receipt_digest: ${receipt.receiptDigest}`)) {
      console.error(`stale receipt_digest in fixtures/harness/${receipt.fixture}.yaml`);
      failed = true;
    }
  }
  if (failed) {
    process.exitCode = 1;
  }
}

if (!check) {
  for (const receipt of receipts) {
    console.log(`${receipt.fixture} body_digest=${receipt.bodyDigest}`);
    console.log(`${receipt.fixture} receipt_digest=${receipt.receiptDigest}`);
  }
}

/**
 * Produce the receipt and its digests from the authoritative Rust binary.
 *
 * `body_digest` is read straight from the binary's emitted `digest` field; the
 * full `receipt_digest` and the canonical oracle body are derived from the
 * binary's own canonical JSON. No receipt is reconstructed in TypeScript, so the
 * committed digests are Rust-true by construction.
 */
function harnessReceiptFromRust(fixture: string): OracleReceipt {
  const fixturePath = path.join(harnessDir, `${fixture}.yaml`);
  const stdout = execFileSync(rustBinary(), ["harness", fixturePath, "--json"], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const receipt = JSON.parse(stdout) as Record<string, Json>;
  const bodyDigest = receipt.digest;
  if (typeof bodyDigest !== "string") {
    throw new Error(`runx harness ${fixture} did not emit a string body digest.`);
  }
  const canonicalJson = canonicalJsonStringify(receipt);
  return {
    fixture,
    bodyDigest,
    receiptDigest: sha256Prefixed(canonicalJson),
    canonicalJson,
  };
}

function rustBinary(): string {
  const fromEnv = process.env.RUNX_RUST_CLI_BIN ?? process.env.RUNX_KERNEL_EVAL_BIN;
  if (fromEnv) {
    return fromEnv;
  }
  const defaultBinary = path.join(
    repoRoot,
    "crates",
    "target",
    "debug",
    process.platform === "win32" ? "runx.exe" : "runx",
  );
  if (existsSync(defaultBinary)) {
    return defaultBinary;
  }
  throw new Error(
    `harness fixtures require the Rust binary; set RUNX_RUST_CLI_BIN or build it at ${relative(defaultBinary)}.`,
  );
}

function oraclePath(receipt: OracleReceipt): string {
  return path.join(oracleDir, `${receipt.fixture}.receipt.json`);
}

function relative(filePath: string): string {
  return path.relative(repoRoot, filePath);
}
