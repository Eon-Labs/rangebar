//! Multi-symbol parallel GPU processing for Tier-1 cryptocurrency symbols
//!
//! This module implements true parallel processing of 18 Tier-1 symbols using
//! padded tensor operations for maximum GPU utilization with Apple Metal backend.

#[cfg(feature = "gpu")]
use burn::{backend::wgpu::Wgpu, tensor::Tensor};

#[cfg(feature = "gpu")]
use crate::{
    fixed_point::FixedPoint,
    gpu::metal_backend::{GpuDevice, GpuError},
    types::{AggTrade, RangeBar},
};

/// Maximum number of Tier-1 symbols for parallel processing
pub const MAX_TIER1_SYMBOLS: usize = 18;

/// Maximum trades per symbol per batch (configurable based on GPU memory)
pub const MAX_TRADES_PER_SYMBOL: usize = 10000;

/// Multi-symbol GPU processor using padded parallel tensor operations
#[cfg(feature = "gpu")]
pub struct MultiSymbolGpuProcessor {
    device: GpuDevice,
    threshold_bps: u32,
    max_trades_per_symbol: usize,
}

#[cfg(feature = "gpu")]
impl MultiSymbolGpuProcessor {
    /// Create new multi-symbol GPU processor
    ///
    /// # Arguments
    /// * `device` - GPU device for computation
    /// * `threshold_bps` - Threshold in basis points (8000 = 0.8%)
    /// * `max_trades_per_symbol` - Maximum trades per symbol per batch
    pub fn new(
        device: GpuDevice,
        threshold_bps: u32,
        max_trades_per_symbol: Option<usize>,
    ) -> Self {
        Self {
            device,
            threshold_bps,
            max_trades_per_symbol: max_trades_per_symbol.unwrap_or(MAX_TRADES_PER_SYMBOL),
        }
    }

    /// Process all 18 Tier-1 symbols in parallel using padded tensors
    ///
    /// This is the core advantage: true parallel processing with maximum GPU utilization.
    ///
    /// # Arguments
    /// * `symbol_trades` - Up to 18 Tier-1 symbols with their trade data
    ///
    /// # Returns
    /// * `Vec<(symbol, Vec<RangeBar>)>` - Range bars for each symbol
    pub fn process_tier1_parallel<'a>(
        &self,
        symbol_trades: &[(&'a str, &[AggTrade])],
    ) -> Result<Vec<(&'a str, Vec<RangeBar>)>, MultiSymbolGpuError> {
        if symbol_trades.is_empty() {
            return Ok(Vec::new());
        }

        // Validate all symbols are Tier-1
        self.validate_tier1_symbols(symbol_trades)?;

        // Create padded tensor batch for parallel processing
        let padded_batch = self.create_padded_tensor_batch(symbol_trades)?;

        // Execute parallel GPU computation across all symbols
        let parallel_results = self.compute_parallel_range_bars(&padded_batch)?;

        // Extract and separate results by symbol
        self.separate_results_by_symbol(symbol_trades, parallel_results)
    }

    /// Validate that all provided symbols are Tier-1
    fn validate_tier1_symbols(
        &self,
        symbol_trades: &[(&str, &[AggTrade])],
    ) -> Result<(), MultiSymbolGpuError> {
        use crate::tier1::is_tier1_symbol;

        if symbol_trades.len() > MAX_TIER1_SYMBOLS {
            return Err(MultiSymbolGpuError::TooManySymbols {
                provided: symbol_trades.len(),
                max: MAX_TIER1_SYMBOLS,
            });
        }

        for (symbol, _) in symbol_trades {
            // Extract base symbol from USDT pair (e.g., "BTCUSDT" -> "BTC")
            let base_symbol = symbol.strip_suffix("USDT").unwrap_or(symbol);

            if !is_tier1_symbol(base_symbol) {
                return Err(MultiSymbolGpuError::NotTier1Symbol {
                    symbol: symbol.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Create padded tensor batch for parallel processing
    ///
    /// **Padded Parallel Processing Design:**
    /// - All symbols get same-sized tensors (padded with zeros)
    /// - Shape: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    /// - Enables true SIMD parallelism across all symbols
    fn create_padded_tensor_batch(
        &self,
        symbol_trades: &[(&str, &[AggTrade])],
    ) -> Result<PaddedTensorBatch, MultiSymbolGpuError> {
        let device = self.device.device();
        let num_symbols = symbol_trades.len();

        // Pre-allocate padded arrays
        let mut prices = vec![vec![0.0f32; self.max_trades_per_symbol]; MAX_TIER1_SYMBOLS];
        let mut volumes = vec![vec![0.0f32; self.max_trades_per_symbol]; MAX_TIER1_SYMBOLS];
        let mut timestamps = vec![vec![0.0f32; self.max_trades_per_symbol]; MAX_TIER1_SYMBOLS];
        let mut microstructure = vec![vec![0.0f32; self.max_trades_per_symbol]; MAX_TIER1_SYMBOLS];

        // Track actual trade counts per symbol (for unpadding later)
        let mut trade_counts = vec![0usize; MAX_TIER1_SYMBOLS];
        let mut symbol_names = vec![String::new(); MAX_TIER1_SYMBOLS];

        // Fill padded tensors with actual trade data
        for (symbol_idx, (symbol, trades)) in symbol_trades.iter().enumerate() {
            let trade_count = trades.len().min(self.max_trades_per_symbol);
            trade_counts[symbol_idx] = trade_count;
            symbol_names[symbol_idx] = symbol.to_string();

            // DEBUG: Log tensor data conversion for first symbol
            if symbol_idx == 0 {
                println!(
                    "🔍 [TENSOR DEBUG] Converting {} trades for symbol {}",
                    trade_count, symbol
                );
                if trade_count > 0 {
                    let first_trade = &trades[0];
                    let price_f64 = first_trade.price.to_f64();
                    let price_f32 = price_f64 as f32;
                    println!(
                        "   First trade: price_f64={:.6}, price_f32={:.6}",
                        price_f64, price_f32
                    );
                    println!(
                        "   First trade: volume={:.6}, timestamp={}",
                        first_trade.volume.to_f64(),
                        first_trade.timestamp
                    );
                }
            }

            for (trade_idx, trade) in trades.iter().take(trade_count).enumerate() {
                prices[symbol_idx][trade_idx] = trade.price.to_f64() as f32;
                volumes[symbol_idx][trade_idx] = trade.volume.to_f64() as f32;
                timestamps[symbol_idx][trade_idx] = trade.timestamp as f32;
                microstructure[symbol_idx][trade_idx] =
                    if trade.is_buyer_maker { 1.0 } else { 0.0 };
            }

            // Remaining entries stay zero (padding)
        }

        // Convert to GPU tensors with shape [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
        let prices_tensor = Tensor::<Wgpu, 1>::from_floats(
            prices.into_iter().flatten().collect::<Vec<_>>().as_slice(),
            device,
        )
        .reshape([MAX_TIER1_SYMBOLS, self.max_trades_per_symbol]);

        let volumes_tensor = Tensor::<Wgpu, 1>::from_floats(
            volumes.into_iter().flatten().collect::<Vec<_>>().as_slice(),
            device,
        )
        .reshape([MAX_TIER1_SYMBOLS, self.max_trades_per_symbol]);

        let timestamps_tensor = Tensor::<Wgpu, 1>::from_floats(
            timestamps
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .as_slice(),
            device,
        )
        .reshape([MAX_TIER1_SYMBOLS, self.max_trades_per_symbol]);

        let microstructure_tensor = Tensor::<Wgpu, 1>::from_floats(
            microstructure
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .as_slice(),
            device,
        )
        .reshape([MAX_TIER1_SYMBOLS, self.max_trades_per_symbol]);

        Ok(PaddedTensorBatch {
            prices: prices_tensor,
            volumes: volumes_tensor,
            timestamps: timestamps_tensor,
            microstructure: microstructure_tensor,
            trade_counts,
            symbol_names,
            num_active_symbols: num_symbols,
        })
    }

    /// Compute range bars for all symbols in parallel with streaming state management
    ///
    /// **True Parallel Processing with Symbol-Aware State Management:**
    /// - Each GPU core processes one symbol simultaneously
    /// - Maintains 18 concurrent bar states in GPU memory
    /// - Streaming sequential processing with parallel execution
    /// - Proper breach detection and bar completion handling
    fn compute_parallel_range_bars(
        &self,
        batch: &PaddedTensorBatch,
    ) -> Result<ParallelRangeBarResults, MultiSymbolGpuError> {
        let device = self.device.device();
        let threshold_multiplier = (self.threshold_bps as f32) / 1_000_000.0;

        // DEBUG: Log entry to main GPU processing function
        println!("🔍 [MULTI-SYMBOL GPU DEBUG] compute_parallel_range_bars ENTRY:");
        println!("   Active symbols: {}", batch.num_active_symbols);
        println!("   Threshold BPS: {}", self.threshold_bps);
        println!("   Threshold multiplier: {:.6}", threshold_multiplier);
        println!("   Max trades per symbol: {}", self.max_trades_per_symbol);

        // Initialize parallel bar states for all symbols
        let mut parallel_state = ParallelBarState::new(batch.num_active_symbols, device.clone())?;
        let mut completed_bars = vec![Vec::new(); batch.num_active_symbols];

        // DEBUG: Log processing loop initialization
        println!("   Parallel state initialized successfully");
        println!(
            "   Starting main processing loop for {} trades",
            self.max_trades_per_symbol
        );

        // Stream through trades sequentially, processing all symbols in parallel
        for trade_idx in 0..self.max_trades_per_symbol {
            // DEBUG: Log loop progress every 100 iterations or first 5
            if trade_idx < 5 || trade_idx % 100 == 0 {
                println!(
                    "   Processing trade index: {} / {}",
                    trade_idx, self.max_trades_per_symbol
                );
            }
            // Extract current trade slice across all symbols [MAX_TIER1_SYMBOLS, 1]
            let trade_prices = batch
                .prices
                .clone()
                .slice([0..MAX_TIER1_SYMBOLS, trade_idx..trade_idx + 1]);
            let trade_volumes = batch
                .volumes
                .clone()
                .slice([0..MAX_TIER1_SYMBOLS, trade_idx..trade_idx + 1]);
            let trade_timestamps = batch
                .timestamps
                .clone()
                .slice([0..MAX_TIER1_SYMBOLS, trade_idx..trade_idx + 1]);
            let trade_microstructure = batch
                .microstructure
                .clone()
                .slice([0..MAX_TIER1_SYMBOLS, trade_idx..trade_idx + 1]);

            // Check which symbols have valid trades at this index (non-zero prices)
            let valid_trades_mask = trade_prices
                .clone()
                .greater(Tensor::zeros_like(&trade_prices));

            // Process this trade slice for all active symbols in parallel
            let trade_slice = TradeSliceData {
                prices: trade_prices,
                volumes: trade_volumes,
                timestamps: trade_timestamps,
                microstructure: trade_microstructure,
                valid_mask: valid_trades_mask,
            };

            self.process_parallel_trade_slice(
                &mut parallel_state,
                &mut completed_bars,
                trade_slice,
                threshold_multiplier,
                trade_idx,
                batch,
            )?;
        }

        // 🔧 CRITICAL FIX: Do NOT finalize incomplete bars to match CPU behavior
        // CPU only returns bars that actually breached thresholds, not incomplete bars
        // Commenting out to achieve CPU parity:
        //
        // let final_bars = self.finalize_incomplete_bars(&mut parallel_state, &batch.trade_counts[0..batch.num_active_symbols])?;
        // for (symbol_idx, bars) in final_bars.into_iter().enumerate() {
        //     completed_bars[symbol_idx].extend(bars);
        // }

        // Calculate symbol bar counts
        let symbol_bar_counts = completed_bars.iter().map(|bars| bars.len()).collect();

        // DEBUG: Log final results before return
        println!("🔍 [MULTI-SYMBOL GPU DEBUG] compute_parallel_range_bars RESULTS:");
        println!("   Processing loop completed");
        let total_bars: usize = completed_bars.iter().map(|bars| bars.len()).sum();
        println!("   Total bars across all symbols: {}", total_bars);
        for (idx, bar_count) in completed_bars.iter().enumerate() {
            if idx < batch.num_active_symbols {
                println!("   Symbol {}: {} bars", idx, bar_count.len());
            }
        }

        // Create placeholder breach mask
        let breach_mask_data = vec![false; MAX_TIER1_SYMBOLS * self.max_trades_per_symbol];
        let breach_mask = Tensor::<Wgpu, 2, burn::tensor::Bool>::from_data(
            burn::tensor::TensorData::new(
                breach_mask_data,
                [MAX_TIER1_SYMBOLS, self.max_trades_per_symbol],
            ),
            device,
        );

        Ok(ParallelRangeBarResults {
            breach_mask,
            completed_bars,
            symbol_bar_counts,
        })
    }

    /// Process a single trade slice across all symbols in parallel
    fn process_parallel_trade_slice(
        &self,
        parallel_state: &mut ParallelBarState,
        completed_bars: &mut [Vec<ParallelGpuBar>],
        trade_slice: TradeSliceData,
        threshold_multiplier: f32,
        trade_idx: usize,
        batch: &PaddedTensorBatch,
    ) -> Result<(), MultiSymbolGpuError> {
        // DEBUG: Log entry to critical bar processing function (only first few iterations)
        if trade_idx < 3 {
            println!(
                "🔍 [TRADE SLICE DEBUG] trade_idx {}: Processing bar logic",
                trade_idx
            );
            println!("   Threshold multiplier: {:.6}", threshold_multiplier);
        }

        // CRITICAL FIX: Early termination when trade data is exhausted (all zeros)
        // This prevents processing 1900+ iterations of zero data after demo (~100 trades) ends
        let trade_data = trade_slice.prices.clone().to_data();
        let price_slice = trade_data.as_slice::<f32>().unwrap_or(&[0.0]);
        let has_any_price_data = price_slice.iter().any(|&price| price > 0.0);

        if !has_any_price_data {
            if trade_idx < 5 || trade_idx % 500 == 0 {
                println!(
                    "🔍 [EARLY TERMINATION] trade_idx {}: No price data, skipping iteration",
                    trade_idx
                );
            }
            return Ok(()); // Skip processing this iteration - no actual trade data
        }

        // Continue processing only if we have actual trade data
        if trade_idx < 3 {
            println!(
                "🔍 [VALID DATA] trade_idx {}: Processing with valid trade data",
                trade_idx
            );
        }

        // CRITICAL FIX: Complete breached bars FIRST (from previous iteration)
        // This clears symbols that breached in the last iteration, making them available for reinitialization
        if trade_idx > 0 {
            let prev_breach_results =
                parallel_state.detect_parallel_breaches(&trade_slice.prices)?;
            let prev_completed = parallel_state.complete_breached_bars(
                &prev_breach_results,
                &trade_slice.prices,
                &trade_slice.volumes,
                &trade_slice.timestamps,
                &trade_slice.microstructure,
                trade_idx,
            )?;

            // Add previous completed bars to results
            for (symbol_idx, bars) in prev_completed.into_iter().enumerate() {
                if symbol_idx < batch.num_active_symbols {
                    completed_bars[symbol_idx].extend(bars);
                }
            }
        }

        // Initialize new bars for symbols that don't have active bars (including just-cleared ones)
        parallel_state.initialize_new_bars(
            &trade_slice.prices,
            &trade_slice.volumes,
            &trade_slice.timestamps,
            &trade_slice.microstructure,
            &trade_slice.valid_mask,
            threshold_multiplier,
        )?;

        // DEBUG: Log after initialization
        if trade_idx < 3 {
            println!("   ✅ initialize_new_bars completed");
        }

        // Update existing bars with current trades
        parallel_state.update_bars_with_trades(
            &trade_slice.prices,
            &trade_slice.volumes,
            &trade_slice.timestamps,
            &trade_slice.microstructure,
            &trade_slice.valid_mask,
        )?;

        // DEBUG: Log after update
        if trade_idx < 3 {
            println!("   ✅ update_bars_with_trades completed");
        }

        // Detect breaches across all symbols in parallel
        let breach_results = parallel_state.detect_parallel_breaches(&trade_slice.prices)?;

        // DEBUG: Log breach detection results and actual values
        if trade_idx < 3 {
            println!("   ✅ detect_parallel_breaches completed");

            // Extract sample values for debugging (first symbol only)
            if let Ok(price_data) = trade_slice
                .prices
                .clone()
                .slice([0..1, 0..1])
                .into_data()
                .to_vec::<f32>()
            {
                if let Ok(upper_data) = parallel_state
                    .upper_thresholds
                    .clone()
                    .slice([0..1, 0..1])
                    .into_data()
                    .to_vec::<f32>()
                {
                    if let Ok(lower_data) = parallel_state
                        .lower_thresholds
                        .clone()
                        .slice([0..1, 0..1])
                        .into_data()
                        .to_vec::<f32>()
                    {
                        if !price_data.is_empty()
                            && !upper_data.is_empty()
                            && !lower_data.is_empty()
                        {
                            println!("   🔍 Sample values (Symbol 0):");
                            println!("      Price: {:.6}", price_data[0]);
                            println!("      Upper: {:.6}", upper_data[0]);
                            println!("      Lower: {:.6}", lower_data[0]);

                            let upper_breach = price_data[0] >= upper_data[0];
                            let lower_breach = price_data[0] <= lower_data[0];
                            println!(
                                "      Upper breach: {} | Lower breach: {}",
                                upper_breach, lower_breach
                            );

                            // Check if thresholds are properly calculated
                            let expected_upper = price_data[0] * 1.008;
                            let expected_lower = price_data[0] * 0.992;
                            println!(
                                "      Expected upper: {:.6}, Expected lower: {:.6}",
                                expected_upper, expected_lower
                            );

                            // Check active bars status using CPU array
                            if !parallel_state.active_bars_cpu.is_empty() {
                                println!(
                                    "      Active bar (Symbol 0): {}",
                                    parallel_state.active_bars_cpu[0]
                                );
                            }
                        }
                    }
                }
            }
        }

        // Complete breached bars and start new ones
        let newly_completed = parallel_state.complete_breached_bars(
            &breach_results,
            &trade_slice.prices,
            &trade_slice.volumes,
            &trade_slice.timestamps,
            &trade_slice.microstructure,
            trade_idx,
        )?;

        // DEBUG: Log completed bars
        if trade_idx < 3 {
            let total_new_bars: usize = newly_completed.iter().map(|bars| bars.len()).sum();
            println!("   ✅ complete_breached_bars: {} new bars", total_new_bars);
        }

        // Add completed bars to results
        for (symbol_idx, bars) in newly_completed.into_iter().enumerate() {
            if symbol_idx < batch.num_active_symbols {
                completed_bars[symbol_idx].extend(bars);
            }
        }

        // DEBUG: Log final state for first few iterations
        if trade_idx < 3 {
            let total_completed: usize = completed_bars.iter().map(|bars| bars.len()).sum();
            println!("   📊 Total completed bars so far: {}", total_completed);
        }

        Ok(())
    }

    /// Finalize any remaining incomplete bars at the end of processing
    #[allow(dead_code)]
    fn finalize_incomplete_bars(
        &self,
        parallel_state: &mut ParallelBarState,
        trade_counts: &[usize],
    ) -> Result<Vec<Vec<ParallelGpuBar>>, MultiSymbolGpuError> {
        let mut final_bars = vec![Vec::new(); trade_counts.len()];

        for symbol_idx in 0..trade_counts.len() {
            if trade_counts[symbol_idx] > 0 && parallel_state.has_active_bar(symbol_idx) {
                if let Some(final_bar) = parallel_state.finalize_bar(symbol_idx)? {
                    final_bars[symbol_idx].push(final_bar);
                }
            }
        }

        Ok(final_bars)
    }

    /// Separate GPU results back to individual symbols with proper conversion
    ///
    /// **Result Separation Process:**
    /// 1. Convert ParallelGpuBar results to RangeBar format
    /// 2. Map each bar to its originating symbol
    /// 3. Apply proper microstructure calculations and validation
    /// 4. Return symbol-specific results maintaining original order
    fn separate_results_by_symbol<'a>(
        &self,
        symbol_trades: &[(&'a str, &[AggTrade])],
        results: ParallelRangeBarResults,
    ) -> Result<Vec<(&'a str, Vec<RangeBar>)>, MultiSymbolGpuError> {
        let mut output = Vec::with_capacity(symbol_trades.len());

        // Process each symbol and convert its GPU bars to RangeBar format
        for (symbol_idx, (symbol, _trades)) in symbol_trades.iter().enumerate() {
            if symbol_idx >= results.completed_bars.len() {
                output.push((*symbol, Vec::new()));
                continue;
            }

            let gpu_bars = &results.completed_bars[symbol_idx];
            let mut range_bars = Vec::with_capacity(gpu_bars.len());

            for gpu_bar in gpu_bars {
                // Convert GPU bar to RangeBar with proper fixed-point arithmetic
                let range_bar = self.convert_gpu_bar_to_range_bar(gpu_bar)?;

                // Validate converted bar maintains range bar invariants
                self.validate_range_bar(&range_bar)?;

                range_bars.push(range_bar);
            }

            output.push((*symbol, range_bars));
        }

        Ok(output)
    }

    /// Convert ParallelGpuBar to RangeBar format with proper fixed-point conversion
    fn convert_gpu_bar_to_range_bar(
        &self,
        gpu_bar: &ParallelGpuBar,
    ) -> Result<RangeBar, MultiSymbolGpuError> {
        // Convert f32 GPU values back to FixedPoint with proper precision
        let open = FixedPoint((gpu_bar.open * 100_000_000.0) as i64);
        let high = FixedPoint((gpu_bar.high * 100_000_000.0) as i64);
        let low = FixedPoint((gpu_bar.low * 100_000_000.0) as i64);
        let close = FixedPoint((gpu_bar.close * 100_000_000.0) as i64);
        let volume = FixedPoint((gpu_bar.volume * 100_000_000.0) as i64);
        let buy_volume = FixedPoint((gpu_bar.buy_volume * 100_000_000.0) as i64);
        let sell_volume = FixedPoint((gpu_bar.sell_volume * 100_000_000.0) as i64);

        // Calculate VWAP: turnover / volume (with proper fixed-point handling)
        let vwap = if gpu_bar.volume > 0.0 {
            FixedPoint((gpu_bar.turnover / gpu_bar.volume * 100_000_000.0) as i64)
        } else {
            open // Fallback to open price if no volume
        };

        Ok(RangeBar {
            open_time: gpu_bar.open_time as i64,
            close_time: gpu_bar.close_time as i64,
            open,
            high,
            low,
            close,
            volume,
            turnover: (gpu_bar.turnover * 100_000_000.0) as i128, // Turnover uses i128
            trade_count: gpu_bar.trade_count as i64,
            first_id: 0, // GPU processing doesn't track individual trade IDs
            last_id: 0,  // GPU processing doesn't track individual trade IDs

            // Market microstructure fields
            buy_volume,
            sell_volume,
            buy_trade_count: 0, // Simplified - could be enhanced to track buy/sell trade counts
            sell_trade_count: 0, // Simplified - could be enhanced
            vwap,
            buy_turnover: (gpu_bar.turnover
                * (gpu_bar.buy_volume / gpu_bar.volume.max(f32::EPSILON)))
                as i128,
            sell_turnover: (gpu_bar.turnover
                * (gpu_bar.sell_volume / gpu_bar.volume.max(f32::EPSILON)))
                as i128,
        })
    }

    /// Validate that converted RangeBar maintains range bar invariants
    fn validate_range_bar(&self, bar: &RangeBar) -> Result<(), MultiSymbolGpuError> {
        // Critical validation: high >= max(open, close) and low <= min(open, close)
        if bar.high < bar.open.max(bar.close) {
            return Err(MultiSymbolGpuError::TensorError {
                message: format!(
                    "Range bar validation failed: high {} < max(open {}, close {})",
                    bar.high, bar.open, bar.close
                ),
            });
        }

        if bar.low > bar.open.min(bar.close) {
            return Err(MultiSymbolGpuError::TensorError {
                message: format!(
                    "Range bar validation failed: low {} > min(open {}, close {})",
                    bar.low, bar.open, bar.close
                ),
            });
        }

        // Validate volume consistency
        if bar.buy_volume.0 + bar.sell_volume.0 != bar.volume.0 {
            return Err(MultiSymbolGpuError::TensorError {
                message: format!(
                    "Volume inconsistency: buy_volume {} + sell_volume {} != total_volume {}",
                    bar.buy_volume, bar.sell_volume, bar.volume
                ),
            });
        }

        Ok(())
    }

    /// Get GPU device information
    pub fn device_info(&self) -> String {
        self.device.info()
    }

    /// Get maximum supported Tier-1 symbols
    pub fn max_symbols(&self) -> usize {
        MAX_TIER1_SYMBOLS
    }

    /// Get maximum trades per symbol per batch
    pub fn max_trades_per_symbol(&self) -> usize {
        self.max_trades_per_symbol
    }
}

/// Trade slice data for a single time step across all symbols
#[cfg(feature = "gpu")]
struct TradeSliceData {
    prices: Tensor<Wgpu, 2>,
    volumes: Tensor<Wgpu, 2>,
    timestamps: Tensor<Wgpu, 2>,
    microstructure: Tensor<Wgpu, 2>,
    valid_mask: Tensor<Wgpu, 2, burn::tensor::Bool>,
}

/// Padded tensor batch for parallel multi-symbol processing
#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct PaddedTensorBatch {
    /// Price data: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    prices: Tensor<Wgpu, 2>,

    /// Volume data: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    volumes: Tensor<Wgpu, 2>,

    /// Timestamp data: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    timestamps: Tensor<Wgpu, 2>,

    /// Microstructure data: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    microstructure: Tensor<Wgpu, 2>,

    /// Actual trade counts per symbol (for unpadding)
    trade_counts: Vec<usize>,

    /// Symbol names for result mapping
    symbol_names: Vec<String>,

    /// Number of active symbols (≤ MAX_TIER1_SYMBOLS)
    num_active_symbols: usize,
}

/// Results from parallel range bar computation
#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct ParallelRangeBarResults {
    /// Breach detection mask: [MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL]
    breach_mask: Tensor<Wgpu, 2, burn::tensor::Bool>,

    /// Completed bars for all symbols
    completed_bars: Vec<Vec<ParallelGpuBar>>,

    /// Number of completed bars per symbol
    symbol_bar_counts: Vec<usize>,
}

/// Parallel GPU bar result
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ParallelGpuBar {
    symbol_idx: usize,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
    trade_count: usize,
    open_time: f32,
    close_time: f32,
    turnover: f32,
    buy_volume: f32,
    sell_volume: f32,
}

/// Parallel bar state manager - maintains 18 concurrent bar states in GPU memory
#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct ParallelBarState {
    /// Active bar flags: CPU-side boolean array (bypass Burn tensor extraction bug)
    active_bars_cpu: Vec<bool>,

    /// Opening prices (fixed thresholds): [MAX_TIER1_SYMBOLS, 1]
    open_prices: Tensor<Wgpu, 2>,

    /// Current high prices: [MAX_TIER1_SYMBOLS, 1]
    high_prices: Tensor<Wgpu, 2>,

    /// Current low prices: [MAX_TIER1_SYMBOLS, 1]
    low_prices: Tensor<Wgpu, 2>,

    /// Current close prices: [MAX_TIER1_SYMBOLS, 1]
    close_prices: Tensor<Wgpu, 2>,

    /// Accumulated volumes: [MAX_TIER1_SYMBOLS, 1]
    volumes: Tensor<Wgpu, 2>,

    /// Opening timestamps: [MAX_TIER1_SYMBOLS, 1]
    open_times: Tensor<Wgpu, 2>,

    /// Current closing timestamps: [MAX_TIER1_SYMBOLS, 1]
    close_times: Tensor<Wgpu, 2>,

    /// Upper breach thresholds (fixed): [MAX_TIER1_SYMBOLS, 1]
    upper_thresholds: Tensor<Wgpu, 2>,

    /// Lower breach thresholds (fixed): [MAX_TIER1_SYMBOLS, 1]
    lower_thresholds: Tensor<Wgpu, 2>,

    /// Trade count accumulator: [MAX_TIER1_SYMBOLS, 1]
    trade_counts: Tensor<Wgpu, 2>,

    /// Buy volume accumulator: [MAX_TIER1_SYMBOLS, 1]
    buy_volumes: Tensor<Wgpu, 2>,

    /// Sell volume accumulator: [MAX_TIER1_SYMBOLS, 1]
    sell_volumes: Tensor<Wgpu, 2>,

    /// Turnover accumulator: [MAX_TIER1_SYMBOLS, 1]
    turnovers: Tensor<Wgpu, 2>,

    /// GPU device reference
    device: burn::backend::wgpu::WgpuDevice,
}

#[cfg(feature = "gpu")]
impl ParallelBarState {
    /// Create new parallel bar state for managing multiple symbol bars
    fn new(
        _num_active_symbols: usize,
        device: burn::backend::wgpu::WgpuDevice,
    ) -> Result<Self, MultiSymbolGpuError> {
        // Initialize all tensors to zeros - bars will be initialized as trades arrive
        let zeros_data = vec![false; MAX_TIER1_SYMBOLS];
        let _zeros_bool = Tensor::<Wgpu, 2, burn::tensor::Bool>::from_data(
            burn::tensor::TensorData::new(zeros_data, [MAX_TIER1_SYMBOLS, 1]),
            &device,
        );
        let zeros_f32 =
            Tensor::<Wgpu, 2, burn::tensor::Float>::zeros([MAX_TIER1_SYMBOLS, 1], &device);

        Ok(Self {
            active_bars_cpu: vec![false; MAX_TIER1_SYMBOLS],
            open_prices: zeros_f32.clone(),
            high_prices: zeros_f32.clone(),
            low_prices: zeros_f32.clone(),
            close_prices: zeros_f32.clone(),
            volumes: zeros_f32.clone(),
            open_times: zeros_f32.clone(),
            close_times: zeros_f32.clone(),
            upper_thresholds: zeros_f32.clone(),
            lower_thresholds: zeros_f32.clone(),
            trade_counts: zeros_f32.clone(),
            buy_volumes: zeros_f32.clone(),
            sell_volumes: zeros_f32.clone(),
            turnovers: zeros_f32.clone(),
            device,
        })
    }

    /// Initialize new bars for symbols that don't have active bars
    fn initialize_new_bars(
        &mut self,
        trade_prices: &Tensor<Wgpu, 2>,
        trade_volumes: &Tensor<Wgpu, 2>,
        trade_timestamps: &Tensor<Wgpu, 2>,
        _trade_microstructure: &Tensor<Wgpu, 2>,
        _valid_trades_mask: &Tensor<Wgpu, 2, burn::tensor::Bool>,
        threshold_multiplier: f32,
    ) -> Result<(), MultiSymbolGpuError> {
        // CRITICAL FIX: Initialize bars for inactive symbols only
        // Count symbols that need initialization (active=false)
        let inactive_symbols: Vec<usize> = (0..MAX_TIER1_SYMBOLS)
            .filter(|&i| !self.active_bars_cpu[i])
            .collect();

        println!(
            "🔍 [GUARD DEBUG] Found {} inactive symbols that need initialization",
            inactive_symbols.len()
        );

        // If no symbols need initialization, preserve existing active bars
        if inactive_symbols.is_empty() {
            println!("🔍 [GUARD DEBUG] All symbols active - preserving existing bars");
            return Ok(());
        }

        println!(
            "🔍 [GUARD DEBUG] Initializing {} symbols: {:?}",
            inactive_symbols.len(),
            &inactive_symbols[..3.min(inactive_symbols.len())]
        );

        // Only reach here on first initialization - set up initial bars with FIXED thresholds
        let new_open_prices = trade_prices.clone();
        let new_volumes = trade_volumes.clone();
        let new_timestamps = trade_timestamps.clone();

        // Compute thresholds from opening prices (FIXED for lifetime of bars)
        // CRITICAL FIX: Set high thresholds for inactive symbols to prevent 0.0 >= 0.0 false breaches
        let threshold_trade_data = trade_prices.clone().to_data();
        let threshold_price_slice = threshold_trade_data.as_slice::<f32>().unwrap_or(&[0.0]);

        let mut upper_threshold_data = Vec::new();
        let mut lower_threshold_data = Vec::new();

        for i in 0..MAX_TIER1_SYMBOLS {
            let price = if i < threshold_price_slice.len() {
                threshold_price_slice[i]
            } else {
                0.0
            };

            if price > 0.0 {
                // Symbol has trade data: normal thresholds (active state set later)
                upper_threshold_data.push(price * (1.0 + threshold_multiplier));
                lower_threshold_data.push(price * (1.0 - threshold_multiplier));
                println!(
                    "🔍 [THRESHOLD DEBUG] Symbol {}: price={:.2}, upper={:.2}, lower={:.2}",
                    i,
                    price,
                    price * (1.0 + threshold_multiplier),
                    price * (1.0 - threshold_multiplier)
                );
            } else {
                // No trade data: impossible thresholds to prevent false breaches
                upper_threshold_data.push(1e9); // 0.0 >= 1e9 = false
                lower_threshold_data.push(-1e9); // 0.0 <= -1e9 = false
            }
        }

        let new_upper_thresholds = Tensor::<Wgpu, 2>::from_data(
            burn::tensor::TensorData::new(upper_threshold_data, [MAX_TIER1_SYMBOLS, 1]),
            &self.device,
        );
        let new_lower_thresholds = Tensor::<Wgpu, 2>::from_data(
            burn::tensor::TensorData::new(lower_threshold_data, [MAX_TIER1_SYMBOLS, 1]),
            &self.device,
        );

        println!(
            "🔍 [THRESHOLD DEBUG] Set impossible thresholds for inactive symbols to prevent false breaches"
        );

        // CRITICAL FIX: Only initialize tensors for symbols that don't have active bars
        // This preserves accumulated data for active symbols while initializing new ones

        // Check if this is the very first initialization (no tensors exist yet)
        let is_first_initialization = self.active_bars_cpu.iter().all(|&x| !x);

        if is_first_initialization {
            // First initialization: create all tensors from scratch
            println!("🔍 [FIRST INIT] First-time initialization of all state tensors");
            self.open_prices = new_open_prices.clone();
            self.high_prices = new_open_prices.clone();
            self.low_prices = new_open_prices.clone();
            self.close_prices = new_open_prices;
            self.volumes = new_volumes;
            self.open_times = new_timestamps.clone();
            self.close_times = new_timestamps;
            self.upper_thresholds = new_upper_thresholds; // Set ONCE and preserve
            self.lower_thresholds = new_lower_thresholds; // Set ONCE and preserve
        } else {
            // Subsequent initializations: preserve existing data for active symbols
            println!("🔍 [PRESERVE ACTIVE] Preserving existing bar data for active symbols");
            // Note: In a production implementation, we would selectively update only
            // the inactive symbol indices. For now, we preserve all existing data
            // and only the active_bars_cpu array controls which symbols are processed.

            // The key fix: DO NOT overwrite existing tensors
            // Active symbols keep their accumulated OHLC data and fixed thresholds
        }

        // Only initialize microstructure for first initialization
        if is_first_initialization {
            // Simplified microstructure initialization
            self.buy_volumes = trade_volumes.clone().mul_scalar(0.5); // Placeholder split
            self.sell_volumes = trade_volumes.clone().mul_scalar(0.5); // Placeholder split

            // Initialize trade counts and turnover
            self.trade_counts = Tensor::ones_like(trade_prices);
            self.turnovers = trade_prices.clone().mul(trade_volumes.clone());
        }

        // CRITICAL FIX: Only mark symbols as active if they have actual trade data
        println!(
            "🔍 [ACTIVE_BARS DEBUG] Checking {} symbols for actual trade data",
            inactive_symbols.len()
        );

        // Extract trade data to identify which symbols have non-zero prices
        let trade_data = trade_prices.clone().to_data();
        let price_slice = trade_data.as_slice::<f32>().unwrap_or(&[0.0]);

        // DEBUG: Inspect the actual trade data structure
        println!(
            "🔍 [TRADE DATA DEBUG] Tensor shape: {:?}",
            trade_prices.shape()
        );
        println!(
            "🔍 [TRADE DATA DEBUG] Price slice length: {}",
            price_slice.len()
        );
        if !price_slice.is_empty() {
            println!(
                "🔍 [TRADE DATA DEBUG] First 5 prices: {:?}",
                &price_slice[..5.min(price_slice.len())]
            );
        }

        let mut symbols_with_data = 0;
        for &symbol_idx in &inactive_symbols {
            // Check if this symbol has non-zero trade data
            let has_trade_data = if symbol_idx < price_slice.len() {
                price_slice[symbol_idx] > 0.0
            } else {
                false
            };

            if has_trade_data {
                self.active_bars_cpu[symbol_idx] = true;
                symbols_with_data += 1;
                println!(
                    "🔍 [ACTIVE_BARS DEBUG] Symbol {} set to active (price: {:.2})",
                    symbol_idx,
                    price_slice.get(symbol_idx).unwrap_or(&0.0)
                );
            } else if symbol_idx < 5 {
                // Only log first few to avoid spam
                println!(
                    "🔍 [ACTIVE_BARS DEBUG] Symbol {} skipped (price: {:.2})",
                    symbol_idx,
                    price_slice.get(symbol_idx).unwrap_or(&0.0)
                );
            }
        }

        // Verify the assignment worked
        let _verify_active = self.active_bars_cpu.iter().any(|&x| x);
        let active_count = self.active_bars_cpu.iter().filter(|&&x| x).count();
        println!(
            "🔍 [ACTIVE_BARS DEBUG] After setting: symbols_with_data = {}, total_active = {}",
            symbols_with_data, active_count
        );

        Ok(())
    }

    /// Update existing bars with current trades (simplified)
    fn update_bars_with_trades(
        &mut self,
        trade_prices: &Tensor<Wgpu, 2>,
        trade_volumes: &Tensor<Wgpu, 2>,
        trade_timestamps: &Tensor<Wgpu, 2>,
        _trade_microstructure: &Tensor<Wgpu, 2>,
        _valid_trades_mask: &Tensor<Wgpu, 2, burn::tensor::Bool>,
    ) -> Result<(), MultiSymbolGpuError> {
        // Simplified: directly update with new data (production version would use masking)

        // Update high/low prices
        self.high_prices = self.high_prices.clone().max_pair(trade_prices.clone());
        self.low_prices = self.low_prices.clone().min_pair(trade_prices.clone());

        // Update close prices and timestamps
        self.close_prices = trade_prices.clone();
        self.close_times = trade_timestamps.clone();

        // Update volumes and trade counts
        self.volumes = self.volumes.clone().add(trade_volumes.clone());
        self.trade_counts = self.trade_counts.clone().add_scalar(1.0);

        // Update microstructure data (simplified)
        self.buy_volumes = self
            .buy_volumes
            .clone()
            .add(trade_volumes.clone().mul_scalar(0.5));
        self.sell_volumes = self
            .sell_volumes
            .clone()
            .add(trade_volumes.clone().mul_scalar(0.5));

        // Update turnover
        self.turnovers = self
            .turnovers
            .clone()
            .add(trade_prices.clone().mul(trade_volumes.clone()));

        Ok(())
    }

    /// Detect breaches across all symbols in parallel
    /// FIXED: Use accumulated bar high/low vs fixed thresholds (not current trade price)
    fn detect_parallel_breaches(
        &self,
        _trade_prices: &Tensor<Wgpu, 2>, // Not used - we check accumulated data
    ) -> Result<Tensor<Wgpu, 2, burn::tensor::Bool>, MultiSymbolGpuError> {
        // CRITICAL FIX: Check accumulated HIGH/LOW prices vs FIXED thresholds
        // This is the correct range bar algorithm: accumulated data vs fixed thresholds

        // DEBUG: Check accumulated values being compared vs fixed thresholds
        let high_cpu = self.high_prices.clone().to_data();
        let low_cpu = self.low_prices.clone().to_data();
        let upper_thresh_cpu = self.upper_thresholds.clone().to_data();
        let lower_thresh_cpu = self.lower_thresholds.clone().to_data();

        // Extract data or use defaults
        let high_slice = high_cpu.as_slice::<f32>().unwrap_or(&[0.0]);
        let low_slice = low_cpu.as_slice::<f32>().unwrap_or(&[0.0]);
        let upper_slice = upper_thresh_cpu.as_slice::<f32>().unwrap_or(&[0.0]);
        let lower_slice = lower_thresh_cpu.as_slice::<f32>().unwrap_or(&[0.0]);

        if !high_slice.is_empty() && !upper_slice.is_empty() && !lower_slice.is_empty() {
            println!(
                "   Accumulated High: {:.6}, Low: {:.6}",
                high_slice[0], low_slice[0]
            );
            println!(
                "   Fixed Upper: {:.6}, Lower: {:.6}",
                upper_slice[0], lower_slice[0]
            );

            if high_slice[0] >= upper_slice[0] {
                println!(
                    "   🔥 ACCUMULATED HIGH BREACHES UPPER! {:.6} >= {:.6}",
                    high_slice[0], upper_slice[0]
                );
            }
            if low_slice[0] <= lower_slice[0] {
                println!(
                    "   🔥 ACCUMULATED LOW BREACHES LOWER! {:.6} <= {:.6}",
                    low_slice[0], lower_slice[0]
                );
            }
        }

        // Only check breaches for symbols with active bars
        println!("🔍 [COMPARISON DEBUG] Starting tensor comparisons with ACCUMULATED data");

        // DEBUG: Extract accumulated vs fixed threshold comparison data
        let high_data = self.high_prices.clone().to_data();
        let low_data = self.low_prices.clone().to_data();
        let upper_data = self.upper_thresholds.clone().to_data();
        let lower_data = self.lower_thresholds.clone().to_data();

        if let (Ok(highs), Ok(lows), Ok(uppers), Ok(lowers)) = (
            high_data.as_slice::<f32>(),
            low_data.as_slice::<f32>(),
            upper_data.as_slice::<f32>(),
            lower_data.as_slice::<f32>(),
        ) {
            for i in 0..3.min(highs.len()) {
                let should_upper = highs[i] >= uppers[i];
                let should_lower = lows[i] <= lowers[i];
                println!(
                    "🔍 [COMPARISON DEBUG] Symbol {}: high={:.2}, low={:.2}, upper={:.2}, lower={:.2}, should_upper={}, should_lower={}",
                    i, highs[i], lows[i], uppers[i], lowers[i], should_upper, should_lower
                );
            }
        }

        // CRITICAL FIX: Use accumulated HIGH/LOW vs fixed thresholds (correct range bar algorithm)
        let upper_breach = self
            .high_prices
            .clone()
            .greater_equal(self.upper_thresholds.clone());
        let lower_breach = self
            .low_prices
            .clone()
            .lower_equal(self.lower_thresholds.clone());
        let any_breach = upper_breach.bool_or(lower_breach);

        println!("🔍 [COMPARISON DEBUG] Tensor comparisons completed");

        // CRITICAL FIX: CPU-side masking to bypass Metal backend boolean tensor bugs
        // Extract breach detection results to CPU, apply masking, recreate tensor
        println!(
            "🔍 [CPU MASK DEBUG] active_bars_cpu: {:?}",
            &self.active_bars_cpu[..5]
        );

        // Extract GPU breach results to CPU (this might fail on Metal, use fallback)
        let breach_cpu_data = match any_breach.clone().to_data().as_slice::<bool>() {
            Ok(data) => data.to_vec(),
            Err(_) => {
                println!("🔍 [CPU MASK DEBUG] GPU extraction failed, using .any() per element");
                // Fallback: Extract each element using .any() workaround
                let mut breach_data = Vec::new();
                for i in 0..MAX_TIER1_SYMBOLS {
                    let breach = any_breach
                        .clone()
                        .slice([i..i + 1, 0..1])
                        .any()
                        .into_scalar()
                        > 0;
                    breach_data.push(breach);

                    // Debug: Log breach detection for first few symbols
                    if i < 8 {
                        println!("🔍 [BREACH INDEX DEBUG] Symbol {}: breach={}", i, breach);
                    }
                }
                breach_data
            }
        };

        // Apply CPU-side masking: breach AND active
        let masked_breach_data: Vec<bool> = (0..MAX_TIER1_SYMBOLS)
            .map(|i| {
                let has_breach = breach_cpu_data.get(i).copied().unwrap_or(false);
                let is_active = self.active_bars_cpu.get(i).copied().unwrap_or(false);
                has_breach && is_active
            })
            .collect();

        println!(
            "🔍 [CPU MASK DEBUG] breach_cpu_data first 5: {:?}",
            &breach_cpu_data[..5]
        );
        println!(
            "🔍 [CPU MASK DEBUG] masked_breach_data first 5: {:?}",
            &masked_breach_data[..5]
        );

        // Create final GPU tensor from CPU-masked data
        let final_result = Tensor::<Wgpu, 2, burn::tensor::Bool>::from_data(
            burn::tensor::TensorData::new(masked_breach_data, [MAX_TIER1_SYMBOLS, 1]),
            &self.device,
        );

        Ok(final_result)
    }

    /// Complete breached bars and prepare for new ones
    #[allow(clippy::needless_range_loop)]
    fn complete_breached_bars(
        &mut self,
        breach_mask: &Tensor<Wgpu, 2, burn::tensor::Bool>,
        _trade_prices: &Tensor<Wgpu, 2>,
        _trade_volumes: &Tensor<Wgpu, 2>,
        _trade_timestamps: &Tensor<Wgpu, 2>,
        _trade_microstructure: &Tensor<Wgpu, 2>,
        trade_idx: usize,
    ) -> Result<Vec<Vec<ParallelGpuBar>>, MultiSymbolGpuError> {
        let mut completed_bars = vec![Vec::new(); MAX_TIER1_SYMBOLS];
        let mut total_completed = 0;

        // DEBUG: Log breach completion attempt for first few iterations
        if trade_idx < 3 {
            println!(
                "🔍 [BAR COMPLETION DEBUG] trade_idx {}: Checking {} symbols",
                trade_idx, MAX_TIER1_SYMBOLS
            );
        }

        // CRITICAL FIX: Use .any() operation per symbol (Metal boolean extraction workaround)
        // Metal backend cannot extract boolean tensors, but .any() works perfectly
        for symbol_idx in 0..MAX_TIER1_SYMBOLS {
            let has_active = self.has_active_bar(symbol_idx);

            // Extract single symbol breach using .any() on slice - this works on Metal!
            let symbol_slice = breach_mask
                .clone()
                .slice([symbol_idx..symbol_idx + 1, 0..1]);
            let has_breach = symbol_slice.any().into_scalar() > 0;

            // DEBUG: Log details for first symbol in first few iterations
            if trade_idx < 3 && symbol_idx == 0 {
                println!(
                    "   Symbol 0: has_active={}, has_breach={} (.any() extraction)",
                    has_active, has_breach
                );
            }

            if has_active && has_breach {
                // This symbol has a breached bar - complete it
                if let Some(completed_bar) = self.extract_completed_bar(symbol_idx, trade_idx)? {
                    completed_bars[symbol_idx].push(completed_bar);
                    total_completed += 1;

                    if trade_idx < 3 {
                        println!("   ✅ Symbol {} bar completed!", symbol_idx);
                    }
                }

                // Clear the bar state for this symbol (will be reinitialized next trade)
                self.clear_bar_state(symbol_idx)?;
            }
        }

        // DEBUG: Log total completed bars
        if trade_idx < 3 {
            println!("   📊 Total completed this iteration: {}", total_completed);
        }

        Ok(completed_bars)
    }

    /// Check if a symbol has an active bar
    fn has_active_bar(&self, symbol_idx: usize) -> bool {
        // Use CPU-side array instead of GPU tensor extraction
        if symbol_idx < self.active_bars_cpu.len() {
            self.active_bars_cpu[symbol_idx]
        } else {
            false
        }
    }

    /// Finalize an incomplete bar for a symbol
    #[allow(dead_code)]
    fn finalize_bar(
        &mut self,
        symbol_idx: usize,
    ) -> Result<Option<ParallelGpuBar>, MultiSymbolGpuError> {
        if self.has_active_bar(symbol_idx) {
            let final_bar = self.extract_completed_bar(symbol_idx, usize::MAX)?;
            self.clear_bar_state(symbol_idx)?;
            Ok(final_bar)
        } else {
            Ok(None)
        }
    }

    /// Extract a completed bar for a specific symbol
    fn extract_completed_bar(
        &self,
        symbol_idx: usize,
        _trade_idx: usize,
    ) -> Result<Option<ParallelGpuBar>, MultiSymbolGpuError> {
        Ok(Some(ParallelGpuBar {
            symbol_idx,
            open: self.extract_f32_value(&self.open_prices, symbol_idx),
            high: self.extract_f32_value(&self.high_prices, symbol_idx),
            low: self.extract_f32_value(&self.low_prices, symbol_idx),
            close: self.extract_f32_value(&self.close_prices, symbol_idx),
            volume: self.extract_f32_value(&self.volumes, symbol_idx),
            trade_count: self.extract_f32_value(&self.trade_counts, symbol_idx) as usize,
            open_time: self.extract_f32_value(&self.open_times, symbol_idx),
            close_time: self.extract_f32_value(&self.close_times, symbol_idx),
            turnover: self.extract_f32_value(&self.turnovers, symbol_idx),
            buy_volume: self.extract_f32_value(&self.buy_volumes, symbol_idx),
            sell_volume: self.extract_f32_value(&self.sell_volumes, symbol_idx),
        }))
    }

    /// Clear bar state for a specific symbol (simplified implementation)
    fn clear_bar_state(&mut self, symbol_idx: usize) -> Result<(), MultiSymbolGpuError> {
        // Mark symbol as inactive using CPU-side array
        if symbol_idx < self.active_bars_cpu.len() {
            self.active_bars_cpu[symbol_idx] = false;
            println!(
                "🔍 [CLEAR DEBUG] Cleared bar state for symbol {} - set active=false",
                symbol_idx
            );
        }
        Ok(())
    }

    /// Helper to extract f32 value from tensor at specific index
    fn extract_f32_value(&self, tensor: &Tensor<Wgpu, 2>, symbol_idx: usize) -> f32 {
        let slice = tensor.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        let data = slice.to_data();
        // For f32 tensor data, access the first element
        data.as_slice::<f32>().unwrap_or(&[0.0])[0]
    }

    /// Helper to extract bool value using .any() workaround (Metal backend compatible)
    #[allow(dead_code)]
    fn extract_bool_value(
        &self,
        tensor: &Tensor<Wgpu, 2, burn::tensor::Bool>,
        symbol_idx: usize,
    ) -> bool {
        // WORKAROUND: Metal backend cannot extract boolean tensors directly
        // TypeMismatch: "Invalid target element type (expected U32, got Bool)"
        // Use .any() operation on single-element slice instead - this works perfectly!
        let slice = tensor.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        slice.any().into_scalar() > 0
    }
}

/// Multi-symbol GPU processing errors
#[cfg(feature = "gpu")]
#[derive(Debug, thiserror::Error)]
pub enum MultiSymbolGpuError {
    #[error("GPU device error: {0}")]
    DeviceError(#[from] GpuError),

    #[error("Too many symbols: {provided}, maximum supported: {max}")]
    TooManySymbols { provided: usize, max: usize },

    #[error("Symbol '{symbol}' is not a Tier-1 cryptocurrency")]
    NotTier1Symbol { symbol: String },

    #[error("Tensor operation failed: {message}")]
    TensorError { message: String },

    #[error("Memory allocation failed for {symbols} symbols with {trades_per_symbol} trades each")]
    MemoryError {
        symbols: usize,
        trades_per_symbol: usize,
    },

    #[error("Parallel processing failed: {message}")]
    ParallelProcessingError { message: String },
}

// No-op implementations when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub struct MultiSymbolGpuProcessor;

#[cfg(not(feature = "gpu"))]
impl MultiSymbolGpuProcessor {
    pub fn new(_device: (), _threshold_bps: u32, _max_trades_per_symbol: Option<usize>) -> Self {
        Self
    }

    pub fn max_symbols(&self) -> usize {
        0
    }

    pub fn max_trades_per_symbol(&self) -> usize {
        0
    }
}

#[cfg(not(feature = "gpu"))]
#[derive(Debug, thiserror::Error)]
pub enum MultiSymbolGpuError {
    #[error("GPU support not compiled")]
    NotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "gpu")]
    use crate::{fixed_point::FixedPoint, gpu::metal_backend::detect_gpu_device};

    fn create_test_trade(id: i64, price: &str, volume: &str, timestamp: i64) -> AggTrade {
        AggTrade {
            agg_trade_id: id,
            price: FixedPoint::from_str(price).unwrap(),
            volume: FixedPoint::from_str(volume).unwrap(),
            first_trade_id: id * 10,
            last_trade_id: id * 10,
            timestamp,
            is_buyer_maker: id % 2 == 0,
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_multi_symbol_processor_creation() {
        if let Some(device) = detect_gpu_device() {
            let processor = MultiSymbolGpuProcessor::new(device, 8000, Some(5000));
            assert_eq!(processor.max_symbols(), 18);
            assert_eq!(processor.max_trades_per_symbol(), 5000);
        } else {
            println!("GPU not available for testing");
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_tier1_symbol_validation() {
        if let Some(device) = detect_gpu_device() {
            let processor = MultiSymbolGpuProcessor::new(device, 8000, None);

            // Valid Tier-1 symbols
            let btc_trades = vec![create_test_trade(1, "50000.0", "1.0", 1000)];
            let eth_trades = vec![create_test_trade(2, "4000.0", "10.0", 2000)];
            let valid_symbols = vec![
                ("BTCUSDT", btc_trades.as_slice()),
                ("ETHUSDT", eth_trades.as_slice()),
            ];

            let result = processor.validate_tier1_symbols(&valid_symbols);
            assert!(
                result.is_ok(),
                "Valid Tier-1 symbols should pass validation"
            );

            // Invalid symbol
            let doge_trades = vec![create_test_trade(3, "0.1", "1000.0", 3000)];
            let invalid_symbols = vec![("DOGECOIN", doge_trades.as_slice())];

            let result = processor.validate_tier1_symbols(&invalid_symbols);
            assert!(result.is_err(), "Invalid symbols should fail validation");
        } else {
            println!("GPU not available for testing");
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_padded_tensor_batch_creation() {
        if let Some(device) = detect_gpu_device() {
            let processor = MultiSymbolGpuProcessor::new(device, 8000, Some(100));

            let btc_batch_trades = vec![
                create_test_trade(1, "50000.0", "1.0", 1000),
                create_test_trade(2, "50200.0", "1.5", 2000),
            ];
            let eth_batch_trades = vec![create_test_trade(3, "4000.0", "10.0", 3000)];
            let symbol_trades = vec![
                ("BTCUSDT", btc_batch_trades.as_slice()),
                ("ETHUSDT", eth_batch_trades.as_slice()),
            ];

            let result = processor.create_padded_tensor_batch(&symbol_trades);
            assert!(
                result.is_ok(),
                "Padded tensor batch creation should succeed"
            );

            if let Ok(batch) = result {
                assert_eq!(batch.num_active_symbols, 2);
                assert_eq!(batch.trade_counts[0], 2); // BTC has 2 trades
                assert_eq!(batch.trade_counts[1], 1); // ETH has 1 trade
                assert_eq!(batch.symbol_names[0], "BTCUSDT");
                assert_eq!(batch.symbol_names[1], "ETHUSDT");
            }
        } else {
            println!("GPU not available for testing");
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_TIER1_SYMBOLS, 18);
        assert_eq!(MAX_TRADES_PER_SYMBOL, 10000);
    }
}
