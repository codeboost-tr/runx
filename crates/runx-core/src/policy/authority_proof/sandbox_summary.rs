use runx_contracts::{
    JsonObject, JsonValue, json_bool_field, json_object as json_value_object,
    json_object_field as object_field,
};

use crate::policy::{
    AuthorityProofApproval, AuthorityProofSandbox, AuthorityProofSandboxDeclaration,
    AuthorityProofSandboxFilesystem, AuthorityProofSandboxNetwork, AuthorityProofSandboxRuntime,
};

pub(super) fn summarize_authority_sandbox(
    metadata: Option<&JsonValue>,
    declaration: Option<&AuthorityProofSandboxDeclaration>,
    approval: Option<&AuthorityProofApproval>,
) -> Option<AuthorityProofSandbox> {
    let record = json_value_object(metadata);
    let profile = string_field(record, "profile")
        .or_else(|| declaration.and_then(|value| optional_string(value.profile.as_deref())))?;
    let network = summarize_network(
        record.and_then(|value| object_field(value, "network")),
        declaration,
    );
    let filesystem =
        summarize_filesystem(record.and_then(|value| object_field(value, "filesystem")));
    let runtime = summarize_runtime(record.and_then(|value| object_field(value, "runtime")));

    Some(AuthorityProofSandbox {
        profile: profile.clone().into(),
        cwd_policy: string_field(record, "cwd_policy")
            .or_else(|| declaration.and_then(|value| optional_string(value.cwd_policy.as_deref())))
            .map(Into::into),
        require_enforcement: bool_field(record, "require_enforcement")
            .or_else(|| declaration.and_then(|value| value.require_enforcement)),
        network,
        filesystem,
        runtime,
        approval_required: bool_field(
            record.and_then(|value| object_field(value, "approval")),
            "required",
        )
        .or(Some(profile == "unrestricted-local-dev")),
        approval_approved: bool_field(
            record.and_then(|value| object_field(value, "approval")),
            "approved",
        )
        .or_else(|| approval.map(|value| value.approved)),
    })
}

fn summarize_network(
    network: Option<&JsonObject>,
    declaration: Option<&AuthorityProofSandboxDeclaration>,
) -> Option<AuthorityProofSandboxNetwork> {
    if network.is_none() && declaration.and_then(|value| value.network).is_none() {
        return None;
    }
    let summary = AuthorityProofSandboxNetwork {
        declared: bool_field(network, "declared")
            .or_else(|| declaration.and_then(|value| value.network)),
        enforcement: string_field(network, "enforcement").map(Into::into),
    };
    if summary.declared.is_none() && summary.enforcement.is_none() {
        None
    } else {
        Some(summary)
    }
}

fn summarize_filesystem(
    filesystem: Option<&JsonObject>,
) -> Option<AuthorityProofSandboxFilesystem> {
    filesystem.and_then(|value| {
        let summary = AuthorityProofSandboxFilesystem {
            enforcement: string_field(Some(value), "enforcement").map(Into::into),
            readonly_paths: bool_field(Some(value), "readonly_paths"),
            writable_paths_enforced: bool_field(Some(value), "writable_paths_enforced"),
            private_tmp: bool_field(Some(value), "private_tmp"),
        };
        if summary.enforcement.is_none()
            && summary.readonly_paths.is_none()
            && summary.writable_paths_enforced.is_none()
            && summary.private_tmp.is_none()
        {
            None
        } else {
            Some(summary)
        }
    })
}

fn summarize_runtime(runtime: Option<&JsonObject>) -> Option<AuthorityProofSandboxRuntime> {
    runtime.and_then(|value| {
        let summary = AuthorityProofSandboxRuntime {
            enforcer: string_field(Some(value), "enforcer").map(Into::into),
            reason: string_field(Some(value), "reason").map(Into::into),
        };
        if summary.enforcer.is_none() && summary.reason.is_none() {
            None
        } else {
            Some(summary)
        }
    })
}

fn string_field(object: Option<&JsonObject>, field: &str) -> Option<String> {
    match object.and_then(|value| value.get(field)) {
        Some(JsonValue::String(value)) if !value.trim().is_empty() => Some(value.trim().to_owned()),
        _ => None,
    }
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn bool_field(object: Option<&JsonObject>, field: &str) -> Option<bool> {
    object.and_then(|value| json_bool_field(value, field))
}
