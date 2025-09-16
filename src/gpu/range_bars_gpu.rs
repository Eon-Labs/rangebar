//! GPU-accelerated range bar processing
//!
//! This module provides GPU implementations of range bar construction using
//! the Burn framework with Metal backend for Mac and WGPU for cross-platform support.

#[cfg(feature = "gpu")]
use burn::{backend::wgpu::Wgpu, tensor::Tensor};

#[cfg(feature = "gpu")]
use crate::{
    gpu::metal_backend::{GpuDevice, GpuError},
    types::{AggTrade, RangeBar},
};

/// GPU-accelerated range bar processor
#[cfg(feature = "gpu")]
pub struct GpuRangeBarProcessor {
    device: GpuDevice,
    threshold_bps: u32,
    batch_size: usize,
}

#[cfg(feature = "gpu")]
impl GpuRangeBarProcessor {
    /// Create new GPU processor
    ///
    /// # Arguments
    /// * `device` - GPU device for computation
    /// * `threshold_bps` - Threshold in basis points (8000 = 0.8%)
    /// * `batch_size` - Number of trades to process in each GPU batch
    pub fn new(device: GpuDevice, threshold_bps: u32, batch_size: Option<usize>) -> Self {
        Self {
            device,
            threshold_bps,
            batch_size: batch_size.unwrap_or(8000), // Default batch size for GPU efficiency
        }
    }

    /// Process multiple symbols in parallel on GPU
    ///
    /// This is the main advantage of GPU processing - batch multiple symbols
    /// simultaneously using tensor operations.
    pub fn process_multi_symbol_batch<'a>(
        &self,
        symbol_trades: &[(&'a str, &[AggTrade])],
    ) -> Result<Vec<(&'a str, Vec<RangeBar>)>, GpuProcessingError> {
        if symbol_trades.is_empty() {
            return Ok(Vec::new());
        }

        // Convert trades to GPU tensors for batch processing
        let tensor_data = self.prepare_tensor_data(symbol_trades)?;

        // Execute GPU computation
        let results = self.compute_range_bars_gpu(&tensor_data)?;

        // Convert results back to RangeBar format
        self.extract_range_bars(symbol_trades, results)
    }

    /// Process single symbol trades using GPU acceleration
    pub fn process_single_symbol(
        &self,
        trades: &[AggTrade],
    ) -> Result<Vec<RangeBar>, GpuProcessingError> {
        if trades.is_empty() {
            return Ok(Vec::new());
        }

        // For single symbol, create batch of 1
        let symbol_trades = vec![("symbol", trades)];
        let mut results = self.process_multi_symbol_batch(&symbol_trades)?;

        if results.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(results.pop().unwrap().1)
        }
    }

    /// Prepare trade data as tensors for GPU processing
    fn prepare_tensor_data(
        &self,
        symbol_trades: &[(&str, &[AggTrade])],
    ) -> Result<GpuTensorData, GpuProcessingError> {
        let total_trades: usize = symbol_trades.iter().map(|(_, trades)| trades.len()).sum();

        // DEBUG: Log input data characteristics
        println!("🔍 [GPU DEBUG] prepare_tensor_data INPUT:");
        println!("   Total symbols: {}", symbol_trades.len());
        println!("   Total trades: {}", total_trades);
        for (i, (symbol, trades)) in symbol_trades.iter().enumerate() {
            println!("   Symbol {}: {} ({} trades)", i, symbol, trades.len());
            if !trades.is_empty() {
                let first_price = trades[0].price.to_f64() as f32;
                let last_price = trades[trades.len() - 1].price.to_f64() as f32;
                println!("     Price range: {:.2} to {:.2}", first_price, last_price);
            }
        }

        // Pre-allocate tensors for all trade data (f32 for Metal optimization)
        let mut prices = Vec::with_capacity(total_trades);
        let mut volumes = Vec::with_capacity(total_trades);
        let mut timestamps = Vec::with_capacity(total_trades);
        let mut trade_ids = Vec::with_capacity(total_trades);
        let mut symbol_indices = Vec::with_capacity(total_trades);
        let mut microstructure = Vec::with_capacity(total_trades);

        // Flatten all symbol data into tensors (convert to f32 for Metal performance)
        for (symbol_idx, (_, trades)) in symbol_trades.iter().enumerate() {
            for trade in trades.iter() {
                let price_f32 = trade.price.to_f64() as f32;
                let volume_f32 = trade.volume.to_f64() as f32;

                prices.push(price_f32);
                volumes.push(volume_f32);
                timestamps.push(trade.timestamp as f32);
                trade_ids.push(trade.agg_trade_id as f32);
                symbol_indices.push(symbol_idx as f32);
                microstructure.push(if trade.is_buyer_maker { 1.0 } else { 0.0 });
            }
        }

        // DEBUG: Log tensor data characteristics
        if !prices.is_empty() {
            let min_price = prices.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_price = prices.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            println!("🔍 [GPU DEBUG] TENSOR DATA:");
            println!("   Price range: {:.2} to {:.2}", min_price, max_price);
            println!("   Sample prices: {:?}", &prices[..prices.len().min(5)]);
        }

        // Create GPU tensors
        let device = self.device.device();

        let price_tensor = Tensor::<Wgpu, 1>::from_floats(prices.as_slice(), device);
        let volume_tensor = Tensor::<Wgpu, 1>::from_floats(volumes.as_slice(), device);
        let timestamp_tensor = Tensor::<Wgpu, 1>::from_floats(timestamps.as_slice(), device);
        let symbol_tensor = Tensor::<Wgpu, 1>::from_floats(symbol_indices.as_slice(), device);
        let microstructure_tensor =
            Tensor::<Wgpu, 1>::from_floats(microstructure.as_slice(), device);

        Ok(GpuTensorData {
            prices: price_tensor,
            volumes: volume_tensor,
            timestamps: timestamp_tensor,
            symbols: symbol_tensor,
            microstructure: microstructure_tensor,
            num_symbols: symbol_trades.len(),
            total_trades,
        })
    }

    /// Core GPU computation for range bar construction using streaming tensor pipeline
    fn compute_range_bars_gpu(
        &self,
        tensor_data: &GpuTensorData,
    ) -> Result<GpuRangeBarResults, GpuProcessingError> {
        // GPU Streaming Pipeline Design:
        // 1. Process trades in chunks to maintain sequential semantics
        // 2. Use GPU state management for current bar
        // 3. Vectorized threshold computation and breach detection per chunk

        let device = self.device.device();
        let chunk_size = self.batch_size.min(1000); // Limit for GPU memory efficiency

        // Initialize GPU bar state containers
        let mut completed_bars = Vec::new();
        let mut current_bar_state: Option<GpuBarState> = None;

        // Stream processing: iterate through trade chunks
        let total_trades = tensor_data.prices.dims()[0];
        let num_chunks = total_trades.div_ceil(chunk_size);

        for chunk_idx in 0..num_chunks {
            let start_idx = chunk_idx * chunk_size;
            let end_idx = (start_idx + chunk_size).min(total_trades);

            // Extract chunk tensors (GPU slice operations)
            let chunk_prices = self.slice_tensor(&tensor_data.prices, start_idx, end_idx)?;
            let chunk_volumes = self.slice_tensor(&tensor_data.volumes, start_idx, end_idx)?;

            // Process chunk with current bar state
            let chunk_results = self.process_trade_chunk(
                &chunk_prices,
                &chunk_volumes,
                &mut current_bar_state,
                device,
            )?;

            // Collect any completed bars from this chunk
            completed_bars.extend(chunk_results.completed_bars);
        }

        // Convert results to output format
        let result_tensor = self.serialize_bars_to_tensor(&completed_bars, device)?;

        Ok(GpuRangeBarResults {
            breach_points: result_tensor.clone(),
            upper_thresholds: result_tensor.clone(), // Placeholder - would be actual threshold data
            lower_thresholds: result_tensor,
            processed_symbols: tensor_data.num_symbols,
        })
    }

    /// GPU tensor slicing operation (preserves device)
    #[allow(clippy::single_range_in_vec_init)]
    fn slice_tensor(
        &self,
        tensor: &Tensor<Wgpu, 1>,
        start: usize,
        end: usize,
    ) -> Result<Tensor<Wgpu, 1>, GpuProcessingError> {
        // Burn tensor slicing - keeps data on GPU
        Ok(tensor.clone().slice([start..end]))
    }

    /// Process a chunk of trades with streaming GPU operations
    #[allow(clippy::single_range_in_vec_init)]
    fn process_trade_chunk(
        &self,
        chunk_prices: &Tensor<Wgpu, 1>,
        chunk_volumes: &Tensor<Wgpu, 1>,
        current_bar_state: &mut Option<GpuBarState>,
        device: &burn::backend::wgpu::WgpuDevice,
    ) -> Result<ChunkProcessingResult, GpuProcessingError> {
        let chunk_size = chunk_prices.dims()[0];
        let mut completed_bars = Vec::new();

        // Convert threshold from basis points to multiplier
        let threshold_multiplier = (self.threshold_bps as f32) / 10_000.0;

        match current_bar_state {
            None => {
                // Initialize first bar with chunk's first trade
                if chunk_size > 0 {
                    let first_price = chunk_prices.clone().slice([0..1]);
                    let first_volume = chunk_volumes.clone().slice([0..1]);

                    *current_bar_state = Some(GpuBarState::new(
                        first_price,
                        first_volume,
                        threshold_multiplier,
                        device,
                    )?);

                    // Process remaining trades in chunk if any
                    if chunk_size > 1 {
                        let remaining_prices = chunk_prices.clone().slice([1..chunk_size]);
                        let remaining_volumes = chunk_volumes.clone().slice([1..chunk_size]);

                        let result = self.process_chunk_with_existing_bar(
                            &remaining_prices,
                            &remaining_volumes,
                            current_bar_state.as_mut().unwrap(),
                        )?;

                        completed_bars.extend(result.completed_bars);
                    }
                }
            }
            Some(bar_state) => {
                // Process chunk against existing bar state
                let result =
                    self.process_chunk_with_existing_bar(chunk_prices, chunk_volumes, bar_state)?;

                completed_bars.extend(result.completed_bars);
            }
        }

        Ok(ChunkProcessingResult { completed_bars })
    }

    /// Process chunk trades against an existing bar state
    #[allow(clippy::single_range_in_vec_init)]
    fn process_chunk_with_existing_bar(
        &self,
        chunk_prices: &Tensor<Wgpu, 1>,
        chunk_volumes: &Tensor<Wgpu, 1>,
        bar_state: &mut GpuBarState,
    ) -> Result<ChunkProcessingResult, GpuProcessingError> {
        let chunk_size = chunk_prices.dims()[0];
        let mut completed_bars = Vec::new();

        // DEBUG: Log chunk characteristics
        println!("🔍 [GPU DEBUG] process_chunk_with_existing_bar:");
        println!("   Chunk size: {}", chunk_size);

        // Extract threshold values for debugging
        let upper_threshold = bar_state.upper_threshold.clone();
        let lower_threshold = bar_state.lower_threshold.clone();

        // DEBUG: Log thresholds
        if let Ok(upper_val) = self.extract_scalar_debug(&upper_threshold) {
            if let Ok(lower_val) = self.extract_scalar_debug(&lower_threshold) {
                println!(
                    "   Thresholds: upper={:.2}, lower={:.2}",
                    upper_val, lower_val
                );
            }
        }

        // DEBUG: Log chunk price range
        if chunk_size > 0 {
            let price_data = chunk_prices.clone().into_data().to_vec::<f32>();
            if let Ok(prices) = price_data {
                let min_price = prices.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_price = prices.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                println!("   Chunk prices: {:.2} to {:.2}", min_price, max_price);
                println!("   Sample prices: {:?}", &prices[..prices.len().min(3)]);
            }
        }

        // Vectorized breach detection across entire chunk
        let upper_breach_mask = chunk_prices.clone().greater_equal(upper_threshold);
        let lower_breach_mask = chunk_prices.clone().lower_equal(lower_threshold);
        let breach_mask = upper_breach_mask.bool_or(lower_breach_mask);

        // Find first breach in chunk (GPU reduction operation)
        let breach_indices = self.find_breach_indices(&breach_mask)?;

        // DEBUG: Log breach detection results
        println!("   Breach indices found: {:?}", breach_indices);

        if let Some(first_breach_idx) = breach_indices.first() {
            // Breach found - process trades up to and including breach
            let breach_idx = *first_breach_idx;

            // Update bar with trades up to breach (inclusive)
            let breach_prices = chunk_prices.clone().slice([0..(breach_idx + 1)]);
            let breach_volumes = chunk_volumes.clone().slice([0..(breach_idx + 1)]);

            bar_state.update_with_trades(&breach_prices, &breach_volumes)?;

            // Close current bar and add to completed
            let completed_bar = bar_state.close_bar()?;
            completed_bars.push(completed_bar);

            // If there are trades after breach, start new bar
            if breach_idx + 1 < chunk_size {
                let next_price = chunk_prices
                    .clone()
                    .slice([(breach_idx + 1)..(breach_idx + 2)]);
                let next_volume = chunk_volumes
                    .clone()
                    .slice([(breach_idx + 1)..(breach_idx + 2)]);

                let threshold_multiplier = (self.threshold_bps as f32) / 10_000.0;
                let device = self.device.device();

                *bar_state =
                    GpuBarState::new(next_price, next_volume, threshold_multiplier, device)?;

                // Process remaining trades in chunk recursively
                if breach_idx + 2 < chunk_size {
                    let remaining_prices =
                        chunk_prices.clone().slice([(breach_idx + 2)..chunk_size]);
                    let remaining_volumes =
                        chunk_volumes.clone().slice([(breach_idx + 2)..chunk_size]);

                    let result = self.process_chunk_with_existing_bar(
                        &remaining_prices,
                        &remaining_volumes,
                        bar_state,
                    )?;

                    completed_bars.extend(result.completed_bars);
                }
            }
        } else {
            // No breach - update current bar with entire chunk
            bar_state.update_with_trades(chunk_prices, chunk_volumes)?;
        }

        Ok(ChunkProcessingResult { completed_bars })
    }

    /// Find breach indices from boolean mask (GPU reduction)
    fn find_breach_indices(
        &self,
        breach_mask: &Tensor<Wgpu, 1, burn::tensor::Bool>,
    ) -> Result<Vec<usize>, GpuProcessingError> {
        // Convert boolean mask to indices
        // This is a simplified implementation - full GPU reduction would be more efficient
        let mask_data = breach_mask.clone().into_data().to_vec().map_err(|e| {
            GpuProcessingError::TensorError {
                message: format!("Failed to extract breach mask: {:?}", e),
            }
        })?;

        let indices: Vec<usize> = mask_data
            .iter()
            .enumerate()
            .filter_map(|(idx, &is_breach)| if is_breach { Some(idx) } else { None })
            .collect();

        Ok(indices)
    }

    /// Convert completed bars to tensor format
    fn serialize_bars_to_tensor(
        &self,
        bars: &[GpuCompletedBar],
        device: &burn::backend::wgpu::WgpuDevice,
    ) -> Result<Tensor<Wgpu, 1>, GpuProcessingError> {
        if bars.is_empty() {
            // Return tensor with single zero element instead of empty
            let empty_data = vec![0.0f32];
            return Ok(Tensor::<Wgpu, 1>::from_floats(
                empty_data.as_slice(),
                device,
            ));
        }

        // Serialize bar data to flat tensor
        // Format: [open1, high1, low1, close1, volume1, open2, high2, ...]
        let mut data = Vec::with_capacity(bars.len() * 5);

        for bar in bars {
            data.push(bar.open);
            data.push(bar.high);
            data.push(bar.low);
            data.push(bar.close);
            data.push(bar.volume);
        }

        Ok(Tensor::<Wgpu, 1>::from_floats(data.as_slice(), device))
    }

    /// Extract range bars from GPU computation results
    fn extract_range_bars<'a>(
        &self,
        symbol_trades: &[(&'a str, &[AggTrade])],
        gpu_results: GpuRangeBarResults,
    ) -> Result<Vec<(&'a str, Vec<RangeBar>)>, GpuProcessingError> {
        // Convert GPU tensor results back to RangeBar format
        let completed_bars = self.deserialize_tensor_to_bars(&gpu_results.breach_points)?;

        let mut results = Vec::with_capacity(symbol_trades.len());

        // For single symbol processing, return all bars under that symbol
        if symbol_trades.len() == 1 {
            let (symbol, trades) = symbol_trades[0];
            let range_bars = self.convert_gpu_bars_to_range_bars(&completed_bars, trades)?;
            results.push((symbol, range_bars));
        } else {
            // Multi-symbol processing: distribute bars by symbol
            // This is a simplified implementation - in practice we'd track which bars belong to which symbol
            for (symbol, trades) in symbol_trades {
                let range_bars = self.convert_gpu_bars_to_range_bars(&completed_bars, trades)?;
                results.push((*symbol, range_bars));
            }
        }

        Ok(results)
    }

    /// Convert GPU tensor data back to completed bars
    fn deserialize_tensor_to_bars(
        &self,
        tensor: &Tensor<Wgpu, 1>,
    ) -> Result<Vec<GpuCompletedBar>, GpuProcessingError> {
        let data = tensor.clone().into_data().to_vec::<f32>().map_err(|e| {
            GpuProcessingError::TensorError {
                message: format!("Failed to deserialize tensor: {:?}", e),
            }
        })?;

        // Each bar is 5 values: [open, high, low, close, volume]
        if data.len() % 5 != 0 {
            return Err(GpuProcessingError::TensorError {
                message: format!(
                    "Invalid tensor data length: {}, expected multiple of 5",
                    data.len()
                ),
            });
        }

        let num_bars = data.len() / 5;
        let mut bars = Vec::with_capacity(num_bars);

        for i in 0..num_bars {
            let offset = i * 5;
            bars.push(GpuCompletedBar {
                open: data[offset],
                high: data[offset + 1],
                low: data[offset + 2],
                close: data[offset + 3],
                volume: data[offset + 4],
                trade_count: 0, // Would be tracked separately in full implementation
                start_timestamp: 0, // Would be tracked separately in full implementation
            });
        }

        Ok(bars)
    }

    /// Convert GPU bars to RangeBar format with proper metadata
    fn convert_gpu_bars_to_range_bars(
        &self,
        gpu_bars: &[GpuCompletedBar],
        trades: &[AggTrade],
    ) -> Result<Vec<RangeBar>, GpuProcessingError> {
        let mut range_bars = Vec::with_capacity(gpu_bars.len());

        for gpu_bar in gpu_bars.iter() {
            let volume_fp = self.f32_to_fixed_point(gpu_bar.volume)?;
            let turnover =
                (volume_fp.0 as i128) * (self.f32_to_fixed_point(gpu_bar.close)?.0 as i128);

            // Create RangeBar with proper metadata
            let range_bar = RangeBar {
                open_time: trades.first().map(|t| t.timestamp).unwrap_or(0),
                close_time: trades.last().map(|t| t.timestamp).unwrap_or(0),
                open: self.f32_to_fixed_point(gpu_bar.open)?,
                high: self.f32_to_fixed_point(gpu_bar.high)?,
                low: self.f32_to_fixed_point(gpu_bar.low)?,
                close: self.f32_to_fixed_point(gpu_bar.close)?,
                volume: volume_fp,
                turnover,
                trade_count: gpu_bar.trade_count.max(1) as i64,
                first_id: trades.first().map(|t| t.agg_trade_id).unwrap_or(0),
                last_id: trades.last().map(|t| t.agg_trade_id).unwrap_or(0),

                // Approximated microstructure data (simplified 50/50 split)
                buy_volume: self.f32_to_fixed_point(gpu_bar.volume * 0.5)?,
                sell_volume: self.f32_to_fixed_point(gpu_bar.volume * 0.5)?,
                buy_trade_count: ((gpu_bar.trade_count / 2).max(1)) as i64,
                sell_trade_count: ((gpu_bar.trade_count / 2).max(1)) as i64,
                vwap: self.f32_to_fixed_point((gpu_bar.open + gpu_bar.close) * 0.5)?, // Approximated VWAP
                buy_turnover: turnover / 2,
                sell_turnover: turnover / 2,
            };

            range_bars.push(range_bar);
        }

        Ok(range_bars)
    }

    /// Convert f32 to FixedPoint (helper function)
    fn f32_to_fixed_point(
        &self,
        value: f32,
    ) -> Result<crate::fixed_point::FixedPoint, GpuProcessingError> {
        use crate::fixed_point::FixedPoint;

        FixedPoint::from_str(&format!("{:.8}", value)).map_err(|e| {
            GpuProcessingError::TensorError {
                message: format!("Failed to convert f32 to FixedPoint: {:?}", e),
            }
        })
    }

    /// Fallback to CPU processing (temporary until full GPU implementation)
    #[allow(dead_code)]
    fn fallback_cpu_processing(
        &self,
        trades: &[AggTrade],
    ) -> Result<Vec<RangeBar>, GpuProcessingError> {
        use crate::range_bars::RangeBarProcessor;

        let mut cpu_processor = RangeBarProcessor::new(self.threshold_bps);
        cpu_processor
            .process_trades(trades)
            .map_err(|e| GpuProcessingError::FallbackError {
                message: e.to_string(),
            })
    }

    /// Get GPU device information
    pub fn device_info(&self) -> String {
        self.device.info()
    }

    /// Check if Metal backend is being used
    pub fn is_using_metal(&self) -> bool {
        self.device.is_metal()
    }

    /// Extract scalar value from tensor for debugging (handles errors gracefully)
    fn extract_scalar_debug(&self, tensor: &Tensor<Wgpu, 1>) -> Result<f32, String> {
        let data = tensor
            .clone()
            .into_data()
            .to_vec::<f32>()
            .map_err(|e| format!("Failed to extract scalar for debug: {:?}", e))?;

        if data.is_empty() {
            return Err("Empty tensor in debug scalar extraction".to_string());
        }

        Ok(data[0])
    }
}

/// GPU tensor data for range bar processing
#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct GpuTensorData {
    prices: Tensor<Wgpu, 1>,
    volumes: Tensor<Wgpu, 1>,
    timestamps: Tensor<Wgpu, 1>,
    symbols: Tensor<Wgpu, 1>,
    microstructure: Tensor<Wgpu, 1>,
    num_symbols: usize,
    total_trades: usize,
}

/// GPU computation results
#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct GpuRangeBarResults {
    breach_points: Tensor<Wgpu, 1>,
    upper_thresholds: Tensor<Wgpu, 1>,
    lower_thresholds: Tensor<Wgpu, 1>,
    processed_symbols: usize,
}

/// GPU-managed bar state for streaming processing
#[cfg(feature = "gpu")]
struct GpuBarState {
    /// Bar OHLCV data as scalars
    open: Tensor<Wgpu, 1>, // [1] tensor
    high: Tensor<Wgpu, 1>,   // [1] tensor
    low: Tensor<Wgpu, 1>,    // [1] tensor
    close: Tensor<Wgpu, 1>,  // [1] tensor
    volume: Tensor<Wgpu, 1>, // [1] tensor

    /// Fixed thresholds computed from open price
    upper_threshold: Tensor<Wgpu, 1>, // [1] tensor
    lower_threshold: Tensor<Wgpu, 1>, // [1] tensor

    /// Metadata
    start_timestamp: i64,
    trade_count: usize,
}

#[cfg(feature = "gpu")]
impl GpuBarState {
    /// Create new bar state from first trade
    fn new(
        price: Tensor<Wgpu, 1>,
        volume: Tensor<Wgpu, 1>,
        threshold_multiplier: f32,
        _device: &burn::backend::wgpu::WgpuDevice,
    ) -> Result<Self, GpuProcessingError> {
        // Initialize OHLC to the opening price
        let open = price.clone();
        let high = price.clone();
        let low = price.clone();
        let close = price;

        // Compute fixed thresholds from opening price
        let upper_threshold = open.clone().mul_scalar(1.0 + threshold_multiplier);
        let lower_threshold = open.clone().mul_scalar(1.0 - threshold_multiplier);

        Ok(Self {
            open,
            high,
            low,
            close,
            volume,
            upper_threshold,
            lower_threshold,
            start_timestamp: 0, // Would be set from actual trade timestamp
            trade_count: 1,
        })
    }

    /// Update bar with new trades (vectorized)
    #[allow(clippy::single_range_in_vec_init)]
    fn update_with_trades(
        &mut self,
        prices: &Tensor<Wgpu, 1>,
        volumes: &Tensor<Wgpu, 1>,
    ) -> Result<(), GpuProcessingError> {
        // Update high: max(current_high, max(new_prices))
        let max_new_price = prices.clone().max_dim(0); // Get maximum across dimension 0
        self.high = self.high.clone().max_pair(max_new_price);

        // Update low: min(current_low, min(new_prices))
        let min_new_price = prices.clone().min_dim(0); // Get minimum across dimension 0
        self.low = self.low.clone().min_pair(min_new_price);

        // Update close: last price in the chunk
        let chunk_size = prices.dims()[0];
        if chunk_size > 0 {
            self.close = prices.clone().slice([(chunk_size - 1)..chunk_size]);
        }

        // Update volume: sum all volumes
        let volume_sum = volumes.clone().sum();
        self.volume = self.volume.clone().add(volume_sum);

        self.trade_count += chunk_size;

        Ok(())
    }

    /// Close current bar and return completed bar data
    fn close_bar(&self) -> Result<GpuCompletedBar, GpuProcessingError> {
        // Extract scalar values from tensors
        let open = self.extract_scalar(&self.open)?;
        let high = self.extract_scalar(&self.high)?;
        let low = self.extract_scalar(&self.low)?;
        let close = self.extract_scalar(&self.close)?;
        let volume = self.extract_scalar(&self.volume)?;

        Ok(GpuCompletedBar {
            open,
            high,
            low,
            close,
            volume,
            trade_count: self.trade_count,
            start_timestamp: self.start_timestamp,
        })
    }

    /// Extract scalar value from single-element tensor
    fn extract_scalar(&self, tensor: &Tensor<Wgpu, 1>) -> Result<f32, GpuProcessingError> {
        let data = tensor.clone().into_data().to_vec::<f32>().map_err(|e| {
            GpuProcessingError::TensorError {
                message: format!("Failed to extract scalar: {:?}", e),
            }
        })?;

        if data.is_empty() {
            return Err(GpuProcessingError::TensorError {
                message: "Empty tensor in scalar extraction".to_string(),
            });
        }

        Ok(data[0])
    }
}

/// Completed bar data (CPU format for output)
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GpuCompletedBar {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
    trade_count: usize,
    start_timestamp: i64,
}

/// Result from processing a chunk of trades
#[cfg(feature = "gpu")]
struct ChunkProcessingResult {
    completed_bars: Vec<GpuCompletedBar>,
}

/// GPU processing errors
#[cfg(feature = "gpu")]
#[derive(Debug, thiserror::Error)]
pub enum GpuProcessingError {
    #[error("GPU device error: {0}")]
    DeviceError(#[from] GpuError),

    #[error("Tensor operation failed: {message}")]
    TensorError { message: String },

    #[error("Memory allocation failed: {message}")]
    MemoryError { message: String },

    #[error("Batch processing failed: {message}")]
    BatchError { message: String },

    #[error("Fallback to CPU failed: {message}")]
    FallbackError { message: String },
}

// No-op implementations when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub struct GpuRangeBarProcessor;

#[cfg(not(feature = "gpu"))]
impl GpuRangeBarProcessor {
    pub fn new(_device: (), _threshold_bps: u32, _batch_size: Option<usize>) -> Self {
        Self
    }
}

#[cfg(not(feature = "gpu"))]
#[derive(Debug, thiserror::Error)]
pub enum GpuProcessingError {
    #[error("GPU support not compiled")]
    NotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "gpu")]
    use crate::{fixed_point::FixedPoint, gpu::metal_backend::detect_gpu_device};

    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_processor_creation() {
        if let Some(device) = detect_gpu_device() {
            let processor = GpuRangeBarProcessor::new(device, 8000, Some(1000));
            assert_eq!(processor.threshold_bps, 8000);
            assert_eq!(processor.batch_size, 1000);
        } else {
            println!("GPU not available for testing");
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_empty_trades() {
        if let Some(device) = detect_gpu_device() {
            let processor = GpuRangeBarProcessor::new(device, 8000, None);
            let result = processor.process_single_symbol(&[]).unwrap();
            assert!(result.is_empty());
        }
    }

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
    fn test_basic_gpu_processing() {
        if let Some(device) = detect_gpu_device() {
            let processor = GpuRangeBarProcessor::new(device, 8000, None);

            let trades = vec![
                create_test_trade(1, "50000.0", "1.0", 1000),
                create_test_trade(2, "50200.0", "1.0", 2000),
                create_test_trade(3, "50400.0", "1.0", 3000), // Should trigger breach
            ];

            let result = processor.process_single_symbol(&trades);
            assert!(result.is_ok(), "GPU processing should succeed");

            let bars = result.unwrap();
            assert!(!bars.is_empty(), "Should produce range bars");
        } else {
            println!("GPU not available for testing");
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_multi_symbol_batch() {
        if let Some(device) = detect_gpu_device() {
            let processor = GpuRangeBarProcessor::new(device, 8000, None);

            let btc_trades = vec![
                create_test_trade(1, "50000.0", "1.0", 1000),
                create_test_trade(2, "50400.0", "1.0", 2000), // Breach
            ];

            let eth_trades = vec![
                create_test_trade(3, "4000.0", "10.0", 1000),
                create_test_trade(4, "4032.0", "10.0", 2000), // 0.8% breach
            ];

            let symbol_trades = vec![
                ("BTCUSDT", btc_trades.as_slice()),
                ("ETHUSDT", eth_trades.as_slice()),
            ];

            let result = processor.process_multi_symbol_batch(&symbol_trades);
            assert!(
                result.is_ok(),
                "Multi-symbol batch processing should succeed"
            );

            let results = result.unwrap();
            assert_eq!(results.len(), 2, "Should process both symbols");
        } else {
            println!("GPU not available for testing");
        }
    }
}
