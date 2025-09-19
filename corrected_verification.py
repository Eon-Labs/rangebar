#!/usr/bin/env python3
"""
CORRECTED range bar verification - fixing the breach detection logic
"""

import json
from decimal import Decimal, getcontext

getcontext().prec = 28

def corrected_breach_check(open_price, high_price, low_price, close_price, threshold_bps=Decimal('80')):
    """
    Corrected breach detection logic.
    Range bars should close when HIGH >= upper_threshold OR LOW <= lower_threshold
    """
    threshold_ratio = threshold_bps / Decimal('10000')  # Convert bps to ratio
    upper_threshold = open_price * (Decimal('1') + threshold_ratio)
    lower_threshold = open_price * (Decimal('1') - threshold_ratio)
    
    # CORRECT: Check if HIGH reached upper OR LOW reached lower threshold
    upper_breach = high_price >= upper_threshold
    lower_breach = low_price <= lower_threshold
    any_breach = upper_breach or lower_breach
    
    return {
        'upper_threshold': upper_threshold,
        'lower_threshold': lower_threshold,
        'upper_breach': upper_breach,
        'lower_breach': lower_breach,
        'any_breach': any_breach,
        'breach_type': 'upper' if upper_breach else ('lower' if lower_breach else 'none')
    }

def corrected_verification():
    """Run corrected verification of all range bars"""
    
    print("🔍 CORRECTED Range Bar Verification")
    print("=" * 50)
    
    with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
        data = json.load(f)
    
    range_bars = data['range_bars']
    total_trades = data['metadata']['statistics']['market_data']['total_trades']
    
    print(f"📊 Analyzing {len(range_bars)} range bars")
    print(f"📈 Total trades: {total_trades:,}")
    print()
    
    print("📋 Corrected Breach Analysis:")
    print("-" * 80)
    print(f"{'Bar':<3} {'Open':<10} {'High':<10} {'Low':<10} {'Close':<10} {'Breach':<8} {'Type':<8}")
    print("-" * 80)
    
    valid_breaches = 0
    total_complete_bars = len(range_bars) - 1  # Exclude last potentially incomplete bar
    
    for i, bar in enumerate(range_bars):
        bar_num = i + 1
        
        # Decode fixed point values
        open_price = Decimal(bar['open']) / Decimal(10**8)
        high_price = Decimal(bar['high']) / Decimal(10**8)
        low_price = Decimal(bar['low']) / Decimal(10**8)
        close_price = Decimal(bar['close']) / Decimal(10**8)
        
        # Run corrected breach check
        breach_result = corrected_breach_check(open_price, high_price, low_price, close_price)
        
        breached = breach_result['any_breach']
        breach_type = breach_result['breach_type']
        
        if breached or bar_num == len(range_bars):  # Allow last bar to not breach
            valid_breaches += 1 if breached else 0
        
        print(f"{bar_num:<3} {float(open_price):<10.2f} {float(high_price):<10.2f} {float(low_price):<10.2f} {float(close_price):<10.2f} "
              f"{'✅' if breached else '❌':<8} {breach_type:<8}")
        
        # Detailed analysis for problematic bars
        if not breached and bar_num < len(range_bars):  # Not the last bar
            print(f"    🚨 Bar {bar_num} closed without breach!")
            print(f"       Upper threshold: ${breach_result['upper_threshold']:.2f}")
            print(f"       Lower threshold: ${breach_result['lower_threshold']:.2f}")
    
    print("-" * 80)
    
    # Final assessment
    print(f"\n🎯 CORRECTED VERIFICATION RESULTS:")
    print(f"   Complete bars (1-{total_complete_bars}): {valid_breaches}/{total_complete_bars} breached properly")
    
    if valid_breaches == total_complete_bars:
        print(f"✅ ALL complete bars correctly breach thresholds - Algorithm working perfectly!")
    else:
        print(f"❌ {total_complete_bars - valid_breaches} complete bars failed to breach - Algorithm bug detected!")
    
    # Check last bar
    last_bar_breached = corrected_breach_check(
        Decimal(range_bars[-1]['open']) / Decimal(10**8),
        Decimal(range_bars[-1]['high']) / Decimal(10**8),
        Decimal(range_bars[-1]['low']) / Decimal(10**8),
        Decimal(range_bars[-1]['close']) / Decimal(10**8)
    )['any_breach']
    
    if last_bar_breached:
        print(f"✅ Last bar also breached (complete)")
    else:
        print(f"✅ Last bar incomplete (no breach yet) - Expected behavior")
    
    return valid_breaches == total_complete_bars

if __name__ == "__main__":
    success = corrected_verification()
    
    if success:
        print(f"\n🎉 ALGORITHM VERIFICATION PASSED!")
        print(f"   All complete range bars properly breach their 0.8% thresholds")
        print(f"   The range bar construction logic is mathematically sound")
    else:
        print(f"\n🚨 ALGORITHM ISSUE CONFIRMED!")
        print(f"   Some complete bars closed without breaching thresholds")
        print(f"   This indicates a bug in the range bar construction logic")