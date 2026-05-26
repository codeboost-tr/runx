import { defineConfig } from "vitest/config";

import { workspaceAliases } from "./vitest.workspace-aliases.js";

export default defineConfig({
  resolve: {
    alias: [...workspaceAliases],
  },
  test: {
    include: ["packages/**/*.test.ts", "tests/kernel-parity-fixtures.test.ts"],
    // These suites shell out to the debug `runx` binary; its cold start under
    // parallel load can exceed the 5s default, so give subprocess work headroom.
    testTimeout: 30_000,
    hookTimeout: 30_000,
  },
});
