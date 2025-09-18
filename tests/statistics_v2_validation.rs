use rangebar::fixed_point::FixedPoint;
use rangebar::statistics_v2::StreamingStatsEngine;
use rangebar::types::{AggTrade, RangeBar};

fn main() {
    println!("🧪 Testing Statistics V2 Module - Sensibility Check");

    let mut engine = StreamingStatsEngine::new();

    // Create realistic trade data with known statistical properties
    let trades = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1609459200000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("50100.0").unwrap(), // +0.2%
            volume: FixedPoint::from_str("2.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 1609459201000,
            is_buyer_maker: true,
        },
        AggTrade {
            agg_trade_id: 3,
            price: FixedPoint::from_str("49900.0").unwrap(), // -0.2%
            volume: FixedPoint::from_str("1.5").unwrap(),
            first_trade_id: 3,
            last_trade_id: 3,
            timestamp: 1609459202000,
            is_buyer_maker: false,
        },
        AggTrade {
            agg_trade_id: 4,
            price: FixedPoint::from_str("50200.0").unwrap(), // +0.4%
            volume: FixedPoint::from_str("3.0").unwrap(),
            first_trade_id: 4,
            last_trade_id: 4,
            timestamp: 1609459203000,
            is_buyer_maker: true,
        },
        AggTrade {
            agg_trade_id: 5,
            price: FixedPoint::from_str("50050.0").unwrap(), // +0.1%
            volume: FixedPoint::from_str("2.5").unwrap(),
            first_trade_id: 5,
            last_trade_id: 5,
            timestamp: 1609459204000,
            is_buyer_maker: false,
        },
    ];

    // Process trades
    println!("\n📈 Processing {} trades...", trades.len());
    for trade in &trades {
        engine.process_trade(trade);
        println!(
            "  Trade: ${:.2} @ {:.1} volume",
            trade.price.to_f64(),
            trade.volume.to_f64()
        );
    }

    // Expected statistics:
    // Prices: [50000, 50100, 49900, 50200, 50050]
    // Mean: ~50050
    // Min: 49900, Max: 50200
    // Range: 300
    // Volumes: [1.0, 2.0, 1.5, 3.0, 2.5]
    // Volume mean: ~2.0

    let snapshot = engine.snapshot();

    println!("\n📊 TRADE STATISTICS:");
    println!("  Count: {}", snapshot.trade_count);
    println!("  Price Rolling Stats:");
    println!("    Mean: ${:.2}", snapshot.price_stats.rolling.mean);
    println!("    Std Dev: ${:.2}", snapshot.price_stats.rolling.std_dev);
    println!("    Count: {}", snapshot.price_stats.rolling.count);
    println!(
        "  Price Range: ${:.2} - ${:.2}",
        snapshot.price_stats.range.0, snapshot.price_stats.range.1
    );

    println!("  Volume Rolling Stats:");
    println!("    Mean: {:.2}", snapshot.volume_stats.rolling.mean);
    println!("    Std Dev: {:.2}", snapshot.volume_stats.rolling.std_dev);
    println!("    Count: {}", snapshot.volume_stats.rolling.count);
    println!(
        "  Volume Range: {:.2} - {:.2}",
        snapshot.volume_stats.range.0, snapshot.volume_stats.range.1
    );

    println!("\n🎯 PERCENTILES (T-Digest):");
    for (name, value) in &snapshot.price_stats.percentiles {
        println!("  Price {}: ${:.2}", name, value);
    }
    for (name, value) in &snapshot.volume_stats.percentiles {
        println!("  Volume {}: {:.2}", name, value);
    }

    // Test range bars
    println!("\n📊 Testing Range Bar Statistics...");

    let range_bars = vec![
        RangeBar {
            open_time: 1609459200000,
            close_time: 1609459201000,
            open: FixedPoint::from_str("50000.0").unwrap(),
            high: FixedPoint::from_str("50150.0").unwrap(),
            low: FixedPoint::from_str("49950.0").unwrap(),
            close: FixedPoint::from_str("50100.0").unwrap(),
            volume: FixedPoint::from_str("10.0").unwrap(),
            turnover: 0,
            trade_count: 42,
            first_id: 1,
            last_id: 42,
            buy_volume: FixedPoint::from_str("6.0").unwrap(),
            buy_turnover: 0,
            sell_volume: FixedPoint::from_str("4.0").unwrap(),
            sell_turnover: 0,
            buy_trade_count: 25,
            sell_trade_count: 17,
            vwap: FixedPoint::from_str("50075.0").unwrap(),
        },
        RangeBar {
            open_time: 1609459201000,
            close_time: 1609459202000,
            open: FixedPoint::from_str("50100.0").unwrap(),
            high: FixedPoint::from_str("50250.0").unwrap(),
            low: FixedPoint::from_str("50000.0").unwrap(),
            close: FixedPoint::from_str("50200.0").unwrap(),
            volume: FixedPoint::from_str("15.0").unwrap(),
            turnover: 0,
            trade_count: 63,
            first_id: 43,
            last_id: 105,
            buy_volume: FixedPoint::from_str("9.0").unwrap(),
            buy_turnover: 0,
            sell_volume: FixedPoint::from_str("6.0").unwrap(),
            sell_turnover: 0,
            buy_trade_count: 38,
            sell_trade_count: 25,
            vwap: FixedPoint::from_str("50125.0").unwrap(),
        },
    ];

    for bar in &range_bars {
        engine.process_bar(bar);
        println!(
            "  Bar: O=${:.2} H=${:.2} L=${:.2} C=${:.2} V={:.1}",
            bar.open.to_f64(),
            bar.high.to_f64(),
            bar.low.to_f64(),
            bar.close.to_f64(),
            bar.volume.to_f64()
        );
    }

    let final_snapshot = engine.snapshot();

    println!("\n📊 RANGE BAR STATISTICS:");
    println!("  Bar Count: {}", final_snapshot.bar_count);
    println!("  OHLC Statistics:");
    println!(
        "    Open Mean: ${:.2}, Std Dev: ${:.2}",
        final_snapshot.ohlc_stats.open.rolling.mean, final_snapshot.ohlc_stats.open.rolling.std_dev
    );
    println!(
        "    High Mean: ${:.2}, Std Dev: ${:.2}",
        final_snapshot.ohlc_stats.high.rolling.mean, final_snapshot.ohlc_stats.high.rolling.std_dev
    );
    println!(
        "    Low Mean: ${:.2}, Std Dev: ${:.2}",
        final_snapshot.ohlc_stats.low.rolling.mean, final_snapshot.ohlc_stats.low.rolling.std_dev
    );
    println!(
        "    Close Mean: ${:.2}, Std Dev: ${:.2}",
        final_snapshot.ohlc_stats.close.rolling.mean,
        final_snapshot.ohlc_stats.close.rolling.std_dev
    );

    println!("\n✅ SENSIBILITY CHECKS:");

    // Check if means are reasonable
    let price_mean = final_snapshot.price_stats.rolling.mean;
    if (49000.0..=51000.0).contains(&price_mean) {
        println!("  ✅ Price mean ({:.2}) is reasonable", price_mean);
    } else {
        println!("  ❌ Price mean ({:.2}) is unreasonable", price_mean);
    }

    // Check if std dev is positive
    let price_std = final_snapshot.price_stats.rolling.std_dev;
    if price_std > 0.0 && price_std < 1000.0 {
        println!("  ✅ Price std dev ({:.2}) is reasonable", price_std);
    } else {
        println!("  ❌ Price std dev ({:.2}) is unreasonable", price_std);
    }

    // Check if volume mean is reasonable
    let volume_mean = final_snapshot.volume_stats.rolling.mean;
    if volume_mean > 0.0 && volume_mean < 100.0 {
        println!("  ✅ Volume mean ({:.2}) is reasonable", volume_mean);
    } else {
        println!("  ❌ Volume mean ({:.2}) is unreasonable", volume_mean);
    }

    // Check if percentiles are ordered
    if let (Some(p50), Some(p95)) = (
        final_snapshot.price_stats.percentiles.get("P50"),
        final_snapshot.price_stats.percentiles.get("P95"),
    ) {
        if p50 < p95 {
            println!(
                "  ✅ Percentiles are ordered (P50: {:.2} < P95: {:.2})",
                p50, p95
            );
        } else {
            println!(
                "  ❌ Percentiles are not ordered (P50: {:.2} >= P95: {:.2})",
                p50, p95
            );
        }
    }

    // Check if counts match
    if final_snapshot.trade_count == trades.len() as u64 {
        println!(
            "  ✅ Trade count matches ({} processed)",
            final_snapshot.trade_count
        );
    } else {
        println!(
            "  ❌ Trade count mismatch (expected {}, got {})",
            trades.len(),
            final_snapshot.trade_count
        );
    }

    if final_snapshot.bar_count == range_bars.len() as u64 {
        println!(
            "  ✅ Bar count matches ({} processed)",
            final_snapshot.bar_count
        );
    } else {
        println!(
            "  ❌ Bar count mismatch (expected {}, got {})",
            range_bars.len(),
            final_snapshot.bar_count
        );
    }

    println!("\n🎯 Statistics V2 Module Validation Complete!");
}
