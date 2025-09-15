//! # Rangebar
//!
//! High-performance non-lookahead bias range bar construction for Binance UM Futures data.
//!
//! ## Quick Start
//!
//! ```rust
//! use rangebar::{RangeBarProcessor, AggTrade, FixedPoint};
//!
//! // Create processor with 0.8% threshold (8000 basis points)
//! let mut processor = RangeBarProcessor::new(8000);
//!
//! // Create sample trade data
//! let trade = AggTrade {
//!     agg_trade_id: 123456789,
//!     price: FixedPoint::from_str("50000.12345").unwrap(),
//!     volume: FixedPoint::from_str("1.50000000").unwrap(),
//!     first_trade_id: 100,
//!     last_trade_id: 105,
//!     timestamp: 1609459200000,
//!     is_buyer_maker: false,
//! };
//!
//! // Process trades into range bars
//! let trades = vec![trade];
//! let bars = processor.process_trades(&trades).unwrap();
//!
//! for bar in bars {
//!     println!("Bar: O={} H={} L={} C={} V={}",
//!              bar.open, bar.high, bar.low, bar.close, bar.volume);
//! }
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
//! ## Features
//!
//! - **137M+ trades/second** processing (2025 benchmarks)
//! - **Fixed-point arithmetic** (no floating-point errors)
//! - **Memory efficient** streaming processing
//! - **Comprehensive statistics** via Polars (when `statistics` feature enabled)
//! - **Data integrity** validation and checksums (when `data-integrity` feature enabled)

pub mod fixed_point;
pub mod range_bars;
pub mod types;

#[cfg(feature = "statistics")]
pub mod statistics;

// TODO: Python bindings module (when python-bindings feature is enabled)
// #[cfg(feature = "python-bindings")]
// pub mod python;

// Re-export commonly used types for convenience
pub use fixed_point::FixedPoint;
pub use range_bars::{ProcessingError, RangeBarProcessor};
pub use types::{AggTrade, RangeBar};

#[cfg(feature = "statistics")]
pub use statistics::{
    AlgorithmConfig, DatasetInfo, FormatMetadata, PerformanceMetrics, QualityMetrics,
    RangeBarMetadata, StatisticalConfig, StatisticalEngine, Statistics,
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

    #[cfg(feature = "statistics")]
    #[test]
    fn test_statistics_export() {
        // Test that statistics module is accessible
        let engine = StatisticalEngine::new();
        assert!(engine.config().parallel_computation);
    }
}
