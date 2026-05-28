//! Project public target-runner views from sealed receipts.

use runx_contracts::{
    ActForm, ClosureDisposition, JsonObject, JsonValue, Receipt, Reference, ReferenceType,
    TargetRepoRunnerPullRequestDisposition,
};

use super::TargetRepoRunnerRuntimeError;
use super::commands::{
    TargetRepoRunnerRevisionReceiptProjection, TargetRepoRunnerSourcePublicationProjection,
};

// rust-style-allow: long-function because projection validates a sealed receipt
// and reconstructs the public target-runner view without partial helpers.
pub fn project_target_repo_runner_revision_receipt(
    receipt: &Receipt,
) -> Result<TargetRepoRunnerRevisionReceiptProjection, TargetRepoRunnerRuntimeError> {
    if matches!(receipt.seal.disposition, ClosureDisposition::Deferred) {
        return Err(TargetRepoRunnerRuntimeError::ReceiptProjection(
            "receipt is not sealed".to_owned(),
        ));
    }
    let act = receipt
        .acts
        .iter()
        .find(|act| act.form == ActForm::Revision)
        .ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection("revision act is required".to_owned())
        })?;
    let metadata = receipt.metadata.clone().ok_or_else(|| {
        TargetRepoRunnerRuntimeError::ReceiptProjection(
            "target runner metadata is required".to_owned(),
        )
    })?;
    let pull_request_ref = find_ref(&act.artifact_refs, ReferenceType::GithubPullRequest)
        .ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "pull request ref is required".to_owned(),
            )
        })?;
    let target_repo_ref =
        find_ref(&act.artifact_refs, ReferenceType::GithubRepo).ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "target repo ref is required".to_owned(),
            )
        })?;
    let source_thread_ref = find_ref(&act.artifact_refs, ReferenceType::SlackThread)
        .or_else(|| act.artifact_refs.first().cloned())
        .ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "source thread ref is required".to_owned(),
            )
        })?;
    let source_issue_ref = find_ref(&act.artifact_refs, ReferenceType::GithubIssue);
    Ok(TargetRepoRunnerRevisionReceiptProjection {
        receipt_ref: Reference::runx(ReferenceType::Receipt, &receipt.id),
        act_id: act.id.to_string(),
        disposition: projection_disposition(&metadata)?,
        target_repo_ref,
        source_issue_ref,
        source_thread_ref,
        pull_request_ref,
        summary: receipt.seal.summary.to_string(),
        metadata,
    })
}

pub fn project_target_repo_runner_source_publication_receipt(
    receipt: &Receipt,
) -> Result<TargetRepoRunnerSourcePublicationProjection, TargetRepoRunnerRuntimeError> {
    if matches!(receipt.seal.disposition, ClosureDisposition::Deferred) {
        return Err(TargetRepoRunnerRuntimeError::ReceiptProjection(
            "source publication receipt is not sealed".to_owned(),
        ));
    }
    let act = receipt
        .acts
        .iter()
        .find(|act| act.form == ActForm::Reply)
        .ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "source publication reply act is required".to_owned(),
            )
        })?;
    let metadata = receipt.metadata.clone().ok_or_else(|| {
        TargetRepoRunnerRuntimeError::ReceiptProjection(
            "source publication metadata is required".to_owned(),
        )
    })?;
    let pull_request_ref = find_ref(&act.artifact_refs, ReferenceType::GithubPullRequest)
        .ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "source publication pull request ref is required".to_owned(),
            )
        })?;
    let source_thread_ref =
        find_ref(&act.artifact_refs, ReferenceType::SlackThread).ok_or_else(|| {
            TargetRepoRunnerRuntimeError::ReceiptProjection(
                "source publication thread ref is required".to_owned(),
            )
        })?;
    let source_issue_ref = find_ref(&act.artifact_refs, ReferenceType::GithubIssue);
    let published_refs = act
        .artifact_refs
        .iter()
        .filter(|reference| {
            !matches!(
                reference.reference_type,
                ReferenceType::GithubPullRequest
                    | ReferenceType::SlackThread
                    | ReferenceType::GithubIssue
            )
        })
        .cloned()
        .collect();
    Ok(TargetRepoRunnerSourcePublicationProjection {
        receipt_ref: Reference::runx(ReferenceType::Receipt, &receipt.id),
        source_issue_ref,
        source_thread_ref,
        pull_request_ref,
        published_refs,
        summary: receipt.seal.summary.to_string(),
        metadata,
    })
}

fn find_ref(refs: &[Reference], reference_type: ReferenceType) -> Option<Reference> {
    refs.iter()
        .find(|reference| reference.reference_type == reference_type)
        .cloned()
}

fn projection_disposition(
    metadata: &JsonObject,
) -> Result<TargetRepoRunnerPullRequestDisposition, TargetRepoRunnerRuntimeError> {
    let Some(JsonValue::Object(target_runner)) = metadata.get("target_runner") else {
        return Err(TargetRepoRunnerRuntimeError::ReceiptProjection(
            "target runner metadata object is required".to_owned(),
        ));
    };
    match target_runner.get("disposition").and_then(JsonValue::as_str) {
        Some("created") => Ok(TargetRepoRunnerPullRequestDisposition::Create),
        Some("reused") => Ok(TargetRepoRunnerPullRequestDisposition::Reuse),
        _ => Err(TargetRepoRunnerRuntimeError::ReceiptProjection(
            "target runner disposition is invalid".to_owned(),
        )),
    }
}
