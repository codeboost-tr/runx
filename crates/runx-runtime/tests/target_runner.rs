use runx_contracts::{
    ActForm, HarnessState, JsonValue, OperationalPolicy, OperationalPolicyAction, Reference,
    ReferenceType, TargetRepoRunnerDedupeLookupObservation, TargetRepoRunnerDedupeLookupPlan,
    TargetRepoRunnerDedupeResult, TargetRepoRunnerExistingPullRequest, TargetRepoRunnerPlanRequest,
    TargetRepoRunnerProvider, TargetRepoRunnerProviderPullRequest,
    TargetRepoRunnerPullRequestDisposition, TargetRepoRunnerReadinessObservation,
    TargetRepoRunnerSourceContext, plan_target_repo_runner, plan_target_repo_runner_execution,
};
use runx_receipts::canonical_receipt_body_digest;
use runx_runtime::target_runner::{
    TargetRepoRunnerAdapter, TargetRepoRunnerAdapterError, TargetRepoRunnerFixtureExecutionInput,
    TargetRepoRunnerGovernedRunnerInvocation, TargetRepoRunnerGovernedRunnerObservation,
    TargetRepoRunnerPullRequestObservationRequest, TargetRepoRunnerRuntimeError,
    TargetRepoRunnerSourcePublicationCommand, TargetRepoRunnerSourcePublicationObservation,
    TargetRepoRunnerSourcePublicationRequest, execute_target_repo_runner_execution_fixture,
    execute_target_repo_runner_fixture, execute_target_repo_runner_with_adapter,
};

const NITROSEND_LIKE: &str =
    include_str!("../../../fixtures/operational-policy/nitrosend-like.json");
const CREATED_AT: &str = "2026-05-20T10:30:00Z";

#[test]
fn runtime_fails_closed_before_dedupe_or_pr_when_checkout_not_scafld_ready()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;

    let error = execute_target_repo_runner_fixture(TargetRepoRunnerFixtureExecutionInput {
        plan,
        readiness: readiness(false),
        dedupe: empty_dedupe_observation("nitrosend/api", "unobserved"),
        created_pull_request: Some(created_pull_request()),
    })
    .err();

    assert!(matches!(
        error,
        Some(TargetRepoRunnerRuntimeError::Plan(
            runx_contracts::TargetRepoRunnerPlanError::NotScafldReady { .. }
        ))
    ));
    Ok(())
}

#[test]
fn runtime_rechecks_execution_readiness_before_mutation_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let execution_plan = plan_target_repo_runner_execution(&plan, &readiness(true))?;

    let error = execute_target_repo_runner_execution_fixture(
        &plan,
        &execution_plan,
        &readiness(false),
        &empty_dedupe_observation("nitrosend/api", &execution_plan.provider_lookup.key),
        Some(&created_pull_request()),
    )
    .err();

    assert!(matches!(
        error,
        Some(TargetRepoRunnerRuntimeError::ReadinessMismatch(_))
    ));
    Ok(())
}

#[test]
fn runtime_creates_pull_request_when_provider_dedupe_has_no_match()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let execution_plan = plan_target_repo_runner_execution(&plan, &readiness(true))?;
    let created = created_pull_request();

    let execution = execute_target_repo_runner_execution_fixture(
        &plan,
        &execution_plan,
        &readiness(true),
        &empty_dedupe_observation("nitrosend/api", &execution_plan.provider_lookup.key),
        Some(&created),
    )?;

    assert_eq!(
        execution.disposition,
        TargetRepoRunnerPullRequestDisposition::Create
    );
    assert_eq!(
        execution.dedupe_execution.result,
        TargetRepoRunnerDedupeResult::LookupRequired
    );
    assert_eq!(execution.pull_request.url, created.url);
    assert_eq!(
        nested_string(
            &execution.pull_request_receipt.metadata,
            &["dedupe", "result"]
        ),
        Some("created")
    );
    assert_eq!(
        nested_string(
            &execution.pull_request_receipt.metadata,
            &["source", "thread_uri"]
        ),
        Some("slack://nitrosend/C0APFMY0V8Q/1778834840.485629")
    );
    assert_eq!(
        execution.source_publication_receipt.pull_request_ref.uri,
        "https://github.com/nitrosend/api/pull/145"
    );
    assert_public_only(&execution)?;
    Ok(())
}

#[test]
fn runtime_reuses_provider_pull_request_when_dedupe_refs_match()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let execution_plan = plan_target_repo_runner_execution(&plan, &readiness(true))?;
    let observed = TargetRepoRunnerProviderPullRequest {
        url: "https://github.com/nitrosend/api/pull/144".to_owned(),
        number: Some(144),
        branch: Some("runx/source-482".to_owned()),
        open: true,
        markers: execution_plan.provider_lookup.query.markers.clone(),
        refs: execution_plan.provider_lookup.query.required_refs.clone(),
    };

    let execution = execute_target_repo_runner_execution_fixture(
        &plan,
        &execution_plan,
        &readiness(true),
        &TargetRepoRunnerDedupeLookupObservation {
            provider: TargetRepoRunnerProvider::Github,
            target_repo: "nitrosend/api".to_owned(),
            key: execution_plan.provider_lookup.key.clone(),
            pull_requests: vec![observed],
        },
        None,
    )?;

    assert_eq!(
        execution.disposition,
        TargetRepoRunnerPullRequestDisposition::Reuse
    );
    assert_eq!(
        execution.dedupe_execution.result,
        TargetRepoRunnerDedupeResult::Reused
    );
    assert_eq!(execution.pull_request.number, Some(144));
    assert_eq!(
        execution
            .deduped_plan
            .dedupe
            .existing_pull_request
            .as_ref()
            .map(|pull_request| pull_request.url.as_str()),
        Some("https://github.com/nitrosend/api/pull/144")
    );
    assert_eq!(
        nested_string(
            &execution.pull_request_receipt.metadata,
            &["dedupe", "result"]
        ),
        Some("reused")
    );
    assert_eq!(
        execution.source_publication_receipt.source_thread_ref.uri,
        "slack://nitrosend/C0APFMY0V8Q/1778834840.485629"
    );
    assert_public_only(&execution)?;
    Ok(())
}

#[test]
fn runtime_requires_created_pull_request_for_create_decision()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let execution_plan = plan_target_repo_runner_execution(&plan, &readiness(true))?;

    let error = execute_target_repo_runner_execution_fixture(
        &plan,
        &execution_plan,
        &readiness(true),
        &empty_dedupe_observation("nitrosend/api", &execution_plan.provider_lookup.key),
        None,
    )
    .err();

    assert!(matches!(
        error,
        Some(TargetRepoRunnerRuntimeError::CreatedPullRequestRequired { .. })
    ));
    Ok(())
}

#[test]
fn live_adapter_composes_observations_into_revision_receipt_without_network()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let mut adapter = FakeTargetRepoRunnerAdapter {
        created_pull_request: created_pull_request(),
        provider_pull_requests: Vec::new(),
        expected_disposition: TargetRepoRunnerPullRequestDisposition::Create,
        omit_source_issue_publication: false,
        events: Vec::new(),
    };

    let live = execute_target_repo_runner_with_adapter(&plan, &mut adapter, CREATED_AT)?;

    assert_eq!(
        adapter.events,
        vec![
            "checkout",
            "dedupe",
            "runner",
            "pull_request",
            "source_publication"
        ]
    );
    assert_eq!(
        live.execution.disposition,
        TargetRepoRunnerPullRequestDisposition::Create
    );
    assert_eq!(
        live.execution.dedupe_execution.result,
        TargetRepoRunnerDedupeResult::LookupRequired
    );
    assert_eq!(
        nested_string(
            &live.execution.pull_request_receipt.metadata,
            &["dedupe", "result"]
        ),
        Some("created")
    );
    assert_eq!(
        live.execution
            .source_publication_receipt
            .pull_request_ref
            .uri,
        "https://github.com/nitrosend/api/pull/145"
    );

    assert_eq!(live.revision_receipt.harness.state, HarnessState::Sealed);
    assert_eq!(live.revision_receipt.harness.revision.sequence, 1);
    assert_eq!(live.revision_receipt.harness.acts.len(), 1);
    let act = &live.revision_receipt.harness.acts[0];
    assert_eq!(act.form, ActForm::Revision);
    assert!(act.revision.is_some());
    assert!(act.verification.is_none());
    assert_eq!(act.closure.reason_code, "target_runner_pr_created");
    assert_eq!(
        live.revision_projection.pull_request_ref.uri,
        "https://github.com/nitrosend/api/pull/145"
    );
    assert_eq!(
        live.revision_projection.source_thread_ref.uri,
        "slack://nitrosend/C0APFMY0V8Q/1778834840.485629"
    );
    assert_eq!(
        live.revision_projection.disposition,
        TargetRepoRunnerPullRequestDisposition::Create
    );
    assert_eq!(
        nested_string(
            &live.revision_projection.metadata,
            &["target_runner", "contract"]
        ),
        Some("runx.target_repo_runner.v1")
    );
    assert_eq!(live.source_publication_request.commands.len(), 2);
    assert_eq!(
        live.source_publication_receipt.harness.state,
        HarnessState::Sealed
    );
    assert_eq!(live.source_publication_receipt.harness.acts.len(), 1);
    assert_eq!(
        live.source_publication_receipt.harness.acts[0].form,
        ActForm::Reply
    );
    assert_eq!(
        live.source_publication_receipt.harness.acts[0]
            .closure
            .reason_code,
        "target_runner_source_published"
    );
    assert_eq!(
        live.source_publication_projection.pull_request_ref.uri,
        "https://github.com/nitrosend/api/pull/145"
    );
    assert_eq!(
        live.source_publication_projection
            .source_issue_ref
            .as_ref()
            .map(|reference| reference.uri.as_str()),
        Some("https://github.com/nitrosend/nitrosend/issues/482")
    );
    assert_eq!(
        live.source_publication_projection.source_thread_ref.uri,
        "slack://nitrosend/C0APFMY0V8Q/1778834840.485629"
    );
    assert_eq!(
        nested_string(
            &live.source_publication_projection.metadata,
            &["target_runner", "contract"]
        ),
        Some("runx.target_repo_runner.source_publication.v1")
    );

    let digest = canonical_receipt_body_digest(&live.revision_receipt)?;
    assert_eq!(live.revision_receipt.seal.digest, digest);
    assert_eq!(
        live.revision_receipt.signature.value,
        format!("sig:{digest}")
    );
    assert_public_only(&live.revision_receipt)?;
    assert_public_only(&live.source_publication_receipt)?;
    assert_hard_cutover_vocabulary_only(&live.revision_receipt)?;
    assert_hard_cutover_vocabulary_only(&live.source_publication_receipt)?;
    Ok(())
}

#[test]
fn live_adapter_reuses_provider_pull_request_without_runner_and_seals_reuse_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let lookup = plan_target_repo_runner_execution(&plan, &readiness(true))?.provider_lookup;
    let existing = TargetRepoRunnerProviderPullRequest {
        url: "https://github.com/nitrosend/api/pull/144".to_owned(),
        number: Some(144),
        branch: Some("runx/source-482".to_owned()),
        open: true,
        markers: lookup.query.markers.clone(),
        refs: lookup.query.required_refs.clone(),
    };
    let mut adapter = FakeTargetRepoRunnerAdapter {
        created_pull_request: created_pull_request(),
        provider_pull_requests: vec![existing],
        expected_disposition: TargetRepoRunnerPullRequestDisposition::Reuse,
        omit_source_issue_publication: false,
        events: Vec::new(),
    };

    let live = execute_target_repo_runner_with_adapter(&plan, &mut adapter, CREATED_AT)?;

    assert_eq!(
        adapter.events,
        vec!["checkout", "dedupe", "pull_request", "source_publication"]
    );
    assert!(live.runner_observation.is_none());
    assert_eq!(
        live.execution.disposition,
        TargetRepoRunnerPullRequestDisposition::Reuse
    );
    assert_eq!(
        live.execution.dedupe_execution.result,
        TargetRepoRunnerDedupeResult::Reused
    );
    assert_eq!(live.execution.pull_request.number, Some(144));
    assert_eq!(
        nested_string(
            &live.execution.source_publication_receipt.metadata,
            &["dedupe", "result"]
        ),
        Some("reused")
    );
    assert_eq!(
        nested_string(&live.revision_projection.metadata, &["dedupe", "result"]),
        Some("reused")
    );
    assert_eq!(
        live.revision_receipt.harness.acts[0].closure.reason_code,
        "target_runner_pr_reused"
    );
    assert_eq!(
        live.revision_projection.pull_request_ref.uri,
        "https://github.com/nitrosend/api/pull/144"
    );
    assert_eq!(
        live.revision_projection.disposition,
        TargetRepoRunnerPullRequestDisposition::Reuse
    );
    assert_eq!(
        nested_string(
            &live.source_publication_projection.metadata,
            &["dedupe", "result"]
        ),
        Some("reused")
    );
    assert_eq!(
        live.source_publication_projection.pull_request_ref.uri,
        "https://github.com/nitrosend/api/pull/144"
    );
    assert_public_only(&live.revision_receipt)?;
    assert_public_only(&live.source_publication_receipt)?;
    assert_hard_cutover_vocabulary_only(&live.revision_receipt)?;
    assert_hard_cutover_vocabulary_only(&live.source_publication_receipt)?;
    Ok(())
}

#[test]
fn live_adapter_fails_when_source_publication_readback_omits_source_issue()
-> Result<(), Box<dyn std::error::Error>> {
    let policy: OperationalPolicy = serde_json::from_str(NITROSEND_LIKE)?;
    let plan = plan_target_repo_runner(&policy, &nitrosend_request("nitrosend/api"))?;
    let mut adapter = FakeTargetRepoRunnerAdapter {
        created_pull_request: created_pull_request(),
        provider_pull_requests: Vec::new(),
        expected_disposition: TargetRepoRunnerPullRequestDisposition::Create,
        omit_source_issue_publication: true,
        events: Vec::new(),
    };

    let error = execute_target_repo_runner_with_adapter(&plan, &mut adapter, CREATED_AT).err();

    assert!(matches!(
        error,
        Some(TargetRepoRunnerRuntimeError::SourcePublicationMismatch(_))
    ));
    assert_eq!(
        adapter.events,
        vec![
            "checkout",
            "dedupe",
            "runner",
            "pull_request",
            "source_publication"
        ]
    );
    Ok(())
}

fn nitrosend_request(target_repo: &str) -> TargetRepoRunnerPlanRequest {
    TargetRepoRunnerPlanRequest {
        source_id: Some("bugs-fixes".to_owned()),
        target_repo: target_repo.to_owned(),
        action: OperationalPolicyAction::IssueToPr,
        runner_id: None,
        source: TargetRepoRunnerSourceContext {
            provider: runx_contracts::OperationalPolicySourceProvider::Slack,
            locator: "slack://nitrosend/C0APFMY0V8Q".to_owned(),
            thread_locator: Some("slack://nitrosend/C0APFMY0V8Q/1778834840.485629".to_owned()),
            thread_ts: Some("1778834840.485629".to_owned()),
            issue_url: Some("https://github.com/nitrosend/nitrosend/issues/482".to_owned()),
        },
        signal_fingerprint: Some("sha256:nitrosend-source-482".to_owned()),
        existing_pull_request: None,
    }
}

struct FakeTargetRepoRunnerAdapter {
    created_pull_request: TargetRepoRunnerExistingPullRequest,
    provider_pull_requests: Vec<TargetRepoRunnerProviderPullRequest>,
    expected_disposition: TargetRepoRunnerPullRequestDisposition,
    omit_source_issue_publication: bool,
    events: Vec<&'static str>,
}

impl TargetRepoRunnerAdapter for FakeTargetRepoRunnerAdapter {
    fn checkout_readiness(
        &mut self,
        plan: &runx_contracts::TargetRepoRunnerPlan,
    ) -> Result<TargetRepoRunnerReadinessObservation, TargetRepoRunnerAdapterError> {
        self.events.push("checkout");
        Ok(TargetRepoRunnerReadinessObservation {
            target_repo: plan.target.repo.clone(),
            runner_id: plan.runner.runner_id.clone(),
            scafld_ready: true,
        })
    }

    fn provider_dedupe_lookup(
        &mut self,
        lookup: &TargetRepoRunnerDedupeLookupPlan,
    ) -> Result<TargetRepoRunnerDedupeLookupObservation, TargetRepoRunnerAdapterError> {
        self.events.push("dedupe");
        Ok(TargetRepoRunnerDedupeLookupObservation {
            provider: lookup.provider,
            target_repo: lookup.target_repo.clone(),
            key: lookup.key.clone(),
            pull_requests: self.provider_pull_requests.clone(),
        })
    }

    fn invoke_governed_runner(
        &mut self,
        invocation: &TargetRepoRunnerGovernedRunnerInvocation,
    ) -> Result<TargetRepoRunnerGovernedRunnerObservation, TargetRepoRunnerAdapterError> {
        self.events.push("runner");
        Ok(TargetRepoRunnerGovernedRunnerObservation {
            runner_id: invocation.execution_plan.readiness.runner_id.clone(),
            target_repo: invocation.execution_plan.checkout.target_repo.clone(),
            summary: "Fake governed target runner prepared a branch for review.".to_owned(),
            revision_refs: vec![Reference {
                reference_type: ReferenceType::GithubPullRequest,
                uri: self.created_pull_request.url.clone(),
                provider: Some("github".to_owned()),
                locator: self
                    .created_pull_request
                    .number
                    .map(|number| format!("nitrosend/api#{number}")),
                label: Some("Nitrosend API target PR".to_owned()),
                observed_at: Some(CREATED_AT.to_owned()),
                proof_kind: None,
            }],
            artifact_refs: vec![Reference {
                reference_type: ReferenceType::Artifact,
                uri: "runx:artifact:target-runner-fake-diff".to_owned(),
                provider: None,
                locator: None,
                label: Some("fake sanitized diff".to_owned()),
                observed_at: Some(CREATED_AT.to_owned()),
                proof_kind: None,
            }],
            verification_refs: vec![Reference {
                reference_type: ReferenceType::Verification,
                uri: "runx:verification:target-runner-fake".to_owned(),
                provider: None,
                locator: None,
                label: Some("fake no-network verification".to_owned()),
                observed_at: Some(CREATED_AT.to_owned()),
                proof_kind: None,
            }],
        })
    }

    fn observe_pull_request(
        &mut self,
        request: &TargetRepoRunnerPullRequestObservationRequest,
    ) -> Result<TargetRepoRunnerExistingPullRequest, TargetRepoRunnerAdapterError> {
        self.events.push("pull_request");
        assert_eq!(request.disposition, self.expected_disposition);
        match request.disposition {
            TargetRepoRunnerPullRequestDisposition::Create => {
                assert!(request.existing_pull_request.is_none());
                assert!(request.runner_observation.is_some());
                Ok(self.created_pull_request.clone())
            }
            TargetRepoRunnerPullRequestDisposition::Reuse => {
                assert!(request.runner_observation.is_none());
                request.existing_pull_request.clone().ok_or_else(|| {
                    TargetRepoRunnerAdapterError::new(
                        "pull_request",
                        "expected existing pull request for reuse",
                    )
                })
            }
        }
    }

    fn publish_source_update(
        &mut self,
        request: &TargetRepoRunnerSourcePublicationRequest,
    ) -> Result<TargetRepoRunnerSourcePublicationObservation, TargetRepoRunnerAdapterError> {
        self.events.push("source_publication");
        assert_source_publication_commands(request);
        Ok(TargetRepoRunnerSourcePublicationObservation {
            source_issue_ref: if self.omit_source_issue_publication {
                None
            } else {
                request.publication.source_issue_ref.clone()
            },
            source_thread_ref: request.publication.source_thread_ref.clone(),
            pull_request_ref: request.publication.pull_request_ref.clone(),
            revision_receipt_ref: request.revision_receipt_ref.clone(),
            published_refs: vec![
                Reference {
                    reference_type: ReferenceType::ExternalUrl,
                    uri: "https://github.com/nitrosend/nitrosend/issues/482#issuecomment-9001"
                        .to_owned(),
                    provider: Some("github".to_owned()),
                    locator: Some("nitrosend/nitrosend#482-comment-9001".to_owned()),
                    label: Some("source issue target PR comment".to_owned()),
                    observed_at: Some(CREATED_AT.to_owned()),
                    proof_kind: None,
                },
                Reference {
                    reference_type: ReferenceType::SlackThread,
                    uri: "slack://nitrosend/C0APFMY0V8Q/1778834840.485629/reply/1778835000.000100"
                        .to_owned(),
                    provider: Some("slack".to_owned()),
                    locator: Some("C0APFMY0V8Q/1778835000.000100".to_owned()),
                    label: Some("source thread target PR reply".to_owned()),
                    observed_at: Some(CREATED_AT.to_owned()),
                    proof_kind: None,
                },
            ],
            metadata: request.publication.metadata.clone(),
        })
    }
}

fn assert_source_publication_commands(request: &TargetRepoRunnerSourcePublicationRequest) {
    assert_eq!(request.commands.len(), 2);
    let mut saw_issue_comment = false;
    let mut saw_thread_reply = false;
    for command in &request.commands {
        match command {
            TargetRepoRunnerSourcePublicationCommand::SourceIssueComment { target, body } => {
                saw_issue_comment = true;
                assert_eq!(
                    target.uri,
                    "https://github.com/nitrosend/nitrosend/issues/482"
                );
                assert!(body.contains(&request.publication.pull_request_ref.uri));
                assert!(body.contains(&request.revision_receipt_ref.uri));
                assert!(body.contains("Human review remains the merge gate."));
            }
            TargetRepoRunnerSourcePublicationCommand::SourceThreadReply { target, body } => {
                saw_thread_reply = true;
                assert_eq!(
                    target.uri,
                    "slack://nitrosend/C0APFMY0V8Q/1778834840.485629"
                );
                assert!(body.contains(&request.publication.pull_request_ref.uri));
                assert!(body.contains("Target repo: nitrosend/api"));
            }
        }
    }
    assert!(saw_issue_comment);
    assert!(saw_thread_reply);
}

fn readiness(scafld_ready: bool) -> TargetRepoRunnerReadinessObservation {
    TargetRepoRunnerReadinessObservation {
        target_repo: "nitrosend/api".to_owned(),
        runner_id: "aster-production".to_owned(),
        scafld_ready,
    }
}

fn empty_dedupe_observation(
    target_repo: &str,
    key: &str,
) -> TargetRepoRunnerDedupeLookupObservation {
    TargetRepoRunnerDedupeLookupObservation {
        provider: TargetRepoRunnerProvider::Github,
        target_repo: target_repo.to_owned(),
        key: key.to_owned(),
        pull_requests: Vec::new(),
    }
}

fn created_pull_request() -> TargetRepoRunnerExistingPullRequest {
    TargetRepoRunnerExistingPullRequest {
        url: "https://github.com/nitrosend/api/pull/145".to_owned(),
        number: Some(145),
        branch: Some("runx/source-482-new".to_owned()),
    }
}

fn nested_string<'a>(
    object: &'a std::collections::BTreeMap<String, JsonValue>,
    path: &[&str],
) -> Option<&'a str> {
    let mut value = object.get(*path.first()?)?;
    for segment in &path[1..] {
        let JsonValue::Object(object) = value else {
            return None;
        };
        value = object.get(*segment)?;
    }
    match value {
        JsonValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn assert_public_only<T: serde::Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(value)?;
    assert!(!json.contains("/Users/"));
    assert!(!json.contains("/tmp/"));
    assert!(!json.contains("RUNX_"));
    assert!(!json.contains("SECRET"));
    Ok(())
}

fn assert_hard_cutover_vocabulary_only<T: serde::Serialize>(
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(value)?;
    for retired in [
        "work_item",
        "matter",
        "engagement",
        "judgment",
        "effect",
        "outcome",
    ] {
        assert!(!json.contains(retired), "retired field leaked: {retired}");
    }
    Ok(())
}
