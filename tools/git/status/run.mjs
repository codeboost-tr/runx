#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const entryCandidates = [
  path.join(here, "src", "index.js"),
  path.join(here, "src", "index.ts"),
];
const entry = entryCandidates.find((candidate) => fs.existsSync(candidate));
if (!entry) {
  throw new Error(`Unable to locate tool entrypoint from ${here}`);
}
const tool = (await import(pathToFileURL(entry).href)).default;
await tool.main();
