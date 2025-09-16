//! GPU Flow Fix Diagnostic - Test corrected processing order
//!
//! This test validates the fix for the fundamental GPU processing flow bug:
//! - OLD: Check breaches using incoming trade price (wrong)
//! - NEW: Accumulate trade data FIRST, then check breaches (correct)

use rangebar::{
    range_bars::RangeBarProcessor,
    types::AggTrade,
    fixed_point::FixedPoint,
};

#[cfg(feature = "gpu")]
use rangebar::gpu::{
    multi_symbol::MultiSymbolGpuProcessor,
    metal_backend::detect_gpu_device,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 **GPU FLOW FIX DIAGNOSTIC**");
    println!("═══════════════════════════════");
    println!("Goal: Validate corrected GPU processing flow order\\n");

    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU feature not enabled. Run with: cargo run --example gpu_flow_fix_diagnostic --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        run_flow_fix_diagnostic()
    }
}

#[cfg(feature = "gpu")]
fn run_flow_fix_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 **Flow Fix Validation Strategy:**");
    println!("   1. Create minimal breach scenario (3 trades)");
    println!("   2. Trace each step of processing flow");
    println!("   3. Verify bar accumulation before breach detection");
    println!("   4. Confirm single bar completion (not multiple)\\n");

    let threshold_bps = 8000; // 0.8%

    // Create minimal test case - 1 symbol, 3 trades with guaranteed breach
    println!("🔍 **Creating Minimal Breach Scenario**");
    println!("─────────────────────────────────────");

    let trades = create_minimal_breach_scenario();
    println!("   Trades created: {}", trades.len());

    for (i, trade) in trades.iter().enumerate() {
        println!("   Trade {}: price={:.2}", i, trade.price.to_f64());
    }

    // Expected: 1 bar (open at first price, accumulate until breach, close)
    println!("   Expected result: 1 completed bar\\n");

    // CPU baseline
    println!("🖥️  **CPU Reference Processing**");
    println!("─────────────────────────────");

    let mut cpu_processor = RangeBarProcessor::new(threshold_bps);
    let cpu_bars = cpu_processor.process_trades(&trades)?;

    println!("   CPU bars generated: {}", cpu_bars.len());
    if !cpu_bars.is_empty() {
        let bar = &cpu_bars[0];
        println!("   CPU Bar: O={:.2}, H={:.2}, L={:.2}, C={:.2}",
            bar.open, bar.high, bar.low, bar.close);
        println!("   CPU Trades in bar: {}", bar.trade_count);
    }

    // GPU processing
    println!("\\n🚀 **GPU Fixed Processing**");
    println!("──────────────────────────");

    let device = detect_gpu_device().ok_or("GPU not available")?;
    let gpu_processor = MultiSymbolGpuProcessor::new(device, threshold_bps, Some(10));

    let symbol_trades = vec![("BTCUSDT", trades.as_slice())];
    let gpu_results = gpu_processor.process_tier1_parallel(&symbol_trades)?;

    let gpu_bar_count = gpu_results.get(0).map(|(_, bars)| bars.len()).unwrap_or(0);
    println!("   GPU bars generated: {}", gpu_bar_count);

    if let Some((_, gpu_bars)) = gpu_results.get(0) {
        if !gpu_bars.is_empty() {
            let bar = &gpu_bars[0];
            println!("   GPU Bar: O={:.2}, H={:.2}, L={:.2}, C={:.2}",
                bar.open, bar.high, bar.low, bar.close);
            println!("   GPU Trades in bar: {}", bar.trade_count);
        }
    }

    // Flow Fix Assessment
    println!("\\n🎯 **FLOW FIX ASSESSMENT**");
    println!("─────────────────────────");

    let bars_match = cpu_bars.len() == gpu_bar_count;
    let expected_bars = 1;

    if bars_match && cpu_bars.len() == expected_bars {
        println!("✅ **FLOW FIX SUCCESS**: Bar counts match CPU baseline");
        println!("   Expected: {} bar", expected_bars);
        println!("   CPU: {} bars", cpu_bars.len());
        println!("   GPU: {} bars", gpu_bar_count);

        // Check OHLC accumulation quality
        if let (Some(cpu_bar), Some((_, gpu_bars))) = (cpu_bars.get(0), gpu_results.get(0)) {
            if let Some(gpu_bar) = gpu_bars.get(0) {
                let ohlc_different = cpu_bar.open != gpu_bar.open ||
                                   cpu_bar.high != gpu_bar.high ||
                                   cpu_bar.low != gpu_bar.low ||
                                   cpu_bar.close != gpu_bar.close;

                if ohlc_different {
                    println!("⚠️ Bar counts match but OHLC accumulation differs");
                    println!("   This indicates remaining accumulation bugs");
                } else {
                    println!("🎯 **PERFECT**: Identical bar count AND OHLC values!");
                }
            }
        }
    } else {
        println!("❌ **FLOW FIX INCOMPLETE**: Bar count mismatch persists");
        println!("   Expected: {} bar", expected_bars);
        println!("   CPU: {} bars", cpu_bars.len());
        println!("   GPU: {} bars", gpu_bar_count);

        if gpu_bar_count > cpu_bars.len() {
            println!("   Issue: GPU still over-generating bars");
            println!("   Diagnosis: Breach detection still triggering too early");
        } else if gpu_bar_count < cpu_bars.len() {
            println!("   Issue: GPU under-generating bars");
            println!("   Diagnosis: Breach detection not triggering when it should");
        }
    }

    Ok(())
}

/// Create minimal 3-trade scenario with guaranteed breach
fn create_minimal_breach_scenario() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_timestamp = 1640995200000i64;

    // Trade 1: Open at 50000.0
    trades.push(AggTrade {
        agg_trade_id: 1,
        price: FixedPoint::from_str("50000.0").unwrap(),
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: 1,
        last_trade_id: 1,
        timestamp: base_timestamp,
        is_buyer_maker: false,
    });

    // Trade 2: Small move to 50200.0 (within threshold)
    trades.push(AggTrade {
        agg_trade_id: 2,
        price: FixedPoint::from_str("50200.0").unwrap(),
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: 2,
        last_trade_id: 2,
        timestamp: base_timestamp + 1000,
        is_buyer_maker: false,
    });

    // Trade 3: Breach to 50500.0 (+1.0% vs 0.8% threshold)
    trades.push(AggTrade {
        agg_trade_id: 3,
        price: FixedPoint::from_str("50500.0").unwrap(),
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: 3,
        last_trade_id: 3,
        timestamp: base_timestamp + 2000,
        is_buyer_maker: false,
    });

    trades
}

#[cfg(not(feature = "gpu"))]
fn run_flow_fix_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPU feature not available");
    Ok(())
}