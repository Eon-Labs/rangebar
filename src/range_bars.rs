//! Core range bar processing algorithm
//!
//! Implements non-lookahead bias range bar construction where bars close when
//! price moves ±threshold% from the bar's OPEN price.

use crate::fixed_point::FixedPoint;
use crate::types::{AggTrade, RangeBar};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use thiserror::Error;

/// Range bar processor with non-lookahead bias guarantee
pub struct RangeBarProcessor {
    /// Threshold in basis points (8000 = 0.8%)
    threshold_bps: u32,
}

impl RangeBarProcessor {
    /// Create new processor with given threshold
    ///
    /// # Arguments
    ///
    /// * `threshold_bps` - Threshold in basis points (8000 = 0.8%)
    pub fn new(threshold_bps: u32) -> Self {
        Self { threshold_bps }
    }

    /// Process a single trade and return completed bar if any
    ///
    /// # Arguments
    ///
    /// * `trade` - Single aggregated trade to process
    ///
    /// # Returns
    ///
    /// `Some(RangeBar)` if a bar was completed, `None` otherwise
    pub fn process_single_trade(
        &mut self,
        trade: AggTrade,
    ) -> Result<Option<RangeBar>, ProcessingError> {
        // Create single-element slice and process
        let trades = vec![trade];
        let mut bars = self.process_trades(&trades)?;

        // Return the last completed bar if any
        if bars.is_empty() {
            Ok(None)
        } else {
            Ok(bars.pop())
        }
    }

    /// Get any incomplete bar currently being processed
    pub fn get_incomplete_bar(&self) -> Option<RangeBar> {
        // This would need internal state tracking - simplified for now
        None
    }

    /// Process trades into range bars
    ///
    /// # Arguments
    ///
    /// * `trades` - Slice of aggregated trades sorted by (timestamp, agg_trade_id)
    ///
    /// # Returns
    ///
    /// Vector of completed range bars
    pub fn process_trades(
        &mut self,
        trades: &[AggTrade],
    ) -> Result<Vec<RangeBar>, ProcessingError> {
        if trades.is_empty() {
            return Ok(Vec::new());
        }

        // Validate trades are sorted
        self.validate_trade_ordering(trades)?;

        let mut bars = Vec::with_capacity(trades.len() / 100); // Heuristic capacity
        let mut current_bar: Option<RangeBarState> = None;
        let mut defer_open = false;

        for trade in trades {
            if defer_open {
                // Previous bar closed, this trade opens new bar
                current_bar = Some(RangeBarState::new(trade, self.threshold_bps));
                defer_open = false;
                continue;
            }

            match current_bar {
                None => {
                    // First bar initialization
                    current_bar = Some(RangeBarState::new(trade, self.threshold_bps));
                }
                Some(ref mut bar_state) => {
                    // CORRECT ALGORITHM: Always update high/low with actual prices first
                    if trade.price > bar_state.bar.high {
                        bar_state.bar.high = trade.price;
                    }
                    if trade.price < bar_state.bar.low {
                        bar_state.bar.low = trade.price;
                    }

                    // Check if this trade breaches the threshold
                    if bar_state.bar.is_breach(
                        trade.price,
                        bar_state.upper_threshold,
                        bar_state.lower_threshold,
                    ) {
                        // Breach detected - close bar with ACTUAL breaching trade price
                        bar_state.bar.close = trade.price; // Actual price, not threshold!
                        bar_state.bar.volume = FixedPoint(bar_state.bar.volume.0 + trade.volume.0);
                        bar_state.bar.trade_count += 1;
                        bar_state.bar.close_time = trade.timestamp;

                        // Validation: Ensure high/low include open/close extremes
                        debug_assert!(
                            bar_state.bar.high >= bar_state.bar.open.max(bar_state.bar.close)
                        );
                        debug_assert!(
                            bar_state.bar.low <= bar_state.bar.open.min(bar_state.bar.close)
                        );

                        bars.push(bar_state.bar.clone());
                        current_bar = None;
                        defer_open = true; // Next trade will open new bar
                    } else {
                        // No breach: normal update (excluding high/low which were already updated)
                        bar_state.bar.close = trade.price;
                        bar_state.bar.volume = FixedPoint(bar_state.bar.volume.0 + trade.volume.0);
                        bar_state.bar.trade_count += 1;
                        bar_state.bar.close_time = trade.timestamp;
                    }
                }
            }
        }

        // Add final partial bar if it exists
        if let Some(bar_state) = current_bar {
            bars.push(bar_state.bar);
        }

        Ok(bars)
    }

    /// Validate that trades are properly sorted for deterministic processing
    fn validate_trade_ordering(&self, trades: &[AggTrade]) -> Result<(), ProcessingError> {
        for i in 1..trades.len() {
            let prev = &trades[i - 1];
            let curr = &trades[i];

            // Check ordering: (timestamp, agg_trade_id) ascending
            if curr.timestamp < prev.timestamp
                || (curr.timestamp == prev.timestamp && curr.agg_trade_id <= prev.agg_trade_id)
            {
                return Err(ProcessingError::UnsortedTrades {
                    index: i,
                    prev_time: prev.timestamp,
                    prev_id: prev.agg_trade_id,
                    curr_time: curr.timestamp,
                    curr_id: curr.agg_trade_id,
                });
            }
        }

        Ok(())
    }
}

/// Internal state for a range bar being built
struct RangeBarState {
    /// The range bar being constructed
    pub bar: RangeBar,

    /// Upper breach threshold (FIXED from bar open)
    pub upper_threshold: FixedPoint,

    /// Lower breach threshold (FIXED from bar open)
    pub lower_threshold: FixedPoint,
}

impl RangeBarState {
    /// Create new range bar state from opening trade
    fn new(trade: &AggTrade, threshold_bps: u32) -> Self {
        let bar = RangeBar::new(trade);

        // Compute FIXED thresholds from opening price
        let (upper_threshold, lower_threshold) = bar.open.compute_range_thresholds(threshold_bps);

        Self {
            bar,
            upper_threshold,
            lower_threshold,
        }
    }
}

/// Processing errors
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error(
        "Trades not sorted at index {index}: prev=({prev_time}, {prev_id}), curr=({curr_time}, {curr_id})"
    )]
    UnsortedTrades {
        index: usize,
        prev_time: i64,
        prev_id: i64,
        curr_time: i64,
        curr_id: i64,
    },

    #[error("Empty trade data")]
    EmptyData,

    #[error("Invalid threshold: {threshold_bps} basis points")]
    InvalidThreshold { threshold_bps: u32 },
}

#[cfg(feature = "python")]
impl From<ProcessingError> for PyErr {
    fn from(err: ProcessingError) -> PyErr {
        match err {
            ProcessingError::UnsortedTrades {
                index,
                prev_time,
                prev_id,
                curr_time,
                curr_id,
            } => pyo3::exceptions::PyValueError::new_err(format!(
                "Trades not sorted at index {}: prev=({}, {}), curr=({}, {})",
                index, prev_time, prev_id, curr_time, curr_id
            )),
            ProcessingError::EmptyData => {
                pyo3::exceptions::PyValueError::new_err("Empty trade data")
            }
            ProcessingError::InvalidThreshold { threshold_bps } => {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid threshold: {} basis points",
                    threshold_bps
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point::FixedPoint;

    fn create_test_trade(id: i64, price: &str, volume: &str, timestamp: i64) -> AggTrade {
        AggTrade {
            agg_trade_id: id,
            price: FixedPoint::from_str(price).unwrap(),
            volume: FixedPoint::from_str(volume).unwrap(),
            first_trade_id: id * 10,
            last_trade_id: id * 10,
            timestamp,
        }
    }

    #[test]
    fn test_single_bar_no_breach() {
        let mut processor = RangeBarProcessor::new(8000); // 0.8%

        let trades = vec![
            create_test_trade(1, "50000.0", "1.0", 1000),
            create_test_trade(2, "50300.0", "1.5", 2000), // +0.6%
            create_test_trade(3, "49700.0", "2.0", 3000), // -0.6%
        ];

        let bars = processor.process_trades(&trades).unwrap();

        // No breach, so only one partial bar
        assert_eq!(bars.len(), 1);

        let bar = &bars[0];
        assert_eq!(bar.open.to_string(), "50000.00000000");
        assert_eq!(bar.high.to_string(), "50300.00000000");
        assert_eq!(bar.low.to_string(), "49700.00000000");
        assert_eq!(bar.close.to_string(), "49700.00000000");
    }

    #[test]
    fn test_exact_breach_upward() {
        let mut processor = RangeBarProcessor::new(8000); // 0.8%

        let trades = vec![
            create_test_trade(1, "50000.0", "1.0", 1000), // Open
            create_test_trade(2, "50200.0", "1.0", 2000), // +0.4%
            create_test_trade(3, "50400.0", "1.0", 3000), // +0.8% BREACH
            create_test_trade(4, "50500.0", "1.0", 4000), // New bar
        ];

        let bars = processor.process_trades(&trades).unwrap();

        assert_eq!(bars.len(), 2);

        // First bar should close at breach
        let bar1 = &bars[0];
        assert_eq!(bar1.open.to_string(), "50000.00000000");
        assert_eq!(bar1.close.to_string(), "50400.00000000"); // Breach tick included
        assert_eq!(bar1.high.to_string(), "50400.00000000");
        assert_eq!(bar1.low.to_string(), "50000.00000000");

        // Second bar should start at next tick price (not breach price)
        let bar2 = &bars[1];
        assert_eq!(bar2.open.to_string(), "50500.00000000"); // Next tick after breach
        assert_eq!(bar2.close.to_string(), "50500.00000000");
    }

    #[test]
    fn test_exact_breach_downward() {
        let mut processor = RangeBarProcessor::new(8000); // 0.8%

        let trades = vec![
            create_test_trade(1, "50000.0", "1.0", 1000), // Open
            create_test_trade(2, "49800.0", "1.0", 2000), // -0.4%
            create_test_trade(3, "49600.0", "1.0", 3000), // -0.8% BREACH
        ];

        let bars = processor.process_trades(&trades).unwrap();

        assert_eq!(bars.len(), 1);

        let bar = &bars[0];
        assert_eq!(bar.open.to_string(), "50000.00000000");
        assert_eq!(bar.close.to_string(), "49600.00000000"); // Breach tick included
        assert_eq!(bar.high.to_string(), "50000.00000000");
        assert_eq!(bar.low.to_string(), "49600.00000000");
    }

    #[test]
    fn test_large_gap_single_bar() {
        let mut processor = RangeBarProcessor::new(8000); // 0.8%

        let trades = vec![
            create_test_trade(1, "50000.0", "1.0", 1000), // Open
            create_test_trade(2, "51000.0", "1.0", 2000), // +2% gap (single bar)
        ];

        let bars = processor.process_trades(&trades).unwrap();

        // Should create exactly ONE bar, not multiple bars to "fill the gap"
        assert_eq!(bars.len(), 1);

        let bar = &bars[0];
        assert_eq!(bar.open.to_string(), "50000.00000000");
        assert_eq!(bar.close.to_string(), "51000.00000000");
        assert_eq!(bar.high.to_string(), "51000.00000000");
        assert_eq!(bar.low.to_string(), "50000.00000000");
    }

    #[test]
    fn test_unsorted_trades_error() {
        let mut processor = RangeBarProcessor::new(8000);

        let trades = vec![
            create_test_trade(1, "50000.0", "1.0", 2000), // Later timestamp first
            create_test_trade(2, "50100.0", "1.0", 1000), // Earlier timestamp second
        ];

        let result = processor.process_trades(&trades);
        assert!(result.is_err());

        match result {
            Err(ProcessingError::UnsortedTrades { index, .. }) => {
                assert_eq!(index, 1);
            }
            _ => panic!("Expected UnsortedTrades error"),
        }
    }

    #[test]
    fn test_threshold_calculation() {
        let processor = RangeBarProcessor::new(8000); // 0.8%

        let trade = create_test_trade(1, "50000.0", "1.0", 1000);
        let bar_state = RangeBarState::new(&trade, processor.threshold_bps);

        // 50000 * 0.008 = 400
        assert_eq!(bar_state.upper_threshold.to_string(), "50400.00000000");
        assert_eq!(bar_state.lower_threshold.to_string(), "49600.00000000");
    }

    #[test]
    fn test_empty_trades() {
        let mut processor = RangeBarProcessor::new(8000);
        let bars = processor.process_trades(&[]).unwrap();
        assert_eq!(bars.len(), 0);
    }
}
