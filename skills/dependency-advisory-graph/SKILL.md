---
name: dependency-advisory-graph
description: Analyze a dependency manifest, compose vulnerability scans, and produce an advisory graph without false positives.
runx:
  category: security
---

# Dependency Advisory Graph

This skill parses a single dependency manifest (like `package.json`, `requirements.txt`, or `Cargo.toml`), composes existing runx vulnerability and research skills to investigate dependencies, and produces a precise advisory packet.

It strictly demands exact version matching to avoid false positives (e.g., no broad package-name-only findings without version overlap).

## Requirements & Quality Profile

- **Exact Version Matching:** Ensure the identified vulnerability applies to the exact installed version specified in the manifest.
- **Composition:** Where possible, compose and delegate to `vuln-scan` or `ecosystem-vuln-scan` for sub-tasks, then synthesize the findings into a graph format.
- **Zero False Positives:** Omit any advisories where the version overlap or exposure is not strictly proven.

## Output

Return an array `advisories` containing objects with exactly these fields:

- `package`: The name of the package.
- `installed_version`: The specific version found in the manifest.
- `advisory_id`: The vulnerability identifier (e.g., GHSA, CVE).
- `evidence_url`: A valid public URL proving the vulnerability exists for this version.
- `advisory_source`: The database or system where this advisory was found.
- `retrieved_at`: ISO timestamp of when this was discovered.
- `severity`: Severity level (low, medium, high, critical).
- `fix_version`: The version that patches the vulnerability (or "unknown").
- `confidence`: "high" (must be high due to strict matching requirements).

## Inputs

- `manifest` (required): The dependency manifest file to analyze.
- `objective` (optional): Specific focus for the advisory graph.
