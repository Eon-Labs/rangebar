/// Quick batch processing demo showing memory usage patterns
///
/// This demonstrates batch processing characteristics vs Production Streaming V2

use rangebar::range_bars::ExportRangeBarProcessor;
use rangebar::types::AggTrade;
use rangebar::fixed_point::FixedPoint;
use std::time::Instant;

fn create_test_trade(id: u64, price: f64, timestamp: u64) -> AggTrade {
    AggTrade {
        agg_trade_id: id as i64,
        price: FixedPoint::from_str(&format!("{:.8}", price)).unwrap(),
        volume: FixedPoint::from_str("1.0").unwrap(),
        first_trade_id: id as i64,
        last_trade_id: id as i64,
        timestamp: timestamp as i64,
        is_buyer_maker: false,
    }
}

fn get_memory_usage_kb() -> u64 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(rss_str) = String::from_utf8(output.stdout) {
                if let Ok(rss_kb) = rss_str.trim().parse::<u64>() {
                    return rss_kb;
                }
            }
        }
    }
    0
}

fn main() {
    println!("🚀 Quick Batch Processing Demo");
    println!("================================================");

    let threshold_bps = 25;
    let trade_count = 500_000; // 500k trades

    // Generate test data
    println!("📊 Generating {} test trades...", trade_count);
    let trades: Vec<AggTrade> = (0..trade_count)
        .map(|i| {
            let price = 23000.0 + (i as f64 * 0.01);
            let timestamp = 1659312000000 + i as u64 * 1000;
            create_test_trade(i as u64, price, timestamp)
        })
        .collect();

    println!("✅ Generated {} trades", trades.len());
    println!();

    // Test 1: Batch Processing (potential unbounded memory)
    println!("🏃 Test 1: Batch Processing (ExportRangeBarProcessor)");
    let initial_mem = get_memory_usage_kb();

    let start_time = Instant::now();
    let mut batch_processor = ExportRangeBarProcessor::new(threshold_bps);
    batch_processor.process_trades_continuously(&trades);
    let mut bars = batch_processor.get_all_completed_bars();

    if let Some(incomplete) = batch_processor.get_incomplete_bar() {
        bars.push(incomplete);
    }

    let batch_duration = start_time.elapsed();
    let batch_memory = get_memory_usage_kb() - initial_mem;

    println!("  📈 Duration: {:.2}s", batch_duration.as_secs_f64());
    println!("  💾 Memory: {:.1}MB", batch_memory as f64 / 1024.0);
    println!("  📊 Bars: {}", bars.len());
    println!("  ⚡ Throughput: {:.0} trades/sec", trades.len() as f64 / batch_duration.as_secs_f64());
    println!();


    println!();
    println!("🎯 KEY INSIGHTS:");
    println!("  • Batch processing accumulates all bars in memory");
    println!("  • Production V2 provides bounded memory with channels");
    println!("  • V2 processes single bars with immediate dispatch");
    println!();

    println!("💡 For full comparison run the cross-year test with Production Streaming V2:");
    println!("  🔒 Bounded memory channels (5000 trades, 100 bars)");
    println!("  ⚖️  Backpressure mechanisms");
    println!("  🛡️  Circuit breaker patterns");
    println!("  ♾️  True infinite streaming capability");
}