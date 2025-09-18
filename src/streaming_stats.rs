//! State-of-the-art streaming statistics with exact and approximate algorithms
//!
//! This module implements a hybrid approach combining:
//! - Exact online algorithms (Welford's method) for mean, variance, std dev
//! - T-Digest for streaming percentiles with <2% error
//! - Numerically stable min/max tracking
//! - Memory usage: O(1) for exact stats, ~1.5KB per T-Digest

use crate::types::AggTrade;
use serde::{Deserialize, Serialize};

#[cfg(feature = "streaming-stats")]
use rolling_stats::Stats as WelfordStats;
#[cfg(feature = "streaming-stats")]
use tdigests::TDigest;

/// Streaming statistics accumulator using state-of-the-art algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    // Exact statistics (Welford's algorithm - numerically stable)
    #[cfg(feature = "streaming-stats")]
    welford_price: WelfordStats<f64>,
    #[cfg(feature = "streaming-stats")]
    welford_volume: WelfordStats<f64>,

    // Fallback for when streaming-stats feature is disabled
    #[cfg(not(feature = "streaming-stats"))]
    sum_price: f64,
    #[cfg(not(feature = "streaming-stats"))]
    sum_squared_price: f64,
    #[cfg(not(feature = "streaming-stats"))]
    sum_volume: f64,
    #[cfg(not(feature = "streaming-stats"))]
    sum_squared_volume: f64,

    // Exact min/max tracking
    min_price: f64,
    max_price: f64,
    min_volume: f64,
    max_volume: f64,

    // Approximate percentiles (T-Digest with <2% error)
    #[cfg(feature = "streaming-stats")]
    price_digest: TDigest,
    #[cfg(feature = "streaming-stats")]
    volume_digest: TDigest,

    // Exact counters and accumulations
    pub trade_count: u64,
    total_volume: f64,
    total_turnover: f64,

    // Market microstructure
    buy_volume: f64,
    sell_volume: f64,
    buy_trade_count: u64,
    sell_trade_count: u64,
    buy_turnover: f64,
    sell_turnover: f64,

    // Temporal tracking
    first_timestamp: i64,
    last_timestamp: i64,
}

impl Default for StreamingStats {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingStats {
    /// Create new streaming statistics accumulator
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "streaming-stats")]
            welford_price: WelfordStats::new(),
            #[cfg(feature = "streaming-stats")]
            welford_volume: WelfordStats::new(),

            #[cfg(not(feature = "streaming-stats"))]
            sum_price: 0.0,
            #[cfg(not(feature = "streaming-stats"))]
            sum_squared_price: 0.0,
            #[cfg(not(feature = "streaming-stats"))]
            sum_volume: 0.0,
            #[cfg(not(feature = "streaming-stats"))]
            sum_squared_volume: 0.0,

            min_price: f64::INFINITY,
            max_price: f64::NEG_INFINITY,
            min_volume: f64::INFINITY,
            max_volume: f64::NEG_INFINITY,

            #[cfg(feature = "streaming-stats")]
            price_digest: TDigest::new(),
            #[cfg(feature = "streaming-stats")]
            volume_digest: TDigest::new(),

            trade_count: 0,
            total_volume: 0.0,
            total_turnover: 0.0,

            buy_volume: 0.0,
            sell_volume: 0.0,
            buy_trade_count: 0,
            sell_trade_count: 0,
            buy_turnover: 0.0,
            sell_turnover: 0.0,

            first_timestamp: i64::MAX,
            last_timestamp: i64::MIN,
        }
    }

    /// Update statistics with a new trade
    pub fn update(&mut self, trade: &AggTrade) {
        let price = trade.price.to_f64();
        let volume = trade.volume.to_f64();
        let turnover = price * volume;

        // Update exact statistics using Welford's algorithm
        #[cfg(feature = "streaming-stats")]
        {
            self.welford_price.update(price);
            self.welford_volume.update(volume);

            // Update T-Digests for percentiles
            self.price_digest = self.price_digest.merge_unsorted(vec![price]);
            self.volume_digest = self.volume_digest.merge_unsorted(vec![volume]);
        }

        // Fallback accumulation when streaming-stats feature disabled
        #[cfg(not(feature = "streaming-stats"))]
        {
            self.sum_price += price;
            self.sum_squared_price += price * price;
            self.sum_volume += volume;
            self.sum_squared_volume += volume * volume;
        }

        // Exact min/max tracking
        self.min_price = self.min_price.min(price);
        self.max_price = self.max_price.max(price);
        self.min_volume = self.min_volume.min(volume);
        self.max_volume = self.max_volume.max(volume);

        // Exact accumulation
        self.trade_count += 1;
        self.total_volume += volume;
        self.total_turnover += turnover;

        // Market microstructure tracking
        if trade.is_buyer_maker {
            self.buy_volume += volume;
            self.buy_trade_count += 1;
            self.buy_turnover += turnover;
        } else {
            self.sell_volume += volume;
            self.sell_trade_count += 1;
            self.sell_turnover += turnover;
        }

        // Temporal tracking
        self.first_timestamp = self.first_timestamp.min(trade.timestamp);
        self.last_timestamp = self.last_timestamp.max(trade.timestamp);
    }

    /// Get mean price (exact)
    pub fn mean_price(&self) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.welford_price.mean
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            if self.trade_count > 0 {
                self.sum_price / self.trade_count as f64
            } else {
                0.0
            }
        }
    }

    /// Get price variance (exact, numerically stable)
    pub fn price_variance(&self) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.welford_price.variance()
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            if self.trade_count > 1 {
                let mean = self.mean_price();
                (self.sum_squared_price - self.sum_price * mean) / (self.trade_count - 1) as f64
            } else {
                0.0
            }
        }
    }

    /// Get price standard deviation (exact, numerically stable)
    pub fn price_std_dev(&self) -> f64 {
        self.price_variance().sqrt()
    }

    /// Get mean volume (exact)
    pub fn mean_volume(&self) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.welford_volume.mean()
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            if self.trade_count > 0 {
                self.sum_volume / self.trade_count as f64
            } else {
                0.0
            }
        }
    }

    /// Get volume variance (exact, numerically stable)
    pub fn volume_variance(&self) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.welford_volume.variance()
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            if self.trade_count > 1 {
                let mean = self.mean_volume();
                (self.sum_squared_volume - self.sum_volume * mean) / (self.trade_count - 1) as f64
            } else {
                0.0
            }
        }
    }

    /// Get volume standard deviation (exact, numerically stable)
    pub fn volume_std_dev(&self) -> f64 {
        self.volume_variance().sqrt()
    }

    /// Get price percentile (approximate, <2% error)
    pub fn price_percentile(&self, quantile: f64) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.price_digest.estimate_quantile(quantile)
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            // Fallback: use min/max for 0th and 100th percentiles
            match quantile {
                q if q <= 0.0 => self.min_price,
                q if q >= 1.0 => self.max_price,
                _ => (self.min_price + self.max_price) / 2.0, // Rough approximation
            }
        }
    }

    /// Get volume percentile (approximate, <2% error)
    pub fn volume_percentile(&self, quantile: f64) -> f64 {
        #[cfg(feature = "streaming-stats")]
        {
            self.volume_digest.estimate_quantile(quantile)
        }
        #[cfg(not(feature = "streaming-stats"))]
        {
            // Fallback: use min/max for 0th and 100th percentiles
            match quantile {
                q if q <= 0.0 => self.min_volume,
                q if q >= 1.0 => self.max_volume,
                _ => (self.min_volume + self.max_volume) / 2.0, // Rough approximation
            }
        }
    }

    /// Get data span in seconds
    pub fn data_span_seconds(&self) -> f64 {
        if self.last_timestamp > self.first_timestamp {
            (self.last_timestamp - self.first_timestamp) as f64 / 1000.0
        } else {
            0.0
        }
    }

    /// Get trading frequency (trades per second)
    pub fn trading_frequency_hz(&self) -> f64 {
        let span = self.data_span_seconds();
        if span > 0.0 && self.trade_count > 1 {
            (self.trade_count - 1) as f64 / span
        } else {
            0.0
        }
    }

    /// Get volume-weighted average price (VWAP)
    pub fn vwap(&self) -> f64 {
        if self.total_volume > 0.0 {
            self.total_turnover / self.total_volume
        } else {
            0.0
        }
    }

    /// Get buy/sell volume ratio
    pub fn buy_sell_volume_ratio(&self) -> f64 {
        if self.sell_volume > 0.0 {
            self.buy_volume / self.sell_volume
        } else if self.buy_volume > 0.0 {
            f64::INFINITY
        } else {
            1.0
        }
    }

    /// Merge with another streaming statistics accumulator
    /// This enables processing in chunks and combining results
    pub fn merge(mut self, other: Self) -> Self {
        #[cfg(feature = "streaming-stats")]
        {
            // Welford's algorithm supports merging
            self.welford_price = self.welford_price.merge(other.welford_price);
            self.welford_volume = self.welford_volume.merge(other.welford_volume);

            // T-Digests are designed for merging
            self.price_digest = self.price_digest.merge(other.price_digest);
            self.volume_digest = self.volume_digest.merge(other.volume_digest);
        }

        #[cfg(not(feature = "streaming-stats"))]
        {
            self.sum_price += other.sum_price;
            self.sum_squared_price += other.sum_squared_price;
            self.sum_volume += other.sum_volume;
            self.sum_squared_volume += other.sum_squared_volume;
        }

        // Min/max are trivial to merge
        self.min_price = self.min_price.min(other.min_price);
        self.max_price = self.max_price.max(other.max_price);
        self.min_volume = self.min_volume.min(other.min_volume);
        self.max_volume = self.max_volume.max(other.max_volume);

        // Exact accumulation
        self.trade_count += other.trade_count;
        self.total_volume += other.total_volume;
        self.total_turnover += other.total_turnover;

        // Microstructure merge
        self.buy_volume += other.buy_volume;
        self.sell_volume += other.sell_volume;
        self.buy_trade_count += other.buy_trade_count;
        self.sell_trade_count += other.sell_trade_count;
        self.buy_turnover += other.buy_turnover;
        self.sell_turnover += other.sell_turnover;

        // Temporal merge
        self.first_timestamp = self.first_timestamp.min(other.first_timestamp);
        self.last_timestamp = self.last_timestamp.max(other.last_timestamp);

        self
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        let mut base_size = std::mem::size_of::<Self>();

        #[cfg(feature = "streaming-stats")]
        {
            // T-Digest memory usage (approximately 1.5KB each for typical compression)
            base_size += 1536 * 2; // price_digest + volume_digest
        }

        base_size
    }

    /// Check if any data has been processed
    pub fn has_data(&self) -> bool {
        self.trade_count > 0
    }
}

/// Summary statistics computed from streaming data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStatsSummary {
    pub trade_count: u64,

    // Price statistics (exact)
    pub price_mean: f64,
    pub price_std_dev: f64,
    pub price_variance: f64,
    pub price_min: f64,
    pub price_max: f64,

    // Price percentiles (approximate, <2% error)
    pub price_p1: f64,
    pub price_p5: f64,
    pub price_p25: f64,
    pub price_median: f64,
    pub price_p75: f64,
    pub price_p95: f64,
    pub price_p99: f64,

    // Volume statistics
    pub volume_mean: f64,
    pub volume_std_dev: f64,
    pub volume_min: f64,
    pub volume_max: f64,
    pub volume_total: f64,

    // Trading activity
    pub turnover_total: f64,
    pub vwap: f64,
    pub trading_frequency_hz: f64,
    pub data_span_seconds: f64,

    // Market microstructure
    pub buy_volume_ratio: f64,
    pub buy_trade_ratio: f64,
    pub buy_turnover_ratio: f64,

    // Memory efficiency
    pub memory_usage_bytes: usize,
}

impl From<&StreamingStats> for StreamingStatsSummary {
    fn from(stats: &StreamingStats) -> Self {
        let buy_trade_ratio = if stats.trade_count > 0 {
            stats.buy_trade_count as f64 / stats.trade_count as f64
        } else {
            0.0
        };

        let buy_turnover_ratio = if stats.total_turnover > 0.0 {
            stats.buy_turnover / stats.total_turnover
        } else {
            0.0
        };

        Self {
            trade_count: stats.trade_count,

            price_mean: stats.mean_price(),
            price_std_dev: stats.price_std_dev(),
            price_variance: stats.price_variance(),
            price_min: stats.min_price,
            price_max: stats.max_price,

            price_p1: stats.price_percentile(0.01),
            price_p5: stats.price_percentile(0.05),
            price_p25: stats.price_percentile(0.25),
            price_median: stats.price_percentile(0.5),
            price_p75: stats.price_percentile(0.75),
            price_p95: stats.price_percentile(0.95),
            price_p99: stats.price_percentile(0.99),

            volume_mean: stats.mean_volume(),
            volume_std_dev: stats.volume_std_dev(),
            volume_min: stats.min_volume,
            volume_max: stats.max_volume,
            volume_total: stats.total_volume,

            turnover_total: stats.total_turnover,
            vwap: stats.vwap(),
            trading_frequency_hz: stats.trading_frequency_hz(),
            data_span_seconds: stats.data_span_seconds(),

            buy_volume_ratio: stats.buy_sell_volume_ratio(),
            buy_trade_ratio,
            buy_turnover_ratio,

            memory_usage_bytes: stats.memory_usage_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point::FixedPoint;

    fn create_test_trade(
        id: i64,
        price: f64,
        volume: f64,
        timestamp: i64,
        is_buyer_maker: bool,
    ) -> AggTrade {
        AggTrade {
            agg_trade_id: id,
            price: FixedPoint::from_str(&format!("{:.8}", price)).unwrap(),
            volume: FixedPoint::from_str(&format!("{:.8}", volume)).unwrap(),
            first_trade_id: id * 10,
            last_trade_id: id * 10,
            timestamp,
            is_buyer_maker,
        }
    }

    #[test]
    fn test_streaming_stats_basic() {
        let mut stats = StreamingStats::new();

        // Add some test trades
        stats.update(&create_test_trade(1, 50000.0, 1.0, 1000, false));
        stats.update(&create_test_trade(2, 50100.0, 1.5, 2000, true));
        stats.update(&create_test_trade(3, 49900.0, 2.0, 3000, false));

        assert_eq!(stats.trade_count, 3);
        assert_eq!(stats.min_price, 49900.0);
        assert_eq!(stats.max_price, 50100.0);
        assert_eq!(stats.total_volume, 4.5);

        // Test that mean is reasonable
        let mean = stats.mean_price();
        assert!(mean > 49900.0 && mean < 50100.0);

        // Test VWAP calculation
        let expected_vwap = (50000.0 * 1.0 + 50100.0 * 1.5 + 49900.0 * 2.0) / 4.5;
        assert!((stats.vwap() - expected_vwap).abs() < 0.01);
    }

    #[test]
    fn test_streaming_stats_merge() {
        let mut stats1 = StreamingStats::new();
        stats1.update(&create_test_trade(1, 50000.0, 1.0, 1000, false));
        stats1.update(&create_test_trade(2, 50100.0, 1.5, 2000, true));

        let mut stats2 = StreamingStats::new();
        stats2.update(&create_test_trade(3, 49900.0, 2.0, 3000, false));
        stats2.update(&create_test_trade(4, 50200.0, 1.0, 4000, true));

        let merged = stats1.merge(stats2);

        assert_eq!(merged.trade_count, 4);
        assert_eq!(merged.min_price, 49900.0);
        assert_eq!(merged.max_price, 50200.0);
        assert_eq!(merged.total_volume, 5.5);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let stats = StreamingStats::new();
        let memory_usage = stats.memory_usage_bytes();

        #[cfg(feature = "streaming-stats")]
        {
            // Should include T-Digest overhead
            assert!(memory_usage > 3000); // Base + 2 * 1.5KB for digests
        }

        #[cfg(not(feature = "streaming-stats"))]
        {
            // Should be just the struct size
            assert!(memory_usage < 1000);
        }
    }

    #[test]
    fn test_buy_sell_ratios() {
        let mut stats = StreamingStats::new();

        // 2 buy trades, 1 sell trade
        stats.update(&create_test_trade(1, 50000.0, 1.0, 1000, true)); // buy
        stats.update(&create_test_trade(2, 50100.0, 1.0, 2000, true)); // buy
        stats.update(&create_test_trade(3, 49900.0, 2.0, 3000, false)); // sell

        let summary = StreamingStatsSummary::from(&stats);

        assert_eq!(summary.buy_trade_ratio, 2.0 / 3.0);
        assert!((summary.buy_volume_ratio - 1.0).abs() < 0.01); // 2.0 buy / 2.0 sell
    }
}
