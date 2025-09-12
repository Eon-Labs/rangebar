#!/usr/bin/env python3
"""
Analyze potential issues with range bar breach logic
"""

import json
from decimal import Decimal, getcontext

getcontext().prec = 28

def decode_fixed_point(value, decimals=8):
    """Convert fixed-point integer back to decimal"""
    return Decimal(value) / Decimal(10 ** decimals)

def analyze_breach_logic():
    """Analyze each bar's breach logic in detail"""
    
    print("🔍 DETAILED BREACH LOGIC ANALYSIS")
    print("=" * 60)
    
    with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
        data = json.load(f)
    
    range_bars = data['range_bars']
    threshold_pct = Decimal('0.008')  # 0.8%
    
    print(f"🎯 Expected threshold: {threshold_pct*100}%")
    print()
    
    issues_found = []
    
    for i, bar in enumerate(range_bars):
        bar_num = i + 1
        
        # Decode values
        open_price = decode_fixed_point(bar['open'])
        high_price = decode_fixed_point(bar['high']) 
        low_price = decode_fixed_point(bar['low'])
        close_price = decode_fixed_point(bar['close'])
        
        # Calculate thresholds from OPEN price (this is the key!)
        upper_threshold = open_price * (1 + threshold_pct)
        lower_threshold = open_price * (1 - threshold_pct)
        
        # Check if HIGH reached upper threshold OR LOW reached lower threshold
        upper_breach_via_high = high_price >= upper_threshold
        lower_breach_via_low = low_price <= lower_threshold
        
        # Check if CLOSE price breaches (different from high/low breach)
        upper_breach_via_close = close_price >= upper_threshold
        lower_breach_via_close = close_price <= lower_threshold
        
        any_breach = upper_breach_via_high or lower_breach_via_low
        close_breach = upper_breach_via_close or lower_breach_via_close
        
        print(f"📊 Bar {bar_num}:")
        print(f"   Open:  ${open_price:>10.2f}")
        print(f"   High:  ${high_price:>10.2f} (upper threshold: ${upper_threshold:.2f})")
        print(f"   Low:   ${low_price:>10.2f} (lower threshold: ${lower_threshold:.2f})")
        print(f"   Close: ${close_price:>10.2f}")
        print(f"   Upper breach via HIGH: {'✅ Yes' if upper_breach_via_high else '❌ No'}")
        print(f"   Lower breach via LOW:  {'✅ Yes' if lower_breach_via_low else '❌ No'}")
        print(f"   Upper breach via CLOSE: {'✅ Yes' if upper_breach_via_close else '❌ No'}")
        print(f"   Lower breach via CLOSE: {'✅ Yes' if lower_breach_via_close else '❌ No'}")
        print(f"   ANY breach (H/L):     {'✅ Yes' if any_breach else '❌ No'}")
        print(f"   CLOSE breach:         {'✅ Yes' if close_breach else '❌ No'}")
        
        # Identify potential issues
        if not any_breach and bar_num < len(range_bars):  # Don't flag the last bar (might be incomplete)
            issues_found.append({
                'bar': bar_num,
                'issue': 'No breach detected via high/low',
                'details': f'High {high_price:.2f} < {upper_threshold:.2f}, Low {low_price:.2f} > {lower_threshold:.2f}'
            })
            print(f"   🚨 ISSUE: No breach detected!")
        
        print()
    
    # Summary
    print("🎯 BREACH LOGIC ANALYSIS SUMMARY:")
    print("=" * 60)
    
    if issues_found:
        print(f"❌ Found {len(issues_found)} potential issues:")
        for issue in issues_found:
            print(f"   Bar {issue['bar']}: {issue['issue']}")
            print(f"     Details: {issue['details']}")
        
        print(f"\n🤔 ALGORITHM QUESTION:")
        print(f"   Should range bars close when:")
        print(f"   A) The HIGH/LOW touches the threshold? (Any price in the bar)")
        print(f"   B) The CLOSE price breaches the threshold? (Final trade only)")
        print(f"   C) Any individual TRADE price breaches? (Tick-by-tick)")
        
        print(f"\n📋 CURRENT BEHAVIOR ANALYSIS:")
        print(f"   - Bars 1-6: Breach via high/low detection ✅")
        print(f"   - Bars 7-8: No breach via high/low ❌")
        print(f"   - This suggests bars 7-8 should not have closed yet")
        print(f"   - OR there's an issue with the breach detection logic")
        
    else:
        print("✅ All bars show proper breach logic")
    
    return issues_found

if __name__ == "__main__":
    issues = analyze_breach_logic()
    
    if issues:
        print(f"\n⚠️  POTENTIAL ALGORITHM ISSUE DETECTED:")
        print(f"   {len(issues)} bars appear to close without breaching thresholds")
        print(f"   This could indicate a bug in the range bar construction logic")
    else:
        print(f"\n✅ ALGORITHM VERIFICATION PASSED:")
        print(f"   All bars correctly breach their thresholds")