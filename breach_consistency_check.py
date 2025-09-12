#!/usr/bin/env python3
"""
Critical consistency check: If high/low breach threshold, close price should also breach.
This validates that breaching trades are properly included in the closing bar.
"""

import json
from decimal import Decimal, getcontext

getcontext().prec = 28

def check_breach_consistency():
    """
    Cross-validate that close prices breach when high/low breach.
    This is a critical algorithm correctness check.
    """
    
    print("🔍 BREACH CONSISTENCY CROSS-VALIDATION")
    print("=" * 60)
    print("Logic: If high/low breach → close should also breach")
    print("(Because breaching trade gets included in closing bar)")
    print()
    
    with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
        data = json.load(f)
    
    range_bars = data['range_bars']
    threshold_pct = Decimal('0.008')
    
    print("📋 Breach Consistency Analysis:")
    print("-" * 100)
    print(f"{'Bar':<3} {'H/L Breach':<10} {'Close Breach':<12} {'Consistent':<10} {'Issue':<20}")
    print("-" * 100)
    
    consistency_issues = []
    
    for i, bar in enumerate(range_bars):
        bar_num = i + 1
        
        # Decode values
        open_price = Decimal(bar['open']) / Decimal(10**8)
        high_price = Decimal(bar['high']) / Decimal(10**8)
        low_price = Decimal(bar['low']) / Decimal(10**8)
        close_price = Decimal(bar['close']) / Decimal(10**8)
        
        # Calculate thresholds
        upper_threshold = open_price * (Decimal('1') + threshold_pct)
        lower_threshold = open_price * (Decimal('1') - threshold_pct)
        
        # Check high/low breach
        upper_breach_hl = high_price >= upper_threshold
        lower_breach_hl = low_price <= lower_threshold
        hl_breach = upper_breach_hl or lower_breach_hl
        
        # Check close breach
        upper_breach_close = close_price >= upper_threshold
        lower_breach_close = close_price <= lower_threshold
        close_breach = upper_breach_close or lower_breach_close
        
        # Determine breach type
        if upper_breach_hl:
            breach_type = "Upper"
            expected_close_breach = upper_breach_close
        elif lower_breach_hl:
            breach_type = "Lower"  
            expected_close_breach = lower_breach_close
        else:
            breach_type = "None"
            expected_close_breach = True  # No expectation if no H/L breach
        
        # Check consistency
        consistent = not hl_breach or close_breach
        issue_description = ""
        
        if hl_breach and not close_breach:
            issue_description = f"{breach_type} H/L breach, close doesn't breach"
            consistency_issues.append({
                'bar': bar_num,
                'issue': issue_description,
                'open': open_price,
                'high': high_price,
                'low': low_price,
                'close': close_price,
                'upper_threshold': upper_threshold,
                'lower_threshold': lower_threshold,
                'breach_type': breach_type
            })
        
        print(f"{bar_num:<3} {breach_type if hl_breach else 'None':<10} "
              f"{('Upper' if upper_breach_close else 'Lower' if lower_breach_close else 'None'):<12} "
              f"{'✅' if consistent else '❌':<10} {issue_description:<20}")
    
    print("-" * 100)
    
    # Detailed analysis of issues
    if consistency_issues:
        print(f"\n🚨 CONSISTENCY ISSUES FOUND ({len(consistency_issues)} bars):")
        print("=" * 60)
        
        for issue in consistency_issues:
            print(f"\n📊 Bar {issue['bar']} - {issue['issue']}:")
            print(f"   Open: ${issue['open']:,.2f}")
            print(f"   High: ${issue['high']:,.2f} (threshold: ${issue['upper_threshold']:,.2f})")
            print(f"   Low:  ${issue['low']:,.2f} (threshold: ${issue['lower_threshold']:,.2f})")
            print(f"   Close: ${issue['close']:,.2f}")
            
            if issue['breach_type'] == 'Upper':
                print(f"   ❌ High breached upper (${issue['high']} ≥ ${issue['upper_threshold']})")
                print(f"   ❌ But close didn't breach upper (${issue['close']} < ${issue['upper_threshold']})")
            else:
                print(f"   ❌ Low breached lower (${issue['low']} ≤ ${issue['lower_threshold']})")
                print(f"   ❌ But close didn't breach lower (${issue['close']} > ${issue['lower_threshold']})")
            
            print(f"   🔍 This suggests breaching trade not included in close price")
    
    # Final assessment
    print(f"\n🎯 BREACH CONSISTENCY RESULTS:")
    
    if not consistency_issues:
        print(f"✅ ALL bars show consistent breach behavior")
        print(f"   • When high/low breach → close also breaches")
        print(f"   • Breaching trades properly included in closing bars")
        print(f"   • Algorithm logic is mathematically sound")
    else:
        print(f"❌ {len(consistency_issues)} bars show inconsistent breach behavior")
        print(f"   • High/low breach but close doesn't breach")
        print(f"   • Suggests algorithm bug in breach inclusion logic")
        print(f"   • Breaching trades may not be included in closing bars")
    
    return len(consistency_issues) == 0

if __name__ == "__main__":
    success = check_breach_consistency()
    
    if success:
        print(f"\n🎉 ALGORITHM CROSS-VALIDATION PASSED!")
        print(f"   Range bar breach inclusion logic is working correctly")
    else:
        print(f"\n🚨 ALGORITHM BUG DETECTED!")
        print(f"   Breach inclusion logic has implementation issues")