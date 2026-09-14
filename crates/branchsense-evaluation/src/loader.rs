//! Deterministic JSONL dataset loading and validation.

use crate::model::EvaluationCase;
use std::collections::BTreeSet;

/// Validation diagnostic emitted during dataset loading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatasetDiagnostic {
    /// A case ID was duplicated.
    DuplicateCaseId(String),
    /// Revisions were malformed or empty.
    MalformedRevision(String),
}

impl std::fmt::Display for DatasetDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCaseId(id) => write!(f, "Duplicate case ID: {id}"),
            Self::MalformedRevision(id) => write!(f, "Malformed or empty revision in case: {id}"),
        }
    }
}

/// A structured evaluation dataset.
#[derive(Clone, Debug)]
pub struct EvaluationDataset {
    cases: Vec<EvaluationCase>,
    diagnostics: Vec<DatasetDiagnostic>,
}

impl EvaluationDataset {
    /// Parses a JSONL string into a dataset, running deterministic validations.
    #[must_use]
    pub fn from_jsonl(content: &str) -> Self {
        let mut cases = Vec::new();
        let mut diagnostics = Vec::new();
        let mut seen_ids = BTreeSet::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            match serde_json::from_str::<EvaluationCase>(line) {
                Ok(case) => {
                    if !seen_ids.insert(case.case_id().to_owned()) {
                        diagnostics.push(DatasetDiagnostic::DuplicateCaseId(case.case_id().to_owned()));
                    } else if case.base_revision().as_str().is_empty()
                        || case.branch_a_revision().as_str().is_empty()
                        || case.branch_b_revision().as_str().is_empty()
                    {
                        diagnostics.push(DatasetDiagnostic::MalformedRevision(case.case_id().to_owned()));
                    } else {
                        cases.push(case);
                    }
                }
                Err(_err) => {
                    // In a production app, we would log the detailed Serde error.
                    // For the harness, silently dropping completely malformed JSON lines
                    // or storing a parse diagnostic is the strategy. We will append a diagnostic.
                    diagnostics.push(DatasetDiagnostic::MalformedRevision("Unparseable JSON".to_owned()));
                }
            }
        }

        Self { cases, diagnostics }
    }

    /// The validated cases, retaining deterministic order.
    #[must_use]
    pub fn cases(&self) -> &[EvaluationCase] {
        &self.cases
    }

    /// Any validation diagnostics encountered.
    #[must_use]
    pub fn diagnostics(&self) -> &[DatasetDiagnostic] {
        &self.diagnostics
    }

    /// Returns `true` if the dataset loaded without any diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
