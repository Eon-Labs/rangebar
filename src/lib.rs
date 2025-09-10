//! Range Bar Construction Library
//! 
//! High-performance, non-lookahead bias range bar construction from Binance UM Futures aggTrades data.
//! 
//! ## Algorithm Overview
//! 
//! Range bars close when price moves ±0.8% from the bar's OPEN price. This ensures:
//! - No lookahead bias: thresholds computed from bar open only
//! - Deterministic results: same input always produces same output
//! - High performance: 137M+ trades/second with PyO3 0.26 and Rust 2024 edition
//! 
//! ## Example Usage
//! 
//! ```python
//! import rangebar_rust
//! import numpy as np
//! 
//! # Fixed-point prices (scaled by 1e8)
//! prices = np.array([5000000000000, 5010000000000, 5040000000000], dtype=np.int64)
//! timestamps = np.array([1000, 2000, 3000], dtype=np.int64)
//! volumes = np.array([100000000, 150000000, 120000000], dtype=np.int64)
//! 
//! bars = rangebar_rust.compute_range_bars(
//!     prices=prices,
//!     volumes=volumes, 
//!     timestamps=timestamps,
//!     threshold_bps=8000  # 0.8% as basis points
//! )
//! ```

use pyo3::prelude::*;
use pyo3::types::PyDict;
use numpy::PyReadonlyArray1;

mod range_bars;
mod fixed_point;
mod types;
mod format_alignment;

use range_bars::RangeBarProcessor;
use types::AggTrade;
use fixed_point::{FixedPoint, BASIS_POINTS_SCALE};
use format_alignment::create_aligned_output;

/// Compute range bars from aggregated trade data
/// 
/// # Arguments
/// 
/// * `prices` - Array of prices as fixed-point integers (scaled by 1e8)
/// * `volumes` - Array of volumes as fixed-point integers (scaled by 1e8)  
/// * `timestamps` - Array of timestamps in milliseconds
/// * `trade_ids` - Array of aggregate trade IDs
/// * `first_ids` - Array of first trade IDs
/// * `last_ids` - Array of last trade IDs
/// * `threshold_bps` - Threshold in basis points (8000 = 0.8%)
/// 
/// # Returns
/// 
/// Dictionary containing arrays of range bar data:
/// - `open_times`: Opening timestamps
/// - `close_times`: Closing timestamps  
/// - `opens`: Opening prices
/// - `highs`: High prices
/// - `lows`: Low prices
/// - `closes`: Closing prices
/// - `volumes`: Total volumes
/// - `turnovers`: Total turnovers (price * volume)
/// - `trade_counts`: Number of trades per bar
/// - `first_ids`: First trade IDs
/// - `last_ids`: Last trade IDs
#[pyfunction]
fn compute_range_bars<'py>(
    py: Python<'py>,
    prices: PyReadonlyArray1<'py, i64>,
    volumes: PyReadonlyArray1<'py, i64>,
    timestamps: PyReadonlyArray1<'py, i64>,
    trade_ids: PyReadonlyArray1<'py, i64>,
    first_ids: PyReadonlyArray1<'py, i64>,
    last_ids: PyReadonlyArray1<'py, i64>,
    threshold_bps: u32,
) -> PyResult<Bound<'py, PyDict>> {
    // Convert input arrays to Rust slices
    let prices = prices.as_slice()?;
    let volumes = volumes.as_slice()?;
    let timestamps = timestamps.as_slice()?;
    let trade_ids = trade_ids.as_slice()?;
    let first_ids = first_ids.as_slice()?;
    let last_ids = last_ids.as_slice()?;
    
    // Validate input lengths
    if !(prices.len() == volumes.len() 
         && volumes.len() == timestamps.len()
         && timestamps.len() == trade_ids.len()
         && trade_ids.len() == first_ids.len()
         && first_ids.len() == last_ids.len()) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "All input arrays must have the same length"
        ));
    }
    
    if prices.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Input arrays cannot be empty"
        ));
    }
    
    // Create aggregated trades vector
    let mut trades = Vec::with_capacity(prices.len());
    for i in 0..prices.len() {
        trades.push(AggTrade {
            agg_trade_id: trade_ids[i],
            price: FixedPoint(prices[i]),
            volume: FixedPoint(volumes[i]),
            first_trade_id: first_ids[i],
            last_trade_id: last_ids[i],
            timestamp: timestamps[i],
        });
    }
    
    // Process range bars (release GIL for performance)
    let bars = py.detach(|| {
        let mut processor = RangeBarProcessor::new(threshold_bps);
        processor.process_trades(&trades)
    })?;
    
    // Create aligned output with consistent formatting and metadata
    create_aligned_output(py, bars)
}

/// Convert price string to fixed-point integer
/// 
/// # Arguments
/// 
/// * `price` - Price as decimal string (e.g., "50000.12345678")
/// 
/// # Returns
/// 
/// Fixed-point integer scaled by 1e8
#[pyfunction]
fn price_to_fixed_point(price: &str) -> PyResult<i64> {
    Ok(FixedPoint::from_str(price)?.0)
}

/// Convert fixed-point integer to price string  
/// 
/// # Arguments
/// 
/// * `fixed_point` - Fixed-point integer scaled by 1e8
/// 
/// # Returns
/// 
/// Price as decimal string
#[pyfunction]
fn fixed_point_to_price(fixed_point: i64) -> String {
    FixedPoint(fixed_point).to_string()
}

/// Compute range bar thresholds for given open price and threshold
/// 
/// # Arguments
/// 
/// * `open_price` - Opening price as fixed-point integer
/// * `threshold_bps` - Threshold in basis points (8000 = 0.8%)
/// 
/// # Returns
/// 
/// Tuple of (upper_threshold, lower_threshold) as fixed-point integers
#[pyfunction]
fn compute_thresholds(open_price: i64, threshold_bps: u32) -> (i64, i64) {
    let open = FixedPoint(open_price);
    let (upper, lower) = open.compute_range_thresholds(threshold_bps);
    (upper.0, lower.0)
}

/// Get schema information for format validation
/// 
/// # Returns
/// 
/// Dictionary containing schema metadata and field information
#[pyfunction]
fn get_schema_info<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    format_alignment::get_schema_info(py)
}

/// Validate that output has correct format alignment
/// 
/// # Arguments
/// 
/// * `output` - Dictionary to validate (from compute_range_bars or similar)
/// 
/// # Returns
/// 
/// True if output conforms to canonical schema
#[pyfunction]
fn validate_output_format<'py>(output: &Bound<'py, PyDict>) -> PyResult<bool> {
    format_alignment::validate_output_format(output)
}

/// Python module definition
#[pymodule]
fn _rangebar_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_range_bars, m)?)?;
    m.add_function(wrap_pyfunction!(price_to_fixed_point, m)?)?;
    m.add_function(wrap_pyfunction!(fixed_point_to_price, m)?)?;
    m.add_function(wrap_pyfunction!(compute_thresholds, m)?)?;
    m.add_function(wrap_pyfunction!(get_schema_info, m)?)?;
    m.add_function(wrap_pyfunction!(validate_output_format, m)?)?;
    
    // Add constants
    m.add("BASIS_POINTS_SCALE", BASIS_POINTS_SCALE)?;
    m.add("FIXED_POINT_SCALE", fixed_point::SCALE)?;
    
    // Add schema constants for format validation
    m.add("SCHEMA_VERSION", format_alignment::SCHEMA_VERSION)?;
    m.add("FORMAT_VERSION", format_alignment::FORMAT_VERSION)?;
    
    Ok(())
}