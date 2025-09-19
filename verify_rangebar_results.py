#!/usr/bin/env python3
"""
Verify range bar algorithm results by decoding fixed-point values
and validating the 0.8% threshold calculations.
"""

import json
import csv
from decimal import Decimal, getcontext
from datetime import datetime

# Set high precision for calculations
getcontext().prec = 28

def decode_fixed_point(value, decimals=8):
    """Convert fixed-point integer back to decimal"""
    return Decimal(value) / Decimal(10 ** decimals)

def verify_range_bar_threshold(open_price, close_price, threshold_bps=80):
    """Verify if close price breaches the threshold from open price (threshold in basis points)"""
    threshold_ratio = Decimal(threshold_bps) / Decimal('10000')
    upper_threshold = open_price * (1 + threshold_ratio)
    lower_threshold = open_price * (1 - threshold_ratio)
    
    upper_breach = close_price >= upper_threshold
    lower_breach = close_price <= lower_threshold
    
    return {
        'open': open_price,
        'close': close_price,
        'upper_threshold': upper_threshold,
        'lower_threshold': lower_threshold,
        'upper_breach': upper_breach,
        'lower_breach': lower_breach,
        'any_breach': upper_breach or lower_breach,
        'breach_magnitude': max(
            (close_price - upper_threshold) / open_price if upper_breach else Decimal(0),
            (lower_threshold - close_price) / open_price if lower_breach else Decimal(0)
        )
    }

def analyze_range_bars():
    """Analyze and verify the generated range bars"""
    
    print("🔍 Range Bar Algorithm Verification")
    print("=" * 50)
    
    # Load JSON results
    with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
        data = json.load(f)
    
    range_bars = data['range_bars']
    threshold_bps = data['metadata']['algorithm']['parameters'].get('threshold_bps',
                                                                   int(data['metadata']['algorithm']['parameters']['threshold_pct'] * 10000))
    
    print(f"📊 Analyzing {len(range_bars)} range bars")
    print(f"🎯 Threshold: {threshold_bps} basis points ({threshold_bps/100:.2f}%)")
    print(f"📈 Total trades processed: {data['metadata']['statistics']['market_data']['total_trades']:,}")
    print()
    
    validation_results = []
    
    for i, bar in enumerate(range_bars):
        # Decode fixed-point values
        open_price = decode_fixed_point(bar['open'])
        high_price = decode_fixed_point(bar['high'])
        low_price = decode_fixed_point(bar['low'])
        close_price = decode_fixed_point(bar['close'])
        volume = decode_fixed_point(bar['volume'])
        trade_count = bar['trade_count']
        
        # Verify threshold calculation
        threshold_check = verify_range_bar_threshold(open_price, close_price, threshold_bps)
        
        # Check OHLCV validity
        ohlcv_valid = (
            low_price <= open_price <= high_price and
            low_price <= close_price <= high_price and
            low_price <= high_price
        )
        
        validation_results.append({
            'bar_index': i,
            'open': open_price,
            'high': high_price,
            'low': low_price,
            'close': close_price,
            'volume': volume,
            'trade_count': trade_count,
            'threshold_check': threshold_check,
            'ohlcv_valid': ohlcv_valid,
            'duration_seconds': (bar['close_time'] - bar['open_time']) / 1000
        })
    
    # Print detailed analysis
    print("📋 Range Bar Validation Results:")
    print("-" * 80)
    print(f"{'Bar':<3} {'Open':<10} {'High':<10} {'Low':<10} {'Close':<10} {'Breach':<8} {'Valid':<5} {'Trades':<8}")
    print("-" * 80)
    
    valid_bars = 0
    breach_bars = 0
    
    for result in validation_results:
        bar_num = result['bar_index'] + 1
        open_val = float(result['open'])
        high_val = float(result['high'])
        low_val = float(result['low'])
        close_val = float(result['close'])
        breached = result['threshold_check']['any_breach']
        valid = result['ohlcv_valid']
        trades = result['trade_count']
        
        print(f"{bar_num:<3} {open_val:<10.2f} {high_val:<10.2f} {low_val:<10.2f} {close_val:<10.2f} "
              f"{'✅' if breached else '❌':<8} {'✅' if valid else '❌':<5} {trades:<8,}")
        
        if valid:
            valid_bars += 1
        if breached:
            breach_bars += 1
    
    print("-" * 80)
    print(f"📊 Summary:")
    print(f"   Valid OHLCV bars: {valid_bars}/{len(range_bars)} ({valid_bars/len(range_bars)*100:.1f}%)")
    print(f"   Threshold breached: {breach_bars}/{len(range_bars)} ({breach_bars/len(range_bars)*100:.1f}%)")
    print()
    
    # Detailed threshold analysis for first few bars
    print("🔬 Detailed Threshold Analysis (First 3 Bars):")
    print("=" * 50)
    
    for i in range(min(3, len(validation_results))):
        result = validation_results[i]
        threshold = result['threshold_check']
        
        print(f"\n📊 Bar {i+1}:")
        print(f"   Open: ${threshold['open']:.2f}")
        print(f"   Close: ${threshold['close']:.2f}")
        print(f"   Upper threshold: ${threshold['upper_threshold']:.2f}")
        print(f"   Lower threshold: ${threshold['lower_threshold']:.2f}")
        print(f"   Breach: {'✅ Yes' if threshold['any_breach'] else '❌ No'}")
        if threshold['any_breach']:
            print(f"   Breach type: {'Upper' if threshold['upper_breach'] else 'Lower'}")
            print(f"   Breach magnitude: {threshold['breach_magnitude']*100:.3f}%")
        print(f"   Duration: {result['duration_seconds']:.1f} seconds")
        print(f"   Trades: {result['trade_count']:,}")
    
    # Final validation
    print(f"\n🎯 Algorithm Validation:")
    if valid_bars == len(range_bars):
        print("✅ All range bars have valid OHLCV data")
    else:
        print(f"❌ {len(range_bars) - valid_bars} bars have invalid OHLCV data")
    
    # Check if most bars (except possibly the last incomplete one) breach the threshold
    complete_bars = range_bars[:-1]  # Exclude last bar which might be incomplete
    complete_breach_count = sum(1 for r in validation_results[:-1] if r['threshold_check']['any_breach'])
    
    if complete_breach_count == len(complete_bars):
        print("✅ All complete range bars correctly breach the 0.8% threshold")
    else:
        print(f"❌ {len(complete_bars) - complete_breach_count} complete bars don't breach threshold")
    
    return validation_results

if __name__ == "__main__":
    try:
        results = analyze_range_bars()
        print(f"\n✅ Range bar verification complete!")
        print(f"📁 Results analyzed from: output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json")
    except Exception as e:
        print(f"❌ Verification failed: {e}")
        import traceback
        traceback.print_exc()