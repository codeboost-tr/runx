//! Phase 1 of `rust-contract-pipeline-inversion`: a bespoke Rust JSON Schema
//! emitter plus a NON-AUTHORITATIVE wire-compatibility drift detector.
//!
//! The 2026-05-22 feasibility spike ruled out vanilla `schemars` (it cannot
//! reproduce the committed schemas: fully inlined, enums as `anyOf` of `const`,
//! custom `$id`/`x-runx-schema`) and `typify` (wrong direction). This emitter
//! controls the document shape directly so it can reproduce the committed
//! `oss/schemas/*.json` exactly, driven by the Rust contract types.
//!
//! This is a PROOF OF MECHANISM over the smallest real contract (`reference`):
//! it does NOT flip the source of truth and deletes nothing. The committed
//! TypeBox-generated schemas remain authoritative. Run:
//!
//!   cargo run -p runx-contracts --example emit_schemas -- --check
//!
//! Exits non-zero if the emitted schema is not wire-compatible with the
//! committed document. The remaining ~58 schemas + the `NonEmptyString` /
//! typed-discriminant constraint model follow this proven pattern.

// This is a command-line drift-detector tool: writing its result to the
// terminal is its purpose, so the workspace print bans are lifted here only.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;

use runx_contracts::ReferenceType;
use serde_json::{Value, json};

/// Every `ReferenceType` variant, in declaration order. Listed here (rather
/// than reflected) because Rust has no enum-variant iteration without a derive
/// macro; productionizing Phase 1 replaces this with a `strum`-style
/// `EnumIter` so the emitter is fully type-driven. The wire names still come
/// from the type itself via `as_str()`.
const REFERENCE_TYPES: [ReferenceType; 34] = [
    ReferenceType::GithubIssue,
    ReferenceType::GithubPullRequest,
    ReferenceType::GithubRepo,
    ReferenceType::SlackThread,
    ReferenceType::SentryEvent,
    ReferenceType::Signal,
    ReferenceType::Act,
    ReferenceType::Receipt,
    ReferenceType::GraphReceipt,
    ReferenceType::Artifact,
    ReferenceType::Verification,
    ReferenceType::Harness,
    ReferenceType::Host,
    ReferenceType::Deployment,
    ReferenceType::Surface,
    ReferenceType::Target,
    ReferenceType::Opportunity,
    ReferenceType::ThesisAssessment,
    ReferenceType::Selection,
    ReferenceType::SkillBinding,
    ReferenceType::TargetTransitionEntry,
    ReferenceType::SelectionCycle,
    ReferenceType::Decision,
    ReferenceType::ReflectionEntry,
    ReferenceType::FeedEntry,
    ReferenceType::Principal,
    ReferenceType::AuthorityProof,
    ReferenceType::ScopeAdmission,
    ReferenceType::Grant,
    ReferenceType::Mandate,
    ReferenceType::Credential,
    ReferenceType::WebhookDelivery,
    ReferenceType::RedactionPolicy,
    ReferenceType::ExternalUrl,
];

const ISO_DATETIME_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$";

/// A `{ "const": <s>, "type": "string" }` leaf: the committed shape for a
/// single string literal (logical-schema markers, fixed discriminants).
fn const_string(value: &str) -> Value {
    json!({ "const": value, "type": "string" })
}

/// A non-empty string (`minLength: 1`), the ubiquitous constraint Phase 1
/// models as a `NonEmptyString` newtype so it cannot be constructed empty.
fn non_empty_string() -> Value {
    json!({ "minLength": 1, "type": "string" })
}

/// A closed string enum rendered as `anyOf` of `const` leaves, matching the
/// committed schemas (which never use JSON Schema `enum`).
fn string_enum<'a>(variants: impl IntoIterator<Item = &'a str>) -> Value {
    let any_of: Vec<Value> = variants.into_iter().map(const_string).collect();
    json!({ "anyOf": any_of })
}

/// Emit `runx.reference.v1`, driven by `ReferenceType` for the type discriminant.
fn reference_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.runx.dev/runx/reference/v1.json",
        "x-runx-schema": "runx.reference.v1",
        "additionalProperties": false,
        "type": "object",
        "required": ["type", "uri"],
        "properties": {
            "schema": const_string("runx.reference.v1"),
            "type": string_enum(REFERENCE_TYPES.iter().map(|variant| variant.as_str())),
            "uri": non_empty_string(),
            "provider": non_empty_string(),
            "locator": non_empty_string(),
            "label": non_empty_string(),
            "observed_at": json!({
                "minLength": 1,
                "pattern": ISO_DATETIME_PATTERN,
                "type": "string",
            }),
            "proof_kind": const_string("payment_rail"),
        },
    })
}

/// A covered contract: the committed schema file, the Rust-emitted schema, and
/// a conformance corpus of representative values. The Phase 1 gate (dod1) is
/// wire-compatibility, not byte-identity: the committed and emitted schemas
/// must agree on accept/reject for every corpus member, and schema identity
/// (`$id`, `x-runx-schema`) must be preserved.
struct CoveredContract {
    file_name: &'static str,
    emitted: Value,
    corpus: Vec<(&'static str, Value)>,
}

/// Expanding Phase 1 means adding a row here as each contract's emitter lands.
fn covered_contracts() -> Vec<CoveredContract> {
    vec![CoveredContract {
        file_name: "reference.schema.json",
        emitted: reference_schema(),
        corpus: reference_corpus(),
    }]
}

/// Representative `reference` values spanning every constraint the schema
/// encodes (required, closed enum, non-empty strings, the timestamp pattern,
/// closed property set, fixed discriminant), so corpus agreement actually
/// exercises the value domain rather than a single happy path.
fn reference_corpus() -> Vec<(&'static str, Value)> {
    vec![
        (
            "minimal valid",
            json!({ "type": "github_issue", "uri": "runx:github_issue:1" }),
        ),
        (
            "full valid",
            json!({
                "type": "act",
                "uri": "runx:act:1",
                "provider": "github",
                "locator": "owner/repo#1",
                "label": "an act",
                "observed_at": "2026-01-01T00:00:00.000Z",
                "proof_kind": "payment_rail",
            }),
        ),
        (
            "optional schema marker",
            json!({ "schema": "runx.reference.v1", "type": "act", "uri": "x" }),
        ),
        ("missing uri", json!({ "type": "act" })),
        ("missing type", json!({ "uri": "x" })),
        (
            "unknown type variant",
            json!({ "type": "not_a_type", "uri": "x" }),
        ),
        ("empty uri", json!({ "type": "act", "uri": "" })),
        (
            "malformed observed_at",
            json!({ "type": "act", "uri": "x", "observed_at": "not-a-timestamp" }),
        ),
        (
            "additional property",
            json!({ "type": "act", "uri": "x", "bogus": true }),
        ),
        (
            "bad proof_kind",
            json!({ "type": "act", "uri": "x", "proof_kind": "wire" }),
        ),
    ]
}

fn committed_dir() -> PathBuf {
    // examples/ lives under crates/runx-contracts; schemas/ is at the oss root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas")
}

fn read_committed(path: &std::path::Path) -> Result<Value, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| format!("cannot read: {error}"))?;
    serde_json::from_str(&raw).map_err(|error| format!("invalid JSON: {error}"))
}

fn main() {
    let check = std::env::args().any(|arg| arg == "--check");
    let dir = committed_dir();
    let contracts = covered_contracts();
    let mut failures = Vec::new();

    for contract in &contracts {
        let name = contract.file_name;
        let committed = match read_committed(&dir.join(name)) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                continue;
            }
        };

        // Schema identity must survive the flip even though the document shape
        // is allowed to differ from the committed one.
        if contract.emitted.get("$id") != committed.get("$id")
            || contract.emitted.get("x-runx-schema") != committed.get("x-runx-schema")
        {
            failures.push(format!(
                "{name}: schema identity ($id / x-runx-schema) diverged"
            ));
            continue;
        }

        let committed_validator = match jsonschema::validator_for(&committed) {
            Ok(validator) => validator,
            Err(error) => {
                failures.push(format!(
                    "{name}: committed schema is not a usable validator: {error}"
                ));
                continue;
            }
        };
        let emitted_validator = match jsonschema::validator_for(&contract.emitted) {
            Ok(validator) => validator,
            Err(error) => {
                failures.push(format!(
                    "{name}: emitted schema is not a usable validator: {error}"
                ));
                continue;
            }
        };

        let mut disagreements = 0;
        for (label, value) in &contract.corpus {
            let committed_accepts = committed_validator.is_valid(value);
            let emitted_accepts = emitted_validator.is_valid(value);
            if committed_accepts != emitted_accepts {
                disagreements += 1;
                failures.push(format!(
                    "{name} / {label}: committed accepts={committed_accepts}, emitted accepts={emitted_accepts}"
                ));
            }
        }

        if disagreements == 0 {
            println!(
                "ok   {name}: {} corpus values agree on accept/reject; schema identity preserved",
                contract.corpus.len()
            );
        }
    }

    if failures.is_empty() {
        println!(
            "\nall {} covered contract(s) are wire-compatible with their committed schemas.",
            contracts.len()
        );
        return;
    }
    eprintln!("\nschema drift detected:");
    for failure in &failures {
        eprintln!("- {failure}");
    }
    if check {
        std::process::exit(1);
    }
}
