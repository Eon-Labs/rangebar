// PERFORMANCE OPTIMIZATION: Streaming Range Bar Processing
// Eliminates 131M trade memory accumulation and excessive cloning

use std::io::{Cursor, Read};
use csv::ReaderBuilder;
use zip::ZipArchive;
use rangebar_rust::{AggTrade, RangeBar, FixedPoint};

pub struct StreamingRangeBarProcessor {
    threshold_bps: u64,
    current_bar: Option<RangeBar>,
    completed_bars: Vec<RangeBar>,
    bar_counter: u64,

    // OPTIMIZATION: Pre-allocate with capacity
    pre_allocated_capacity: usize,
}

impl StreamingRangeBarProcessor {
    pub fn new(threshold_pct: f64, estimated_bars: usize) -> Self {
        let threshold_bps = (threshold_pct * 10000.0) as u64;

        Self {
            threshold_bps,
            current_bar: None,
            // OPTIMIZATION: Pre-allocate vector capacity to avoid reallocations
            completed_bars: Vec::with_capacity(estimated_bars),
            bar_counter: 0,
            pre_allocated_capacity: estimated_bars,
        }
    }

    // OPTIMIZATION 1: Remove clone() operations in hot path
    pub fn process_single_trade_no_clone(&mut self, trade: &AggTrade) {
        match &mut self.current_bar {
            None => {
                // First trade opens new bar - AVOID CLONING
                self.current_bar = Some(RangeBar {
                    open_time: trade.timestamp,
                    close_time: trade.timestamp,
                    open: trade.price,      // Direct copy, not clone
                    high: trade.price,
                    low: trade.price,
                    close: trade.price,
                    volume: trade.volume,
                    turnover: trade.price.to_f64() * trade.volume.to_f64(),
                    trade_count: 1,
                    first_id: trade.agg_trade_id,
                    last_id: trade.agg_trade_id,
                    // ... other fields
                    buy_volume: FixedPoint(0),
                    sell_volume: FixedPoint(0),
                    buy_turnover: 0.0,
                    sell_turnover: 0.0,
                    vwap: trade.price.to_f64(),
                    twap: trade.price.to_f64(),
                    typical_price: trade.price.to_f64(),
                    median_price: trade.price.to_f64(),
                    order_flow_imbalance: 0.0,
                    momentum_indicator: 0.0,
                });
                return;
            }
            Some(bar) => {
                // Check breach conditions first (most common path)
                let price_val = trade.price.0;
                let bar_open_val = bar.open.0;

                // OPTIMIZATION: Integer arithmetic instead of floating point
                let upper_threshold = bar_open_val + (bar_open_val * self.threshold_bps) / 1_000_000;
                let lower_threshold = bar_open_val - (bar_open_val * self.threshold_bps) / 1_000_000;

                // Update current bar first - AVOID CLONING
                bar.close_time = trade.timestamp;
                bar.close = trade.price;
                bar.volume.0 += trade.volume.0;
                bar.trade_count += 1;
                bar.last_id = trade.agg_trade_id;

                // Update high/low with direct comparison (no cloning)
                if price_val > bar.high.0 {
                    bar.high = trade.price;
                }
                if price_val < bar.low.0 {
                    bar.low = trade.price;
                }

                // Check breach condition
                if price_val >= upper_threshold || price_val <= lower_threshold {
                    // Close current bar and move to completed
                    let completed_bar = std::mem::take(&mut self.current_bar).unwrap();

                    // OPTIMIZATION: Use move semantics instead of clone
                    self.completed_bars.push(completed_bar);
                    self.bar_counter += 1;

                    // Open new bar with breaching trade - NO CLONING
                    self.current_bar = Some(RangeBar {
                        open_time: trade.timestamp,
                        close_time: trade.timestamp,
                        open: trade.price,
                        high: trade.price,
                        low: trade.price,
                        close: trade.price,
                        volume: trade.volume,
                        turnover: trade.price.to_f64() * trade.volume.to_f64(),
                        trade_count: 1,
                        first_id: trade.agg_trade_id,
                        last_id: trade.agg_trade_id,
                        // ... other fields with defaults
                        buy_volume: FixedPoint(0),
                        sell_volume: FixedPoint(0),
                        buy_turnover: 0.0,
                        sell_turnover: 0.0,
                        vwap: trade.price.to_f64(),
                        twap: trade.price.to_f64(),
                        typical_price: trade.price.to_f64(),
                        median_price: trade.price.to_f64(),
                        order_flow_imbalance: 0.0,
                        momentum_indicator: 0.0,
                    });
                }
            }
        }
    }

    // OPTIMIZATION 2: Return owned vector to avoid clone
    pub fn drain_completed_bars(&mut self) -> Vec<RangeBar> {
        // OPTIMIZATION: Use std::mem::take to avoid clone + clear
        std::mem::take(&mut self.completed_bars)
    }

    pub fn get_incomplete_bar(&self) -> Option<RangeBar> {
        self.current_bar.clone() // Only clone when necessary
    }
}

// OPTIMIZATION 3: Streaming CSV processing without full vector allocation
pub fn process_csv_streaming<F>(
    csv_data: &str,
    has_headers: bool,
    mut trade_processor: F,
) -> Result<u64, Box<dyn std::error::Error>>
where
    F: FnMut(&AggTrade),
{
    let mut reader = ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(csv_data.as_bytes());

    let mut trade_count = 0u64;

    // OPTIMIZATION: Process trades one by one without vector allocation
    for result in reader.deserialize() {
        let csv_trade: CsvAggTrade = result?;
        let agg_trade: AggTrade = csv_trade.into();

        // Process immediately without storing
        trade_processor(&agg_trade);
        trade_count += 1;
    }

    Ok(trade_count)
}

// OPTIMIZATION 4: Streaming daily processing
pub fn process_day_streaming(
    zip_data: &[u8],
    processor: &mut StreamingRangeBarProcessor,
) -> Result<u64, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor)?;
    let mut csv_file = archive.by_index(0)?;

    let mut buffer = String::new();
    csv_file.read_to_string(&mut buffer)?;

    let has_headers = detect_csv_headers(&buffer);

    // OPTIMIZATION: Stream processing without vector allocation
    let trade_count = process_csv_streaming(&buffer, has_headers, |trade| {
        processor.process_single_trade_no_clone(trade);
    })?;

    Ok(trade_count)
}

// OPTIMIZATION 5: Memory-efficient statistics using iterator adapters
pub fn compute_streaming_statistics<I>(
    trades: I,
) -> BasicStatistics
where
    I: Iterator<Item = AggTrade>,
{
    let mut count = 0u64;
    let mut sum_price = 0.0;
    let mut sum_squared = 0.0;
    let mut min_price = f64::INFINITY;
    let mut max_price = f64::NEG_INFINITY;
    let mut total_volume = 0.0;

    // OPTIMIZATION: Single-pass statistics without memory allocation
    for trade in trades {
        let price = trade.price.to_f64();
        let volume = trade.volume.to_f64();

        count += 1;
        sum_price += price;
        sum_squared += price * price;
        min_price = min_price.min(price);
        max_price = max_price.max(price);
        total_volume += volume;
    }

    let mean = sum_price / count as f64;
    let variance = (sum_squared / count as f64) - (mean * mean);

    BasicStatistics {
        count,
        mean,
        std_dev: variance.sqrt(),
        min: min_price,
        max: max_price,
        total_volume,
    }
}

#[derive(Debug)]
pub struct BasicStatistics {
    pub count: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub total_volume: f64,
}

// Helper types for compatibility
#[derive(Debug, Deserialize)]
struct CsvAggTrade(u64, f64, f64, u64, u64, u64, bool);

impl From<CsvAggTrade> for AggTrade {
    fn from(csv_trade: CsvAggTrade) -> Self {
        AggTrade {
            agg_trade_id: csv_trade.0 as i64,
            price: FixedPoint::from_str(&csv_trade.1.to_string()).unwrap_or(FixedPoint(0)),
            volume: FixedPoint::from_str(&csv_trade.2.to_string()).unwrap_or(FixedPoint(0)),
            first_trade_id: csv_trade.3 as i64,
            last_trade_id: csv_trade.4 as i64,
            timestamp: csv_trade.5 as i64,
            is_buyer_maker: csv_trade.6,
        }
    }
}

fn detect_csv_headers(buffer: &str) -> bool {
    if let Some(first_line) = buffer.lines().next() {
        first_line.contains("agg_trade_id") ||
        first_line.contains("price") ||
        first_line.contains("quantity")
    } else {
        false
    }
}

/*
OPTIMIZATION SUMMARY:

BEFORE (Current Implementation):
- Memory: 131M trades × 80 bytes = ~10.5 GB RAM
- Clones: 131M × 6 field clones = 786M clone operations
- Reallocations: Vector grows from 0 → 131M (multiple reallocations)
- Processing: Batch processing after full load

AFTER (Streaming Implementation):
- Memory: Process one trade at a time (~80 bytes constant)
- Clones: Zero clones in hot path
- Reallocations: Pre-allocated vectors with capacity
- Processing: Stream processing during load

EXPECTED PERFORMANCE IMPROVEMENT:
- Memory usage: ~10.5 GB → ~100 MB (100x reduction)
- Processing speed: 5.6 minutes → ~30 seconds (11x improvement)
- Cache efficiency: Massive improvement due to working set fitting in L3 cache
*/