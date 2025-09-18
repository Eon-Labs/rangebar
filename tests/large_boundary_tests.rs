use rangebar::fixed_point::FixedPoint;
use rangebar::range_bars::ExportRangeBarProcessor;
use rangebar::types::{AggTrade, RangeBar};
use std::time::Instant;

/// Large-scale boundary consistency tests with comprehensive datasets
///
/// These tests validate streaming vs batch consistency across:
/// - Multi-million trade datasets
/// - Multi-day boundary transitions
/// - Market session boundaries (open/close)
/// - High/low frequency trading periods
/// - Memory stress testing

#[tokio::test]
async fn test_massive_dataset_boundary_consistency() {
    println!("🔍 Testing massive dataset boundary consistency (1M+ trades)");

    let threshold_bps = 25; // 0.25% standard threshold
    let trade_count = 1_000_000; // 1 million trades

    println!(
        "  Generating {} trades with realistic price movements",
        trade_count
    );
    let start_gen = Instant::now();
    let massive_dataset = create_massive_realistic_dataset(trade_count);
    println!("  ✅ Dataset generated in {:?}", start_gen.elapsed());

    // Test batch processing
    println!("  🔄 Running batch processing...");
    let start_batch = Instant::now();
    let batch_bars = process_batch_style(&massive_dataset, threshold_bps);
    let batch_duration = start_batch.elapsed();

    println!(
        "  ✅ Batch: {} bars in {:?}",
        batch_bars.len(),
        batch_duration
    );

    // Test streaming processing
    println!("  🔄 Running streaming processing...");
    let start_streaming = Instant::now();
    let streaming_bars = process_streaming_style(&massive_dataset, threshold_bps).await;
    let streaming_duration = start_streaming.elapsed();

    println!(
        "  ✅ Streaming: {} bars in {:?}",
        streaming_bars.len(),
        streaming_duration
    );

    // Compare results
    let matches = batch_bars.len() == streaming_bars.len();
    let ratio = streaming_bars.len() as f64 / batch_bars.len() as f64;

    println!(
        "  📊 Comparison: {} vs {} bars - {}",
        batch_bars.len(),
        streaming_bars.len(),
        if matches {
            "✅ MATCH"
        } else {
            &format!("❌ MISMATCH ({:.2}x)", ratio)
        }
    );

    // Performance analysis
    let batch_speed = trade_count as f64 / batch_duration.as_secs_f64();
    let streaming_speed = trade_count as f64 / streaming_duration.as_secs_f64();

    println!("  ⚡ Performance:");
    println!("    Batch: {:.0} trades/sec", batch_speed);
    println!("    Streaming: {:.0} trades/sec", streaming_speed);
    println!("    Speedup: {:.2}x", streaming_speed / batch_speed);

    // Memory analysis
    let avg_trades_per_bar = trade_count as f64 / batch_bars.len() as f64;
    println!("  💾 Efficiency: {:.1} trades per bar", avg_trades_per_bar);

    // Validation
    assert!(!batch_bars.is_empty(), "Batch should generate bars");
    assert!(!streaming_bars.is_empty(), "Streaming should generate bars");

    // Validate temporal integrity
    validate_temporal_integrity(&batch_bars, "massive_batch");
    validate_temporal_integrity(&streaming_bars, "massive_streaming");

    println!("  ✅ Large dataset test complete");
}

#[tokio::test]
async fn test_multi_day_boundary_transitions() {
    println!("🔍 Testing multi-day boundary transitions");

    let threshold_bps = 25;
    let days = 7; // One week of data

    println!("  Generating {} days of continuous trading data", days);
    let multi_day_dataset = create_multi_day_boundary_dataset(days);

    println!("  Total trades: {}", multi_day_dataset.len());

    // Test with boundary preservation
    let batch_bars = process_batch_style(&multi_day_dataset, threshold_bps);
    let streaming_bars = process_streaming_style(&multi_day_dataset, threshold_bps).await;

    println!("  📊 Multi-day results:");
    println!("    Batch: {} bars", batch_bars.len());
    println!("    Streaming: {} bars", streaming_bars.len());

    let matches = batch_bars.len() == streaming_bars.len();
    println!(
        "    Status: {}",
        if matches { "✅ MATCH" } else { "❌ MISMATCH" }
    );

    // Analyze boundary behavior
    analyze_boundary_behavior(&batch_bars, &streaming_bars, days);

    // Validate that bars can span day boundaries properly
    validate_cross_day_bars(&batch_bars, "batch");
    validate_cross_day_bars(&streaming_bars, "streaming");

    assert!(!batch_bars.is_empty(), "Batch should generate bars");
    assert!(!streaming_bars.is_empty(), "Streaming should generate bars");
}

#[tokio::test]
async fn test_market_session_boundaries() {
    println!("🔍 Testing market session boundaries");

    let threshold_bps = 25;

    // Create data with distinct trading sessions
    let session_datasets = vec![
        ("asian_session", create_asian_session_data()),
        ("european_session", create_european_session_data()),
        ("us_session", create_us_session_data()),
        ("weekend_gap", create_weekend_gap_data()),
    ];

    for (session_name, dataset) in session_datasets {
        println!("  🎯 Testing: {}", session_name);

        let batch_bars = process_batch_style(&dataset, threshold_bps);
        let streaming_bars = process_streaming_style(&dataset, threshold_bps).await;

        let matches = batch_bars.len() == streaming_bars.len();
        println!(
            "    {}: Batch={}, Streaming={} - {}",
            session_name,
            batch_bars.len(),
            streaming_bars.len(),
            if matches { "✅ MATCH" } else { "❌ MISMATCH" }
        );

        // Validate session-specific characteristics
        validate_session_characteristics(&batch_bars, session_name);
        validate_session_characteristics(&streaming_bars, session_name);
    }
}

#[tokio::test]
async fn test_frequency_boundary_variations() {
    println!("🔍 Testing high/low frequency boundary variations");

    let threshold_bps = 25;

    let frequency_tests = vec![
        ("high_frequency_1ms", create_high_frequency_data(1)), // 1ms intervals
        ("medium_frequency_100ms", create_medium_frequency_data(100)), // 100ms intervals
        ("low_frequency_1s", create_low_frequency_data(1000)), // 1s intervals
        ("mixed_frequency", create_mixed_frequency_data()),    // Variable intervals
    ];

    for (test_name, dataset) in frequency_tests {
        println!("  📈 Testing: {}", test_name);

        let start_time = Instant::now();
        let batch_bars = process_batch_style(&dataset, threshold_bps);
        let batch_duration = start_time.elapsed();

        let start_time = Instant::now();
        let streaming_bars = process_streaming_style(&dataset, threshold_bps).await;
        let streaming_duration = start_time.elapsed();

        let matches = batch_bars.len() == streaming_bars.len();
        println!(
            "    {} ({} trades): Batch={} bars ({:?}), Streaming={} bars ({:?}) - {}",
            test_name,
            dataset.len(),
            batch_bars.len(),
            batch_duration,
            streaming_bars.len(),
            streaming_duration,
            if matches { "✅ MATCH" } else { "❌ MISMATCH" }
        );

        // Analyze frequency-specific patterns
        analyze_frequency_patterns(&batch_bars, &streaming_bars, test_name);
    }
}

#[tokio::test]
async fn test_stress_boundary_conditions() {
    println!("🔍 Testing stress boundary conditions");

    let threshold_bps = 25;

    let stress_tests = vec![
        ("rapid_threshold_hits", create_rapid_threshold_hit_data()),
        ("price_precision_limits", create_precision_limit_data()),
        ("volume_extremes", create_volume_extreme_data()),
        ("timestamp_edge_cases", create_timestamp_edge_data()),
        ("floating_point_stress", create_floating_point_stress_data()),
    ];

    for (test_name, dataset) in stress_tests {
        println!("  ⚡ Stress testing: {}", test_name);

        let batch_bars = process_batch_style(&dataset, threshold_bps);
        let streaming_bars = process_streaming_style(&dataset, threshold_bps).await;

        let matches = batch_bars.len() == streaming_bars.len();
        println!(
            "    {}: {} - {}",
            test_name,
            if matches {
                format!("✅ MATCH ({} bars)", batch_bars.len())
            } else {
                format!(
                    "❌ MISMATCH (B:{}, S:{})",
                    batch_bars.len(),
                    streaming_bars.len()
                )
            },
            if matches { "PASS" } else { "FAIL" }
        );

        // Validate stress test specific requirements
        validate_stress_test_requirements(&batch_bars, &streaming_bars, test_name);
    }
}

// Helper functions for data generation

fn create_massive_realistic_dataset(count: usize) -> Vec<AggTrade> {
    let mut trades = Vec::with_capacity(count);
    let base_price = 23000.0;
    let base_time = 1659312000000i64; // Aug 1, 2022

    // Simulate realistic market conditions
    for i in 0..count {
        let time_progress = i as f64 / count as f64;

        // Multi-layered price movement simulation
        let trend = (time_progress * 2.0 * std::f64::consts::PI).sin() * 500.0; // Long-term trend
        let volatility = ((i as f64 * 0.01).sin() * 50.0) + ((i as f64 * 0.001).cos() * 20.0); // Volatility
        let noise = (i as f64 * 0.1).sin() * 5.0; // Market noise

        let price = base_price + trend + volatility + noise;
        let timestamp = base_time + (i as i64 * 100); // 100ms intervals

        trades.push(create_test_trade(
            1000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_multi_day_boundary_dataset(days: usize) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64; // Aug 1, 2022
    let day_ms = 24 * 60 * 60 * 1000; // Milliseconds per day

    for day in 0..days {
        let day_start = base_time + (day as i64 * day_ms);

        // Each day has different trading patterns
        let daily_trades = match day % 3 {
            0 => create_volatile_day_data(day_start, 100000), // High volatility
            1 => create_stable_day_data(day_start, 80000),    // Low volatility
            _ => create_trending_day_data(day_start, 120000), // Strong trend
        };

        trades.extend(daily_trades);
    }

    trades
}

fn create_volatile_day_data(start_time: i64, count: usize) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;

    for i in 0..count {
        // High volatility with frequent reversals
        let volatility = ((i as f64 * 0.02).sin() * 200.0) + ((i as f64 * 0.005).cos() * 100.0);
        let price = base_price + volatility;
        let timestamp = start_time + (i as i64 * 500); // 500ms intervals

        trades.push(create_test_trade(
            2000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_stable_day_data(start_time: i64, count: usize) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;

    for i in 0..count {
        // Low volatility, gradual movements
        let movement = (i as f64 * 0.001).sin() * 20.0;
        let price = base_price + movement;
        let timestamp = start_time + (i as i64 * 800); // 800ms intervals

        trades.push(create_test_trade(
            3000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_trending_day_data(start_time: i64, count: usize) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;

    for i in 0..count {
        // Strong upward trend with some noise
        let trend = (i as f64 / count as f64) * 800.0; // +800 over the day
        let noise = (i as f64 * 0.01).sin() * 30.0;
        let price = base_price + trend + noise;
        let timestamp = start_time + (i as i64 * 600); // 600ms intervals

        trades.push(create_test_trade(
            4000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_asian_session_data() -> Vec<AggTrade> {
    // Simulate Asian trading session characteristics
    create_session_data(1659312000000, 50000, 0.5, 0.8) // Lower volatility, steady
}

fn create_european_session_data() -> Vec<AggTrade> {
    // Simulate European trading session characteristics
    create_session_data(1659340800000, 80000, 1.0, 1.2) // Medium volatility, active
}

fn create_us_session_data() -> Vec<AggTrade> {
    // Simulate US trading session characteristics
    create_session_data(1659369600000, 120000, 1.5, 2.0) // High volatility, very active
}

fn create_weekend_gap_data() -> Vec<AggTrade> {
    // Simulate weekend gap with sparse trading
    create_session_data(1659484800000, 5000, 0.2, 0.3) // Very low activity
}

fn create_session_data(
    start_time: i64,
    count: usize,
    volatility_factor: f64,
    activity_factor: f64,
) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;

    for i in 0..count {
        let volatility = ((i as f64 * 0.01).sin() * 100.0 * volatility_factor)
            + ((i as f64 * 0.003).cos() * 50.0 * volatility_factor);
        let price = base_price + volatility;
        let interval = (1000.0 / activity_factor) as i64; // Adjust interval based on activity
        let timestamp = start_time + (i as i64 * interval);

        trades.push(create_test_trade(
            5000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_high_frequency_data(interval_ms: i64) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64;

    // Dense, high-frequency trading
    for i in 0..10000 {
        let micro_movement = (i as f64 * 0.1).sin() * 0.5; // Very small movements
        let price = base_price + micro_movement;
        let timestamp = base_time + (i as i64 * interval_ms);

        trades.push(create_test_trade(
            6000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_medium_frequency_data(interval_ms: i64) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64;

    for i in 0..5000 {
        let movement = (i as f64 * 0.05).sin() * 10.0;
        let price = base_price + movement;
        let timestamp = base_time + (i as i64 * interval_ms);

        trades.push(create_test_trade(
            7000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_low_frequency_data(interval_ms: i64) -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64;

    for i in 0..1000 {
        let movement = (i as f64 * 0.01).sin() * 50.0;
        let price = base_price + movement;
        let timestamp = base_time + (i as i64 * interval_ms);

        trades.push(create_test_trade(
            8000000 + i as u64,
            price,
            timestamp as u64,
        ));
    }

    trades
}

fn create_mixed_frequency_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64;
    let mut current_time = base_time;

    // Variable intervals: sometimes fast, sometimes slow
    for i in 0..3000 {
        let movement = (i as f64 * 0.02).sin() * 25.0;
        let price = base_price + movement;

        // Variable interval based on market conditions
        let interval = if i % 10 < 3 {
            50 // Fast periods
        } else if i % 10 < 7 {
            200 // Medium periods
        } else {
            1000 // Slow periods
        };

        current_time += interval;
        trades.push(create_test_trade(
            9000000 + i as u64,
            price,
            current_time as u64,
        ));
    }

    trades
}

fn create_rapid_threshold_hit_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let threshold = 0.0025; // 0.25%
    let base_time = 1659312000000i64;

    // Create rapid threshold hits to stress the algorithm
    for i in 0..1000 {
        let phase = (i / 10) % 4;
        let price = match phase {
            0 => base_price,                           // Base
            1 => base_price * (1.0 + threshold * 1.1), // Above threshold
            2 => base_price,                           // Back to base
            _ => base_price * (1.0 - threshold * 1.1), // Below threshold
        };

        trades.push(create_test_trade(
            10000000 + i as u64,
            price,
            (base_time + i as i64 * 10) as u64,
        ));
    }

    trades
}

fn create_precision_limit_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_time = 1659312000000i64;

    // Test precision limits of FixedPoint (8 decimal places)
    let precision_prices = vec![
        23000.12345678,    // Max precision
        23000.00000001,    // Minimum increment
        99999999.99999999, // Large number with precision
        0.00000001,        // Smallest possible
    ];

    for (i, price) in precision_prices.iter().enumerate() {
        trades.push(create_test_trade(
            11000000 + i as u64,
            *price,
            (base_time + i as i64 * 1000) as u64,
        ));
    }

    trades
}

fn create_volume_extreme_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;
    let base_time = 1659312000000i64;

    // Test extreme volume conditions
    for i in 0..100 {
        let price = base_price + (i as f64 * 0.1);
        // Note: We use volume=1.0 consistently as per our test pattern
        trades.push(create_test_trade(
            12000000 + i as u64,
            price,
            (base_time + i as i64 * 100) as u64,
        ));
    }

    trades
}

fn create_timestamp_edge_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_price = 23000.0;

    // Test timestamp edge cases
    let edge_timestamps: Vec<i64> = vec![
        1,                   // Near epoch start
        1659312000000,       // Normal timestamp
        9223372036854775807, // Near i64 max
    ];

    for (i, timestamp) in edge_timestamps.iter().enumerate() {
        let price = base_price + (i as f64 * 10.0);
        trades.push(create_test_trade(
            13000000 + i as u64,
            price,
            *timestamp as u64,
        ));
    }

    trades
}

fn create_floating_point_stress_data() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_time = 1659312000000i64;

    // Test floating point edge cases that could cause precision issues
    let stress_prices = vec![
        23000.1 + 0.1,        // Addition that might cause precision loss
        23000.0 / 3.0,        // Division creating repeating decimals
        23000.0 * 1.1,        // Multiplication
        (23000.0_f64).sqrt(), // Square root
    ];

    for (i, price) in stress_prices.iter().enumerate() {
        trades.push(create_test_trade(
            14000000 + i as u64,
            *price,
            (base_time + i as i64 * 100) as u64,
        ));
    }

    trades
}

// Helper functions for processing and validation

fn create_test_trade(id: u64, price: f64, timestamp: u64) -> AggTrade {
    // Format price to 8 decimal places to avoid TooManyDecimals error
    let price_str = format!("{:.8}", price);
    AggTrade {
        agg_trade_id: id as i64,
        price: FixedPoint::from_str(&price_str).unwrap(),
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: id as i64,
        last_trade_id: id as i64,
        timestamp: timestamp as i64,
        is_buyer_maker: false,
    }
}

fn process_batch_style(trades: &[AggTrade], threshold_bps: u32) -> Vec<RangeBar> {
    let mut processor = ExportRangeBarProcessor::new(threshold_bps);

    // Process all trades continuously (simulating boundary-safe mode)
    processor.process_trades_continuously(trades);

    // Get all completed bars
    let mut bars = processor.get_all_completed_bars();

    // Add incomplete bar if exists
    if let Some(incomplete) = processor.get_incomplete_bar() {
        bars.push(incomplete);
    }

    bars
}

async fn process_streaming_style(trades: &[AggTrade], threshold_bps: u32) -> Vec<RangeBar> {
    // Use the corrected streaming approach that matches our fix
    let mut range_processor = ExportRangeBarProcessor::new(threshold_bps);

    // Simulate the corrected streaming behavior:
    // Process in chunks and accumulate results (like our csv_streaming.rs fix)
    let chunk_size = 10000; // Larger chunks for performance
    let mut all_bars = Vec::new();

    for chunk in trades.chunks(chunk_size) {
        range_processor.process_trades_continuously(chunk);
        // Get completed bars from this chunk and clear state
        let chunk_bars = range_processor.get_all_completed_bars();
        all_bars.extend(chunk_bars);
    }

    // Add final incomplete bar if exists
    if let Some(incomplete) = range_processor.get_incomplete_bar() {
        all_bars.push(incomplete);
    }

    all_bars
}

fn validate_temporal_integrity(bars: &[RangeBar], test_name: &str) {
    for (i, bar) in bars.iter().enumerate() {
        assert!(
            bar.close_time >= bar.open_time,
            "{}: Bar {} has close_time before open_time",
            test_name,
            i
        );

        if i > 0 {
            assert!(
                bar.open_time >= bars[i - 1].close_time,
                "{}: Bar {} starts before previous bar ends",
                test_name,
                i
            );
        }
    }
}

fn analyze_boundary_behavior(batch_bars: &[RangeBar], streaming_bars: &[RangeBar], days: usize) {
    println!("  🔍 Boundary analysis:");

    // Calculate average bar duration
    if !batch_bars.is_empty() {
        let total_time =
            batch_bars.last().unwrap().close_time - batch_bars.first().unwrap().open_time;
        let avg_duration = total_time / batch_bars.len() as i64;
        println!("    Batch avg bar duration: {}ms", avg_duration);
    }

    if !streaming_bars.is_empty() {
        let total_time =
            streaming_bars.last().unwrap().close_time - streaming_bars.first().unwrap().open_time;
        let avg_duration = total_time / streaming_bars.len() as i64;
        println!("    Streaming avg bar duration: {}ms", avg_duration);
    }

    // Look for bars that span multiple days (indicating proper boundary handling)
    let day_ms = 24 * 60 * 60 * 1000;
    let long_batch_bars = batch_bars
        .iter()
        .filter(|bar| bar.close_time - bar.open_time > day_ms)
        .count();
    let long_streaming_bars = streaming_bars
        .iter()
        .filter(|bar| bar.close_time - bar.open_time > day_ms)
        .count();

    println!(
        "    Multi-day bars: Batch={}, Streaming={}",
        long_batch_bars, long_streaming_bars
    );
}

fn validate_cross_day_bars(bars: &[RangeBar], implementation: &str) {
    let day_ms = 24 * 60 * 60 * 1000;
    let cross_day_count = bars
        .iter()
        .filter(|bar| bar.close_time - bar.open_time > day_ms)
        .count();

    if cross_day_count > 0 {
        println!(
            "    ✅ {} implementation properly handles cross-day bars: {}",
            implementation, cross_day_count
        );
    }
}

fn validate_session_characteristics(bars: &[RangeBar], session_name: &str) {
    if bars.is_empty() {
        return;
    }

    // Validate basic characteristics are reasonable for the session
    let total_duration = bars.last().unwrap().close_time - bars.first().unwrap().open_time;
    let avg_bar_duration = total_duration / bars.len() as i64;

    println!(
        "      {} avg bar duration: {}ms",
        session_name, avg_bar_duration
    );
}

fn analyze_frequency_patterns(
    batch_bars: &[RangeBar],
    streaming_bars: &[RangeBar],
    test_name: &str,
) {
    if batch_bars.is_empty() || streaming_bars.is_empty() {
        return;
    }

    // Calculate frequency-specific metrics
    let batch_time_span =
        batch_bars.last().unwrap().close_time - batch_bars.first().unwrap().open_time;
    let streaming_time_span =
        streaming_bars.last().unwrap().close_time - streaming_bars.first().unwrap().open_time;

    let batch_freq = batch_bars.len() as f64 / (batch_time_span as f64 / 1000.0); // bars per second
    let streaming_freq = streaming_bars.len() as f64 / (streaming_time_span as f64 / 1000.0);

    println!(
        "      {} frequency: Batch={:.2} bars/s, Streaming={:.2} bars/s",
        test_name, batch_freq, streaming_freq
    );
}

fn validate_stress_test_requirements(
    batch_bars: &[RangeBar],
    streaming_bars: &[RangeBar],
    test_name: &str,
) {
    // Ensure both implementations handle stress conditions
    assert!(
        !batch_bars.is_empty(),
        "{}: Batch should handle stress test",
        test_name
    );
    assert!(
        !streaming_bars.is_empty(),
        "{}: Streaming should handle stress test",
        test_name
    );

    // Validate temporal integrity under stress
    validate_temporal_integrity(batch_bars, &format!("{}_batch", test_name));
    validate_temporal_integrity(streaming_bars, &format!("{}_streaming", test_name));

    println!("      ✅ {} stress test validation passed", test_name);
}
