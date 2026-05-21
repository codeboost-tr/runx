#![cfg(feature = "external-adapter")]

use std::fs;
use std::path::{Path, PathBuf};

use runx_contracts::{
    EXTERNAL_ADAPTER_PROTOCOL_VERSION, ExternalAdapterInvocation, ExternalAdapterManifest,
    ExternalAdapterResponse, ExternalAdapterSandboxIntent, ExternalAdapterStatus,
    ExternalAdapterTimeouts, ExternalAdapterTransport, ExternalAdapterTransportKind, JsonNumber,
    JsonObject, JsonValue, Reference, ReferenceType,
};
use runx_core::state_machine::GraphStatus;
use runx_parser::SkillSource;
use runx_runtime::adapters::external_adapter::{
    ExternalAdapterProcessSupervisor, ExternalAdapterSkillAdapter, ExternalAdapterSupervisorError,
};
use runx_runtime::{
    CredentialDelivery, Runtime, RuntimeError, RuntimeOptions, SkillAdapter, SkillInvocation,
};

const MANIFEST_SCHEMA: &str = "runx.external_adapter.manifest.v1";
const INVOCATION_SCHEMA: &str = "runx.external_adapter.invocation.v1";
const RESPONSE_SCHEMA: &str = "runx.external_adapter.response.v1";

#[test]
fn external_adapter_process_supervisor_invokes_contract_process()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let capture_path = temp.path().join("captured-invocation.json");
    let response_path = temp.path().join("response.json");
    fs::write(&response_path, serde_json::to_vec(&completed_response())?)?;
    let script = write_script(
        temp.path(),
        r#"set -eu
IFS= read -r invocation
printf '%s' "$invocation" > "$RUNX_CAPTURE_INVOCATION"
/bin/cat "$RUNX_RESPONSE_PATH"
"#,
    )?;
    let invocation = invocation_with_env([
        (
            "RUNX_CAPTURE_INVOCATION",
            path_string(capture_path.as_path())?,
        ),
        ("RUNX_RESPONSE_PATH", path_string(response_path.as_path())?),
    ]);

    let outcome =
        ExternalAdapterProcessSupervisor.invoke(&manifest_for_script(&script)?, &invocation)?;

    assert_eq!(outcome.response.status, ExternalAdapterStatus::Completed);
    assert_eq!(
        outcome.response.stdout.as_deref(),
        Some("{\"summary\":\"Issue needs triage\"}")
    );
    assert_eq!(outcome.process_exit_code, Some(0));
    let captured: ExternalAdapterInvocation =
        serde_json::from_slice(&fs::read(capture_path.as_path())?)?;
    assert_eq!(captured, invocation);
    Ok(())
}

#[test]
fn external_adapter_process_supervisor_rejects_mismatched_response_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let response_path = temp.path().join("response.json");
    let mut response = completed_response();
    response.adapter_id = "adapter.other".to_owned();
    fs::write(&response_path, serde_json::to_vec(&response)?)?;
    let script = write_cat_response_script(temp.path())?;
    let invocation = invocation_with_env([("RUNX_RESPONSE_PATH", path_string(&response_path)?)]);

    let Err(error) =
        ExternalAdapterProcessSupervisor.invoke(&manifest_for_script(&script)?, &invocation)
    else {
        return Err("mismatched response identity must fail closed".into());
    };

    assert!(matches!(
        error,
        ExternalAdapterSupervisorError::ResponseMismatch {
            field: "adapter_id",
            ..
        }
    ));
    Ok(())
}

#[test]
fn external_adapter_process_supervisor_rejects_unexpected_credential_request()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let response_path = temp.path().join("credential-request.json");
    fs::write(
        &response_path,
        br#"{"schema":"runx.external_adapter.credential_request.v1","protocol_version":"runx.external_adapter.v1","request_id":"cred_req_1","adapter_id":"adapter.github.issue-intake","invocation_id":"external_inv_123","credential_refs":[],"requested_at":"2026-05-21T15:00:01Z"}"#,
    )?;
    let script = write_cat_response_script(temp.path())?;
    let invocation = invocation_with_env([("RUNX_RESPONSE_PATH", path_string(&response_path)?)]);

    let Err(error) =
        ExternalAdapterProcessSupervisor.invoke(&manifest_for_script(&script)?, &invocation)
    else {
        return Err("credential request frame on response channel must fail closed".into());
    };

    assert!(matches!(
        error,
        ExternalAdapterSupervisorError::UnexpectedCredentialRequest { request_id }
            if request_id == "cred_req_1"
    ));
    Ok(())
}

#[test]
fn external_adapter_process_supervisor_times_out_and_maps_cancellation()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let script = write_script(
        temp.path(),
        r#"set -eu
IFS= read -r _invocation
/bin/sleep 5
"#,
    )?;
    let mut manifest = manifest_for_script(&script)?;
    manifest.timeouts.invocation_ms = 50;
    let invocation = base_invocation();

    let Err(error) = ExternalAdapterProcessSupervisor.invoke(&manifest, &invocation) else {
        return Err("timed out process must fail closed".into());
    };

    let ExternalAdapterSupervisorError::TimedOut {
        timeout_ms,
        cancellation,
    } = error
    else {
        return Err(format!("unexpected timeout error: {error}").into());
    };
    assert_eq!(timeout_ms, 50);
    assert_eq!(
        cancellation.protocol_version,
        EXTERNAL_ADAPTER_PROTOCOL_VERSION
    );
    assert_eq!(cancellation.schema, "runx.external_adapter.cancellation.v1");
    assert_eq!(cancellation.adapter_id, "adapter.github.issue-intake");
    assert_eq!(cancellation.invocation_id, "external_inv_123");
    assert_eq!(cancellation.reason, "invocation timeout after 50ms");
    Ok(())
}

#[test]
fn external_adapter_process_supervisor_rejects_crashed_process()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let script = write_script(
        temp.path(),
        r#"set -eu
IFS= read -r _invocation
exit 12
"#,
    )?;

    let Err(error) =
        ExternalAdapterProcessSupervisor.invoke(&manifest_for_script(&script)?, &base_invocation())
    else {
        return Err("crashed process must fail closed".into());
    };

    assert!(matches!(
        error,
        ExternalAdapterSupervisorError::ProcessFailed { .. }
    ));
    Ok(())
}

#[test]
fn external_adapter_process_supervisor_rejects_unknown_protocol_before_spawn()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let marker_path = temp.path().join("should-not-exist");
    let script = write_script(
        temp.path(),
        r#"set -eu
printf spawned > "$RUNX_MARKER_PATH"
"#,
    )?;
    let mut manifest = manifest_for_script(&script)?;
    manifest.protocol_version = "runx.external_adapter.v2".to_owned();
    let invocation = invocation_with_env([("RUNX_MARKER_PATH", path_string(&marker_path)?)]);

    let Err(error) = ExternalAdapterProcessSupervisor.invoke(&manifest, &invocation) else {
        return Err("unknown manifest protocol must fail before process spawn".into());
    };

    assert!(matches!(
        error,
        ExternalAdapterSupervisorError::UnsupportedManifestProtocol { actual }
            if actual == "runx.external_adapter.v2"
    ));
    assert!(!marker_path.exists());
    Ok(())
}

#[test]
fn external_adapter_graph_invocation_reaches_process_supervisor()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let skill_dir = temp.path().join("external-skill");
    fs::create_dir_all(&skill_dir)?;
    let capture_path = skill_dir.join("captured-invocation.json");
    let response_path = skill_dir.join("response.json");
    let mut response = completed_response();
    response.invocation_id = "external_adapter.external-smoke.invoke".to_owned();
    response.stdout = Some("{\"summary\":\"graph reached supervisor\"}".to_owned());
    let mut output = JsonObject::new();
    output.insert(
        "summary".to_owned(),
        JsonValue::String("graph reached supervisor".to_owned()),
    );
    response.output = Some(output);
    fs::write(&response_path, serde_json::to_vec(&response)?)?;
    write_script(
        &skill_dir,
        r#"set -eu
IFS= read -r invocation
printf '%s' "$invocation" > captured-invocation.json
/bin/cat response.json
"#,
    )?;
    write_external_adapter_skill(&skill_dir)?;
    let graph_path = temp.path().join("graph.yaml");
    fs::write(
        &graph_path,
        "name: external-adapter-graph\nsteps:\n  - id: invoke\n    skill: ./external-skill\n    inputs:\n      issue_number: 77\n",
    )?;

    let run = Runtime::new(
        ExternalAdapterSkillAdapter::default(),
        RuntimeOptions::default(),
    )
    .run_graph_file(&graph_path)?;

    assert_eq!(run.state.status, GraphStatus::Succeeded);
    assert_eq!(run.steps.len(), 1);
    assert_eq!(
        run.steps[0].output.stdout,
        "{\"summary\":\"graph reached supervisor\"}"
    );
    let captured: ExternalAdapterInvocation = serde_json::from_slice(&fs::read(capture_path)?)?;
    assert_eq!(captured.source_type, "external-adapter");
    assert_eq!(captured.adapter_id, "adapter.github.issue-intake");
    assert_eq!(captured.skill_ref, "external-smoke");
    assert_eq!(
        captured.inputs.get("issue_number"),
        Some(&JsonValue::Number(JsonNumber::I64(77)))
    );
    Ok(())
}

#[test]
fn external_adapter_skill_adapter_fails_closed_without_inline_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;

    let Err(error) =
        ExternalAdapterSkillAdapter::default().invoke(skill_invocation(temp.path(), None, [])?)
    else {
        return Err("external-adapter source without manifest must fail closed".into());
    };

    assert!(matches!(
        error,
        RuntimeError::SkillFailed { message, .. }
            if message.contains("missing an inline manifest")
    ));
    Ok(())
}

#[test]
fn external_adapter_skill_adapter_preserves_supervisor_fail_closed_response_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let response_path = temp.path().join("response.json");
    let mut response = completed_response();
    response.invocation_id = "external_adapter.external-smoke.invoke".to_owned();
    response.adapter_id = "adapter.other".to_owned();
    fs::write(&response_path, serde_json::to_vec(&response)?)?;
    let script = write_cat_response_script(temp.path())?;
    let manifest = manifest_for_script(&script)?;

    let Err(error) = ExternalAdapterSkillAdapter::default().invoke(skill_invocation(
        temp.path(),
        Some(manifest),
        [("RUNX_RESPONSE_PATH", path_string(&response_path)?)],
    )?) else {
        return Err("mismatched response identity must fail closed through SkillAdapter".into());
    };

    assert!(matches!(
        error,
        RuntimeError::SkillFailed { message, .. }
            if message.contains("adapter_id") && message.contains("adapter.other")
    ));
    Ok(())
}

fn write_cat_response_script(dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    write_script(
        dir,
        r#"set -eu
IFS= read -r _invocation
/bin/cat "$RUNX_RESPONSE_PATH"
"#,
    )
}

fn write_script(dir: &Path, body: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = dir.join("adapter.sh");
    fs::write(path.as_path(), body)?;
    Ok(path)
}

fn write_external_adapter_skill(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        dir.join("SKILL.md"),
        r#"---
name: external-smoke
source:
  type: external-adapter
  external_adapter:
    manifest:
      schema: runx.external_adapter.manifest.v1
      protocol_version: runx.external_adapter.v1
      adapter_id: adapter.github.issue-intake
      name: GitHub issue intake adapter
      version: 0.1.0
      supported_source_types:
        - external-adapter
      transport:
        kind: process
        command: /bin/sh
        args:
          - adapter.sh
      timeouts:
        startup_ms: 500
        invocation_ms: 2000
      sandbox_intent:
        profile: readonly
        network: false
        cwd_policy: skill-directory
---

Exercise the external adapter runtime wiring path.
"#,
    )?;
    Ok(())
}

fn manifest_for_script(
    script: &Path,
) -> Result<ExternalAdapterManifest, Box<dyn std::error::Error>> {
    Ok(ExternalAdapterManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        protocol_version: EXTERNAL_ADAPTER_PROTOCOL_VERSION.to_owned(),
        adapter_id: "adapter.github.issue-intake".to_owned(),
        name: "GitHub issue intake adapter".to_owned(),
        version: "0.1.0".to_owned(),
        supported_source_types: vec!["external-adapter".to_owned()],
        transport: ExternalAdapterTransport {
            kind: ExternalAdapterTransportKind::Process,
            command: Some("/bin/sh".to_owned()),
            args: Some(vec![path_string(script)?]),
            endpoint: None,
        },
        timeouts: ExternalAdapterTimeouts {
            startup_ms: 500,
            invocation_ms: 2_000,
        },
        credential_needs: None,
        sandbox_intent: ExternalAdapterSandboxIntent {
            profile: "readonly".to_owned(),
            network: false,
            cwd_policy: "skill-directory".to_owned(),
            writable_paths: None,
        },
        metadata: None,
    })
}

fn skill_invocation<const N: usize>(
    skill_dir: &Path,
    manifest: Option<ExternalAdapterManifest>,
    env: [(&str, String); N],
) -> Result<SkillInvocation, Box<dyn std::error::Error>> {
    Ok(SkillInvocation {
        skill_name: "external-smoke".to_owned(),
        source: skill_source(manifest)?,
        inputs: [(
            "issue_number".to_owned(),
            JsonValue::Number(JsonNumber::I64(77)),
        )]
        .into_iter()
        .collect(),
        resolved_inputs: JsonObject::new(),
        skill_directory: skill_dir.to_path_buf(),
        env: env
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
        credential_delivery: CredentialDelivery::none(),
    })
}

fn skill_source(
    manifest: Option<ExternalAdapterManifest>,
) -> Result<SkillSource, Box<dyn std::error::Error>> {
    let mut raw = JsonObject::new();
    raw.insert(
        "type".to_owned(),
        JsonValue::String("external-adapter".to_owned()),
    );
    if let Some(manifest) = manifest {
        let mut external_adapter = JsonObject::new();
        external_adapter.insert("manifest".to_owned(), contract_json_value(&manifest)?);
        raw.insert(
            "external_adapter".to_owned(),
            JsonValue::Object(external_adapter),
        );
    }
    Ok(SkillSource {
        source_type: "external-adapter".to_owned(),
        command: None,
        args: Vec::new(),
        cwd: None,
        timeout_seconds: None,
        input_mode: None,
        sandbox: None,
        server: None,
        catalog_ref: None,
        tool: None,
        arguments: None,
        agent_card_url: None,
        agent_identity: None,
        agent: None,
        task: None,
        hook: None,
        outputs: None,
        graph: None,
        raw,
    })
}

fn base_invocation() -> ExternalAdapterInvocation {
    invocation_with_env([])
}

fn invocation_with_env<const N: usize>(env: [(&str, String); N]) -> ExternalAdapterInvocation {
    ExternalAdapterInvocation {
        schema: INVOCATION_SCHEMA.to_owned(),
        protocol_version: EXTERNAL_ADAPTER_PROTOCOL_VERSION.to_owned(),
        invocation_id: "external_inv_123".to_owned(),
        adapter_id: "adapter.github.issue-intake".to_owned(),
        run_id: "run_123".to_owned(),
        step_id: "issue-intake".to_owned(),
        source_type: "external-adapter".to_owned(),
        skill_ref: "runx/github-issue-intake".to_owned(),
        harness_ref: reference(ReferenceType::Harness, "runx:harness:hrn_123"),
        host_ref: reference(ReferenceType::Host, "runx:host:local-cli"),
        inputs: [
            (
                "repo".to_owned(),
                JsonValue::String("runxhq/runx".to_owned()),
            ),
            (
                "issue_number".to_owned(),
                JsonValue::Number(JsonNumber::I64(77)),
            ),
        ]
        .into_iter()
        .collect(),
        resolved_inputs: Some(
            [(
                "repo".to_owned(),
                JsonValue::String("runxhq/runx".to_owned()),
            )]
            .into_iter()
            .collect(),
        ),
        cwd: None,
        receipt_dir: Some("/workspace/.runx/receipts".to_owned()),
        env: Some(
            env.into_iter()
                .map(|(key, value)| (key.to_owned(), JsonValue::String(value)))
                .collect(),
        ),
        credential_refs: None,
        metadata: None,
    }
}

fn completed_response() -> ExternalAdapterResponse {
    let mut output = JsonObject::new();
    output.insert(
        "decision".to_owned(),
        JsonValue::String("request_review".to_owned()),
    );
    output.insert(
        "summary".to_owned(),
        JsonValue::String("Issue needs triage".to_owned()),
    );

    ExternalAdapterResponse {
        schema: RESPONSE_SCHEMA.to_owned(),
        protocol_version: EXTERNAL_ADAPTER_PROTOCOL_VERSION.to_owned(),
        invocation_id: "external_inv_123".to_owned(),
        adapter_id: "adapter.github.issue-intake".to_owned(),
        status: ExternalAdapterStatus::Completed,
        stdout: Some("{\"summary\":\"Issue needs triage\"}".to_owned()),
        stderr: Some(String::new()),
        exit_code: Some(Some(0)),
        output: Some(output),
        artifacts: None,
        errors: None,
        telemetry: None,
        metadata: None,
        observed_at: "2026-05-21T15:00:00Z".to_owned(),
    }
}

fn reference(reference_type: ReferenceType, uri: &str) -> Reference {
    Reference {
        reference_type,
        uri: uri.to_owned(),
        provider: None,
        locator: None,
        label: None,
        observed_at: None,
        proof_kind: None,
    }
}

fn path_string(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(path
        .to_str()
        .ok_or("test path must be valid UTF-8")?
        .to_owned())
}

fn contract_json_value(value: &impl serde::Serialize) -> Result<JsonValue, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    serde_json::from_value(value)
}
