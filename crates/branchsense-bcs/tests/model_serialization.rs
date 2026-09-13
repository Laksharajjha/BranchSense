//! Model serialization tests.
use branchsense_bcs::{BcsExplanation, BcsOrdinalBand};

#[test]
fn test_ordinal_band_serialization() {
    assert_eq!(serde_json::to_string(&BcsOrdinalBand::None).unwrap(), "\"none\"");
    assert_eq!(serde_json::to_string(&BcsOrdinalBand::Moderate).unwrap(), "\"moderate\"");
    assert_eq!(serde_json::to_string(&BcsOrdinalBand::Critical).unwrap(), "\"critical\"");
    assert_eq!(serde_json::to_string(&BcsOrdinalBand::Indeterminate).unwrap(), "\"indeterminate\"");
}

#[test]
fn test_ordinal_band_ordering() {
    assert!(BcsOrdinalBand::None < BcsOrdinalBand::Low);
    assert!(BcsOrdinalBand::Low < BcsOrdinalBand::Moderate);
    assert!(BcsOrdinalBand::Moderate < BcsOrdinalBand::High);
    assert!(BcsOrdinalBand::High < BcsOrdinalBand::Critical);
    assert!(BcsOrdinalBand::Critical < BcsOrdinalBand::Indeterminate);
}

#[test]
fn test_explanation_structure() {
    let explanation = BcsExplanation::new(vec!["A".to_string(), "B".to_string()]);
    assert_eq!(explanation.reasons(), &["A".to_string(), "B".to_string()]);
}
