#!/usr/bin/env python3
"""
Validate range bar algorithm compliance from generated JSON files
"""

import json
import sys

def validate_range_bar_algorithm(json_file, threshold_bps):
    """Validate that range bars comply with the algorithm specification"""
    print(f"🔍 Validating range bars in {json_file}")
    print(f"📊 Expected threshold: {threshold_bps} bps ({threshold_bps/100:.3}%)")

    with open(json_file, 'r') as f:
        data = json.load(f)

    range_bars = data['range_bars']
    violations = []

    print(f"📈 Total bars to validate: {len(range_bars)}")

    for i, bar in enumerate(range_bars):
        # Convert fixed-point to float (divide by 1e8)
        open_price = bar['open'] / 1e8
        high_price = bar['high'] / 1e8
        low_price = bar['low'] / 1e8
        close_price = bar['close'] / 1e8

        # Calculate expected thresholds from open price (convert BPS to decimal ratio)
        threshold_ratio = threshold_bps / 10000.0
        upper_threshold = open_price * (1 + threshold_ratio)
        lower_threshold = open_price * (1 - threshold_ratio)

        # Check if high or low breached the thresholds
        high_breach = high_price >= upper_threshold
        low_breach = low_price <= lower_threshold

        any_breach = high_breach or low_breach

        # Verify breach consistency (if high/low breach, close must also breach)
        if high_breach and close_price < upper_threshold:
            if close_price < open_price:  # Close is below open, check if it breached lower
                if close_price <= lower_threshold:
                    breach_consistent = True
                else:
                    breach_consistent = False
            else:
                breach_consistent = False
        elif low_breach and close_price > lower_threshold:
            if close_price > open_price:  # Close is above open, check if it breached upper
                if close_price >= upper_threshold:
                    breach_consistent = True
                else:
                    breach_consistent = False
            else:
                breach_consistent = False
        else:
            breach_consistent = True

        if not any_breach:
            violations.append({
                'bar_index': i,
                'issue': 'No threshold breach',
                'open': open_price,
                'high': high_price,
                'low': low_price,
                'close': close_price,
                'expected_upper': upper_threshold,
                'expected_lower': lower_threshold,
                'high_breach': high_breach,
                'low_breach': low_breach
            })
        elif not breach_consistent:
            violations.append({
                'bar_index': i,
                'issue': 'Breach inconsistency',
                'open': open_price,
                'high': high_price,
                'low': low_price,
                'close': close_price,
                'expected_upper': upper_threshold,
                'expected_lower': lower_threshold,
                'high_breach': high_breach,
                'low_breach': low_breach
            })

    print(f"\n📋 Validation Results:")
    if not violations:
        print("✅ ALL BARS COMPLY with range bar algorithm")
        print("✅ No algorithm violations detected")
    else:
        print(f"❌ FOUND {len(violations)} ALGORITHM VIOLATIONS:")
        for v in violations[:5]:  # Show first 5 violations
            print(f"   Bar {v['bar_index']}: {v['issue']}")
            print(f"      OHLC: O={v['open']:.2f} H={v['high']:.2f} L={v['low']:.2f} C={v['close']:.2f}")
            print(f"      Thresholds: Upper={v['expected_upper']:.2f} Lower={v['expected_lower']:.2f}")
            print(f"      Breaches: High={v['high_breach']} Low={v['low_breach']}")
        if len(violations) > 5:
            print(f"   ... and {len(violations) - 5} more violations")

    return len(violations) == 0

def main():
    print("🎯 Range Bar Algorithm Validation Tool")
    print("=====================================")

    # Test edge case with different threshold
    files_to_test = [
        ('./output/edge_case_test/spot_BTCUSDT_rangebar_20241030_20241031_0050bps.json', 50),  # 0.5% = 50 bps
    ]

    all_valid = True

    for json_file, threshold_bps in files_to_test:
        try:
            is_valid = validate_range_bar_algorithm(json_file, threshold_bps)
            all_valid = all_valid and is_valid
            print()
        except FileNotFoundError:
            print(f"❌ File not found: {json_file}")
            all_valid = False
        except Exception as e:
            print(f"❌ Error processing {json_file}: {e}")
            all_valid = False

    print("🏁 FINAL VERDICT:")
    if all_valid:
        print("✅ ALL RANGE BAR FILES COMPLY WITH ALGORITHM")
        sys.exit(0)
    else:
        print("❌ ALGORITHM VIOLATIONS DETECTED")
        sys.exit(1)

if __name__ == '__main__':
    main()