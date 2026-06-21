# Security Policy

## Security model

runx keeps execution, state, and receipts on your machine. Fetching a skill from the registry is the one call that ever leaves it. A run reaches runx only when you choose to publish its receipt.

The crate that holds receipts has no network access by design, so there is no telemetry to send. This is a property of the build, not a setting you toggle.

Credentials are supplied per run with `runx skill <ref> --secret-env` and `runx skill <ref> --credential`. They are never persisted.

Authority narrows at every hop. A hop's scopes are a subset of the grant it inherits, and widening is denied by construction, so a skill deep in a graph cannot reach past the authority its caller held. Every act produces a signed, reproducible receipt.

Hosted brokerage and the browser connect flow are opt-in and never sit between you and a local run. The result is a small attack surface: most of what runx does has no network edge to attack, and the parts that do are bounded grants you choose to make.

## Supported versions

runx ships from one rolling `cli-vX.Y.Z` release line. Security fixes land on the latest released CLI. There is no separate LTS line yet.

## Reporting a vulnerability

Do not open a public issue for a vulnerability.

Report privately through GitHub's private vulnerability reporting on the repository: open the **Security** tab and choose **Report a vulnerability**. That keeps the report confidential and routes it to the maintainers.

Include enough for us to confirm and fix the issue:

- The affected version (the `cli-vX.Y.Z` you are running).
- Steps to reproduce, with a minimal case if you can.
- The impact: what an attacker gains and under what conditions.

Disclosure is coordinated. We prepare a fix privately, then disclose the issue and the fix together.

## Scope

This policy covers the open-source CLI, the trusted local Rust runtime, and the generated contracts in this repository. The hosted runx service is governed separately and is not covered here.
