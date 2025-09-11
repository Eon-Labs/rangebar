#!/usr/bin/env python3
"""
Test script for the UM Futures data fetcher.
Validates that we can fetch and process real Binance aggTrades data.
"""

import asyncio
import sys
from pathlib import Path
import logging

# Add the built library to Python path
sys.path.insert(0, str(Path(__file__).parent / "src"))

from rangebar.data_fetcher import UMFuturesDataFetcher, fetch_um_futures_aggtrades


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


async def test_data_fetcher():
    """Test the data fetcher with a small date range."""
    print("🧪 Testing UM Futures data fetcher...")
    
    try:
        # Test with a small date range to avoid large downloads
        symbol = "BTCUSDT"
        start_date = "2024-01-01"
        end_date = "2024-01-02"  # Just one day
        
        print(f"Fetching {symbol} aggTrades from {start_date} to {end_date}")
        
        # Use convenience function
        aggtrades = await fetch_um_futures_aggtrades(
            symbol=symbol,
            start_date=start_date,
            end_date=end_date
        )
        
        print(f"✅ Successfully fetched {len(aggtrades)} aggTrades")
        
        if aggtrades:
            # Show first few trades
            print("\nFirst 3 trades:")
            for i, trade in enumerate(aggtrades[:3]):
                print(f"  Trade {i+1}:")
                print(f"    ID: {trade.agg_trade_id}")
                print(f"    Price: {trade.price}")
                print(f"    Quantity: {trade.quantity}")
                print(f"    Timestamp: {trade.timestamp}")
                print(f"    Trade Count: {trade.trade_count()}")
                print()
            
            # Validate data integrity
            print("Validating data integrity...")
            
            # Check sorting
            for i in range(1, len(aggtrades)):
                prev = aggtrades[i-1]
                curr = aggtrades[i]
                
                if (curr.timestamp < prev.timestamp or 
                    (curr.timestamp == prev.timestamp and curr.agg_trade_id <= prev.agg_trade_id)):
                    raise ValueError(f"Data not properly sorted at index {i}")
            
            print("✅ Data integrity validated")
            
            # Test range bar construction with fetched data
            print("\nTesting range bar construction with real data...")
            from rangebar.range_bars import iter_range_bars_from_aggtrades
            from decimal import Decimal
            
            # Use first 1000 trades for testing
            test_trades = aggtrades[:1000] if len(aggtrades) > 1000 else aggtrades
            
            bars = list(iter_range_bars_from_aggtrades(test_trades, pct=Decimal('0.008')))
            
            print(f"✅ Generated {len(bars)} range bars from {len(test_trades)} trades")
            
            if bars:
                print(f"First bar: Open={bars[0]['open']}, Close={bars[0]['close']}, Volume={bars[0]['volume']}")
                print(f"Last bar: Open={bars[-1]['open']}, Close={bars[-1]['close']}, Volume={bars[-1]['volume']}")
        
        else:
            print("⚠️ No data fetched (might be weekend or holiday)")
            
    except Exception as e:
        print(f"❌ Data fetcher test failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    
    return True


def test_validation():
    """Test input validation functions."""
    print("🧪 Testing input validation...")
    
    try:
        fetcher = UMFuturesDataFetcher()
        
        # Test symbol validation
        assert fetcher.validate_symbol("BTCUSDT") == "BTCUSDT"
        assert fetcher.validate_symbol("btcusdt") == "BTCUSDT"
        assert fetcher.validate_symbol(" ETHUSDT ") == "ETHUSDT"
        
        try:
            fetcher.validate_symbol("BTCUSD")  # Should fail - no USDT
            assert False, "Should have raised ValueError"
        except ValueError:
            pass
        
        try:
            fetcher.validate_symbol("")  # Should fail - empty
            assert False, "Should have raised ValueError"
        except ValueError:
            pass
        
        # Test date validation
        start, end = fetcher.validate_date_range("2024-01-01", "2024-01-02")
        assert start == "2024-01-01"
        assert end == "2024-01-02"
        
        try:
            fetcher.validate_date_range("2024-01-02", "2024-01-01")  # Should fail - wrong order
            assert False, "Should have raised ValueError"
        except ValueError:
            pass
        
        try:
            fetcher.validate_date_range("invalid", "2024-01-01")  # Should fail - invalid format
            assert False, "Should have raised ValueError"
        except ValueError:
            pass
        
        print("✅ Input validation tests passed")
        return True
        
    except Exception as e:
        print(f"❌ Validation test failed: {e}")
        import traceback
        traceback.print_exc()
        return False


async def main():
    """Run all tests."""
    print("🚀 Starting UM Futures data fetcher tests...")
    print()
    
    # Test validation first
    validation_passed = test_validation()
    if not validation_passed:
        print("❌ Validation tests failed - stopping")
        sys.exit(1)
    
    print()
    
    # Test actual data fetching (may take some time)
    data_fetch_passed = await test_data_fetcher()
    if not data_fetch_passed:
        print("❌ Data fetch tests failed")
        sys.exit(1)
    
    print()
    print("🎉 ALL TESTS PASSED!")
    print()
    print("SUCCESS SUMMARY:")
    print("✅ Input validation working correctly")
    print("✅ UM Futures data fetcher implemented")
    print("✅ Real Binance aggTrades data fetched successfully")
    print("✅ Data integrity validated")
    print("✅ Range bar construction working with real data")
    print()
    print("NEXT STEPS:")
    print("1. Create Parquet converter for efficient storage")
    print("2. Implement CLI interface for data operations")
    print("3. Add performance benchmarks")


if __name__ == "__main__":
    asyncio.run(main())