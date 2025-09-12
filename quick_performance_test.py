#!/usr/bin/env python3
"""
Quick performance test for range bar processing without requiring Rust build.
Tests the pure Python implementation to get baseline performance measurements.
"""

import time
import numpy as np
from decimal import Decimal
import sys
import os

# Add src to path to import rangebar modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

try:
    from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
    print("✅ Successfully imported rangebar Python modules")
except ImportError as e:
    print(f"❌ Could not import rangebar modules: {e}")
    print("This test requires the Python rangebar package to be available")
    sys.exit(1)

def generate_test_trades(num_trades: int, base_price: float = 50000.0) -> list:
    """Generate realistic test trade data"""
    print(f"📊 Generating {num_trades:,} test trades...")
    
    np.random.seed(42)  # Reproducible results
    
    # Generate realistic price movements
    price_changes = np.random.normal(0, 0.001, num_trades)  # 0.1% std dev
    prices = [base_price]
    
    for change in price_changes[1:]:
        new_price = prices[-1] * (1 + change)
        prices.append(new_price)
    
    # Generate trade data
    trades_data = []
    base_timestamp = 1640995200000  # 2022-01-01
    
    for i in range(num_trades):
        trades_data.append({
            'a': i + 1,                    # aggTradeId
            'p': f"{prices[i]:.8f}",       # price
            'q': f"{np.random.uniform(0.1, 5.0):.8f}",  # quantity
            'f': i + 1,                    # firstTradeId
            'l': i + 1,                    # lastTradeId
            'T': base_timestamp + (i * 100),  # timestamp (100ms apart)
            'm': bool(i % 2)               # is_buyer_maker
        })
    
    # Convert to AggTrade objects
    trades = [AggTrade(data) for data in trades_data]
    print(f"✅ Generated {len(trades):,} AggTrade objects")
    
    return trades

def benchmark_python_processing(trades: list, threshold_pct: Decimal) -> dict:
    """Benchmark the Python range bar processing"""
    print(f"🐍 Testing Python range bar processing with {len(trades):,} trades...")
    
    start_time = time.perf_counter()
    
    # Process trades into range bars
    bars = list(iter_range_bars_from_aggtrades(trades, pct=threshold_pct))
    
    end_time = time.perf_counter()
    
    duration = end_time - start_time
    trades_per_sec = len(trades) / duration if duration > 0 else 0
    
    print(f"⏱️  Processing time: {duration:.4f} seconds")
    print(f"📈 Throughput: {trades_per_sec:,.0f} trades/sec")
    print(f"📊 Generated: {len(bars)} range bars")
    
    # Show first few bars for validation
    if bars:
        print(f"📋 First bar: open={bars[0].open}, high={bars[0].high}, low={bars[0].low}, close={bars[0].close}")
        if len(bars) > 1:
            print(f"📋 Last bar:  open={bars[-1].open}, high={bars[-1].high}, low={bars[-1].low}, close={bars[-1].close}")
    
    return {
        'num_trades': len(trades),
        'num_bars': len(bars),
        'duration_sec': duration,
        'trades_per_sec': trades_per_sec
    }

def run_performance_test():
    """Run performance test suite"""
    print("🚀 Quick Range Bar Performance Test")
    print("=" * 50)
    
    # Test with different trade volumes
    test_sizes = [1_000, 5_000, 10_000]
    threshold_pct = Decimal('0.008')  # 0.8%
    
    results = []
    
    for num_trades in test_sizes:
        print(f"\n📈 Testing {num_trades:,} trades")
        print("-" * 30)
        
        # Generate test data
        trades = generate_test_trades(num_trades)
        
        # Benchmark processing
        result = benchmark_python_processing(trades, threshold_pct)
        results.append(result)
        
        print()
    
    # Summary
    print("📊 PERFORMANCE SUMMARY")
    print("=" * 50)
    
    for result in results:
        print(f"💻 {result['num_trades']:>6,} trades: {result['trades_per_sec']:>8,.0f} trades/sec "
              f"({result['duration_sec']:.3f}s) → {result['num_bars']} bars")
    
    # Best performance
    best_result = max(results, key=lambda r: r['trades_per_sec'])
    print(f"\n🏆 Peak Performance: {best_result['trades_per_sec']:,.0f} trades/sec")
    
    # Performance assessment
    if best_result['trades_per_sec'] >= 50_000:
        print("✅ Python performance: Good (>50K trades/sec)")
    elif best_result['trades_per_sec'] >= 10_000:
        print("⚠️  Python performance: Moderate (>10K trades/sec)")
    else:
        print("❌ Python performance: Low (<10K trades/sec)")
    
    print(f"\n📝 Note: This is pure Python performance.")
    print(f"   Rust implementation should be 100-1000x faster.")
    
    return best_result['trades_per_sec']

if __name__ == "__main__":
    try:
        peak_performance = run_performance_test()
        print(f"\n🎯 Python Baseline: {peak_performance/1000:.1f}K trades/sec")
    except Exception as e:
        print(f"❌ Performance test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)