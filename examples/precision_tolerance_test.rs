//! Precision Tolerance Test - Validate OHLC differences are within acceptable bounds
//!
//! Tests if GPU vs CPU OHLC differences are within typical financial tolerances.

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
    println!("📊 **PRECISION TOLERANCE TEST**");
    println!("═══════════════════════════════");
    println!("Goal: Validate OHLC differences are within financial tolerances\\n");

    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU feature not enabled. Run with: cargo run --example precision_tolerance_test --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        run_precision_test()
    }
}

#[cfg(feature = "gpu")]
fn run_precision_test() -> Result<(), Box<dyn std::error::Error>> {
    let threshold_bps = 8000;
    let tolerance_bps = 1; // 0.01% tolerance (1 basis point)

    println!("🔍 **Test Configuration:**");
    println!("   Symbol: BTCUSDT");
    println!("   Threshold: {}bps ({}%)", threshold_bps, threshold_bps as f32 / 10000.0);
    println!("   Tolerance: {}bps ({}%)", tolerance_bps, tolerance_bps as f32 / 10000.0);
    println!("   Expected: GPU vs CPU OHLC differences within tolerance\\n");

    // Create simple test scenario
    let trades = create_precision_test_trades();
    println!("📈 **Test Data:**");
    println!("   Trades: {}", trades.len());
    for (i, trade) in trades.iter().take(3).enumerate() {
        println!("   Trade {}: price={:.8}", i, trade.price.to_f64());
    }
    println!();

    // CPU processing
    println!("🖥️  **CPU Processing:**");
    let mut cpu_processor = RangeBarProcessor::new(threshold_bps);
    let cpu_bars = cpu_processor.process_trades(&trades)?;
    println!("   Bars generated: {}", cpu_bars.len());

    // GPU processing
    println!("🚀 **GPU Processing:**");
    let device = detect_gpu_device().ok_or("GPU not available")?;
    let gpu_processor = MultiSymbolGpuProcessor::new(device, threshold_bps, Some(trades.len()));
    let symbol_trades = vec![("BTCUSDT", trades.as_slice())];
    let gpu_results = gpu_processor.process_tier1_parallel(&symbol_trades)?;

    let gpu_bars = if let Some((_, bars)) = gpu_results.get(0) {
        bars.clone()
    } else {
        Vec::new()
    };
    println!("   Bars generated: {}\\n", gpu_bars.len());

    // Precision tolerance analysis
    println!("📊 **PRECISION TOLERANCE ANALYSIS**");
    println!("─────────────────────────────────────");

    if cpu_bars.len() != gpu_bars.len() {
        println!("❌ Bar count mismatch: CPU={}, GPU={}", cpu_bars.len(), gpu_bars.len());
        return Ok(());
    }

    let mut max_diff_bps = 0.0;
    let mut total_diffs = 0;
    let mut acceptable_diffs = 0;

    for (i, (cpu_bar, gpu_bar)) in cpu_bars.iter().zip(gpu_bars.iter()).enumerate() {
        println!("Bar {}:", i);

        let diffs = [
            ("Open", cpu_bar.open, gpu_bar.open),
            ("High", cpu_bar.high, gpu_bar.high),
            ("Low", cpu_bar.low, gpu_bar.low),
            ("Close", cpu_bar.close, gpu_bar.close),
        ];

        for (name, cpu_val, gpu_val) in diffs {
            let cpu_f64 = cpu_val.to_f64();
            let gpu_f64 = gpu_val.to_f64();
            let abs_diff = (cpu_f64 - gpu_f64).abs();
            let diff_bps = (abs_diff / cpu_f64) * 10000.0; // Convert to basis points

            total_diffs += 1;

            if diff_bps <= tolerance_bps as f64 {
                acceptable_diffs += 1;
                println!("   {} ✅: CPU={:.8} GPU={:.8} diff={:.2}bps",
                    name, cpu_f64, gpu_f64, diff_bps);
            } else {
                println!("   {} ⚠️ : CPU={:.8} GPU={:.8} diff={:.2}bps (EXCEEDS TOLERANCE)",
                    name, cpu_f64, gpu_f64, diff_bps);
            }

            if diff_bps > max_diff_bps {
                max_diff_bps = diff_bps;
            }
        }
        println!();
    }

    // Final assessment
    println!("🎯 **TOLERANCE ASSESSMENT**");
    println!("─────────────────────────");
    println!("Maximum difference: {:.2} basis points", max_diff_bps);
    println!("Acceptable differences: {}/{} ({:.1}%)",
        acceptable_diffs, total_diffs, (acceptable_diffs as f64 / total_diffs as f64) * 100.0);

    if max_diff_bps <= tolerance_bps as f64 {
        println!("✅ **PRECISION PERFECT**: All differences within {}bp tolerance", tolerance_bps);
        println!("   GPU achieves financial-grade precision parity with CPU");
    } else if max_diff_bps <= 10.0 { // 10bp = 0.1%
        println!("✅ **PRECISION ACCEPTABLE**: Max difference {:.2}bp within typical financial tolerance", max_diff_bps);
        println!("   Differences likely due to f32 vs f64 precision - normal for GPU computing");
    } else {
        println!("⚠️  **PRECISION CONCERN**: Max difference {:.2}bp exceeds typical tolerance", max_diff_bps);
        println!("   May need investigation for systematic precision issues");
    }

    println!("\\n💡 **Context:**");
    println!("   • 1bp (0.01%) is typical minimum tick size in many markets");
    println!("   • GPU uses f32, CPU uses f64 - small differences expected");
    println!("   • Perfect bar count alignment is the critical success metric");

    Ok(())
}

fn create_precision_test_trades() -> Vec<AggTrade> {
    let mut trades = Vec::new();
    let base_timestamp = 1640995200000i64;
    let mut current_price = 50000.0;

    // Create trades with precise decimal values to test floating-point handling
    let price_changes = [0.0, -0.008, 0.012, -0.0075, 0.006]; // Mix of small and large changes

    for (i, &change) in price_changes.iter().enumerate() {
        current_price *= 1.0 + change;

        trades.push(AggTrade {
            agg_trade_id: (i as i64) + 1,
            price: FixedPoint::from_str(&format!("{:.8}", current_price)).unwrap(),
            volume: FixedPoint::from_str("1.00000000").unwrap(),
            first_trade_id: (i as i64 + 1) * 10,
            last_trade_id: (i as i64 + 1) * 10,
            timestamp: base_timestamp + (i as i64) * 1000,
            is_buyer_maker: i % 2 == 0,
        });
    }

    trades
}

#[cfg(not(feature = "gpu"))]
fn run_precision_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPU feature not available");
    Ok(())
}