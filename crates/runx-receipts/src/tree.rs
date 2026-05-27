// rust-style-allow: large-file -- the public facade is small; the remaining
// size is the adversarial receipt-tree test matrix.
mod findings;
mod proof;
mod resolver;
mod traversal;

use std::collections::BTreeSet;

use runx_contracts::{Receipt, Reference};

use crate::{
    ReceiptFinding, ReceiptProofContext, ReceiptVerification, verify_receipt, verify_receipt_proof,
};
use findings::{child_receipt_findings, duplicate_child_findings, orphan_child_findings};
use proof::{StrictChildProofPolicy, StructuralChildProofPolicy, child_receipt_proof_findings};
use resolver::SliceReceiptResolver;
use traversal::TreeTraversal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptTreeConfig {
    pub max_depth: usize,
    pub max_breadth: usize,
    pub require_parent_links: bool,
}

impl Default for ReceiptTreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 64,
            max_breadth: 1024,
            require_parent_links: false,
        }
    }
}

pub trait ReceiptResolver {
    fn resolve_child<'a>(&'a self, reference: &Reference) -> ReceiptResolveResult<'a>;
    fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>>;
}

pub trait ReceiptProofContextProvider {
    fn proof_context<'a>(&'a self, receipt: &Receipt) -> ReceiptProofContext<'a>;
}

#[derive(Clone, Debug)]
pub struct ResolvedReceipt<'a> {
    pub path: String,
    pub receipt: &'a Receipt,
}

#[derive(Clone, Debug)]
pub enum ReceiptResolveResult<'a> {
    Found(ResolvedReceipt<'a>),
    Missing,
    Malformed,
    Ambiguous,
    ResolverError,
}

pub fn validate_receipt_tree(
    root: &Receipt,
    children: &[Receipt],
) -> Result<(), ReceiptVerification> {
    let resolver = SliceReceiptResolver { children };
    validate_receipt_tree_with_resolver(root, &resolver, ReceiptTreeConfig::default())
}

#[must_use]
pub fn verify_receipt_tree(root: &Receipt, children: &[Receipt]) -> ReceiptVerification {
    let resolver = SliceReceiptResolver { children };
    verify_receipt_tree_with_resolver(root, &resolver, ReceiptTreeConfig::default())
}

pub fn validate_receipt_tree_proof(
    root: &Receipt,
    children: &[Receipt],
    proof_contexts: &impl ReceiptProofContextProvider,
) -> Result<(), ReceiptVerification> {
    let resolver = SliceReceiptResolver { children };
    validate_receipt_tree_proof_with_resolver(
        root,
        &resolver,
        ReceiptTreeConfig::default(),
        proof_contexts,
    )
}

#[must_use]
pub fn verify_receipt_tree_proof(
    root: &Receipt,
    children: &[Receipt],
    proof_contexts: &impl ReceiptProofContextProvider,
) -> ReceiptVerification {
    let resolver = SliceReceiptResolver { children };
    verify_receipt_tree_proof_with_resolver(
        root,
        &resolver,
        ReceiptTreeConfig::default(),
        proof_contexts,
    )
}

pub fn validate_receipt_tree_with_resolver(
    root: &Receipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
) -> Result<(), ReceiptVerification> {
    let verification = verify_receipt_tree_with_resolver(root, resolver, config);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_receipt_tree_with_resolver(
    root: &Receipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
) -> ReceiptVerification {
    let mut findings = verify_receipt(root).findings;
    let supplied = resolver.supplied_receipts();
    findings.extend(duplicate_child_findings(&supplied));
    findings.extend(child_receipt_findings(&supplied));
    verify_tree_relationships(root, resolver, config, &supplied, findings)
}

pub fn validate_receipt_tree_proof_with_resolver(
    root: &Receipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
    proof_contexts: &impl ReceiptProofContextProvider,
) -> Result<(), ReceiptVerification> {
    let verification =
        verify_receipt_tree_proof_with_resolver(root, resolver, config, proof_contexts);
    if verification.valid {
        Ok(())
    } else {
        Err(verification)
    }
}

#[must_use]
pub fn verify_receipt_tree_proof_with_resolver(
    root: &Receipt,
    resolver: &impl ReceiptResolver,
    config: ReceiptTreeConfig,
    proof_contexts: &impl ReceiptProofContextProvider,
) -> ReceiptVerification {
    let root_context = proof_contexts.proof_context(root);
    let mut findings = verify_receipt_proof(root, &root_context).findings;
    let supplied = resolver.supplied_receipts();
    findings.extend(duplicate_child_findings(&supplied));
    findings.extend(child_receipt_proof_findings(&supplied, proof_contexts));
    verify_tree_relationships_with_proof(
        root,
        resolver,
        config,
        &supplied,
        findings,
        proof_contexts,
    )
}

fn verify_tree_relationships<R: ReceiptResolver>(
    root: &Receipt,
    resolver: &R,
    config: ReceiptTreeConfig,
    supplied: &[ResolvedReceipt<'_>],
    mut findings: Vec<ReceiptFinding>,
) -> ReceiptVerification {
    let mut traversal = TreeTraversal {
        resolver,
        config,
        proof_policy: StructuralChildProofPolicy,
        visiting: BTreeSet::new(),
        reached: BTreeSet::new(),
    };
    findings.extend(traversal.subtree_findings("", root, 0));
    findings.extend(orphan_child_findings(supplied, &traversal.reached));
    ReceiptVerification::from_findings(findings)
}

fn verify_tree_relationships_with_proof<R: ReceiptResolver>(
    root: &Receipt,
    resolver: &R,
    config: ReceiptTreeConfig,
    supplied: &[ResolvedReceipt<'_>],
    mut findings: Vec<ReceiptFinding>,
    proof_contexts: &impl ReceiptProofContextProvider,
) -> ReceiptVerification {
    let mut traversal = TreeTraversal {
        resolver,
        config,
        proof_policy: StrictChildProofPolicy::new(supplied, proof_contexts),
        visiting: BTreeSet::new(),
        reached: BTreeSet::new(),
    };
    findings.extend(traversal.subtree_findings("", root, 0));
    findings.extend(orphan_child_findings(supplied, &traversal.reached));
    ReceiptVerification::from_findings(findings)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{
        ReceiptProofContextProvider, ReceiptResolveResult, ReceiptResolver, ReceiptTreeConfig,
        ResolvedReceipt, validate_receipt_tree_proof, validate_receipt_tree_with_resolver,
        verify_receipt_tree, verify_receipt_tree_proof, verify_receipt_tree_proof_with_resolver,
        verify_receipt_tree_with_resolver,
    };
    use crate::{
        ReceiptFindingCode, ReceiptProofContext, ReceiptSignature, ReceiptVerification,
        SignatureVerificationFailure, SignatureVerifier, canonical_receipt_body_digest,
    };
    use runx_contracts::{Receipt, ReceiptIssuer, Reference, ReferenceType};

    const SUCCESS_RECEIPT: &str =
        include_str!("../../../fixtures/contracts/harness-spine/receipt-success.json");
    const ABNORMAL_RECEIPT: &str =
        include_str!("../../../fixtures/contracts/harness-spine/receipt-abnormal.json");

    #[derive(Debug, Deserialize)]
    struct Fixture {
        expected: Receipt,
    }

    fn child_refs_mut(receipt: &mut Receipt) -> &mut Vec<Reference> {
        &mut receipt
            .lineage
            .get_or_insert_with(Default::default)
            .children
    }

    #[test]
    fn slice_adapter_accepts_only_typed_receipt_uri() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        child_refs_mut(&mut root)[0].uri = "hrn_rcpt_child_1".to_owned().into();
        let verification = verify_receipt_tree(&root, std::slice::from_ref(&child));
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "lineage.children[0]",
        );

        child_refs_mut(&mut root)[0].uri = "runx:receipt:hrn_rcpt_child_1".to_owned().into();
        assert!(verify_receipt_tree(&root, &[child]).valid);
        Ok(())
    }

    #[test]
    fn malformed_and_wrong_namespace_refs_are_stable_findings() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        child_refs_mut(&mut root)[0].uri = "runx:graph_receipt:hrn_rcpt_child_1".to_owned().into();
        let verification = verify_receipt_tree(&root, std::slice::from_ref(&child));
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "lineage.children[0]",
        );

        child_refs_mut(&mut root)[0].uri = ":hrn_rcpt_child_1".to_owned().into();
        let verification = verify_receipt_tree(&root, &[child]);
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn suffix_only_refs_are_malformed_not_aliases() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        child_refs_mut(&mut root)[0].uri = "child_1".to_owned().into();
        let child = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree(&root, &[child]);

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn duplicate_ids_make_slice_resolution_ambiguous() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let first = child("hrn_rcpt_child_1")?;
        let second = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree(&root, &[first, second]);

        assert_finding(
            &verification,
            ReceiptFindingCode::DuplicateChildReceipt,
            "children[1].id",
        );
        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptAmbiguous,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn resolver_ambiguous_result_is_a_stable_finding() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &AmbiguousResolver,
            ReceiptTreeConfig::default(),
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptAmbiguous,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn resolver_error_result_is_a_stable_finding() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &ResolverErrorResolver,
            ReceiptTreeConfig::default(),
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptResolverError,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn strict_mode_rejects_mismatched_parent_link() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child = child("hrn_rcpt_child_1")?;
        child.lineage.get_or_insert_with(Default::default).parent =
            Some(reference(ReferenceType::Receipt, "other"));

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: std::slice::from_ref(&child),
            },
            ReceiptTreeConfig {
                require_parent_links: true,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptParentMismatch,
            "lineage.children[0].lineage.parent",
        );
        Ok(())
    }

    #[test]
    fn strict_mode_requires_present_parent_link() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let child = child("hrn_rcpt_child_1")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: std::slice::from_ref(&child),
            },
            ReceiptTreeConfig {
                require_parent_links: true,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptParentMismatch,
            "lineage.children[0].lineage.parent",
        );
        Ok(())
    }

    #[test]
    fn depth_limit_blocks_hostile_nested_tree() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child_receipt = child("hrn_rcpt_child_1")?;
        child_refs_mut(&mut child_receipt).push(reference(ReferenceType::Receipt, "grandchild"));
        let grandchild = child("grandchild")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: &[child_receipt, grandchild],
            },
            ReceiptTreeConfig {
                max_depth: 1,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptDepthLimit,
            "children[0].lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn breadth_limit_blocks_hostile_fanout() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        child_refs_mut(&mut root).push(reference(ReferenceType::Receipt, "second"));
        let first = child("hrn_rcpt_child_1")?;
        let second = child("second")?;

        let verification = verify_receipt_tree_with_resolver(
            &root,
            &super::SliceReceiptResolver {
                children: &[first, second],
            },
            ReceiptTreeConfig {
                max_breadth: 1,
                ..ReceiptTreeConfig::default()
            },
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptBreadthLimit,
            "lineage.children",
        );
        Ok(())
    }

    #[test]
    fn positive_nested_tree_verifies() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child_receipt = child("hrn_rcpt_child_1")?;
        child_refs_mut(&mut child_receipt).push(reference(ReferenceType::Receipt, "grandchild"));
        let grandchild = child("grandchild")?;

        assert!(verify_receipt_tree(&root, &[child_receipt, grandchild]).valid);
        Ok(())
    }

    #[test]
    fn positive_fanout_tree_verifies() -> Result<(), serde_json::Error> {
        let mut root = fixture(SUCCESS_RECEIPT)?;
        child_refs_mut(&mut root).push(reference(ReferenceType::Receipt, "second"));
        let first = child("hrn_rcpt_child_1")?;
        let second = child("second")?;

        assert!(verify_receipt_tree(&root, &[first, second]).valid);
        Ok(())
    }

    #[test]
    fn strict_parent_links_can_verify_cleanly() -> Result<(), serde_json::Error> {
        let root = fixture(SUCCESS_RECEIPT)?;
        let mut child = child("hrn_rcpt_child_1")?;
        child.lineage.get_or_insert_with(Default::default).parent =
            Some(Reference::runx(ReferenceType::Receipt, &root.id));

        assert!(
            validate_receipt_tree_with_resolver(
                &root,
                &super::SliceReceiptResolver {
                    children: std::slice::from_ref(&child),
                },
                ReceiptTreeConfig {
                    require_parent_links: true,
                    ..ReceiptTreeConfig::default()
                },
            )
            .is_ok()
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_accepts_root_and_child() -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        let proof_contexts = FixtureProofContexts::default();

        assert!(validate_receipt_tree_proof(&root, &[child], &proof_contexts).is_ok());
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_missing_child() -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof(&root, &[], &proof_contexts);

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptMissing,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_extra_child() -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        let extra = proof_child("hrn_rcpt_extra")?;
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof(&root, &[child, extra], &proof_contexts);

        assert_finding(
            &verification,
            ReceiptFindingCode::OrphanChildReceipt,
            "children[1].id",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_legacy_exact_id_child_ref() -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        child_refs_mut(&mut root)[0].uri = child.id.clone();
        refresh_proof_digest_and_signature(&mut root)?;
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof(&root, &[child], &proof_contexts);

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptRefMalformed,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_structurally_valid_child_proof_mismatch()
    -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let mut child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        child.acts[0].summary = "tampered child proof body".into();
        let proof_contexts = FixtureProofContexts::default();

        assert!(verify_receipt_tree(&root, std::slice::from_ref(&child)).valid);
        let verification = verify_receipt_tree_proof(&root, &[child], &proof_contexts);

        assert_finding(
            &verification,
            ReceiptFindingCode::SealDigestMismatch,
            "children[0].digest",
        );
        assert_finding(
            &verification,
            ReceiptFindingCode::SignatureInvalid,
            "children[0].signature.value",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_valid_alternate_child_with_same_id()
    -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let original = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &original)?;
        let mut alternate = proof_child("hrn_rcpt_child_1")?;
        alternate.acts[0].summary = "valid alternate child body".into();
        refresh_proof_digest_and_signature(&mut alternate)?;
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof(&root, &[alternate], &proof_contexts);

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptDigestMismatch,
            "children[0].locator",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_custom_resolver_child_not_in_supplied_receipts()
    -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let mut child = proof_child("hrn_rcpt_child_1")?;
        link_child_digest(&mut root, 0, &child)?;
        child.acts[0].summary = "hidden tampered child".into();
        let resolver = HiddenChildResolver { child: &child };
        let proof_contexts = FixtureProofContexts::default();

        assert!(
            verify_receipt_tree_with_resolver(&root, &resolver, ReceiptTreeConfig::default()).valid
        );
        let verification = verify_receipt_tree_proof_with_resolver(
            &root,
            &resolver,
            ReceiptTreeConfig::default(),
            &proof_contexts,
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::SealDigestMismatch,
            "hidden_child.digest",
        );
        assert_finding(
            &verification,
            ReceiptFindingCode::SignatureInvalid,
            "hidden_child.signature.value",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_resolver_error() -> Result<(), serde_json::Error> {
        let root = proof_root()?;
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof_with_resolver(
            &root,
            &ResolverErrorResolver,
            ReceiptTreeConfig::default(),
            &proof_contexts,
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::ChildReceiptResolverError,
            "lineage.children[0]",
        );
        Ok(())
    }

    #[test]
    fn strict_tree_proof_rejects_custom_resolver_duplicate_id_child_after_reached()
    -> Result<(), serde_json::Error> {
        let mut root = proof_root()?;
        let first = proof_child("shared_child")?;
        let mut second = proof_child("shared_child")?;
        *child_refs_mut(&mut root) = vec![
            reference(ReferenceType::Receipt, "first"),
            reference(ReferenceType::Receipt, "second"),
        ];
        child_refs_mut(&mut root)[0].locator = Some(first.digest.clone());
        child_refs_mut(&mut root)[1].locator = Some(second.digest.clone());
        refresh_proof_digest_and_signature(&mut root)?;
        second.acts[0].summary = "hidden duplicate-id tamper".into();
        let resolver = DuplicateIdResolver {
            first: &first,
            second: &second,
        };
        let proof_contexts = FixtureProofContexts::default();

        let verification = verify_receipt_tree_proof_with_resolver(
            &root,
            &resolver,
            ReceiptTreeConfig::default(),
            &proof_contexts,
        );

        assert_finding(
            &verification,
            ReceiptFindingCode::SealDigestMismatch,
            "hidden_second.digest",
        );
        assert_finding(
            &verification,
            ReceiptFindingCode::SignatureInvalid,
            "hidden_second.signature.value",
        );
        Ok(())
    }

    fn fixture(json: &str) -> Result<Receipt, serde_json::Error> {
        let mut receipt = serde_json::from_str::<Fixture>(json).map(|fixture| fixture.expected)?;
        // The flat success fixture carries no children; the tree tests need one
        // typed child ref to mutate, so seed a single receipt ref.
        if receipt
            .lineage
            .as_ref()
            .is_none_or(|lineage| lineage.children.is_empty())
        {
            child_refs_mut(&mut receipt)
                .push(Reference::runx(ReferenceType::Receipt, "hrn_rcpt_child_1"));
        }
        Ok(receipt)
    }

    fn child(id: &str) -> Result<Receipt, serde_json::Error> {
        let mut receipt = fixture(ABNORMAL_RECEIPT)?;
        receipt.id = id.into();
        child_refs_mut(&mut receipt).clear();
        Ok(receipt)
    }

    fn proof_root() -> Result<Receipt, serde_json::Error> {
        let mut receipt = fixture(SUCCESS_RECEIPT)?;
        refresh_proof_digest_and_signature(&mut receipt)?;
        Ok(receipt)
    }

    fn proof_child(id: &str) -> Result<Receipt, serde_json::Error> {
        let mut receipt = fixture(SUCCESS_RECEIPT)?;
        receipt.id = id.into();
        child_refs_mut(&mut receipt).clear();
        refresh_proof_digest_and_signature(&mut receipt)?;
        Ok(receipt)
    }

    fn link_child_digest(
        root: &mut Receipt,
        index: usize,
        child: &Receipt,
    ) -> Result<(), serde_json::Error> {
        child_refs_mut(root)[index].locator = Some(child.digest.clone());
        refresh_proof_digest_and_signature(root)
    }

    fn refresh_proof_digest_and_signature(receipt: &mut Receipt) -> Result<(), serde_json::Error> {
        let digest = canonical_receipt_body_digest(receipt)
            .map_err(|error| serde_json::Error::io(std::io::Error::other(error.to_string())))?;
        receipt.digest = digest.clone().into();
        receipt.signature.value = format!("sig:{digest}").into();
        Ok(())
    }

    fn reference(reference_type: ReferenceType, id: &str) -> Reference {
        Reference::runx(reference_type, id)
    }

    fn assert_finding(verification: &ReceiptVerification, code: ReceiptFindingCode, path: &str) {
        assert!(
            verification
                .findings
                .iter()
                .any(|finding| finding.code == code && finding.path == path),
            "expected finding {code:?} at {path}; got {:?}",
            verification.findings
        );
    }

    #[derive(Default)]
    struct FixtureProofContexts {
        verifier: FixtureSignatureVerifier,
    }

    impl ReceiptProofContextProvider for FixtureProofContexts {
        fn proof_context<'a>(&'a self, _receipt: &Receipt) -> ReceiptProofContext<'a> {
            ReceiptProofContext {
                signature_verifier: Some(&self.verifier),
                authority_verified: true,
                external_attestations_verified: true,
                verified_redaction_refs: std::collections::BTreeSet::new(),
                verified_hash_commitments: std::collections::BTreeSet::new(),
            }
        }
    }

    #[derive(Default)]
    struct FixtureSignatureVerifier;

    impl SignatureVerifier for FixtureSignatureVerifier {
        fn verify(
            &self,
            _issuer: &ReceiptIssuer,
            signature: &ReceiptSignature,
            body_digest: &str,
        ) -> Result<(), SignatureVerificationFailure> {
            if signature.value == format!("sig:{body_digest}") {
                Ok(())
            } else {
                Err(SignatureVerificationFailure::SignatureMismatch)
            }
        }
    }

    struct AmbiguousResolver;

    impl ReceiptResolver for AmbiguousResolver {
        fn resolve_child<'a>(&'a self, _reference: &Reference) -> ReceiptResolveResult<'a> {
            ReceiptResolveResult::Ambiguous
        }

        fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
            Vec::new()
        }
    }

    struct ResolverErrorResolver;

    impl ReceiptResolver for ResolverErrorResolver {
        fn resolve_child<'a>(&'a self, _reference: &Reference) -> ReceiptResolveResult<'a> {
            ReceiptResolveResult::ResolverError
        }

        fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
            Vec::new()
        }
    }

    struct HiddenChildResolver<'a> {
        child: &'a Receipt,
    }

    impl ReceiptResolver for HiddenChildResolver<'_> {
        fn resolve_child<'a>(&'a self, _reference: &Reference) -> ReceiptResolveResult<'a> {
            ReceiptResolveResult::Found(ResolvedReceipt {
                path: "hidden_child".to_owned(),
                receipt: self.child,
            })
        }

        fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
            Vec::new()
        }
    }

    struct DuplicateIdResolver<'a> {
        first: &'a Receipt,
        second: &'a Receipt,
    }

    impl ReceiptResolver for DuplicateIdResolver<'_> {
        fn resolve_child<'a>(&'a self, reference: &Reference) -> ReceiptResolveResult<'a> {
            if reference.uri.ends_with(":first") {
                return ReceiptResolveResult::Found(ResolvedReceipt {
                    path: "hidden_first".to_owned(),
                    receipt: self.first,
                });
            }
            ReceiptResolveResult::Found(ResolvedReceipt {
                path: "hidden_second".to_owned(),
                receipt: self.second,
            })
        }

        fn supplied_receipts<'a>(&'a self) -> Vec<ResolvedReceipt<'a>> {
            Vec::new()
        }
    }
}
