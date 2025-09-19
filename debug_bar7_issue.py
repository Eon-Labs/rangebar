#!/usr/bin/env python3
"""
Debug the Bar 7 breach detection discrepancy
"""

import json
from decimal import Decimal, getcontext

getcontext().prec = 28

def debug_bar7_breach():
    """Debug the specific issue with Bar 7 breach detection"""
    
    print("🔍 DEBUGGING BAR 7 BREACH DETECTION DISCREPANCY")
    print("=" * 60)
    
    with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
        data = json.load(f)
    
    # Focus on Bar 7 (index 6)
    bar7 = data['range_bars'][6]
    threshold_bps = Decimal('80')  # 80 basis points
    threshold_ratio = threshold_bps / Decimal('10000')  # Convert to decimal ratio
    
    print("📊 Bar 7 Raw Data:")
    for key, value in bar7.items():
        print(f"   {key}: {value}")
    print()
    
    # Decode with high precision
    open_fp = bar7['open']  # Fixed point
    high_fp = bar7['high']
    low_fp = bar7['low'] 
    close_fp = bar7['close']
    
    print(f"📊 Fixed Point Values:")
    print(f"   Open (FP):  {open_fp}")
    print(f"   High (FP):  {high_fp}")
    print(f"   Low (FP):   {low_fp}")
    print(f"   Close (FP): {close_fp}")
    print()
    
    # Decode to decimal
    open_price = Decimal(open_fp) / Decimal(10**8)
    high_price = Decimal(high_fp) / Decimal(10**8)
    low_price = Decimal(low_fp) / Decimal(10**8)
    close_price = Decimal(close_fp) / Decimal(10**8)
    
    print(f"📊 Decoded Decimal Values:")
    print(f"   Open:  {open_price}")
    print(f"   High:  {high_price}")
    print(f"   Low:   {low_price}")
    print(f"   Close: {close_price}")
    print()
    
    # Calculate thresholds with high precision
    upper_threshold = open_price * (Decimal('1') + threshold_ratio)
    lower_threshold = open_price * (Decimal('1') - threshold_ratio)
    
    print(f"📊 Threshold Calculations:")
    print(f"   Open price: {open_price}")
    print(f"   Threshold: {threshold_bps} bps = {threshold_ratio * 100}%")
    print(f"   Upper threshold: {upper_threshold}")
    print(f"   Lower threshold: {lower_threshold}")
    print()
    
    # Check breaches with exact precision
    upper_breach_high = high_price >= upper_threshold
    lower_breach_low = low_price <= lower_threshold
    upper_breach_close = close_price >= upper_threshold
    lower_breach_close = close_price <= lower_threshold
    
    print(f"🔍 Precise Breach Analysis:")
    print(f"   High >= Upper threshold: {high_price} >= {upper_threshold} = {upper_breach_high}")
    print(f"   Low <= Lower threshold:  {low_price} <= {lower_threshold} = {lower_breach_low}")
    print(f"   Close >= Upper threshold: {close_price} >= {upper_threshold} = {upper_breach_close}")
    print(f"   Close <= Lower threshold: {close_price} <= {lower_threshold} = {lower_breach_close}")
    print()
    
    # Check the exact differences
    high_vs_upper = high_price - upper_threshold
    low_vs_lower = lower_threshold - low_price
    
    print(f"🔬 Exact Differences:")
    print(f"   High - Upper threshold: {high_vs_upper}")
    print(f"   Lower threshold - Low: {low_vs_lower}")
    print()
    
    # Determine if this should be a completed bar
    any_breach = upper_breach_high or lower_breach_low
    
    if any_breach:
        print(f"✅ Bar 7 SHOULD have breached and closed")
        if not upper_breach_high and not lower_breach_low:
            print(f"🚨 ALGORITHM BUG: Bar shows breach but calculations don't match!")
    else:
        print(f"❌ Bar 7 should NOT have closed - no breach detected")
        print(f"🚨 ALGORITHM BUG: Bar closed without valid breach!")
    
    # Check if this is exactly at threshold (floating point precision issue)
    if abs(high_vs_upper) < Decimal('0.00000001'):
        print(f"⚠️  PRECISION ISSUE: High price is within rounding error of threshold")
        print(f"   Difference: {high_vs_upper}")
        print(f"   This could be a floating-point precision edge case")
    
    return {
        'bar_number': 7,
        'should_breach': any_breach,
        'precision_issue': abs(high_vs_upper) < Decimal('0.00000001') or abs(low_vs_lower) < Decimal('0.00000001')
    }

if __name__ == "__main__":
    result = debug_bar7_breach()
    
    print("\n🎯 DIAGNOSIS:")
    if result['should_breach']:
        print("✅ Bar 7 correctly breached threshold - algorithm working properly")
    else:
        if result['precision_issue']:
            print("⚠️  Bar 7 has precision/rounding issue at threshold boundary")
        else:
            print("🚨 Bar 7 closed without breach - ALGORITHM BUG DETECTED!")