//! GPU benchmarking and performance analysis
//!
//! This module provides benchmarking tools to compare GPU vs CPU performance
//! for range bar processing and validate GPU acceleration benefits.

#[cfg(feature = "gpu")]
use std::time::Instant;

#[cfg(feature = "gpu")]
use crate::{
    fixed_point::FixedPoint,
    gpu::{GpuRangeBarProcessor, metal_backend::detect_gpu_device},
    range_bars::RangeBarProcessor,
    types::AggTrade,
};

/// Comprehensive GPU vs CPU benchmark results
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub trade_count: usize,
    pub symbol_count: usize,

    // CPU Performance
    pub cpu_processing_time_ms: f64,
    pub cpu_throughput_trades_per_sec: f64,
    pub cpu_bars_generated: usize,

    // GPU Performance
    pub gpu_processing_time_ms: f64,
    pub gpu_throughput_trades_per_sec: f64,
    pub gpu_bars_generated: usize,

    // Comparison Metrics
    pub speedup_factor: f64,
    pub throughput_improvement: f64,
    pub result_consistency: bool,

    // System Information
    pub gpu_device: String,
    pub gpu_backend: String,
    pub batch_size: usize,
}

#[cfg(feature = "gpu")]
impl BenchmarkResults {
    /// Create summary report of benchmark results
    pub fn summary(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Trades: {}, Symbols: {}\n\
             CPU: {:.2}ms ({:.0} trades/sec, {} bars)\n\
             GPU: {:.2}ms ({:.0} trades/sec, {} bars)\n\
             Speedup: {:.2}x, Throughput: +{:.1}%\n\
             Results Match: {}\n\
             Device: {} ({})",
            self.test_name,
            self.trade_count,
            self.symbol_count,
            self.cpu_processing_time_ms,
            self.cpu_throughput_trades_per_sec,
            self.cpu_bars_generated,
            self.gpu_processing_time_ms,
            self.gpu_throughput_trades_per_sec,
            self.gpu_bars_generated,
            self.speedup_factor,
            self.throughput_improvement * 100.0,
            self.result_consistency,
            self.gpu_device,
            self.gpu_backend
        )
    }

    /// Check if GPU showed meaningful improvement
    pub fn is_gpu_beneficial(&self) -> bool {
        self.speedup_factor > 1.1 && self.result_consistency
    }
}

/// GPU benchmarking suite
#[cfg(feature = "gpu")]
#[allow(dead_code)]
pub struct GpuBenchmarkSuite {
    gpu_processor: Option<GpuRangeBarProcessor>,
    cpu_processor: RangeBarProcessor,
    threshold_bps: u32,
}

#[cfg(feature = "gpu")]
impl GpuBenchmarkSuite {
    /// Create new benchmark suite
    pub fn new(threshold_bps: u32) -> Self {
        let gpu_processor = detect_gpu_device()
            .map(|device| GpuRangeBarProcessor::new(device, threshold_bps, None));

        let cpu_processor = RangeBarProcessor::new(threshold_bps);

        Self {
            gpu_processor,
            cpu_processor,
            threshold_bps,
        }
    }

    /// Run comprehensive benchmark suite
    pub fn run_all_benchmarks(&mut self) -> Vec<BenchmarkResults> {
        let mut results = Vec::new();

        if self.gpu_processor.is_none() {
            println!("GPU not available - skipping GPU benchmarks");
            return results;
        }

        // Single symbol benchmarks with varying data sizes
        for size in [1_000, 10_000, 100_000, 1_000_000] {
            let test_name = format!("single_symbol_{}k_trades", size / 1000);
            if let Ok(result) = self.benchmark_single_symbol(&test_name, size) {
                results.push(result);
            }
        }

        // Multi-symbol benchmarks (GPU advantage)
        for symbol_count in [2, 5, 10, 18] {
            // 18 = Tier-1 USDT pairs
            let test_name = format!("multi_symbol_{}symbols", symbol_count);
            if let Ok(result) = self.benchmark_multi_symbol(&test_name, symbol_count, 10_000) {
                results.push(result);
            }
        }

        // Stress test with realistic Tier-1 data volumes
        let test_name = "tier1_realistic_6month".to_string();
        if let Ok(result) = self.benchmark_tier1_realistic(&test_name) {
            results.push(result);
        }

        results
    }

    /// Benchmark single symbol processing
    pub fn benchmark_single_symbol(
        &mut self,
        test_name: &str,
        trade_count: usize,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        let trades = generate_test_trades(trade_count, 50000.0, 100.0);

        // CPU benchmark
        let cpu_start = Instant::now();
        let cpu_bars = self.cpu_processor.process_trades(&trades).map_err(|e| {
            BenchmarkError::ProcessingError {
                message: e.to_string(),
            }
        })?;
        let cpu_time = cpu_start.elapsed().as_millis() as f64;

        // GPU benchmark
        let gpu_processor = self.gpu_processor.as_ref().unwrap();
        let gpu_start = Instant::now();
        let gpu_bars =
            gpu_processor
                .process_single_symbol(&trades)
                .map_err(|e| BenchmarkError::GpuError {
                    message: e.to_string(),
                })?;
        let gpu_time = gpu_start.elapsed().as_millis() as f64;

        // Calculate metrics
        let cpu_throughput = (trade_count as f64) / (cpu_time / 1000.0);
        let gpu_throughput = (trade_count as f64) / (gpu_time / 1000.0);
        let speedup = cpu_time / gpu_time;
        let throughput_improvement = (gpu_throughput - cpu_throughput) / cpu_throughput;

        // Verify result consistency (simplified check)
        let result_consistency = cpu_bars.len() == gpu_bars.len();

        Ok(BenchmarkResults {
            test_name: test_name.to_string(),
            trade_count,
            symbol_count: 1,
            cpu_processing_time_ms: cpu_time,
            cpu_throughput_trades_per_sec: cpu_throughput,
            cpu_bars_generated: cpu_bars.len(),
            gpu_processing_time_ms: gpu_time,
            gpu_throughput_trades_per_sec: gpu_throughput,
            gpu_bars_generated: gpu_bars.len(),
            speedup_factor: speedup,
            throughput_improvement,
            result_consistency,
            gpu_device: gpu_processor.device_info(),
            gpu_backend: if gpu_processor.is_using_metal() {
                "Metal"
            } else {
                "WGPU"
            }
            .to_string(),
            batch_size: 8000,
        })
    }

    /// Benchmark multi-symbol batch processing (GPU advantage)
    pub fn benchmark_multi_symbol(
        &mut self,
        test_name: &str,
        symbol_count: usize,
        trades_per_symbol: usize,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        // Generate test data for multiple symbols
        let mut symbol_trades = Vec::with_capacity(symbol_count);
        let mut all_trades = Vec::new();

        for i in 0..symbol_count {
            let symbol = format!("SYM{}USDT", i);
            let trades =
                generate_test_trades(trades_per_symbol, 50000.0 + (i as f64 * 1000.0), 100.0);
            all_trades.extend(trades.clone());
            symbol_trades.push((symbol, trades));
        }

        let total_trades = symbol_count * trades_per_symbol;

        // CPU benchmark (sequential processing)
        let cpu_start = Instant::now();
        let mut cpu_total_bars = 0;
        for (_, trades) in &symbol_trades {
            let bars = self.cpu_processor.process_trades(trades).map_err(|e| {
                BenchmarkError::ProcessingError {
                    message: e.to_string(),
                }
            })?;
            cpu_total_bars += bars.len();
        }
        let cpu_time = cpu_start.elapsed().as_millis() as f64;

        // GPU benchmark (batch processing)
        let gpu_processor = self.gpu_processor.as_ref().unwrap();
        let symbol_refs: Vec<(&str, &[AggTrade])> = symbol_trades
            .iter()
            .map(|(symbol, trades)| (symbol.as_str(), trades.as_slice()))
            .collect();

        let gpu_start = Instant::now();
        let gpu_results = gpu_processor
            .process_multi_symbol_batch(&symbol_refs)
            .map_err(|e| BenchmarkError::GpuError {
                message: e.to_string(),
            })?;
        let gpu_time = gpu_start.elapsed().as_millis() as f64;

        let gpu_total_bars: usize = gpu_results.iter().map(|(_, bars)| bars.len()).sum();

        // Calculate metrics
        let cpu_throughput = (total_trades as f64) / (cpu_time / 1000.0);
        let gpu_throughput = (total_trades as f64) / (gpu_time / 1000.0);
        let speedup = cpu_time / gpu_time;
        let throughput_improvement = (gpu_throughput - cpu_throughput) / cpu_throughput;

        // Verify result consistency
        let result_consistency = cpu_total_bars == gpu_total_bars;

        Ok(BenchmarkResults {
            test_name: test_name.to_string(),
            trade_count: total_trades,
            symbol_count,
            cpu_processing_time_ms: cpu_time,
            cpu_throughput_trades_per_sec: cpu_throughput,
            cpu_bars_generated: cpu_total_bars,
            gpu_processing_time_ms: gpu_time,
            gpu_throughput_trades_per_sec: gpu_throughput,
            gpu_bars_generated: gpu_total_bars,
            speedup_factor: speedup,
            throughput_improvement,
            result_consistency,
            gpu_device: gpu_processor.device_info(),
            gpu_backend: if gpu_processor.is_using_metal() {
                "Metal"
            } else {
                "WGPU"
            }
            .to_string(),
            batch_size: 8000,
        })
    }

    /// Benchmark with realistic Tier-1 cryptocurrency volumes
    pub fn benchmark_tier1_realistic(
        &mut self,
        test_name: &str,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        // Simulate 18 Tier-1 USDT pairs with 6-month realistic volumes
        let tier1_symbols = vec![
            "BTCUSDT",
            "ETHUSDT",
            "BNBUSDT",
            "ADAUSDT",
            "XRPUSDT",
            "SOLUSDT",
            "DOGEUSDT",
            "DOTUSDT",
            "AVAXUSDT",
            "SHIBUSDT",
            "MATICUSDT",
            "LTCUSDT",
            "UNIUSDT",
            "LINKUSDT",
            "ATOMUSDT",
            "ETCUSDT",
            "XLMUSDT",
            "NEARUSDT",
        ];

        // Realistic 6-month trade volumes per symbol (simplified)
        let base_trades_per_symbol = 50_000; // Scaled down for testing

        self.benchmark_multi_symbol(test_name, tier1_symbols.len(), base_trades_per_symbol)
    }

    /// Check if GPU is available for benchmarking
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_processor.is_some()
    }

    /// Get GPU device information
    pub fn gpu_info(&self) -> String {
        self.gpu_processor
            .as_ref()
            .map(|p| p.device_info())
            .unwrap_or_else(|| "GPU not available".to_string())
    }
}

/// Generate realistic test trades for benchmarking
#[cfg(feature = "gpu")]
fn generate_test_trades(count: usize, base_price: f64, volatility: f64) -> Vec<AggTrade> {
    let mut trades = Vec::with_capacity(count);
    let mut price = base_price;
    let mut rng = 0x12345678u64; // Simple deterministic RNG

    for i in 0..count {
        // Simple LCG for deterministic price movements
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let random = (rng >> 16) as f64 / 65536.0; // [0, 1)
        let price_change = (random - 0.5) * volatility * 0.001; // Much smaller changes
        price += price_change;

        // Ensure price stays in reasonable range
        price = price.max(base_price * 0.8).min(base_price * 1.2);

        trades.push(AggTrade {
            agg_trade_id: i as i64,
            price: FixedPoint::from_str(&format!("{:.2}", price.max(0.01))).unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: i as i64,
            last_trade_id: i as i64,
            timestamp: 1640995200000 + (i as i64 * 100), // 100ms intervals
            is_buyer_maker: i % 2 == 0,                  // Alternate buy/sell pressure
        });
    }

    trades
}

/// Benchmark execution errors
#[cfg(feature = "gpu")]
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("GPU processing error: {message}")]
    GpuError { message: String },

    #[error("CPU processing error: {message}")]
    ProcessingError { message: String },

    #[error("Benchmark setup error: {message}")]
    SetupError { message: String },

    #[error("GPU not available")]
    GpuNotAvailable,
}

// No-op implementations when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub struct GpuBenchmarkSuite;

#[cfg(not(feature = "gpu"))]
impl GpuBenchmarkSuite {
    pub fn new(_threshold_bps: u32) -> Self {
        Self
    }

    pub fn run_all_benchmarks(&mut self) -> Vec<()> {
        Vec::new()
    }

    pub fn is_gpu_available(&self) -> bool {
        false
    }

    pub fn gpu_info(&self) -> String {
        "GPU support not compiled".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "gpu")]
    fn test_benchmark_suite_creation() {
        let suite = GpuBenchmarkSuite::new(8000);
        println!("GPU Available: {}", suite.is_gpu_available());
        println!("GPU Info: {}", suite.gpu_info());
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_generate_test_trades() {
        let trades = generate_test_trades(1000, 50000.0, 100.0);
        assert_eq!(trades.len(), 1000);
        assert!(trades[0].price.to_f64() > 0.0);
        assert!(trades.windows(2).all(|w| w[1].timestamp > w[0].timestamp));
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_small_benchmark() {
        let mut suite = GpuBenchmarkSuite::new(8000);

        if suite.is_gpu_available() {
            let result = suite.benchmark_single_symbol("test_small", 100);
            match result {
                Ok(benchmark) => {
                    println!("Small benchmark result:\n{}", benchmark.summary());
                    assert!(benchmark.trade_count == 100);
                    assert!(benchmark.cpu_bars_generated > 0 || benchmark.gpu_bars_generated > 0);
                }
                Err(e) => println!("Benchmark failed: {}", e),
            }
        } else {
            println!("GPU not available for testing");
        }
    }
}
