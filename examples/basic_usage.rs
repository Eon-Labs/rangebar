//! Basic Range Bar Processing Example
//!
//! This example demonstrates how to use the rangebar library to process
//! cryptocurrency trades into range bars using the non-lookahead algorithm.

use rangebar::{AggTrade, FixedPoint, RangeBarProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Range Bar Processing Example");
    println!("================================");

    // Create a range bar processor with 0.8% threshold (8000 basis points)
    let mut processor = RangeBarProcessor::new(8000);
    println!("✅ Created processor with 0.8% threshold");

    // Create sample trade data simulating real market conditions
    let sample_trades = create_sample_trades()?;
    println!("✅ Generated {} sample trades", sample_trades.len());

    // Process trades into range bars
    let range_bars = processor.process_trades(&sample_trades)?;
    println!(
        "✅ Processed {} trades into {} range bars",
        sample_trades.len(),
        range_bars.len()
    );

    // Display results
    println!("\n📊 Range Bar Results:");
    println!("{:-<80}", "");
    println!(
        "{:>4} | {:>12} | {:>12} | {:>12} | {:>12} | {:>8}",
        "Bar", "Open", "High", "Low", "Close", "Volume"
    );
    println!("{:-<80}", "");

    for (i, bar) in range_bars.iter().enumerate() {
        println!(
            "{:>4} | {:>12} | {:>12} | {:>12} | {:>12} | {:>8}",
            i + 1,
            format!("{:.2}", bar.open.to_f64()),
            format!("{:.2}", bar.high.to_f64()),
            format!("{:.2}", bar.low.to_f64()),
            format!("{:.2}", bar.close.to_f64()),
            format!("{:.3}", bar.volume.to_f64())
        );
    }

    println!("\n🎯 Key Features Demonstrated:");
    println!("   • Non-lookahead bias: Thresholds computed from bar open only");
    println!("   • Fixed-point arithmetic: No floating-point precision errors");
    println!("   • High-frequency ready: Handles millisecond-precision timestamps");

    Ok(())
}

/// Create sample trade data simulating realistic market conditions
fn create_sample_trades() -> Result<Vec<AggTrade>, Box<dyn std::error::Error>> {
    let base_price = 50000.0;
    let base_timestamp = 1609459200000; // 2021-01-01 00:00:00 UTC
    let mut trades = Vec::new();

    // Simulate price movements that will trigger range bar closes
    let price_changes = [
        0.0,  // Start price
        0.2,  // Small move
        0.5,  // Medium move
        0.9,  // Large move - should trigger range bar close (>0.8%)
        -0.3, // Retracement
        -0.9, // Large down move - should trigger range bar close
        0.1,  // Small recovery
    ];

    for (i, &price_change_pct) in price_changes.iter().enumerate() {
        let price = base_price * (1.0 + price_change_pct / 100.0);
        let trade = AggTrade {
            agg_trade_id: i as i64 + 1,
            price: FixedPoint::from_str(&format!("{:.8}", price))?,
            volume: FixedPoint::from_str("1.50000000")?,
            first_trade_id: i as i64 * 10 + 1,
            last_trade_id: i as i64 * 10 + 5,
            timestamp: base_timestamp + (i as i64 * 1000), // 1 second apart
            is_buyer_maker: i % 2 == 0,                    // Alternate buy/sell
        };
        trades.push(trade);
    }

    Ok(trades)
}
