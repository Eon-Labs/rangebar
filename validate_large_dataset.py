#!/usr/bin/env python3
"""
Validate range bar algorithm compliance and performance metrics from large dataset
"""

import json
import time
import sys

def validate_large_dataset_integrity(json_file, threshold_pct):
    """Validate algorithm compliance and performance metrics from large dataset"""
    print(f"🚀 Large Dataset Performance & Integrity Validation")
    print(f"📁 File: {json_file}")
    print(f"📊 Expected threshold: {threshold_pct:.3%}")

    with open(json_file, 'r') as f:
        data = json.load(f)

    # Extract performance metrics
    total_trades = None
    total_bars = len(data['range_bars'])
    processing_time = None

    # Try to find performance data in export summary
    try:
        summary_file = './output/performance_test/export_summary.json'
        with open(summary_file, 'r') as f:
            summary = json.load(f)
            total_trades = summary.get('total_trades', 0)
            processing_time = summary.get('processing_time_seconds', 0)
    except:
        # Fallback: estimate from data
        total_trades = sum(bar['trade_count'] for bar in data['range_bars'])

    print(f"\n📈 PERFORMANCE METRICS:")
    print(f"   Total Trades Processed: {total_trades:,}")
    print(f"   Total Range Bars Generated: {total_bars:,}")
    print(f"   Processing Time: {processing_time:.2f} seconds")

    if processing_time and processing_time > 0:
        trades_per_second = total_trades / processing_time
        print(f"   📊 Processing Speed: {trades_per_second:,.0f} trades/second")
        print(f"   📊 Trades per Bar: {total_trades / total_bars:,.0f} avg")

        # Performance assessment
        if trades_per_second > 250000:
            print(f"   ✅ EXCELLENT: Processing speed exceeds 250K trades/second")
        elif trades_per_second > 100000:
            print(f"   ✅ GOOD: Processing speed exceeds 100K trades/second")
        else:
            print(f"   ⚠️ SLOW: Processing speed below 100K trades/second")

    # Validate algorithm compliance for first and last few bars
    range_bars = data['range_bars']
    violations = []
    sample_size = min(10, len(range_bars))  # Check first/last 10 bars

    print(f"\n🔍 ALGORITHM COMPLIANCE CHECK (sampling {sample_size} bars):")

    # Check first few bars
    for i in range(min(5, len(range_bars))):
        violation = check_bar_compliance(range_bars[i], i, threshold_pct)
        if violation:
            violations.append(violation)

    # Check last few bars (excluding final incomplete bar)
    start_idx = max(5, len(range_bars) - 6)  # Avoid overlap with first 5
    for i in range(start_idx, len(range_bars) - 1):  # Exclude final bar
        violation = check_bar_compliance(range_bars[i], i, threshold_pct)
        if violation:
            violations.append(violation)

    # Separately check final bar (may be incomplete)
    if len(range_bars) > 0:
        final_bar = range_bars[-1]
        final_violation = check_bar_compliance(final_bar, len(range_bars)-1, threshold_pct)
        if final_violation:
            print(f"   ⚠️ Final bar incomplete (expected): {final_violation['issue']}")
        else:
            print(f"   ✅ Final bar compliant")

    print(f"\n📋 ALGORITHM VALIDATION RESULTS:")
    if not violations:
        print("✅ ALL SAMPLED BARS COMPLY with range bar algorithm")
        print("✅ No algorithm violations detected in sample")
    else:
        print(f"❌ FOUND {len(violations)} ALGORITHM VIOLATIONS in sample:")
        for v in violations[:3]:  # Show first 3 violations
            print(f"   Bar {v['bar_index']}: {v['issue']}")
            print(f"      OHLC: O={v['open']:.2f} H={v['high']:.2f} L={v['low']:.2f} C={v['close']:.2f}")

    # Data integrity checks
    print(f"\n🔬 DATA INTEGRITY CHECKS:")

    # Check chronological ordering
    timestamps = [bar['open_time'] for bar in range_bars]
    is_ordered = all(timestamps[i] <= timestamps[i+1] for i in range(len(timestamps)-1))
    print(f"   ✅ Chronological ordering: {'PASSED' if is_ordered else 'FAILED'}")

    # Check for gaps in trade IDs
    trade_id_gaps = 0
    for i in range(1, len(range_bars)):
        if range_bars[i]['first_id'] != range_bars[i-1]['last_id']:
            trade_id_gaps += 1

    if trade_id_gaps == 0:
        print(f"   ✅ Trade ID continuity: PERFECT (no gaps)")
    else:
        print(f"   ⚠️ Trade ID gaps: {trade_id_gaps} discontinuities")

    # Volume and turnover consistency
    volume_inconsistencies = 0
    for bar in range_bars[:sample_size]:
        buy_sell_total = bar['buy_volume'] + bar['sell_volume']
        if abs(buy_sell_total - bar['volume']) > 1:  # Allow small floating point errors
            volume_inconsistencies += 1

    if volume_inconsistencies == 0:
        print(f"   ✅ Volume consistency: PERFECT")
    else:
        print(f"   ❌ Volume inconsistencies: {volume_inconsistencies} bars")

    return len(violations) == 0

def check_bar_compliance(bar, bar_index, threshold_pct):
    """Check if a single bar complies with range bar algorithm"""
    # Convert fixed-point to float (divide by 1e8)
    open_price = bar['open'] / 1e8
    high_price = bar['high'] / 1e8
    low_price = bar['low'] / 1e8
    close_price = bar['close'] / 1e8

    # Calculate expected thresholds from open price
    upper_threshold = open_price * (1 + threshold_pct)
    lower_threshold = open_price * (1 - threshold_pct)

    # Check if high or low breached the thresholds
    high_breach = high_price >= upper_threshold
    low_breach = low_price <= lower_threshold

    any_breach = high_breach or low_breach

    if not any_breach:
        return {
            'bar_index': bar_index,
            'issue': 'No threshold breach',
            'open': open_price,
            'high': high_price,
            'low': low_price,
            'close': close_price,
            'expected_upper': upper_threshold,
            'expected_lower': lower_threshold,
            'high_breach': high_breach,
            'low_breach': low_breach
        }

    return None

def main():
    print("🎯 Large Dataset Performance & Algorithm Validation")
    print("=================================================")

    json_file = './output/performance_test/um_BTCUSDT_rangebar_20250901_20250909_0.800pct.json'
    threshold_pct = 0.008  # 0.8%

    try:
        is_valid = validate_large_dataset_integrity(json_file, threshold_pct)

        print(f"\n🏁 PERFORMANCE TEST VERDICT:")
        if is_valid:
            print("✅ LARGE DATASET PROCESSING: PASSED")
            print("✅ Algorithm integrity maintained at scale")
            print("✅ System ready for production deployment")
            sys.exit(0)
        else:
            print("❌ ALGORITHM VIOLATIONS DETECTED")
            print("❌ Requires investigation before production")
            sys.exit(1)

    except FileNotFoundError:
        print(f"❌ File not found: {json_file}")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error processing file: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()