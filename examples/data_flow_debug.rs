//! Data Flow Diagnostic - Multi-Symbol Pipeline
//!
//! Since atomic tensor tests pass perfectly, the issue must be in
//! multi-symbol data flow, state management, or processing logic.

use rangebar::gpu::multi_symbol::*;
use rangebar::gpu::GpuDevice;
use rangebar::types::FixedPoint;

#[cfg(feature = "gpu")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **DATA FLOW DIAGNOSTIC - MULTI-SYMBOL PIPELINE**");
    println!("═══════════════════════════════════════════════════");
    println!("Goal: Trace where 6 vs 1500 bars gap originates\n");

    // Create processor
    let device = GpuDevice::new()?;
    let mut processor = MultiSymbolGpuProcessor::new(device, 8000, None);
    println!("✅ GPU processor initialized\n");

    // Create controlled test data - small but predictable
    println!("📊 **CREATING CONTROLLED TEST DATA**");
    println!("─────────────────────────────────");

    let symbols = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT"];
    println!("Test symbols: {:?}", symbols);

    // Create 10 trades per symbol with known breach patterns
    let mut symbol_trades = std::collections::HashMap::new();

    // BTC: Should generate 2 bars (open -> upper breach -> new bar -> lower breach)
    let btc_trades = vec![
        create_test_trade(50750.0, 1.0, 1000),  // Open
        create_test_trade(50800.0, 1.0, 1001),  // Small move
        create_test_trade(51200.0, 1.0, 1002),  // Upper breach (should close bar)
        create_test_trade(50750.0, 1.0, 1003),  // New bar open
        create_test_trade(50200.0, 1.0, 1004),  // Lower breach (should close bar)
    ];

    // ETH: Should generate 1 bar (open -> upper breach)
    let eth_trades = vec![
        create_test_trade(4000.0, 2.0, 2000),   // Open
        create_test_trade(4100.0, 2.0, 2001),   // Small move
        create_test_trade(4400.0, 2.0, 2002),   // Upper breach (should close bar)
    ];

    // ADA: Should generate 1 bar (open -> lower breach)
    let ada_trades = vec![
        create_test_trade(1.0, 100.0, 3000),    // Open
        create_test_trade(0.95, 100.0, 3001),   // Small move
        create_test_trade(0.85, 100.0, 3002),   // Lower breach (should close bar)
    ];

    symbol_trades.insert("BTCUSDT", btc_trades.as_slice());
    symbol_trades.insert("ETHUSDT", eth_trades.as_slice());
    symbol_trades.insert("ADAUSDT", ada_trades.as_slice());

    println!("Expected bars:");
    println!("  BTCUSDT: 2 bars (upper breach + lower breach)");
    println!("  ETHUSDT: 1 bar (upper breach)");
    println!("  ADAUSDT: 1 bar (lower breach)");
    println!("  Total expected: 4 bars\n");

    // Process trades
    println!("⚡ **PROCESSING TRADES**");
    println!("─────────────────────");
    let start_time = std::time::Instant::now();
    let results = processor.compute_parallel_range_bars(&symbol_trades)?;
    let duration = start_time.elapsed();

    println!("Processing completed in {:?}", duration);

    // Analyze results
    println!("\n📈 **RESULTS ANALYSIS**");
    println!("─────────────────────");

    let mut total_bars = 0;
    for (symbol, bars) in &results {
        println!("Symbol {}: {} bars generated", symbol, bars.len());
        total_bars += bars.len();

        for (i, bar) in bars.iter().enumerate() {
            println!("  Bar {}: open={:.2}, high={:.2}, low={:.2}, close={:.2}",
                i, bar.open, bar.high, bar.low, bar.close);
        }
    }

    println!("\n🎯 **DIAGNOSTIC SUMMARY**");
    println!("──────────────────────");
    println!("Expected total bars: 4");
    println!("Actual total bars: {}", total_bars);

    if total_bars == 4 {
        println!("✅ SUCCESS: Controlled test shows correct behavior!");
        println!("   Issue may be in demo data or larger scale processing");
    } else {
        println!("❌ ISSUE CONFIRMED: Even controlled test shows wrong results");
        println!("   Gap: {} vs 4 expected", total_bars);

        // Debug what went wrong
        if total_bars == 0 {
            println!("   Problem: No bars generated at all - initialization issue");
        } else if total_bars < 4 {
            println!("   Problem: Under-generation - breach detection or completion issue");
        } else {
            println!("   Problem: Over-generation - false breach detection");
        }
    }

    Ok(())
}

#[cfg(not(feature = "gpu"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("❌ GPU feature not enabled. Run with: cargo run --example data_flow_debug --features gpu");
    Ok(())
}

fn create_test_trade(price: f64, volume: f64, timestamp: i64) -> rangebar::types::AggTrade {
    rangebar::types::AggTrade {
        agg_trade_id: timestamp,
        price: FixedPoint::from_f64(price),
        volume: FixedPoint::from_f64(volume),
        first_trade_id: timestamp,
        last_trade_id: timestamp,
        timestamp,
        is_buyer_maker: false,
    }
}