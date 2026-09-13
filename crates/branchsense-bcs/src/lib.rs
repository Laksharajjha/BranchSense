//! Deterministic Branch Collision Score (BCS) assessment and normalization.
//!
//! This crate aggregates semantic evidence from multiple subsystems (collision, impact, history, ownership)
//! and evaluates an ordinal, explainable collision score.

pub mod model;

pub use model::{BcsAssessment, BcsExplanation, BcsOrdinalBand};

pub mod normalization;

pub use normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
pub mod adapter;
pub mod ledger;
