use std::collections::BTreeSet;

use runx_contracts::{Receipt, Reference};

use super::{ReceiptProofContextProvider, ResolvedReceipt};
use crate::tree::findings::{child_digest_link_findings, child_finding};
use crate::{ReceiptFinding, verify_receipt_proof};

pub(super) fn child_receipt_proof_findings(
    children: &[ResolvedReceipt<'_>],
    proof_contexts: &impl ReceiptProofContextProvider,
) -> Vec<ReceiptFinding> {
    children
        .iter()
        .flat_map(|child| {
            let context = proof_contexts.proof_context(child.receipt);
            verify_receipt_proof(child.receipt, &context)
                .findings
                .into_iter()
                .map(|finding| child_finding(&child.path, finding))
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) trait ChildProofPolicy {
    fn findings(
        &mut self,
        path: &str,
        reference: &Reference,
        receipt: &Receipt,
    ) -> Vec<ReceiptFinding>;
}

pub(super) struct StructuralChildProofPolicy;

impl ChildProofPolicy for StructuralChildProofPolicy {
    fn findings(
        &mut self,
        _path: &str,
        _reference: &Reference,
        _receipt: &Receipt,
    ) -> Vec<ReceiptFinding> {
        Vec::new()
    }
}

pub(super) struct StrictChildProofPolicy<'a, P: ReceiptProofContextProvider> {
    proof_contexts: &'a P,
    verified_receipts: BTreeSet<usize>,
}

impl<'a, P: ReceiptProofContextProvider> StrictChildProofPolicy<'a, P> {
    pub(super) fn new(supplied: &[ResolvedReceipt<'_>], proof_contexts: &'a P) -> Self {
        Self {
            proof_contexts,
            verified_receipts: supplied
                .iter()
                .map(|child| receipt_address(child.receipt))
                .collect(),
        }
    }
}

impl<P: ReceiptProofContextProvider> ChildProofPolicy for StrictChildProofPolicy<'_, P> {
    fn findings(
        &mut self,
        path: &str,
        reference: &Reference,
        receipt: &Receipt,
    ) -> Vec<ReceiptFinding> {
        let mut findings = child_digest_link_findings(path, reference, receipt);
        if !self.verified_receipts.insert(receipt_address(receipt)) {
            return findings;
        }
        let context = self.proof_contexts.proof_context(receipt);
        findings.extend(
            verify_receipt_proof(receipt, &context)
                .findings
                .into_iter()
                .map(|finding| child_finding(path, finding)),
        );
        findings
    }
}

fn receipt_address(receipt: &Receipt) -> usize {
    receipt as *const Receipt as usize
}
