mod connect_support;

use runx_contracts::JsonValue;
use runx_core::policy::{
    AdmissionDecision, LocalAdmissionOptions, LocalAdmissionSkill, LocalAdmissionSource,
    admit_local_skill,
};
use runx_runtime::connect::connect_grant_to_local_admission;
use serde_json::json;

use connect_support::grant_fixture;

#[test]
fn converted_targeted_grant_admits_exact_requirement() -> Result<(), Box<dyn std::error::Error>> {
    let grant = grant_fixture("grant_exact");
    let skill = connected_skill(json!({
        "provider": "github",
        "scopes": ["repo:read"],
        "scope_family": "github_repo",
        "authority_kind": "read_only",
        "target_repo": "runxhq/aster",
        "target_locator": "github:repo:runxhq/aster"
    }))?;
    let decision = admit_local_skill(
        &skill,
        &LocalAdmissionOptions {
            connected_grants: Some(vec![connect_grant_to_local_admission(&grant)]),
            ..LocalAdmissionOptions::default()
        },
    );

    assert!(matches!(decision, AdmissionDecision::Allow { .. }));
    Ok(())
}

#[test]
fn converted_targeted_grant_does_not_admit_untargeted_requirement()
-> Result<(), Box<dyn std::error::Error>> {
    let grant = grant_fixture("grant_targeted");
    let skill = connected_skill(json!({
        "provider": "github",
        "scopes": ["repo:read"]
    }))?;
    let decision = admit_local_skill(
        &skill,
        &LocalAdmissionOptions {
            connected_grants: Some(vec![connect_grant_to_local_admission(&grant)]),
            ..LocalAdmissionOptions::default()
        },
    );

    assert!(matches!(decision, AdmissionDecision::Deny { .. }));
    Ok(())
}

#[test]
fn converted_revoked_grant_denies() -> Result<(), Box<dyn std::error::Error>> {
    let mut grant = grant_fixture("grant_revoked");
    grant.status = runx_runtime::connect::ConnectGrantStatus::Revoked;
    let skill = connected_skill(json!({
        "provider": "github",
        "scopes": ["repo:read"],
        "scope_family": "github_repo",
        "authority_kind": "read_only",
        "target_repo": "runxhq/aster",
        "target_locator": "github:repo:runxhq/aster"
    }))?;
    let decision = admit_local_skill(
        &skill,
        &LocalAdmissionOptions {
            connected_grants: Some(vec![connect_grant_to_local_admission(&grant)]),
            ..LocalAdmissionOptions::default()
        },
    );

    assert!(matches!(decision, AdmissionDecision::Deny { .. }));
    Ok(())
}

fn connected_skill(auth: serde_json::Value) -> Result<LocalAdmissionSkill, serde_json::Error> {
    Ok(LocalAdmissionSkill {
        name: "connected-review".to_owned(),
        source: LocalAdmissionSource {
            source_type: "cli-tool".to_owned(),
            command: Some("true".to_owned()),
            args: None,
            timeout_seconds: None,
            sandbox: None,
        },
        auth: Some(serde_json::from_value::<JsonValue>(auth)?),
        runtime: None,
    })
}
