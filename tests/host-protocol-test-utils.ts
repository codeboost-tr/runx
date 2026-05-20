import { spawnSync } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { createDefaultSkillAdapters } from "@runxhq/adapters";
import type { HostBridge } from "@runxhq/host-adapters";
import { createRunxSdk, createHostBridge } from "@runxhq/runtime-local/sdk";

export interface HostHarness {
  readonly bridge: HostBridge;
  readonly cleanup: () => Promise<void>;
}

export const workspaceRoot = process.cwd();
const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
export const runxBinary = path.join(
  workspaceRoot,
  "crates",
  "target",
  "debug",
  process.platform === "win32" ? "runx.exe" : "runx",
);
let runxBinaryBuilt = false;

export function kernelTestEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  return {
    ...process.env,
    RUNX_CWD: process.cwd(),
    RUNX_KERNEL_EVAL_BIN: runxBinary,
    ...extra,
  };
}

export async function createHostHarness(): Promise<HostHarness> {
  ensureRunxBinary();
  const tempDir = await mkdtemp(path.join(os.tmpdir(), "runx-host-protocol-"));
  const sdk = createRunxSdk({
    env: {
      ...kernelTestEnv(),
      RUNX_HOME: path.join(tempDir, "home"),
    },
    receiptDir: path.join(tempDir, "receipts"),
    adapters: createDefaultSkillAdapters(),
  });

  return {
    bridge: createHostBridge({ execute: sdk.runSkill.bind(sdk) }),
    cleanup: async () => {
      await rm(tempDir, { recursive: true, force: true });
    },
  };
}

export function ensureRunxBinary(): void {
  if (runxBinaryBuilt) {
    return;
  }
  const result = spawnSync(
    cargo,
    ["build", "--quiet", "--manifest-path", "crates/Cargo.toml", "-p", "runx-cli", "--bin", "runx"],
    {
      cwd: workspaceRoot,
      encoding: "utf8",
      env: process.env,
    },
  );
  if (result.status !== 0) {
    throw new Error(`failed to build runx binary for host protocol tests: ${result.stderr || result.stdout}`);
  }
  runxBinaryBuilt = true;
}
