#!/usr/bin/env python3
import csv
import sys

def validate_range_bar(open_price, high, low, close, threshold_pct=0.0025):
    """Validate range bar breach consistency rule"""
    upper_threshold = open_price * (1 + threshold_pct)
    lower_threshold = open_price * (1 - threshold_pct)
    
    high_breach = high >= upper_threshold
    low_breach = low <= lower_threshold
    close_high_breach = close >= upper_threshold  
    close_low_breach = close <= lower_threshold
    
    # Breach consistency rule validation
    if high_breach and not close_high_breach:
        return False, f"High breached upper ({high:.8f} >= {upper_threshold:.8f}) but close didn't ({close:.8f})"
    elif low_breach and not close_low_breach:
        return False, f"Low breached lower ({low:.8f} <= {lower_threshold:.8f}) but close didn't ({close:.8f})"
    
    return True, "Valid"

def validate_file(filename, max_rows=10):
    """Validate range bars in a CSV file"""
    violations = 0
    valid_bars = 0
    
    print(f"Validating {filename}...")
    
    with open(filename, 'r') as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader):
            if i >= max_rows:
                break
                
            try:
                open_price = float(row['open']) / 1e11  # Convert from fixed-point
                high = float(row['high']) / 1e11
                low = float(row['low']) / 1e11  
                close = float(row['close']) / 1e11
                
                is_valid, message = validate_range_bar(open_price, high, low, close)
                
                if is_valid:
                    valid_bars += 1
                    print(f"  ✅ Bar {i+1}: {message}")
                else:
                    violations += 1
                    print(f"  ❌ Bar {i+1}: {message}")
                    print(f"     Open: {open_price:.8f}, High: {high:.8f}, Low: {low:.8f}, Close: {close:.8f}")
                    
            except (ValueError, KeyError) as e:
                print(f"  ⚠️  Bar {i+1}: Parse error - {e}")
    
    return valid_bars, violations

if __name__ == "__main__":
    files_to_check = [
        "./output/multi_year_concurrent_2022_2025_025pct/um_BTCUSDT_rangebar_20220801_20250911_0.250pct.csv",
        "./output/adversarial_audit_2022_2025/um_BTCUSDT_rangebar_20220801_20250911_0.250pct.csv",
        "./output/range_bars_batched.csv"
    ]
    
    for filename in files_to_check:
        try:
            valid, violations = validate_file(filename, max_rows=15)
            print(f"📊 {filename}: {valid} valid, {violations} violations\n")
        except FileNotFoundError:
            print(f"❌ File not found: {filename}\n")
