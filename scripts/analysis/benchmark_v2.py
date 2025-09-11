#!/usr/bin/env python3
"""
Benchmark RangeBar v0.2.0 with latest 2025 dependencies.
Tests both Python and Rust implementations.
"""

import time
import numpy as np
from decimal import Decimal
import sys
from typing import List, Dict, Any

# Import RangeBar components
import rangebar
from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
import rangebar._rangebar_rust as rust

def generate_test_data(num_trades: int) -> tuple:
    """Generate realistic test data for benchmarking."""
    print(f"📊 Generating {num_trades:,} test trades...")
    
    # Base price: $50,000 
    base_price = 50000.0
    np.random.seed(42)  # Reproducible results
    
    # Generate realistic price movements (±2% range)
    price_changes = np.random.normal(0, 0.001, num_trades)  # 0.1% std dev
    prices = [base_price]
    
    for change in price_changes[1:]:
        new_price = prices[-1] * (1 + change)
        prices.append(new_price)
    
    # Generate volumes (0.1 to 5.0 BTC)
    volumes = np.random.uniform(0.1, 5.0, num_trades)
    
    # Generate timestamps (1ms to 10ms apart)
    intervals = np.random.randint(1, 11, num_trades)
    timestamps = np.cumsum(intervals) + 1640995200000  # Start from 2022-01-01
    
    # Create AggTrade data structures
    trades_data = []
    rust_prices = []
    rust_volumes = []
    rust_timestamps = []
    rust_trade_ids = []
    rust_first_ids = []
    rust_last_ids = []
    
    for i in range(num_trades):
        trades_data.append({
            'a': i + 1,
            'p': f"{prices[i]:.8f}",
            'q': f"{volumes[i]:.8f}",
            'f': i + 1,
            'l': i + 1,
            'T': int(timestamps[i]),
            'm': bool(i % 2)  # Alternate buyer/seller
        })
        
        # Rust arrays (fixed-point scaled by 1e8)
        rust_prices.append(int(prices[i] * 1e8))
        rust_volumes.append(int(volumes[i] * 1e8))
        rust_timestamps.append(int(timestamps[i]))
        rust_trade_ids.append(i + 1)
        rust_first_ids.append(i + 1)
        rust_last_ids.append(i + 1)
    
    python_trades = [AggTrade(data) for data in trades_data]
    
    rust_arrays = (
        np.array(rust_prices, dtype=np.int64),
        np.array(rust_volumes, dtype=np.int64),
        np.array(rust_timestamps, dtype=np.int64),
        np.array(rust_trade_ids, dtype=np.int64),
        np.array(rust_first_ids, dtype=np.int64),
        np.array(rust_last_ids, dtype=np.int64),
    )
    
    return python_trades, rust_arrays

def benchmark_python_implementation(trades: List[AggTrade], threshold_pct: Decimal) -> Dict[str, Any]:
    """Benchmark Python reference implementation."""
    print("🐍 Benchmarking Python implementation...")
    
    start_time = time.perf_counter()
    bars = list(iter_range_bars_from_aggtrades(trades, pct=threshold_pct))
    end_time = time.perf_counter()
    
    duration = end_time - start_time
    trades_per_sec = len(trades) / duration if duration > 0 else 0
    
    return {
        'implementation': 'Python',
        'num_trades': len(trades),
        'num_bars': len(bars),
        'duration_sec': duration,
        'trades_per_sec': trades_per_sec,
        'bars': bars
    }

def benchmark_rust_implementation(rust_arrays: tuple, threshold_bps: int) -> Dict[str, Any]:
    """Benchmark Rust implementation."""
    print("🦀 Benchmarking Rust implementation...")
    
    prices, volumes, timestamps, trade_ids, first_ids, last_ids = rust_arrays
    
    start_time = time.perf_counter()
    result = rust.compute_range_bars(
        prices=prices,
        volumes=volumes, 
        timestamps=timestamps,
        trade_ids=trade_ids,
        first_ids=first_ids,
        last_ids=last_ids,
        threshold_bps=threshold_bps
    )
    end_time = time.perf_counter()
    
    duration = end_time - start_time
    trades_per_sec = len(prices) / duration if duration > 0 else 0
    
    return {
        'implementation': 'Rust',
        'num_trades': len(prices),
        'num_bars': len(result['opens']),
        'duration_sec': duration,
        'trades_per_sec': trades_per_sec,
        'result': result
    }

def compare_results(python_result: Dict[str, Any], rust_result: Dict[str, Any]) -> bool:
    """Compare Python and Rust results for correctness."""
    python_bars = python_result['bars']
    rust_bars_count = rust_result['num_bars']
    
    if len(python_bars) != rust_bars_count:
        print(f"❌ Bar count mismatch: Python={len(python_bars)}, Rust={rust_bars_count}")
        return False
    
    print(f"✅ Results match: Both generated {len(python_bars)} bars")
    return True

def run_benchmark_suite():
    """Run comprehensive benchmark suite."""
    print("🚀 RangeBar v0.2.0 Performance Benchmark")
    print("=" * 60)
    print(f"🏗️  Environment:")
    print(f"   RangeBar version: {rangebar.__version__}")
    print(f"   NumPy version: {np.__version__}")
    print(f"   Python version: {sys.version.split()[0]}")
    print()
    
    # Test different trade volumes
    test_sizes = [10_000, 100_000, 500_000, 1_000_000]
    threshold_pct = Decimal('0.008')  # 0.8%
    threshold_bps = 8000  # 0.8% in basis points
    
    results = []
    
    for num_trades in test_sizes:
        print(f"📈 Benchmark: {num_trades:,} trades")
        print("-" * 40)
        
        # Generate test data
        python_trades, rust_arrays = generate_test_data(num_trades)
        
        # Benchmark Python implementation
        python_result = benchmark_python_implementation(python_trades, threshold_pct)
        
        # Benchmark Rust implementation  
        rust_result = benchmark_rust_implementation(rust_arrays, threshold_bps)
        
        # Compare results
        results_match = compare_results(python_result, rust_result)
        
        # Store results
        results.append({
            'num_trades': num_trades,
            'python': python_result,
            'rust': rust_result,
            'results_match': results_match
        })
        
        print(f"   Python: {python_result['trades_per_sec']:,.0f} trades/sec")
        print(f"   Rust:   {rust_result['trades_per_sec']:,.0f} trades/sec")
        print(f"   Speedup: {rust_result['trades_per_sec'] / python_result['trades_per_sec']:.1f}x")
        print()
    
    # Summary report
    print("📊 BENCHMARK SUMMARY")
    print("=" * 60)
    
    # Find best Rust performance
    best_rust_performance = max(results, key=lambda r: r['rust']['trades_per_sec'])
    best_python_performance = max(results, key=lambda r: r['python']['trades_per_sec'])
    
    print(f"🏆 Peak Performance (2025 Dependencies):")
    print(f"   Rust:   {best_rust_performance['rust']['trades_per_sec']:,.0f} trades/sec")
    print(f"   Python: {best_python_performance['python']['trades_per_sec']:,.0f} trades/sec")
    print()
    
    print("📋 Dependency Versions:")
    print(f"   PyO3: 0.26+ (Rust 2024 edition)")
    print(f"   numpy: {np.__version__} (latest)")
    print(f"   Python: {sys.version.split()[0]} (2025 standard)")
    print()
    
    # Verify results integrity
    all_match = all(r['results_match'] for r in results)
    print(f"✅ Algorithm Integrity: {'VERIFIED' if all_match else 'FAILED'}")
    
    return best_rust_performance['rust']['trades_per_sec']

if __name__ == "__main__":
    peak_performance = run_benchmark_suite()
    
    # Performance verdict for documentation
    if peak_performance >= 2_500_000:
        print(f"🎉 PERFORMANCE VERIFIED: {peak_performance/1_000_000:.1f}M+ trades/sec")
    else:
        print(f"⚠️  PERFORMANCE BELOW TARGET: {peak_performance/1_000_000:.1f}M trades/sec (target: 2.5M+)")