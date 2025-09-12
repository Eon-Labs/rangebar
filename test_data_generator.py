#!/usr/bin/env python3
"""
Generate realistic test trade data for range bar algorithm validation.
Creates CSV files that match Binance aggTrades format.
"""

import csv
import os
from datetime import datetime, timedelta
import random
import math

def generate_realistic_trades(symbol: str, start_date: str, num_trades: int = 10000) -> str:
    """Generate realistic aggTrades data and save to CSV"""
    
    # Create output directory
    output_dir = f"test_data/{symbol}"
    os.makedirs(output_dir, exist_ok=True)
    
    # Parse date and create filename
    date_obj = datetime.strptime(start_date, "%Y-%m-%d")
    filename = f"{output_dir}/{symbol}_aggTrades_{date_obj.strftime('%Y%m%d')}.csv"
    
    print(f"📊 Generating {num_trades:,} realistic trades for {symbol}")
    print(f"💾 Output: {filename}")
    
    # Initial parameters for BTC-like data
    base_price = 50000.0  # Starting around $50k
    min_volume = 0.001    # Minimum trade size
    max_volume = 10.0     # Maximum trade size
    volatility = 0.002    # 0.2% price volatility per trade
    
    # Start timestamp (milliseconds)
    current_timestamp = int(date_obj.timestamp() * 1000)
    
    trades = []
    current_price = base_price
    
    for i in range(num_trades):
        # Generate price movement (random walk with some mean reversion)
        price_change_pct = random.gauss(0, volatility)
        # Add slight mean reversion
        if current_price > base_price * 1.05:  # If 5% above base, bias downward
            price_change_pct -= 0.001
        elif current_price < base_price * 0.95:  # If 5% below base, bias upward
            price_change_pct += 0.001
            
        current_price = current_price * (1 + price_change_pct)
        current_price = max(current_price, 1.0)  # Prevent negative prices
        
        # Generate volume (log-normal distribution)
        volume = random.lognormvariate(math.log(0.5), 0.8)
        volume = max(min_volume, min(volume, max_volume))
        
        # Generate timestamp (random intervals between 10ms and 5000ms)
        time_increment = random.randint(10, 5000)
        current_timestamp += time_increment
        
        # Create trade record
        trade = {
            'a': i + 1,  # aggTradeId
            'p': f"{current_price:.8f}",  # price
            'q': f"{volume:.8f}",  # quantity
            'f': i + 1,  # firstTradeId
            'l': i + 1,  # lastTradeId
            'T': current_timestamp,  # timestamp
            'm': random.choice([True, False])  # is_buyer_maker
        }
        trades.append(trade)
    
    # Write to CSV file
    with open(filename, 'w', newline='') as csvfile:
        fieldnames = ['a', 'p', 'q', 'f', 'l', 'T', 'm']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(trades)
    
    print(f"✅ Generated {len(trades):,} trades")
    print(f"📈 Price range: ${min(float(t['p']) for t in trades):,.2f} - ${max(float(t['p']) for t in trades):,.2f}")
    print(f"📊 Volume range: {min(float(t['q']) for t in trades):.6f} - {max(float(t['q']) for t in trades):.6f}")
    print(f"⏰ Time span: {(trades[-1]['T'] - trades[0]['T']) / 1000 / 60:.1f} minutes")
    
    return filename

def create_test_scenarios():
    """Create multiple test scenarios for algorithm validation"""
    
    scenarios = [
        {
            'symbol': 'BTCUSDT',
            'start_date': '2025-09-01',
            'num_trades': 5000,
            'description': 'Moderate volume scenario'
        },
        {
            'symbol': 'ETHUSDT', 
            'start_date': '2025-09-01',
            'num_trades': 10000,
            'description': 'High volume scenario'
        }
    ]
    
    generated_files = []
    
    for scenario in scenarios:
        print(f"\n🎯 Creating scenario: {scenario['description']}")
        print(f"   Symbol: {scenario['symbol']}")
        print(f"   Date: {scenario['start_date']}")
        print(f"   Trades: {scenario['num_trades']:,}")
        
        filename = generate_realistic_trades(
            scenario['symbol'],
            scenario['start_date'],
            scenario['num_trades']
        )
        generated_files.append({
            'file': filename,
            'symbol': scenario['symbol'],
            'date': scenario['start_date']
        })
    
    return generated_files

if __name__ == "__main__":
    print("🚀 Range Bar Test Data Generator")
    print("=" * 50)
    
    # Generate test scenarios
    test_files = create_test_scenarios()
    
    print(f"\n📋 Generated Files Summary:")
    print("=" * 50)
    for i, file_info in enumerate(test_files, 1):
        print(f"{i}. {file_info['file']}")
        print(f"   Symbol: {file_info['symbol']}")
        print(f"   Date: {file_info['date']}")
        
        # Show file size
        if os.path.exists(file_info['file']):
            size_kb = os.path.getsize(file_info['file']) / 1024
            print(f"   Size: {size_kb:.1f} KB")
    
    print(f"\n✅ Test data generation complete!")
    print(f"📁 Data directory: test_data/")
    print(f"\nNext step: Run range bar processing with:")
    print(f"./target/release/rangebar BTCUSDT 2025-09-01 2025-09-01 0.008 ./output")