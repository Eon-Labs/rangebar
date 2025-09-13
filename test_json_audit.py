#!/usr/bin/env python3
"""
Quick test script to generate range bars from our test data and output JSON
for auditing the microstructure fields.
"""

import csv
import json
from typing import List, Dict, Any
from dataclasses import dataclass
from decimal import Decimal, getcontext

# Set high precision for financial calculations
getcontext().prec = 28

@dataclass
class AggTrade:
    agg_trade_id: int
    price: Decimal
    volume: Decimal
    first_trade_id: int
    last_trade_id: int
    timestamp: int
    is_buyer_maker: bool

    def trade_count(self) -> int:
        return self.last_trade_id - self.first_trade_id + 1

    def turnover(self) -> Decimal:
        return self.price * self.volume

@dataclass
class RangeBar:
    open_time: int
    close_time: int
    open: Decimal
    high: Decimal
    low: Decimal
    close: Decimal
    volume: Decimal
    turnover: Decimal
    trade_count: int
    first_id: int
    last_id: int
    # Market microstructure fields
    buy_volume: Decimal
    sell_volume: Decimal
    buy_trade_count: int
    sell_trade_count: int
    vwap: Decimal
    buy_turnover: Decimal
    sell_turnover: Decimal

def load_trades_from_csv(filename: str) -> List[AggTrade]:
    """Load aggregate trades from CSV file"""
    trades = []

    with open(filename, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            trade = AggTrade(
                agg_trade_id=int(row['a']),
                price=Decimal(row['p']),
                volume=Decimal(row['q']),
                first_trade_id=int(row['f']),
                last_trade_id=int(row['l']),
                timestamp=int(row['T']),
                is_buyer_maker=(row['m'] == 'True')
            )
            trades.append(trade)

    return trades

def create_range_bar_from_trade(trade: AggTrade) -> RangeBar:
    """Create new range bar from opening trade"""
    trade_turnover = trade.turnover()
    trade_count = trade.trade_count()

    # Segregate order flow based on is_buyer_maker
    if trade.is_buyer_maker:
        buy_volume, sell_volume = Decimal(0), trade.volume
        buy_trade_count, sell_trade_count = 0, trade_count
        buy_turnover, sell_turnover = Decimal(0), trade_turnover
    else:
        buy_volume, sell_volume = trade.volume, Decimal(0)
        buy_trade_count, sell_trade_count = trade_count, 0
        buy_turnover, sell_turnover = trade_turnover, Decimal(0)

    return RangeBar(
        open_time=trade.timestamp,
        close_time=trade.timestamp,
        open=trade.price,
        high=trade.price,
        low=trade.price,
        close=trade.price,
        volume=trade.volume,
        turnover=trade_turnover,
        trade_count=trade_count,
        first_id=trade.agg_trade_id,
        last_id=trade.agg_trade_id,
        # Market microstructure fields
        buy_volume=buy_volume,
        sell_volume=sell_volume,
        buy_trade_count=buy_trade_count,
        sell_trade_count=sell_trade_count,
        vwap=trade.price,  # Initial VWAP equals opening price
        buy_turnover=buy_turnover,
        sell_turnover=sell_turnover
    )

def update_bar_with_trade(bar: RangeBar, trade: AggTrade) -> None:
    """Update bar with new trade (includes microstructure)"""
    trade_turnover = trade.turnover()
    trade_count = trade.trade_count()

    # Update price extremes
    if trade.price > bar.high:
        bar.high = trade.price
    if trade.price < bar.low:
        bar.low = trade.price

    # Update closing data
    bar.close = trade.price
    bar.close_time = trade.timestamp
    bar.last_id = trade.agg_trade_id

    # Update total volume and trade count
    bar.volume += trade.volume
    bar.turnover += trade_turnover
    bar.trade_count += trade_count

    # Update order flow segregation
    if trade.is_buyer_maker:
        # Seller aggressive = sell pressure
        bar.sell_volume += trade.volume
        bar.sell_trade_count += trade_count
        bar.sell_turnover += trade_turnover
    else:
        # Buyer aggressive = buy pressure
        bar.buy_volume += trade.volume
        bar.buy_trade_count += trade_count
        bar.buy_turnover += trade_turnover

    # Update VWAP incrementally
    if bar.volume > 0:
        bar.vwap = bar.turnover / bar.volume

def process_trades_to_range_bars(trades: List[AggTrade], threshold_pct: float) -> List[RangeBar]:
    """Process trades into range bars with given threshold"""
    if not trades:
        return []

    bars = []
    current_bar = None

    for trade in trades:
        if current_bar is None:
            # Start new bar
            current_bar = create_range_bar_from_trade(trade)
            continue

        # Check if this trade breaches the threshold
        upper_threshold = current_bar.open * (Decimal(1) + Decimal(str(threshold_pct)))
        lower_threshold = current_bar.open * (Decimal(1) - Decimal(str(threshold_pct)))

        if trade.price >= upper_threshold or trade.price <= lower_threshold:
            # Breach detected - update bar and close it
            update_bar_with_trade(current_bar, trade)
            bars.append(current_bar)

            # Start new bar
            current_bar = create_range_bar_from_trade(trade)
        else:
            # No breach - normal update
            update_bar_with_trade(current_bar, trade)

    # Add final partial bar if it exists
    if current_bar is not None:
        bars.append(current_bar)

    return bars

def serialize_bar(bar: RangeBar) -> Dict[str, Any]:
    """Convert RangeBar to JSON-serializable dict"""
    return {
        'open_time': bar.open_time,
        'close_time': bar.close_time,
        'open': float(bar.open),
        'high': float(bar.high),
        'low': float(bar.low),
        'close': float(bar.close),
        'volume': float(bar.volume),
        'turnover': float(bar.turnover),
        'trade_count': bar.trade_count,
        'first_id': bar.first_id,
        'last_id': bar.last_id,
        # Market microstructure fields
        'buy_volume': float(bar.buy_volume),
        'sell_volume': float(bar.sell_volume),
        'buy_trade_count': bar.buy_trade_count,
        'sell_trade_count': bar.sell_trade_count,
        'vwap': float(bar.vwap),
        'buy_turnover': float(bar.buy_turnover),
        'sell_turnover': float(bar.sell_turnover)
    }

def main():
    print("🔍 Range Bar JSON Audit Tool")
    print("=" * 40)

    # Load test data
    print("📊 Loading test data...")
    trades = load_trades_from_csv('test_data/BTCUSDT/BTCUSDT_aggTrades_20250901.csv')
    print(f"✅ Loaded {len(trades)} trades")

    # Process to range bars
    threshold_pct = 0.008  # 0.8%
    print(f"🔄 Processing with {threshold_pct*100}% threshold...")
    bars = process_trades_to_range_bars(trades, threshold_pct)
    print(f"✅ Generated {len(bars)} range bars")

    # Convert to JSON format
    json_data = {
        'metadata': {
            'symbol': 'BTCUSDT',
            'threshold_pct': threshold_pct,
            'total_bars': len(bars),
            'total_trades': len(trades),
            'microstructure_fields_included': True
        },
        'range_bars': [serialize_bar(bar) for bar in bars[:5]]  # First 5 bars for audit
    }

    # Output for auditing
    print("\n📋 JSON Output Audit (First 5 bars):")
    print("=" * 50)
    print(json.dumps(json_data, indent=2))

    # Save full output
    with open('audit_output.json', 'w') as f:
        full_data = {**json_data, 'range_bars': [serialize_bar(bar) for bar in bars]}
        json.dump(full_data, f, indent=2)

    print(f"\n💾 Full output saved to: audit_output.json")
    print(f"📊 Total range bars generated: {len(bars)}")

if __name__ == "__main__":
    main()