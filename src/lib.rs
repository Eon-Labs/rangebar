//! Rangebar Rust Library
//!
//! High-performance range bar construction with comprehensive statistical analysis
//! and market microstructure metrics for financial time series data.

pub mod fixed_point;
pub mod range_bars;
pub mod types;

#[cfg(feature = "statistics")]
pub mod statistics;

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
