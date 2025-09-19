//! Non-lookahead range bar construction for cryptocurrency trading.
//!
//! This crate provides algorithms for constructing range bars from trade data
//! with temporal integrity guarantees, ensuring no lookahead bias in financial backtesting.
//!
//! ## Features
//!
//! - Non-lookahead bias range bar construction
//! - Fixed-point arithmetic for precision
//! - Streaming and batch processing modes
//! - Tier-1 cryptocurrency symbol discovery
//! - Pure Rust implementation
//!
//! ## Basic Usage
//!
//! ```rust
//! use rangebar::{RangeBarProcessor, AggTrade, FixedPoint};
//!
//! // Create processor with 250 basis points threshold
//! let mut processor = RangeBarProcessor::new(250);
//!
//! // Create sample trade
//! let trade = AggTrade {
//!     agg_trade_id: 1,
//!     price: FixedPoint::from_str("50000.0").unwrap(),
//!     volume: FixedPoint::from_str("1.0").unwrap(),
//!     first_trade_id: 1,
//!     last_trade_id: 1,
//!     timestamp: 1609459200000,
//!     is_buyer_maker: false,
//! };
//!
//! // Process trades into range bars
//! let trades = vec![trade];
//! let bars = processor.process_trades(&trades).unwrap();
//! ```
//!
//! ## Tier-1 Symbols
//!
//! ```rust
//! use rangebar::{is_tier1_symbol, get_tier1_symbols};
//!
//! // Check if a symbol is Tier-1 (available across all Binance futures markets)
//! assert!(is_tier1_symbol("BTC"));
//! assert!(is_tier1_symbol("ETH"));
//! assert!(!is_tier1_symbol("SHIB"));
//!
//! // Get all Tier-1 symbols
//! let symbols = get_tier1_symbols();
//! assert_eq!(symbols.len(), 18);
//! ```
//!
//! ## Algorithm
//!
//! Range bars close when price moves ±threshold% from the bar's **opening price**:
//!
//! 1. **Non-lookahead bias**: Thresholds computed only from bar open price
//! 2. **Breach inclusion**: Breaching trade included in closing bar
//! 3. **Fixed thresholds**: Never recalculated during bar lifetime
//!

pub mod config;
pub mod fixed_point;
pub mod range_bars;
pub mod range_bars_debug;
pub mod tier1;
pub mod types;

#[cfg(feature = "statistics")]
pub mod statistics;

// Streaming statistics are now part of the main statistics module

// Production-ready streaming architecture (bounded memory, backpressure, circuit breaker)
pub mod streaming_processor;

#[cfg(feature = "api")]
pub mod api;

// TODO: Python bindings module (when python-bindings feature is enabled)
// #[cfg(feature = "python-bindings")]
// pub mod python;

// Re-export commonly used types for convenience
pub use config::Settings;
pub use fixed_point::FixedPoint;
pub use range_bars::{ExportRangeBarProcessor, ProcessingError, RangeBarProcessor};
pub use tier1::{TIER1_SYMBOLS, get_tier1_symbols, get_tier1_usdt_pairs, is_tier1_symbol};
pub use types::{AggTrade, RangeBar};

// Legacy statistics exports removed - now use streaming-stats feature

#[cfg(feature = "streaming-stats")]
pub use statistics::{
    BarStats, OhlcStatistics, PriceStatistics, RollingStats, StatisticsSnapshot,
    StreamingStatsEngine, TradeStats, VolumeStatistics,
};

// Streaming processor exports
pub use streaming_processor::{
    MetricsSummary, RangeBarStream, StreamingError, StreamingMetrics, StreamingProcessor,
    StreamingProcessorConfig,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Library initialization and configuration
pub fn init() {
    // Future: Initialize logging, metrics, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
        assert!(!DESCRIPTION.is_empty());
    }

    #[test]
    fn test_types_export() {
        // Test that we can create and use exported types
        let fp = FixedPoint::from_str("123.456").unwrap();
        assert_eq!(fp.to_string(), "123.45600000");
    }

    // Legacy statistics test disabled
    // #[cfg(feature = "statistics")]
    // #[test]
    // fn test_statistics_export() {
    //     // Test that statistics module is accessible
    //     let engine = StatisticalEngine::new();
    //     assert!(engine.config().parallel_computation);
    // }
}
