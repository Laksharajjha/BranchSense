#![allow(clippy::manual_let_else)]
//! Deterministic pipeline runner for historical evaluation cases.

use std::path::Path;

use branchsense_bcs::adapter::{
    normalize_collision, normalize_history, normalize_impact, normalize_overlap,
    normalize_ownership,
};
use branchsense_bcs::ledger::BcsEvidenceAggregator;
use branchsense_bcs::score::BcsEngine;
use branchsense_collision::CollisionAnalyzer;
use branchsense_diff::SemanticDiffer;
use branchsense_git::{GitRepository, GitSnapshotIndexer};
use branchsense_history::{HistoricalAnalyzer, HistoricalOptions};
use branchsense_impact::ImpactAnalyzer;
use branchsense_overlap::SemanticOverlapAnalyzer;
use branchsense_ownership::{ResponsibilityAnalyzer, ResponsibilityOptions};

use crate::model::{EvaluationCase, EvaluationDiagnostic, EvaluationResult};

/// Runs a single evaluation case strictly without modifying the repository.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn run_evaluation_case(case: &EvaluationCase, repo_path: &Path) -> EvaluationResult {
    let repository = match GitRepository::discover(repo_path) {
        Ok(repo) => repo,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::UnavailableRepository,
                None,
                case.observed_outcome().clone(),
            );
        }
    };

    let base = case.base_revision().hash();
    let head_a = case.branch_a_revision().hash();
    let head_b = case.branch_b_revision().hash();

    let resolved_base = match repository.resolve(base) {
        Ok(r) => r,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };
    let resolved_a = match repository.resolve(head_a) {
        Ok(r) => r,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };
    let resolved_b = match repository.resolve(head_b) {
        Ok(r) => r,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };

    let indexer = GitSnapshotIndexer::default();
    let base_snapshot = match indexer.index_revision(&repository, &resolved_base, None) {
        Ok(s) => s,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };
    let snapshot_a = match indexer.index_revision(&repository, &resolved_a, None) {
        Ok(s) => s,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };
    let snapshot_b = match indexer.index_revision(&repository, &resolved_b, None) {
        Ok(s) => s,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };

    let differ = SemanticDiffer::new();
    let diff_a = differ.diff_git(&base_snapshot, &snapshot_a);
    let diff_b = differ.diff_git(&base_snapshot, &snapshot_b);

    let impact_analyzer = ImpactAnalyzer::new();
    let impact_a =
        impact_analyzer.analyze(&diff_a, base_snapshot.semantic(), snapshot_a.semantic());
    let impact_b =
        impact_analyzer.analyze(&diff_b, base_snapshot.semantic(), snapshot_b.semantic());

    let overlaps = SemanticOverlapAnalyzer::new().analyze(&diff_a, &impact_a, &diff_b, &impact_b);
    let assessment = CollisionAnalyzer::new().analyze(&overlaps);

    let history = match HistoricalAnalyzer::new().analyze(
        &repository,
        &resolved_base,
        HistoricalOptions::new(100),
    ) {
        Ok(h) => h,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };

    let ownership = match ResponsibilityAnalyzer::new().analyze(
        &repository,
        &resolved_base,
        ResponsibilityOptions::new(100),
    ) {
        Ok(o) => o,
        Err(_) => {
            return EvaluationResult::new(
                case.case_id(),
                EvaluationDiagnostic::FailedAnalysis,
                None,
                case.observed_outcome().clone(),
            );
        }
    };

    let mut aggregator = BcsEvidenceAggregator::new();
    for ev in normalize_collision(&assessment) {
        aggregator.add(ev);
    }
    for ev in normalize_impact(&impact_a) {
        aggregator.add(ev);
    }
    for ev in normalize_impact(&impact_b) {
        aggregator.add(ev);
    }
    for ev in normalize_overlap(&overlaps) {
        aggregator.add(ev);
    }
    for ev in normalize_history(&history) {
        aggregator.add(ev);
    }
    for ev in normalize_ownership(&ownership) {
        aggregator.add(ev);
    }

    let engine = BcsEngine::new();
    let bcs_result = engine.assess(&aggregator);

    let diagnostic = if bcs_result.band() == branchsense_bcs::BcsOrdinalBand::Indeterminate {
        EvaluationDiagnostic::Abstained
    } else {
        EvaluationDiagnostic::Success
    };

    EvaluationResult::new(
        case.case_id(),
        diagnostic,
        Some(bcs_result),
        case.observed_outcome().clone(),
    )
}

/// Runs a sequence of cases deterministically.
#[must_use]
pub fn run_dataset(cases: &[EvaluationCase], repo_path: &Path) -> Vec<EvaluationResult> {
    cases.iter().map(|case| run_evaluation_case(case, repo_path)).collect()
}
