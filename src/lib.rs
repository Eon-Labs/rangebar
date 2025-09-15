//! # Rangebar - High-Performance Range Bar Construction & Tier-1 Analysis
//!
//! Complete solution for non-lookahead bias range bar construction and multi-market cryptocurrency analysis.
//!
//! ## Core Features
//!
//! - 🚀 **137M+ trades/second** range bar processing
//! - 📊 **Tier-1 symbol discovery** across Binance futures markets
//! - 🔒 **Non-lookahead bias** temporal integrity guarantee
//! - ⚡ **Fixed-point arithmetic** (no floating-point errors)
//! - 🔧 **Pure Rust** implementation with optional Python bindings
//!
//! ## Binaries
//!
//! ### `tier1-symbol-discovery`
//!
//! Discovers Tier-1 cryptocurrency symbols available across **all three** Binance futures markets:
//!
//! - **UM Futures USDT-margined**: BTCUSDT, ETHUSDT, etc.
//! - **UM Futures USDC-margined**: BTCUSDC, ETHUSDC, etc.
//! - **CM Futures Coin-margined**: BTCUSD_PERP, ETHUSD_PERP, etc.
//!
//! ```bash
//! # Discover all Tier-1 symbols (comprehensive database)
//! cargo run --bin tier1-symbol-discovery
//!
//! # Generate minimal output for pipeline integration
//! cargo run --bin tier1-symbol-discovery -- --format minimal
//!
//! # Include single-market symbols in analysis
//! cargo run --bin tier1-symbol-discovery -- --include-single-market
//!
//! # Add custom suffix to output files
//! cargo run --bin tier1-symbol-discovery -- --custom-suffix range_bar_ready
//! ```
//!
//! **Output**: Machine-discoverable JSON databases in `output/symbol_analysis/current/`
//! **Pipeline Integration**: `/tmp/tier1_usdt_pairs.txt` for downstream processing
//!
//! ### `rangebar-analyze`
//!
//! Parallel range bar analysis across all Tier-1 symbols using Rayon:
//!
//! ```bash
//! # Requires: /tmp/range_bar_analysis_config.json (configuration)
//! # Consumes: /tmp/tier1_usdt_pairs.txt (from tier1-symbol-discovery)
//! cargo run --bin rangebar-analyze
//! ```
//!
//! ### `rangebar-export`
//!
//! Export range bar data for visualization and analysis:
//!
//! ```bash
//! cargo run --bin rangebar-export -- --help
//! ```
//!
//! ## Library Usage
//!
//! ### Tier-1 Symbol Discovery
//!
//! ```rust
//! use rangebar::{is_tier1_symbol, get_tier1_symbols, get_tier1_usdt_pairs, TIER1_SYMBOLS};
//!
//! // Check if a symbol is Tier-1
//! assert!(is_tier1_symbol("BTC"));
//! assert!(is_tier1_symbol("ETH"));
//! assert!(!is_tier1_symbol("SHIB"));
//!
//! // Get all Tier-1 symbols (18 total)
//! let symbols = get_tier1_symbols();
//! println!("Tier-1 symbols: {:?}", symbols);
//!
//! // Get USDT perpetual pairs for Tier-1 symbols
//! let usdt_pairs = get_tier1_usdt_pairs();
//! println!("USDT pairs: {:?}", usdt_pairs); // ["BTCUSDT", "ETHUSDT", ...]
//!
//! // Access the constant array directly
//! println!("Total Tier-1 symbols: {}", TIER1_SYMBOLS.len()); // 18
//! ```
//!
//! ### Range Bar Processing
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
//! ### Combined Usage: Tier-1 Range Bar Analysis
//!
//! ```rust
//! use rangebar::{RangeBarProcessor, get_tier1_usdt_pairs};
//!
//! // Process range bars for all Tier-1 USDT pairs
//! let tier1_pairs = get_tier1_usdt_pairs();
//!
//! for pair in tier1_pairs {
//!     println!("Processing range bars for {}...", pair);
//!     let mut processor = RangeBarProcessor::new(8000); // 0.8% threshold
//!
//!     // Load trades for this pair and process
//!     // let trades = load_binance_aggtrades(&pair);
//!     // let bars = processor.process_trades(&trades).unwrap();
//!     // analyze_bars(&bars);
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
//! ## Tier-1 Instruments Definition
//!
//! **Tier-1 instruments** are cryptocurrency assets that Binance lists across **ALL THREE** futures markets:
//!
//! 1. **UM Futures (USDT-margined)**: Traditional perpetual contracts
//! 2. **UM Futures (USDC-margined)**: Stablecoin-margined perpetuals
//! 3. **CM Futures (Coin-margined)**: Inverse perpetual contracts
//!
//! **Current Count**: 18 Tier-1 symbols (BTC, ETH, SOL, ADA, AVAX, etc.)
//!
//! **Key Characteristics**:
//! - Multi-market availability indicates Binance's highest confidence
//! - Suitable for cross-market arbitrage and reliability analysis
//! - Premium liquidity and institutional interest
//!
//! ## Performance Benchmarks
//!
//! - **Range Bar Processing**: 137M+ trades/second
//! - **Symbol Discovery**: 577 UM + 59 CM symbols in ~1 second
//! - **Memory Efficiency**: Fixed-point arithmetic, optimized data structures
//! - **CPU Efficiency**: 42% more efficient than Python implementations

pub mod fixed_point;
pub mod range_bars;
pub mod tier1;
pub mod types;

#[cfg(feature = "statistics")]
pub mod statistics;

// TODO: Python bindings module (when python-bindings feature is enabled)
// #[cfg(feature = "python-bindings")]
// pub mod python;

// Re-export commonly used types for convenience
pub use fixed_point::FixedPoint;
pub use range_bars::{ProcessingError, RangeBarProcessor};
pub use tier1::{TIER1_SYMBOLS, get_tier1_symbols, get_tier1_usdt_pairs, is_tier1_symbol};
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
