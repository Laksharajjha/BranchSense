#![allow(clippy::similar_names)]
//! Evaluation dataset schemas and models.

use branchsense_bcs::model::BcsAssessment;
use branchsense_semantic::{
    DatasetSchemaVersion, EvalOutcome, EvalRepositoryIdentity, EvalRevision,
};
use serde::{Deserialize, Serialize};

/// An explicitly structured, historical evaluation case acting as input.
///
/// This type ensures there is zero cross-contamination between a pre-computed
/// prediction and the inputs strictly necessary for the runner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvaluationCase {
    case_id: String,
    schema_version: DatasetSchemaVersion,
    repository: EvalRepositoryIdentity,
    base_revision: EvalRevision,
    branch_a_revision: EvalRevision,
    branch_b_revision: EvalRevision,
    merge_base: EvalRevision,
    /// The actual historical integration outcome (ground truth).
    observed_outcome: EvalOutcome,
}

impl EvaluationCase {
    /// Creates a new historical evaluation case.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        case_id: impl Into<String>,
        schema_version: DatasetSchemaVersion,
        repository: EvalRepositoryIdentity,
        base_revision: EvalRevision,
        branch_a_revision: EvalRevision,
        branch_b_revision: EvalRevision,
        merge_base: EvalRevision,
        observed_outcome: EvalOutcome,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            schema_version,
            repository,
            base_revision,
            branch_a_revision,
            branch_b_revision,
            merge_base,
            observed_outcome,
        }
    }

    /// The unique identifier for this evaluation scenario.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// The schema version of the case format.
    #[must_use]
    pub const fn schema_version(&self) -> &DatasetSchemaVersion {
        &self.schema_version
    }

    /// The repository the case applies to.
    #[must_use]
    pub const fn repository(&self) -> &EvalRepositoryIdentity {
        &self.repository
    }

    /// The base revision.
    #[must_use]
    pub const fn base_revision(&self) -> &EvalRevision {
        &self.base_revision
    }

    /// The branch A revision.
    #[must_use]
    pub const fn branch_a_revision(&self) -> &EvalRevision {
        &self.branch_a_revision
    }

    /// The branch B revision.
    #[must_use]
    pub const fn branch_b_revision(&self) -> &EvalRevision {
        &self.branch_b_revision
    }

    /// The three-way merge base revision.
    #[must_use]
    pub const fn merge_base(&self) -> &EvalRevision {
        &self.merge_base
    }

    /// The historical integration outcome (ground truth label).
    #[must_use]
    pub const fn observed_outcome(&self) -> &EvalOutcome {
        &self.observed_outcome
    }
}

/// Evaluation diagnostics reflecting issues encountered during pipeline execution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationDiagnostic {
    /// Execution succeeded normally.
    Success,
    /// Repository directory or Git environment was missing/unavailable.
    UnavailableRepository,
    /// Semantic pipeline failed (e.g., parsing/indexing errors).
    FailedAnalysis,
    /// The BCS engine properly ingested evidence but formally abstained.
    Abstained,
}

/// The structured result encapsulating predictions and ground truth side-by-side.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvaluationResult {
    case_id: String,
    diagnostic: EvaluationDiagnostic,
    /// The BCS assessment produced strictly from repository inputs.
    assessment: Option<BcsAssessment>,
    /// The original observed outcome isolated from the pipeline.
    observed_outcome: EvalOutcome,
}

impl EvaluationResult {
    /// Creates a complete evaluation result containing both predictions and observations.
    #[must_use]
    pub fn new(
        case_id: impl Into<String>,
        diagnostic: EvaluationDiagnostic,
        assessment: Option<BcsAssessment>,
        observed_outcome: EvalOutcome,
    ) -> Self {
        Self { case_id: case_id.into(), diagnostic, assessment, observed_outcome }
    }

    /// Returns the associated case ID.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns the diagnostic status.
    #[must_use]
    pub const fn diagnostic(&self) -> &EvaluationDiagnostic {
        &self.diagnostic
    }

    /// Returns the resulting BCS assessment, if execution succeeded.
    #[must_use]
    pub const fn assessment(&self) -> Option<&BcsAssessment> {
        self.assessment.as_ref()
    }

    /// Returns the independently observed integration outcome.
    #[must_use]
    pub const fn observed_outcome(&self) -> &EvalOutcome {
        &self.observed_outcome
    }
}
