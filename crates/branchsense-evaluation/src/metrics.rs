//! Descriptive metrics for evaluating BCS assessment against historical outcomes.

use std::collections::BTreeMap;

use branchsense_bcs::BcsOrdinalBand;

use crate::model::{EvaluationDiagnostic, EvaluationResult};

/// Aggregated metrics across an entire evaluation dataset run.
#[derive(Clone, Debug, Default)]
pub struct EvaluationMetrics {
    total_cases: usize,
    successful_cases: usize,
    abstained_cases: usize,
    failed_analysis: usize,
    unavailable_repository: usize,

    band_distribution: BTreeMap<BcsOrdinalBand, usize>,
    
    // Outcome tracking (True = issue occurred, False = clean, None = unknown)
    outcome_build_failure: usize,
    outcome_test_failure: usize,
    outcome_semantic_issue: usize,

    // Basic agreement tables: (BCS Band, Had Semantic Issue) -> Count
    agreement_semantic_issue: BTreeMap<(BcsOrdinalBand, bool), usize>,
}

impl EvaluationMetrics {
    /// Computes metrics from a sequence of results.
    #[must_use]
    pub fn compute(results: &[EvaluationResult]) -> Self {
        let mut metrics = Self::default();
        metrics.total_cases = results.len();

        for result in results {
            match result.diagnostic() {
                EvaluationDiagnostic::Success => {
                    metrics.successful_cases += 1;
                    if let Some(assessment) = result.assessment() {
                        *metrics.band_distribution.entry(assessment.band()).or_insert(0) += 1;
                        
                        // Agreement tracking (only if outcome is definitively known)
                        if let Some(had_issue) = result.observed_outcome().semantic_integration_issue {
                            *metrics.agreement_semantic_issue.entry((assessment.band(), had_issue)).or_insert(0) += 1;
                        }
                    }
                }
                EvaluationDiagnostic::Abstained => {
                    metrics.abstained_cases += 1;
                    *metrics.band_distribution.entry(BcsOrdinalBand::Indeterminate).or_insert(0) += 1;
                }
                EvaluationDiagnostic::FailedAnalysis => {
                    metrics.failed_analysis += 1;
                }
                EvaluationDiagnostic::UnavailableRepository => {
                    metrics.unavailable_repository += 1;
                }
            }

            // Outcome distributions
            if result.observed_outcome().build_failure == Some(true) {
                metrics.outcome_build_failure += 1;
            }
            if result.observed_outcome().test_failure == Some(true) {
                metrics.outcome_test_failure += 1;
            }
            if result.observed_outcome().semantic_integration_issue == Some(true) {
                metrics.outcome_semantic_issue += 1;
            }
        }

        metrics
    }

    /// Total number of cases processed.
    #[must_use]
    pub const fn total_cases(&self) -> usize {
        self.total_cases
    }

    /// Number of successfully analyzed cases (including abstentions).
    #[must_use]
    pub const fn successful_cases(&self) -> usize {
        self.successful_cases
    }

    /// Number of abstained cases.
    #[must_use]
    pub const fn abstained_cases(&self) -> usize {
        self.abstained_cases
    }

    /// Count of cases that failed analysis.
    #[must_use]
    pub const fn failed_analysis(&self) -> usize {
        self.failed_analysis
    }

    /// The frequency of each BCS band across successful runs.
    #[must_use]
    pub const fn band_distribution(&self) -> &BTreeMap<BcsOrdinalBand, usize> {
        &self.band_distribution
    }

    /// The basic agreement table for semantic issues.
    #[must_use]
    pub const fn agreement_semantic_issue(&self) -> &BTreeMap<(BcsOrdinalBand, bool), usize> {
        &self.agreement_semantic_issue
    }
}
