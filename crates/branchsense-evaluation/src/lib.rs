//! Evaluation dataset harness and analytical runner.
//!
//! Exclusively consumes immutable Git repositories and evaluates the BCS
//! engine accuracy against explicit historical labels (ground truth) without
//! leaking outcome knowledge into the scoring engine.

pub mod model;
pub mod loader;
