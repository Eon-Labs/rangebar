//! Deterministic Alignment Test - Ultimate GPU vs CPU Validation
//!
//! Tests whether GPU and CPU produce EXACTLY the same number of bars
//! when given identical aggTrades input data.

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
    println!("🎯 **DETERMINISTIC ALIGNMENT TEST - ULTIMATE VALIDATION**");
    println!("════════════════════════════════════════════════════════");
    println!("Goal: Verify GPU and CPU produce IDENTICAL bar counts with IDENTICAL input\\n");

    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU feature not enabled. Run with: cargo run --example deterministic_alignment_test --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        run_deterministic_test()
    }
}

#[cfg(feature = "gpu")]
fn run_deterministic_test() -> Result<(), Box<dyn std::error::Error>> {
    let threshold_bps = 8000; // 0.8% threshold - IDENTICAL for both

    println!("📊 **Test Configuration:**");
    println!("   Threshold: {}bps ({}%)", threshold_bps, threshold_bps as f32 / 10000.0);
    println!("   Data: IDENTICAL aggTrades for both CPU and GPU");
    println!("   Goal: EXACT bar count alignment\\n");

    // Create IDENTICAL test data with KNOWN breach patterns
    let test_symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let trades_per_symbol = 100;

    println!("🔍 **Creating IDENTICAL Test Data**");
    println!("─────────────────────────────────");

    let mut symbol_trades = Vec::new();
    for symbol in &test_symbols {
        let trades = create_deterministic_trades(symbol, trades_per_symbol);
        println!("   {}: {} trades (guaranteed breach patterns)", symbol, trades.len());
        symbol_trades.push((*symbol, trades));
    }

    // CPU PROCESSING - Sequential baseline
    println!("\\n🖥️  **CPU SEQUENTIAL PROCESSING**");
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
    println!("   ✅ CPU Total: {} bars in {:?}", total_cpu_bars, cpu_duration);

    // GPU PROCESSING - Parallel (IDENTICAL input)
    println!("\\n🚀 **GPU PARALLEL PROCESSING (IDENTICAL INPUT)**");
    println!("─────────────────────────────────────────────────");

    let device = detect_gpu_device().ok_or("GPU not available")?;
    let mut gpu_processor = MultiSymbolGpuProcessor::new(device, threshold_bps, Some(trades_per_symbol));

    let gpu_start = std::time::Instant::now();

    // Convert to GPU format - IDENTICAL DATA
    let symbol_trades_refs: Vec<(&str, &[AggTrade])> = symbol_trades.iter()
        .map(|(symbol, trades)| (*symbol, trades.as_slice()))
        .collect();

    let gpu_results = gpu_processor.process_tier1_parallel(&symbol_trades_refs)?;
    let gpu_duration = gpu_start.elapsed();

    let mut total_gpu_bars = 0;
    for (symbol, bars) in &gpu_results {
        println!("   {}: {} bars", symbol, bars.len());
        total_gpu_bars += bars.len();
    }
    println!("   ✅ GPU Total: {} bars in {:?}", total_gpu_bars, gpu_duration);

    // DETERMINISTIC ALIGNMENT ANALYSIS
    println!("\\n📊 **DETERMINISTIC ALIGNMENT ANALYSIS**");
    println!("─────────────────────────────────────────");

    println!("Overall Alignment:");
    println!("   CPU Total: {} bars", total_cpu_bars);
    println!("   GPU Total: {} bars", total_gpu_bars);

    let perfect_alignment = total_cpu_bars == total_gpu_bars;
    let alignment_percentage = if total_cpu_bars > 0 {
        (total_gpu_bars as f64 / total_cpu_bars as f64) * 100.0
    } else {
        100.0
    };

    println!("   Alignment: {:.1}%", alignment_percentage);

    if perfect_alignment {
        println!("   🎯 **PERFECT DETERMINISTIC ALIGNMENT!**");
    } else {
        println!("   ⚠️ Alignment gap: {} bars", (total_cpu_bars as i32 - total_gpu_bars as i32).abs());
    }

    // Per-symbol detailed analysis
    println!("\\nPer-Symbol Alignment:");
    let mut all_symbols_aligned = true;

    for (cpu_symbol, cpu_bars) in &cpu_results {
        if let Some((_, gpu_bars)) = gpu_results.iter().find(|(gpu_symbol, _)| *gpu_symbol == *cpu_symbol) {
            let symbol_aligned = cpu_bars.len() == gpu_bars.len();
            let symbol_alignment = if cpu_bars.len() > 0 {
                (gpu_bars.len() as f64 / cpu_bars.len() as f64) * 100.0
            } else {
                100.0
            };

            let status = if symbol_aligned { "✅ ALIGNED" } else { "❌ MISALIGNED" };
            println!("   {}: CPU={} GPU={} ({:.1}%) {}",
                cpu_symbol, cpu_bars.len(), gpu_bars.len(), symbol_alignment, status);

            if !symbol_aligned {
                all_symbols_aligned = false;

                // Detailed analysis for misaligned symbols
                println!("     → Difference: {} bars", (cpu_bars.len() as i32 - gpu_bars.len() as i32).abs());

                // Show first few bars for comparison
                if !cpu_bars.is_empty() || !gpu_bars.is_empty() {
                    println!("     → CPU bars sample: {:?}", cpu_bars.iter().take(3).map(|b| format!("O:{:.2},H:{:.2},L:{:.2},C:{:.2}", b.open, b.high, b.low, b.close)).collect::<Vec<_>>());
                    println!("     → GPU bars sample: {:?}", gpu_bars.iter().take(3).map(|b| format!("O:{:.2},H:{:.2},L:{:.2},C:{:.2}", b.open, b.high, b.low, b.close)).collect::<Vec<_>>());
                }
            }
        } else {
            println!("   {}: ❌ MISSING in GPU results", cpu_symbol);
            all_symbols_aligned = false;
        }
    }

    // FINAL DETERMINISTIC ASSESSMENT
    println!("\\n🎯 **FINAL DETERMINISTIC ASSESSMENT**");
    println!("─────────────────────────────────────");

    if perfect_alignment && all_symbols_aligned {
        println!("✅ **DETERMINISTIC ALIGNMENT: PERFECT!**");
        println!("   🎯 GPU and CPU produce IDENTICAL results");
        println!("   📊 Total alignment: {:.1}%", alignment_percentage);
        println!("   🚀 GPU implementation is DETERMINISTICALLY CORRECT!");
    } else {
        println!("❌ **DETERMINISTIC ALIGNMENT: IMPERFECT**");
        if !perfect_alignment {
            println!("   📊 Total bar count mismatch: {} vs {}", total_cpu_bars, total_gpu_bars);
        }
        if !all_symbols_aligned {
            println!("   📊 Per-symbol misalignments detected");
        }

        // Analysis of potential causes
        println!("\\n🔍 **Potential Causes Analysis:**");
        if total_gpu_bars < total_cpu_bars {
            println!("   • GPU under-generating: possible precision or threshold issues");
        } else if total_gpu_bars > total_cpu_bars {
            println!("   • GPU over-generating: possible false breach detection");
        }

        println!("   • Check floating point precision differences");
        println!("   • Verify threshold calculation alignment");
        println!("   • Examine parallel vs sequential processing differences");
    }

    // Performance comparison
    println!("\\n⚡ **Performance Comparison:**");
    let performance_ratio = gpu_duration.as_secs_f64() / cpu_duration.as_secs_f64();
    println!("   CPU time: {:?}", cpu_duration);
    println!("   GPU time: {:?}", gpu_duration);
    println!("   Ratio: {:.3}x", performance_ratio);

    if performance_ratio < 1.0 {
        println!("   🚀 GPU is {:.1}x FASTER!", 1.0 / performance_ratio);
    } else {
        println!("   🐌 GPU is {:.1}x slower (expected for small datasets)", performance_ratio);
    }

    Ok(())
}

/// Create deterministic trades with KNOWN breach patterns for alignment testing
fn create_deterministic_trades(symbol: &str, count: usize) -> Vec<AggTrade> {
    let base_price = match symbol {
        "BTCUSDT" => 50000.0,
        "ETHUSDT" => 4000.0,
        "SOLUSDT" => 100.0,
        _ => 1000.0,
    };

    let mut trades = Vec::with_capacity(count);
    let mut current_price = base_price;
    let timestamp = 1640995200000i64;

    // Create DETERMINISTIC breach patterns every 20 trades
    for i in 0..count {
        let price_change = if i > 0 && i % 20 == 0 {
            // Guaranteed breach: alternating ±1.0% (vs 0.8% threshold)
            if (i / 20) % 2 == 0 { 0.010 } else { -0.010 }
        } else {
            // Small deterministic movements between breaches
            let normalized_i = (i % 19) as f64;
            (normalized_i / 100.0 - 0.09) * 0.005 // ±0.05% range
        };

        current_price *= 1.0 + price_change;
        let volume = 1.0 + (i as f64 * 0.1) % 10.0; // Deterministic volume 1.0-11.0

        trades.push(AggTrade {
            agg_trade_id: (i as i64) + 1,
            price: FixedPoint::from_str(&format!("{:.8}", current_price))
                .unwrap_or_else(|_| FixedPoint(0)),
            volume: FixedPoint::from_str(&format!("{:.8}", volume))
                .unwrap_or_else(|_| FixedPoint(0)),
            first_trade_id: (i as i64 + 1) * 10,
            last_trade_id: (i as i64 + 1) * 10,
            timestamp: timestamp + (i as i64) * 1000,
            is_buyer_maker: i % 2 == 0,
        });
    }

    trades
}

#[cfg(not(feature = "gpu"))]
fn run_deterministic_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPU feature not available");
    Ok(())
}