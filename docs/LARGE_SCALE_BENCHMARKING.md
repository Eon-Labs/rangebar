# Large-Scale GPU vs CPU Benchmarking Framework

Comprehensive performance validation framework for range bar GPU algorithmic CPU parity with production-scale data volumes.

## Overview

The large-scale benchmarking framework provides systematic performance validation between GPU and CPU range bar implementations with:

- **Production-scale data volumes** (1K to 50M+ trades)
- **Multi-symbol parallel processing** (1-18 Tier-1 symbols)
- **Real Binance UM Futures data validation**
- **Financial-grade precision verification** (≤1bp tolerance)
- **Statistical significance testing** (multiple repetitions)
- **Cloud cost estimation** and resource usage analysis
- **Comprehensive reporting** with performance curves and anomaly detection

## Prerequisites

### System Requirements

- **Memory**: 16GB+ recommended for large-scale tests
- **Storage**: 10GB+ for output files and real data
- **GPU**: Optional but recommended for GPU vs CPU comparison
- **CPU**: Multi-core recommended for parallel processing

### Dependencies

- Rust 1.80+ with release optimization
- Optional: GPU drivers (Metal on macOS, CUDA/Vulkan on Linux)
- Optional: Real Binance UM Futures data in `./output/gpu_benchmark_real_data/`

## Quick Start

### 1. Build the Framework

```bash
# Build optimized release version
cargo build --release --bin large-scale-gpu-cpu-benchmark

# Or with GPU support
cargo build --release --features gpu --bin large-scale-gpu-cpu-benchmark
```

### 2. Run Quick Test

```bash
# Quick validation test (1K-100K trades, limited symbols)
cargo run --release --bin large-scale-gpu-cpu-benchmark -- --quick
```

### 3. Run Full Production Test

```bash
# Full production-scale test (up to 50M trades, all Tier-1 symbols)
cargo run --release --features gpu --bin large-scale-gpu-cpu-benchmark -- --production
```

### 4. Run Default Configuration

```bash
# Balanced test configuration
cargo run --release --features gpu --bin large-scale-gpu-cpu-benchmark
```

## Configuration Options

### Command Line Arguments

- `--quick`: Fast validation (1K-100K trades, 1-5 symbols, 1 repetition)
- `--production`: Full production scale (1K-50M trades, 1-18 symbols, 5 repetitions)
- Default: Balanced configuration (1K-10M trades, 1-18 symbols, 3 repetitions)

### Data Volumes Tested

| Scale | Trades | Use Case | Estimated Time |
|-------|--------|----------|----------------|
| Small | 1K-10K | Development validation | <1 minute |
| Medium | 100K | Production validation | 1-5 minutes |
| Large | 1M | High-frequency trading | 5-30 minutes |
| Very Large | 10M | Daily aggregation | 30-120 minutes |
| Extreme | 50M+ | Multi-day analysis | 2-8 hours |

### Symbol Counts

- **Single Symbol** (1): Baseline performance
- **Small Batch** (5): Small-scale parallel processing
- **Medium Batch** (10): Medium-scale parallel processing
- **Tier-1 Full** (18): All Tier-1 USDT perpetual pairs

### Thresholds Tested

- **0.5%** (500 bps): High-frequency bars
- **0.8%** (800 bps): Standard configuration
- **1.0%** (1000 bps): Medium-frequency bars
- **1.5%** (1500 bps): Low-frequency bars
- **2.0%** (2000 bps): Very low-frequency bars

## Test Types

### 1. Single Symbol Scaling Tests

Validates performance scaling with increasing data volumes for single symbols.

**Metrics**:
- Processing time (CPU vs GPU)
- Throughput (trades/second)
- Memory usage
- Speedup factor
- Algorithmic parity

### 2. Multi-Symbol Parallel Processing Tests

Tests GPU's advantage in parallel multi-symbol processing vs sequential CPU processing.

**Configuration**:
- 2, 5, 10, 18 symbols simultaneously
- Various trades per symbol (10K, 100K, 1M)
- Parallel GPU batch processing vs sequential CPU

### 3. Real Data Validation Tests

Uses authentic Binance UM Futures aggTrades data for validation.

**Data Sources**:
- BTCUSDT, ETHUSDT, SOLUSDT, ADAUSDT, BNBUSDT
- Real market volatility patterns
- Production timestamp sequences
- Authentic volume distributions

### 4. Production Stress Testing

Simulates production workloads with realistic Tier-1 symbol volumes.

**Scenarios**:
- All 18 Tier-1 USDT pairs
- 6-month realistic trade volumes
- Peak trading period simulation
- Memory constraint testing

### 5. Memory Scaling Analysis

Analyzes memory usage patterns and scaling behavior.

**Analysis**:
- Memory usage per trade
- Peak memory consumption
- Memory efficiency improvements
- Out-of-memory boundary detection

## Output and Reporting

### Directory Structure

```
./output/large_scale_benchmark/
├── large_scale_benchmark_YYYYMMDD_HHMMSS.json  # Detailed results
├── performance_curves/                          # Performance graphs
├── validation_reports/                          # Precision validation
└── system_reports/                             # System information
```

### JSON Results Schema

```json
{
  "config": { ... },                    // Test configuration
  "system_info": { ... },               // Hardware/software info
  "test_results": [ ... ],               // Individual test results
  "summary_statistics": { ... },        // Aggregated performance metrics
  "performance_curves": { ... },        // Scaling analysis
  "validation_results": { ... },        // Precision/parity validation
  "timestamp": "2025-09-16T23:30:00Z"
}
```

### Key Metrics

#### Performance Metrics
- **Speedup Factor**: GPU time / CPU time
- **Throughput Improvement**: (GPU throughput - CPU throughput) / CPU throughput
- **Memory Efficiency**: GPU memory usage vs CPU memory usage
- **Cost Efficiency**: Performance improvement vs resource cost

#### Validation Metrics
- **Algorithmic Parity**: 100% = perfect GPU/CPU algorithm match
- **Precision Validation**: OHLC values within ≤1bp tolerance
- **Bar Count Validation**: Identical number of bars generated
- **Temporal Consistency**: Identical bar timing and sequence

#### System Metrics
- **Peak Memory Usage**: Maximum memory consumption
- **Processing Time**: Total benchmark execution time
- **Cloud Cost Estimate**: Estimated AWS/GCP compute cost
- **Resource Utilization**: CPU/GPU/memory efficiency

## Performance Analysis

### Interpreting Results

#### Speedup Factor
- **>2.0x**: Significant GPU advantage
- **1.1-2.0x**: Moderate GPU advantage
- **<1.1x**: Minimal or no GPU advantage

#### Validation Success Rates
- **100%**: Perfect algorithmic parity
- **>99%**: Acceptable with investigation
- **<99%**: Algorithmic issues requiring fixes

#### Precision Tolerance
- **≤1bp**: Financial-grade precision maintained
- **>1bp**: Precision degradation - investigate

### Troubleshooting Common Issues

#### Low GPU Performance
1. Check GPU memory availability
2. Verify GPU feature compilation (`--features gpu`)
3. Ensure proper GPU drivers
4. Check for thermal throttling

#### Precision Violations
1. Review threshold calculations
2. Check fixed-point arithmetic precision
3. Validate breach detection logic
4. Examine edge case handling

#### Memory Issues
1. Reduce `max_memory_mb` in configuration
2. Use smaller data volumes
3. Enable memory monitoring
4. Check for memory leaks

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Large-Scale GPU Benchmark

on:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday 2 AM

jobs:
  benchmark:
    runs-on: gpu-runner
    steps:
    - uses: actions/checkout@v3
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - name: Run Quick Benchmark
      run: cargo run --release --features gpu --bin large-scale-gpu-cpu-benchmark -- --quick
    - name: Upload Results
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: ./output/large_scale_benchmark/
```

### Performance Regression Detection

```bash
# Compare current benchmark with baseline
cargo run --release --bin large-scale-gpu-cpu-benchmark -- --quick > current_results.json

# Analysis script to detect regressions
python scripts/compare_benchmark_results.py baseline_results.json current_results.json
```

## Advanced Configuration

### Custom Configuration File

Create `benchmark_config.json`:

```json
{
  "data_volumes": [1000, 50000, 100000],
  "symbol_counts": [1, 5],
  "thresholds_bps": [800],
  "repetitions": 3,
  "max_memory_mb": 8000,
  "precision_tolerance_bps": 1,
  "use_real_data": true,
  "output_dir": "./output/custom_benchmark"
}
```

### Environment Variables

```bash
# Memory limits
export MAX_BENCHMARK_MEMORY_MB=16000

# GPU selection
export CUDA_VISIBLE_DEVICES=0

# Output verbosity
export RUST_LOG=debug
```

## Performance Baselines

### Expected Results (MacBook Pro M3 Max)

| Test Type | Volume | CPU (trades/sec) | GPU (trades/sec) | Speedup |
|-----------|--------|------------------|------------------|---------|
| Single Symbol | 100K | 50,000 | 500,000 | 10x |
| Single Symbol | 1M | 45,000 | 1,200,000 | 26x |
| Multi-Symbol (5) | 500K | 40,000 | 800,000 | 20x |
| Multi-Symbol (18) | 1.8M | 35,000 | 1,000,000 | 28x |

### Expected Results (NVIDIA RTX 4090)

| Test Type | Volume | CPU (trades/sec) | GPU (trades/sec) | Speedup |
|-----------|--------|------------------|------------------|---------|
| Single Symbol | 100K | 35,000 | 800,000 | 22x |
| Single Symbol | 1M | 32,000 | 2,500,000 | 78x |
| Multi-Symbol (5) | 500K | 30,000 | 1,500,000 | 50x |
| Multi-Symbol (18) | 1.8M | 28,000 | 2,000,000 | 71x |

## Best Practices

### 1. Pre-Benchmark Checklist

- [ ] System has sufficient memory (16GB+ for large tests)
- [ ] GPU drivers are up to date
- [ ] No other intensive processes running
- [ ] Real data is available (if using `--production`)
- [ ] Output directory has sufficient disk space

### 2. Benchmark Execution

- [ ] Run benchmarks on dedicated hardware
- [ ] Use release builds only (`--release`)
- [ ] Enable GPU features if available (`--features gpu`)
- [ ] Monitor system resources during execution
- [ ] Save results with timestamps

### 3. Results Analysis

- [ ] Verify algorithmic parity is 100%
- [ ] Check precision validation success rates
- [ ] Review performance curves for scaling behavior
- [ ] Investigate any anomalies or outliers
- [ ] Compare against baseline performance

### 4. Continuous Monitoring

- [ ] Set up automated weekly benchmarks
- [ ] Track performance regressions
- [ ] Monitor precision degradation
- [ ] Update baselines after algorithmic changes
- [ ] Archive historical results

## Contributing

### Adding New Test Types

1. Extend `TestType` enum in `large_scale_gpu_cpu_benchmark.rs`
2. Implement test execution method
3. Add configuration options
4. Update documentation

### Performance Optimizations

1. Profile bottlenecks using `cargo flamegraph`
2. Optimize data structures and algorithms
3. Validate changes with benchmark suite
4. Document performance improvements

## Support

For issues with the large-scale benchmarking framework:

1. Check system requirements and dependencies
2. Review troubleshooting guide above
3. Enable debug logging (`RUST_LOG=debug`)
4. Create issue with benchmark configuration and results
5. Include system information and error logs

## Related Documentation

- [GPU Implementation Guide](./GPU_IMPLEMENTATION.md)
- [Range Bar Algorithm Specification](./ALGORITHM.md)
- [Performance Optimization Guide](./PERFORMANCE.md)
- [Precision Validation Documentation](./PRECISION.md)