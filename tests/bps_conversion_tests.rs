//! Tests for basis points (BPS) conversion and validation
//!
//! This module ensures correct conversion between percentage and basis points
//! and validates that the core algorithm uses the correct scaling.

use rangebar::fixed_point::FixedPoint;
use rangebar::range_bars::RangeBarProcessor;

#[test]
fn test_basis_points_scaling_constants() {
    // Verify that BASIS_POINTS_SCALE is correct
    assert_eq!(rangebar::fixed_point::BASIS_POINTS_SCALE, 10_000);
}

#[test]
fn test_compute_range_thresholds_standard_values() {
    let price = FixedPoint::from_str("100.0").unwrap();

    // Test 0.25% = 25 basis points
    let (upper, lower) = price.compute_range_thresholds(25);
    assert_eq!(upper.to_f64(), 100.25);
    assert_eq!(lower.to_f64(), 99.75);

    // Test 0.80% = 80 basis points (common rangebar threshold)
    let (upper, lower) = price.compute_range_thresholds(80);
    assert_eq!(upper.to_f64(), 100.80);
    assert_eq!(lower.to_f64(), 99.20);

    // Test 1.00% = 100 basis points
    let (upper, lower) = price.compute_range_thresholds(100);
    assert_eq!(upper.to_f64(), 101.0);
    assert_eq!(lower.to_f64(), 99.0);
}

#[test]
fn test_range_bar_processor_uses_correct_bps() {
    // Create processor with 25 BPS (0.25%)
    let mut processor = RangeBarProcessor::new(25);

    // Create test trades at exactly the threshold
    let trades = vec![
        rangebar::types::AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("100.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // This trade should trigger bar closure (0.25% above)
        rangebar::types::AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("100.25").unwrap(), // Exactly 25 BPS above
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
    ];

    let bars = processor.process_trades(&trades).unwrap();

    // Should produce exactly 1 bar
    assert_eq!(bars.len(), 1);

    // Verify the bar closed at the threshold
    let bar = &bars[0];
    assert_eq!(bar.open.to_f64(), 100.0);
    assert_eq!(bar.close.to_f64(), 100.25);
    assert_eq!(bar.high.to_f64(), 100.25);
    assert_eq!(bar.low.to_f64(), 100.0);
}

#[test]
fn test_percentage_to_bps_conversion_formulas() {
    // Test the conversion formulas used in the codebase

    // 0.25% = 25 basis points
    let pct_025 = 0.0025_f64;
    let bps_from_pct = (pct_025 * 10_000.0) as u32;
    assert_eq!(bps_from_pct, 25);

    // 0.80% = 80 basis points (common rangebar threshold)
    let pct_080 = 0.008_f64;
    let bps_from_pct = (pct_080 * 10_000.0) as u32;
    assert_eq!(bps_from_pct, 80);

    // 1.00% = 100 basis points
    let pct_100 = 0.01_f64;
    let bps_from_pct = (pct_100 * 10_000.0) as u32;
    assert_eq!(bps_from_pct, 100);

    // Reverse conversion: BPS to percentage
    let bps = 25_u32;
    let pct_from_bps = bps as f64 / 10_000.0;
    assert_eq!(pct_from_bps, 0.0025);
}

#[test]
fn test_api_conversion_formula() {
    // Test the corrected API conversion formula

    // User inputs 0.008 (0.8% as decimal) for testing conversion
    let threshold_decimal = 0.008_f64;

    // CORRECTED: Multiply by 10,000 not 1,000,000
    let threshold_bps = (threshold_decimal * 10_000.0) as u32;
    assert_eq!(threshold_bps, 80);

    // WRONG (the bug we fixed): Would have been 8000
    let wrong_bps = (threshold_decimal * 1_000_000.0) as u32;
    assert_eq!(wrong_bps, 8000); // This was the bug!
}

#[test]
fn test_edge_cases_bps_conversion() {
    // Test very small basis points
    let price = FixedPoint::from_str("50000.0").unwrap();
    let (upper, lower) = price.compute_range_thresholds(1); // 0.01%
    assert_eq!(upper.to_f64(), 50005.0);
    assert_eq!(lower.to_f64(), 49995.0);

    // Test large basis points (but still realistic)
    let (upper, lower) = price.compute_range_thresholds(1000); // 10%
    assert_eq!(upper.to_f64(), 55000.0);
    assert_eq!(lower.to_f64(), 45000.0);
}

#[test]
fn test_fixed_point_precision_with_bps() {
    // Test that BPS calculations maintain precision
    let price = FixedPoint::from_str("12345.67890123").unwrap(); // Max precision

    let (upper, lower) = price.compute_range_thresholds(25); // 0.25%

    // Should maintain precision in the calculation
    let expected_upper = 12345.67890123 * 1.0025;
    let expected_lower = 12345.67890123 * 0.9975;

    // Allow small floating point tolerance
    assert!((upper.to_f64() - expected_upper).abs() < 1e-6);
    assert!((lower.to_f64() - expected_lower).abs() < 1e-6);
}
