//! Debug module to reproduce and fix the 0.5% threshold algorithm bug

use crate::fixed_point::FixedPoint;
use crate::range_bars::RangeBarProcessor;
use crate::types::AggTrade;

/// Test the fix for the 0.5% threshold algorithm bug
pub fn test_algorithm_fix() {
    println!("✅ Testing algorithm fix for 0.5% threshold bug...");

    // Test the FIXED behavior with strict algorithm compliance
    test_fixed_algorithm_behavior();

    // Test backward compatibility with analysis mode
    test_analysis_mode_compatibility();
}

/// Test that the fixed algorithm only creates bars on threshold breach
fn test_fixed_algorithm_behavior() {
    println!("🧪 Testing FIXED algorithm (strict compliance)...");

    let mut processor = RangeBarProcessor::new(5000); // True 0.5%

    let trades = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("111441.5").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("111692.3").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("111201.6").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 3000,
            is_buyer_maker: true,
        },
        AggTrade {
            agg_trade_id: 4,
            price: FixedPoint::from_str("111499.9").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 4,
            last_trade_id: 4,
            timestamp: 4000,
            is_buyer_maker: false,
        },
    ];

    // Use the FIXED algorithm (strict compliance)
    let bars = processor.process_trades(&trades).unwrap();

    println!("   Fixed algorithm result: {} bars", bars.len());

    if bars.is_empty() {
        println!("   ✅ FIXED: No bars created (no threshold breach)");
        println!("   ✅ Algorithm integrity preserved");
    } else {
        println!(
            "   ❌ UNEXPECTED: {} bars created without breach",
            bars.len()
        );
        for (i, bar) in bars.iter().enumerate() {
            println!(
                "   Bar {}: O={} H={} L={} C={}",
                i + 1,
                bar.open,
                bar.high,
                bar.low,
                bar.close
            );
        }
    }

    // Test with analysis mode (includes incomplete bars)
    let bars_with_incomplete = processor.process_trades_with_incomplete(&trades).unwrap();
    println!(
        "   Analysis mode result: {} bars",
        bars_with_incomplete.len()
    );

    if bars_with_incomplete.len() == 1 {
        let bar = &bars_with_incomplete[0];
        println!("   ✅ Analysis mode includes incomplete bar for study");
        println!(
            "   Bar: O={} H={} L={} C={}",
            bar.open, bar.high, bar.low, bar.close
        );
    }
}

/// Test compatibility with analysis workflows that may need incomplete bars
fn test_analysis_mode_compatibility() {
    println!("\n🔬 Testing analysis mode compatibility...");

    let mut processor = RangeBarProcessor::new(8000); // 0.8%

    let trades = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // Small movement within threshold
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("50200.0").unwrap(), // +0.4%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
    ];

    let strict_bars = processor.process_trades(&trades).unwrap();
    let analysis_bars = processor.process_trades_with_incomplete(&trades).unwrap();

    println!(
        "   Strict mode: {} bars (algorithm compliant)",
        strict_bars.len()
    );
    println!(
        "   Analysis mode: {} bars (includes incomplete)",
        analysis_bars.len()
    );

    assert_eq!(
        strict_bars.len(),
        0,
        "Strict mode should not include incomplete bars"
    );
    assert_eq!(
        analysis_bars.len(),
        1,
        "Analysis mode should include incomplete bar"
    );

    println!("   ✅ Both modes work as expected");
}

/// Test case to reproduce the exact bug found in the audit
pub fn reproduce_audit_bug() {
    println!("🐛 Reproducing 0.5% threshold algorithm bug from audit...");

    // The audit findings show:
    // Bar: O=111441.5 H=111692.3 L=111201.6 C=111499.9
    // Expected Upper Breach: 111998.71 (suggests 5000 basis points)
    // Expected Lower Breach: 110884.29 (suggests 5000 basis points)

    // But the result was "Bar closed without any breach occurring!"

    println!("🧪 Testing TRUE 0.5% threshold (5000 basis points)...");
    test_with_audit_data(5000);

    println!("\n🧪 Testing MISUNDERSTOOD 0.05% threshold (500 basis points)...");
    test_with_audit_data(500);

    println!("\n🧪 Testing end-of-data scenario...");
    test_end_of_data_scenario();
}

fn test_with_audit_data(threshold_bps: u32) {
    let mut processor = RangeBarProcessor::new(threshold_bps);

    let trades = vec![
        // Opening trade
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("111441.5").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // Trade that creates the high
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("111692.3").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
        // Trade that creates the low
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("111201.6").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 3000,
            is_buyer_maker: true,
        },
        // Final trade that becomes close
        AggTrade {
            agg_trade_id: 4,
            price: FixedPoint::from_str("111499.9").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 4,
            last_trade_id: 4,
            timestamp: 4000,
            is_buyer_maker: false,
        },
    ];

    // Calculate thresholds
    let open_price = FixedPoint::from_str("111441.5").unwrap();
    let (upper_threshold, lower_threshold) = open_price.compute_range_thresholds(threshold_bps);
    let threshold_pct = threshold_bps as f64 / 10000.0;

    println!(
        "   Threshold: {} bps ({:.3}%)",
        threshold_bps, threshold_pct
    );
    println!("   Upper threshold: {}", upper_threshold);
    println!("   Lower threshold: {}", lower_threshold);

    // Check if actual prices breach thresholds
    let high = FixedPoint::from_str("111692.3").unwrap();
    let low = FixedPoint::from_str("111201.6").unwrap();

    let high_breaches_upper = high >= upper_threshold;
    let high_breaches_lower = high <= lower_threshold;
    let low_breaches_upper = low >= upper_threshold;
    let low_breaches_lower = low <= lower_threshold;

    println!(
        "   High (111692.3): breaches upper={}, breaches lower={}",
        high_breaches_upper, high_breaches_lower
    );
    println!(
        "   Low (111201.6): breaches upper={}, breaches lower={}",
        low_breaches_upper, low_breaches_lower
    );

    // Process trades
    let bars = processor.process_trades(&trades).unwrap();
    println!("   Result: {} bars created", bars.len());

    if !bars.is_empty() {
        for (i, bar) in bars.iter().enumerate() {
            println!(
                "   Bar {}: O={} H={} L={} C={}",
                i + 1,
                bar.open,
                bar.high,
                bar.low,
                bar.close
            );

            // Check if this bar violates the algorithm
            let bar_upper = bar.open.compute_range_thresholds(threshold_bps).0;
            let bar_lower = bar.open.compute_range_thresholds(threshold_bps).1;

            let high_violation = bar.high < bar_upper && bar.low > bar_lower;
            if high_violation {
                println!("   ❌ ALGORITHM VIOLATION: Bar closed without breach!");
                println!(
                    "      Expected upper: {}, actual high: {}",
                    bar_upper, bar.high
                );
                println!(
                    "      Expected lower: {}, actual low: {}",
                    bar_lower, bar.low
                );
            }
        }
    }
}

fn test_end_of_data_scenario() {
    let mut processor = RangeBarProcessor::new(5000); // True 0.5%

    // Create scenario where data ends before breach
    let trades = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("111441.5").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // Small price movements that don't breach 0.5% threshold
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("111500.0").unwrap(), // +0.052%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("111400.0").unwrap(), // -0.037%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 3000,
            is_buyer_maker: true,
        },
        AggTrade {
            agg_trade_id: 4,
            price: FixedPoint::from_str("111499.9").unwrap(), // +0.052%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 4,
            last_trade_id: 4,
            timestamp: 4000,
            is_buyer_maker: false,
        },
    ];

    let open_price = FixedPoint::from_str("111441.5").unwrap();
    let (upper_threshold, lower_threshold) = open_price.compute_range_thresholds(5000);

    println!("   Scenario: Small movements within 0.5% threshold");
    println!("   Upper threshold: {} (breach needed)", upper_threshold);
    println!("   Lower threshold: {} (breach needed)", lower_threshold);

    let bars = processor.process_trades(&trades).unwrap();

    if bars.len() == 1 {
        let bar = &bars[0];
        println!("   Result: 1 bar (end-of-data closure)");
        println!(
            "   Bar: O={} H={} L={} C={}",
            bar.open, bar.high, bar.low, bar.close
        );

        // This is the expected behavior - bar closes due to end of data
        let high_breach = bar.high >= upper_threshold;
        let low_breach = bar.low <= lower_threshold;

        if !high_breach && !low_breach {
            println!("   ✅ EXPECTED: Bar closed due to end-of-data, not breach");
            println!("   ✅ This is normal behavior, not a bug");
        }
    } else {
        println!("   Unexpected result: {} bars", bars.len());
    }
}

/// Test very small thresholds to identify precision issues
pub fn test_small_thresholds() {
    println!("\n🔬 Testing small threshold precision...");

    let price = FixedPoint::from_str("111441.5").unwrap();
    let small_thresholds = [10, 50, 100, 500, 1000, 5000]; // 0.001% to 0.5%

    for &threshold_bps in &small_thresholds {
        let (upper, lower) = price.compute_range_thresholds(threshold_bps);
        let threshold_pct = threshold_bps as f64 / 10000.0; // Convert to percentage

        println!(
            "   {} bps ({:.3}%): upper={}, lower={}, delta={}",
            threshold_bps,
            threshold_pct,
            upper,
            lower,
            upper.0 - price.0
        );
    }
}

/// Test the exact scenario from the audit to understand the discrepancy
pub fn analyze_audit_discrepancy() {
    println!("\n🔍 Analyzing audit discrepancy...");

    // The audit said:
    // "Expected Upper Breach: 111998.71"
    // "Expected Lower Breach: 110884.29"
    // For open price 111441.5

    let open_price = FixedPoint::from_str("111441.5").unwrap();

    // Calculate what threshold would produce these exact values
    let expected_upper = FixedPoint::from_str("111998.71").unwrap();
    let expected_lower = FixedPoint::from_str("110884.29").unwrap();

    let upper_diff = expected_upper.0 - open_price.0;
    let lower_diff = open_price.0 - expected_lower.0;

    println!("   Open price: {}", open_price);
    println!(
        "   Expected upper: {} (diff: {})",
        expected_upper, upper_diff
    );
    println!(
        "   Expected lower: {} (diff: {})",
        expected_lower, lower_diff
    );

    // Calculate implied threshold
    let implied_threshold_upper = (upper_diff as f64 / open_price.0 as f64) * 10000.0;
    let implied_threshold_lower = (lower_diff as f64 / open_price.0 as f64) * 10000.0;

    println!(
        "   Implied threshold from upper: {:.1} bps ({:.3}%)",
        implied_threshold_upper,
        implied_threshold_upper / 100.0
    );
    println!(
        "   Implied threshold from lower: {:.1} bps ({:.3}%)",
        implied_threshold_lower,
        implied_threshold_lower / 100.0
    );

    // Test with our calculated thresholds to see if we match
    let test_threshold = 5000; // 0.5%
    let (our_upper, our_lower) = open_price.compute_range_thresholds(test_threshold);

    println!(
        "   Our calculation (5000 bps): upper={}, lower={}",
        our_upper, our_lower
    );

    let upper_match = (our_upper.0 - expected_upper.0).abs() < 1000; // Within 0.00001
    let lower_match = (our_lower.0 - expected_lower.0).abs() < 1000;

    if upper_match && lower_match {
        println!("   ✅ Our calculation matches audit expectations");
    } else {
        println!("   ❌ Calculation mismatch - investigating precision...");
    }
}

/// Adversarial testing with very small thresholds (0.25%, 0.3%)
pub fn test_small_threshold_adversarial() {
    println!("🎯 Adversarial testing with very small thresholds...");

    // Test both 0.25% (2500 basis points) and 0.3% (3000 basis points)
    // Note: This system uses threshold_bps / 1,000,000 = percentage
    // So 2500 = 0.25%, 3000 = 0.3% (not standard basis point definition)
    let test_thresholds = [(2500, "0.25%"), (3000, "0.3%")];

    for (threshold_bps, threshold_desc) in test_thresholds {
        println!(
            "\n🧪 Testing {} threshold ({} basis points)...",
            threshold_desc, threshold_bps
        );
        test_specific_small_threshold(threshold_bps, threshold_desc);
    }

    // Test extreme scenarios with these thresholds
    test_extreme_scenarios_small_thresholds();
}

fn test_specific_small_threshold(threshold_bps: u32, threshold_desc: &str) {
    let mut processor = RangeBarProcessor::new(threshold_bps);

    // Create trades that should NOT breach the small threshold
    let trades_no_breach = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("100000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // Small movement within threshold
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("100200.0").unwrap(), // +0.2%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("99900.0").unwrap(), // -0.1%
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 3000,
            is_buyer_maker: true,
        },
    ];

    // Test strict algorithm compliance
    let bars_strict = processor.process_trades(&trades_no_breach).unwrap();
    println!(
        "   Strict mode ({}): {} bars created",
        threshold_desc,
        bars_strict.len()
    );

    if bars_strict.is_empty() {
        println!("   ✅ CORRECT: No bars created without breach");
    } else {
        println!(
            "   ❌ VIOLATION: {} bars created without breach!",
            bars_strict.len()
        );
        for (i, bar) in bars_strict.iter().enumerate() {
            println!(
                "      Bar {}: O={} H={} L={} C={}",
                i + 1,
                bar.open,
                bar.high,
                bar.low,
                bar.close
            );
        }
    }

    // Test analysis mode
    let bars_analysis = processor
        .process_trades_with_incomplete(&trades_no_breach)
        .unwrap();
    println!(
        "   Analysis mode ({}): {} bars created",
        threshold_desc,
        bars_analysis.len()
    );

    if bars_analysis.len() == 1 {
        println!("   ✅ CORRECT: Analysis mode includes incomplete bar");
    } else {
        println!(
            "   ❌ UNEXPECTED: Analysis mode created {} bars",
            bars_analysis.len()
        );
    }

    // Now test with trades that SHOULD breach the threshold
    let threshold_fraction = threshold_bps as f64 / 10000.0;
    let breach_price = 100000.0 * (1.0 + threshold_fraction + 0.01); // Slightly above threshold

    println!(
        "   Testing with breach price: {:.2} (threshold: {:.3}%)",
        breach_price,
        threshold_fraction * 100.0
    );

    let trades_with_breach = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("100000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str(&format!("{:.2}", breach_price)).unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
    ];

    let bars_with_breach = processor.process_trades(&trades_with_breach).unwrap();
    println!("   With breach: {} bars created", bars_with_breach.len());

    if bars_with_breach.len() == 1 {
        println!("   ✅ CORRECT: Breach creates exactly 1 bar");
        let bar = &bars_with_breach[0];
        println!(
            "      Bar: O={} H={} L={} C={}",
            bar.open, bar.high, bar.low, bar.close
        );
    } else {
        println!(
            "   ❌ UNEXPECTED: Breach created {} bars",
            bars_with_breach.len()
        );
    }
}

fn test_extreme_scenarios_small_thresholds() {
    println!("\n🔬 Testing extreme scenarios with small thresholds...");

    // Test with 0.1% threshold (1000 basis points in this system)
    let mut processor = RangeBarProcessor::new(1000);

    // Create trades with very small price movements
    let tiny_movements = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
            is_buyer_maker: false,
        },
        // 0.05% movement (half the threshold)
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("50025.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
            is_buyer_maker: false,
        },
        // Another 0.05% movement (still within threshold)
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("50049.99").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 3000,
            is_buyer_maker: false,
        },
    ];

    let bars_tiny = processor.process_trades(&tiny_movements).unwrap();
    println!(
        "   0.1% threshold with tiny movements: {} bars",
        bars_tiny.len()
    );

    if bars_tiny.is_empty() {
        println!("   ✅ EXCELLENT: Tiny movements within threshold don't create bars");
    } else {
        println!(
            "   ❌ ISSUE: Tiny movements created {} bars",
            bars_tiny.len()
        );
    }

    // Test precision boundaries
    test_precision_boundaries();
}

fn test_precision_boundaries() {
    println!("\n🎯 Testing precision boundaries...");

    let price = FixedPoint::from_str("111441.5").unwrap();
    let critical_thresholds = [1000, 2500, 3000, 5000]; // 0.1%, 0.25%, 0.3%, 0.5%

    for &threshold_bps in &critical_thresholds {
        let (upper, lower) = price.compute_range_thresholds(threshold_bps);
        let threshold_pct = threshold_bps as f64 / 10000.0;

        println!(
            "   {} bps ({:.3}%): range [{}, {}]",
            threshold_bps, threshold_pct, lower, upper
        );

        // Calculate the actual delta in fixed-point
        let delta = upper.0 - price.0;
        let delta_f64 = delta as f64 / 100_000_000.0; // Convert to actual price difference

        println!(
            "      Price delta: {:.8} (fixed-point: {})",
            delta_f64, delta
        );

        // Verify precision is sufficient
        if delta < 1000 {
            // Less than 0.00001 precision
            println!("      ⚠️ WARNING: Very small delta may cause precision issues");
        } else {
            println!("      ✅ GOOD: Sufficient precision for threshold");
        }
    }
}

/// Debug the specific threshold calculation issue
pub fn debug_threshold_calculation_issue() {
    println!("🐛 Debugging threshold calculation issue...");

    let open_price = FixedPoint::from_str("100000.0").unwrap();
    let test_price = FixedPoint::from_str("100200.0").unwrap(); // +0.2%
    let threshold_bps = 250; // 0.25%

    println!("   Open price: {}", open_price);
    println!("   Test price: {} (+0.2%)", test_price);
    println!("   Threshold: {} bps (0.25%)", threshold_bps);

    let (upper_threshold, lower_threshold) = open_price.compute_range_thresholds(threshold_bps);

    println!("   Calculated upper threshold: {}", upper_threshold);
    println!("   Calculated lower threshold: {}", lower_threshold);

    // Check if test price breaches thresholds
    let upper_breach = test_price >= upper_threshold;
    let lower_breach = test_price <= lower_threshold;

    println!(
        "   Does {} >= {}? {}",
        test_price, upper_threshold, upper_breach
    );
    println!(
        "   Does {} <= {}? {}",
        test_price, lower_threshold, lower_breach
    );

    let is_breach = upper_breach || lower_breach;
    println!("   Overall breach: {}", is_breach);

    // Verify our manual calculation
    let expected_upper = 100000.0 * 1.0025; // Should be 100250
    println!("   Expected upper (manual): {}", expected_upper);

    // Check fixed-point calculation details
    let basis_points_scale = 1_000_000;
    let delta = (open_price.0 as i128 * threshold_bps as i128) / basis_points_scale as i128;
    println!(
        "   Delta calculation: ({} * {}) / {} = {}",
        open_price.0, threshold_bps, basis_points_scale, delta
    );

    let manual_upper = open_price.0 + delta as i64;
    println!(
        "   Manual upper fixed-point: {} + {} = {}",
        open_price.0, delta, manual_upper
    );
    println!("   Library upper fixed-point: {}", upper_threshold.0);

    if manual_upper != upper_threshold.0 {
        println!("   ❌ MISMATCH: Manual and library calculations differ!");
    } else {
        println!("   ✅ MATCH: Calculations are consistent");
    }

    // Now test the actual breach detection
    println!("\n🔍 Testing breach detection directly...");

    use crate::types::RangeBar;

    let dummy_trade = crate::types::AggTrade {
        agg_trade_id: 1,
        price: open_price,
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: 1,
        last_trade_id: 1,
        timestamp: 1000,
        is_buyer_maker: false,
    };

    let bar = RangeBar::new(&dummy_trade);
    let breach_result = bar.is_breach(test_price, upper_threshold, lower_threshold);

    println!(
        "   RangeBar::is_breach({}, {}, {}) = {}",
        test_price, upper_threshold, lower_threshold, breach_result
    );

    if breach_result && !is_breach {
        println!("   ❌ INCONSISTENCY: RangeBar::is_breach differs from manual calculation!");
    } else if !breach_result && is_breach {
        println!("   ❌ INCONSISTENCY: Manual calculation differs from RangeBar::is_breach!");
    } else {
        println!("   ✅ CONSISTENT: Both methods agree");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_bug_reproduction() {
        reproduce_audit_bug();
        test_small_thresholds();
        analyze_audit_discrepancy();
    }

    #[test]
    fn test_algorithm_bug_fix() {
        test_algorithm_fix();
    }

    #[test]
    fn test_adversarial_small_thresholds() {
        test_small_threshold_adversarial();
    }

    #[test]
    fn test_debug_threshold_calculation() {
        debug_threshold_calculation_issue();
    }

    #[test]
    fn test_threshold_precision_issue() {
        // Test case: very small threshold might cause integer truncation
        let price = FixedPoint::from_str("111441.5").unwrap();
        let (upper, _lower) = price.compute_range_thresholds(500); // 0.05%

        // Expected: 111441.5 * 0.0005 = 55.7207 delta
        let expected_delta = 5572075; // In fixed point (55.7207575 * 1e8)
        let actual_delta = upper.0 - price.0;

        println!(
            "Expected delta: {}, Actual delta: {}",
            expected_delta, actual_delta
        );

        // Check for precision loss
        let precision_loss = expected_delta - actual_delta;
        if precision_loss.abs() > 1000 {
            // More than 0.00001 precision loss
            println!("❌ Significant precision loss detected: {}", precision_loss);
        }
    }
}
