"""
Python reference implementation for testing and validation.
This serves as a reference to validate the Rust implementation.
"""

from decimal import Decimal, getcontext
from typing import Iterator, Dict, Any, List
import json

# Set high precision for decimal calculations
getcontext().prec = 28


class AggTrade:
    """Represents a Binance UM Futures aggTrade"""
    
    def __init__(self, data: Dict[str, Any]):
        self.agg_trade_id = int(data['a'])
        self.price = Decimal(str(data['p']))
        self.quantity = Decimal(str(data['q']))
        self.first_trade_id = int(data['f'])
        self.last_trade_id = int(data['l'])
        self.timestamp = int(data['T'])
        self.is_buyer_maker = bool(data['m'])
    
    def trade_count(self) -> int:
        """Number of individual trades aggregated"""
        return self.last_trade_id - self.first_trade_id + 1
    
    def turnover(self) -> Decimal:
        """Price * quantity"""
        return self.price * self.quantity


def iter_range_bars_from_aggtrades(
    trades: List[AggTrade], 
    pct: Decimal = Decimal('0.008')
) -> Iterator[Dict[str, Any]]:
    """
    Non-lookahead range bar construction.
    
    Args:
        trades: List of AggTrade objects, sorted by (timestamp, agg_trade_id)
        pct: Percentage threshold (0.008 = 0.8%)
    
    Yields:
        Range bar dictionaries with OHLCV data
    """
    if not trades:
        return
    
    bar = None
    defer_open = False
    
    def _new_bar_from_trade(trade: AggTrade) -> Dict[str, Any]:
        """Create new bar from trade"""
        return {
            'open_time': trade.timestamp,
            'close_time': trade.timestamp,
            'open': trade.price,
            'high': trade.price,
            'low': trade.price,
            'close': trade.price,
            'volume': trade.quantity,
            'turnover': trade.turnover(),
            'trade_count': trade.trade_count(),
            'first_id': trade.agg_trade_id,
            'last_id': trade.agg_trade_id,
        }
    
    for trade in trades:
        if defer_open:
            # This trade opens new bar after previous bar closed
            bar = _new_bar_from_trade(trade)
            # Compute FIXED thresholds from open price
            bar['upper_threshold'] = bar['open'] * (Decimal('1') + pct)
            bar['lower_threshold'] = bar['open'] * (Decimal('1') - pct)
            defer_open = False
            continue
        
        if bar is None:
            # First bar initialization
            bar = _new_bar_from_trade(trade)
            # Compute FIXED thresholds from open price
            bar['upper_threshold'] = bar['open'] * (Decimal('1') + pct)
            bar['lower_threshold'] = bar['open'] * (Decimal('1') - pct)
            continue
        
        # CRITICAL: Update bar with current trade FIRST (always include)
        bar['high'] = max(bar['high'], trade.price)
        bar['low'] = min(bar['low'], trade.price)
        bar['close'] = trade.price
        bar['close_time'] = trade.timestamp
        bar['volume'] += trade.quantity
        bar['turnover'] += trade.turnover()
        bar['trade_count'] += trade.trade_count()
        bar['last_id'] = trade.agg_trade_id
        
        # Check breach using FIXED thresholds (computed from open)
        if trade.price >= bar['upper_threshold'] or trade.price <= bar['lower_threshold']:
            # Remove threshold metadata before yielding
            result_bar = {k: v for k, v in bar.items() 
                         if k not in ['upper_threshold', 'lower_threshold']}
            yield result_bar
            bar = None
            defer_open = True  # Next trade opens new bar
    
    # Yield final partial bar if exists
    if bar is not None:
        result_bar = {k: v for k, v in bar.items() 
                     if k not in ['upper_threshold', 'lower_threshold']}
        yield result_bar


def test_algorithm():
    """Test the Python reference implementation"""
    print("Testing Python reference implementation...")
    
    # Test data: exact 0.8% breach scenario
    test_trades_data = [
        {'a': 1, 'p': '50000.0', 'q': '1.0', 'f': 1, 'l': 1, 'T': 1000, 'm': False},
        {'a': 2, 'p': '50200.0', 'q': '1.0', 'f': 2, 'l': 2, 'T': 2000, 'm': False},
        {'a': 3, 'p': '50400.0', 'q': '1.0', 'f': 3, 'l': 3, 'T': 3000, 'm': False},  # Exact breach
        {'a': 4, 'p': '50500.0', 'q': '1.0', 'f': 4, 'l': 4, 'T': 4000, 'm': False},  # New bar
    ]
    
    trades = [AggTrade(data) for data in test_trades_data]
    bars = list(iter_range_bars_from_aggtrades(trades))
    
    print(f"Generated {len(bars)} bars")
    
    for i, bar in enumerate(bars):
        print(f"Bar {i}:")
        print(f"  Open: {bar['open']}")
        print(f"  High: {bar['high']}")
        print(f"  Low: {bar['low']}")
        print(f"  Close: {bar['close']}")
        print(f"  Volume: {bar['volume']}")
        print(f"  Trades: {bar['trade_count']}")
        print()
    
    # Validate results
    assert len(bars) == 2, f"Expected 2 bars, got {len(bars)}"
    
    # First bar should close at breach
    assert bars[0]['open'] == Decimal('50000.0')
    assert bars[0]['close'] == Decimal('50400.0')  # Breach tick included
    assert bars[0]['high'] == Decimal('50400.0')
    assert bars[0]['low'] == Decimal('50000.0')
    
    # Second bar should start at next tick
    assert bars[1]['open'] == Decimal('50500.0')  # Next tick after breach
    assert bars[1]['close'] == Decimal('50500.0')
    
    print("✅ Python reference implementation tests passed!")


if __name__ == "__main__":
    test_algorithm()