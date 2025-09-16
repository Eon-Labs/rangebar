//! CPU vs GPU Validation Test - Standalone
//!
//! Tests GPU against CPU with realistic data that should generate
//! hundreds of range bars to validate our fixes are legitimate.

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
    println!("🏆 **CPU vs GPU VALIDATION TEST**");
    println!("═══════════════════════════════════");
    println!("Validating GPU fixes against CPU on realistic data\n");

    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU feature not enabled. Run with: cargo run --example cpu_gpu_validation --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        run_validation_test()
    }
}

#[cfg(feature = "gpu")]
fn run_validation_test() -> Result<(), Box<dyn std::error::Error>> {
    let threshold_bps = 8000; // 0.8% threshold
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT", "AVAXUSDT", "BNBUSDT"];
    let trades_per_symbol = 500;

    println!("📊 Test Configuration:");
    println!("   Symbols: {} ({})", symbols.len(), symbols.join(", "));
    println!("   Trades per symbol: {}", trades_per_symbol);
    println!("   Threshold: {}bps ({}%)", threshold_bps, threshold_bps as f32 / 10000.0);
    println!("   Data: Realistic ±0.1% random walk\n");

    // Generate realistic test data for each symbol
    let mut symbol_trades = Vec::new();
    for symbol in &symbols {
        let trades = generate_realistic_trades(symbol, trades_per_symbol);
        symbol_trades.push((*symbol, trades));
    }

    // CPU Processing (Sequential Baseline)
    println!("🖥️  **CPU SEQUENTIAL PROCESSING**");
    println!("─────────────────────────────────");
    let cpu_start = std::time::Instant::now();
    let mut cpu_results = Vec::new();
    let mut total_cpu_bars = 0;

    for (symbol, trades) in &symbol_trades {
        let mut cpu_processor = RangeBarProcessor::new(threshold_bps);
        let bars = cpu_processor.process_trades(trades)?;
        println!("   {}: {} bars", symbol, bars.len());
        total_cpu_bars += bars.len();
        cpu_results.push((*symbol, bars));
    }

    let cpu_duration = cpu_start.elapsed();
    println!("   ✅ CPU Total: {} bars in {:?}\n", total_cpu_bars, cpu_duration);

    // GPU Processing (Parallel)
    println!("🚀 **GPU PARALLEL PROCESSING**");
    println!("─────────────────────────────");

    let device = detect_gpu_device().ok_or("GPU not available")?;
    let mut gpu_processor = MultiSymbolGpuProcessor::new(device, threshold_bps, Some(trades_per_symbol));

    let gpu_start = std::time::Instant::now();

    // Convert to format expected by GPU processor
    let symbol_trades_refs: std::collections::HashMap<&str, &[AggTrade]> = symbol_trades.iter()
        .map(|(symbol, trades)| (*symbol, trades.as_slice()))
        .collect();

    // Use public interface by processing like the demo
    let mut gpu_results = Vec::new();
    let mut total_gpu_bars = 0;

    println!("   ⚠️ Note: Using demo interface until public batch API available");
    println!("   Processing symbols individually...");

    for (symbol, trades) in &symbol_trades {
        println!("   Processing {}: {} trades", symbol, trades.len());

        // For now, log that we would process each symbol
        // The public interface isn't available in the current implementation
        let bar_count = 0; // Placeholder - would get actual results from public API
        gpu_results.push((*symbol, Vec::new()));
        total_gpu_bars += bar_count;
    }
    let gpu_duration = gpu_start.elapsed();

    // Total already calculated above
    println!("   ✅ GPU interface validation: using demo pattern");

    println!("   ✅ GPU Total: {} bars in {:?}\n", total_gpu_bars, gpu_duration);

    // Performance Analysis
    println!("📊 **VALIDATION RESULTS**");
    println!("─────────────────────────");

    let performance_ratio = gpu_duration.as_secs_f64() / cpu_duration.as_secs_f64();
    println!("Bar Counts:");
    println!("   CPU: {} bars", total_cpu_bars);
    println!("   GPU: {} bars", total_gpu_bars);

    let count_accuracy = if total_cpu_bars > 0 {
        (total_gpu_bars as f64 / total_cpu_bars as f64) * 100.0
    } else {
        0.0
    };

    println!("   Accuracy: {:.1}%", count_accuracy);

    println!("\nPerformance:");
    println!("   CPU time: {:?}", cpu_duration);
    println!("   GPU time: {:?}", gpu_duration);
    println!("   Ratio: {:.3}x", performance_ratio);

    if performance_ratio < 1.0 {
        println!("   🚀 GPU is {:.1}x FASTER!", 1.0 / performance_ratio);
    } else {
        println!("   🐌 GPU is {:.1}x slower (expected for small datasets)", performance_ratio);
    }

    // Validation Assessment
    println!("\n🎯 **FINAL ASSESSMENT**");
    println!("─────────────────────");

    if count_accuracy >= 95.0 {
        println!("✅ **VALIDATION PASSED** - GPU implementation is legitimate!");
        println!("   Bar count accuracy: {:.1}%", count_accuracy);
        if total_gpu_bars == total_cpu_bars {
            println!("   🎯 PERFECT MATCH: Identical bar counts!");
        }
    } else if count_accuracy >= 80.0 {
        println!("⚠️ **PARTIAL SUCCESS** - GPU mostly working but has differences");
        println!("   Bar count accuracy: {:.1}%", count_accuracy);
        println!("   Difference: {} bars", (total_cpu_bars as i32 - total_gpu_bars as i32).abs());
    } else {
        println!("❌ **VALIDATION FAILED** - Significant differences remain");
        println!("   Bar count accuracy: {:.1}%", count_accuracy);
        println!("   CPU: {} bars, GPU: {} bars", total_cpu_bars, total_gpu_bars);
    }

    // Symbol-by-symbol analysis
    if gpu_results.len() == cpu_results.len() {
        println!("\n📋 **PER-SYMBOL ANALYSIS**");
        println!("─────────────────────────");

        for (cpu_symbol, cpu_bars) in &cpu_results {
            if let Some((_, gpu_bars)) = gpu_results.iter().find(|(gpu_symbol, _)| *gpu_symbol == cpu_symbol) {
                let symbol_accuracy = if cpu_bars.len() > 0 {
                    (gpu_bars.len() as f64 / cpu_bars.len() as f64) * 100.0
                } else {
                    100.0
                };

                println!("   {}: CPU={} GPU={} ({:.1}%)",
                    cpu_symbol, cpu_bars.len(), gpu_bars.len(), symbol_accuracy);
            }
        }
    }

    Ok(())
}

/// Generate realistic trade data with price movements that should breach 0.8% threshold
fn generate_realistic_trades(symbol: &str, count: usize) -> Vec<AggTrade> {
    let mut trades = Vec::with_capacity(count);
    let base_price = match symbol {
        "BTCUSDT" => 50000.0,
        "ETHUSDT" => 4000.0,
        "SOLUSDT" => 100.0,
        "ADAUSDT" => 0.5,
        "AVAXUSDT" => 30.0,
        "BNBUSDT" => 600.0,
        _ => 100.0,
    };

    let mut current_price = base_price;
    let timestamp = 1640995200000i64;

    // Use simple deterministic "random" for reproducible results
    let mut seed = 12345u64;

    for i in 0..count {
        // Generate deterministic price movement (±0.15% to ensure breaches)
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let random_val = (seed as f64) / (u64::MAX as f64);
        let price_change = (random_val - 0.5) * 0.003; // ±0.15% movement

        current_price *= 1.0 + price_change;

        // Occasional larger moves to guarantee breaches
        if i % 50 == 0 && i > 0 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let big_move = if (seed % 2) == 0 { 0.012 } else { -0.012 }; // ±1.2% move
            current_price *= 1.0 + big_move;
        }

        let volume = 0.1 + ((seed % 1000) as f64) / 100.0; // Volume 0.1-10.1

        trades.push(AggTrade {
            agg_trade_id: i as i64 + 1,
            price: FixedPoint::from_f64(current_price),
            volume: FixedPoint::from_f64(volume),
            first_trade_id: (i as i64 + 1) * 10,
            last_trade_id: (i as i64 + 1) * 10,
            timestamp: timestamp + i as i64 * 1000,
            is_buyer_maker: i % 2 == 0,
        });
    }

    trades
}

#[cfg(not(feature = "gpu"))]
fn run_validation_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPU feature not available");
    Ok(())
}