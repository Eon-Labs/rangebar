#!/usr/bin/env python3
"""
Comprehensive analysis of RangeBar output format, data types, and future-proofness.
"""

import sys
import json
import numpy as np
import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
from decimal import Decimal
from typing import Dict, Any, List
from pathlib import Path

# Import RangeBar components
sys.path.insert(0, 'src')
from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
import rangebar._rangebar_rust as rust

def create_test_data():
    """Create comprehensive test data for analysis."""
    return [
        # Normal trades
        {'a': 1, 'p': '50000.12345678', 'q': '1.50000000', 'f': 1, 'l': 1, 'T': 1640995200000, 'm': False},
        {'a': 2, 'p': '50200.87654321', 'q': '2.25000000', 'f': 2, 'l': 2, 'T': 1640995201000, 'm': True},
        
        # Exact breach (should close bar)
        {'a': 3, 'p': '50400.00000000', 'q': '0.75000000', 'f': 3, 'l': 3, 'T': 1640995202000, 'm': False},  # +0.8%
        
        # New bar
        {'a': 4, 'p': '50500.99999999', 'q': '3.12345678', 'f': 4, 'l': 4, 'T': 1640995203000, 'm': True},
        
        # Large volume trade
        {'a': 5, 'p': '50300.50000000', 'q': '100.00000000', 'f': 5, 'l': 5, 'T': 1640995204000, 'm': False},
        
        # Multi-trade aggregate
        {'a': 6, 'p': '49700.25000000', 'q': '0.12345678', 'f': 6, 'l': 10, 'T': 1640995205000, 'm': True},  # 5 trades
    ]

def analyze_python_output():
    """Analyze Python implementation output format."""
    print("🐍 PYTHON IMPLEMENTATION OUTPUT ANALYSIS")
    print("=" * 60)
    
    trades_data = create_test_data()
    trades = [AggTrade(data) for data in trades_data]
    bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
    
    print(f"Generated {len(bars)} range bars from {len(trades)} trades")
    print()
    
    for i, bar in enumerate(bars):
        print(f"📊 Bar {i + 1}:")
        print(f"  Raw structure: {type(bar)}")
        print(f"  Keys: {list(bar.keys())}")
        print()
        
        for key, value in bar.items():
            print(f"  {key:12} = {value!r:20} (type: {type(value).__name__})")
        print()
        
        # Check precision
        if hasattr(bar['open'], 'quantize'):
            print(f"  💰 Decimal precision:")
            print(f"    open precision: {bar['open'].as_tuple()}")
        print()
    
    return bars

def analyze_rust_output():
    """Analyze Rust implementation output format."""
    print("🦀 RUST IMPLEMENTATION OUTPUT ANALYSIS")
    print("=" * 60)
    
    trades_data = create_test_data()
    
    # Convert to Rust arrays
    prices = np.array([int(float(t['p']) * 1e8) for t in trades_data], dtype=np.int64)
    volumes = np.array([int(float(t['q']) * 1e8) for t in trades_data], dtype=np.int64)
    timestamps = np.array([t['T'] for t in trades_data], dtype=np.int64)
    trade_ids = np.array([t['a'] for t in trades_data], dtype=np.int64)
    first_ids = np.array([t['f'] for t in trades_data], dtype=np.int64)
    last_ids = np.array([t['l'] for t in trades_data], dtype=np.int64)
    
    result = rust.compute_range_bars(
        prices=prices,
        volumes=volumes,
        timestamps=timestamps,
        trade_ids=trade_ids,
        first_ids=first_ids,
        last_ids=last_ids,
        threshold_bps=8000
    )
    
    print(f"Generated {len(result['opens'])} range bars")
    print(f"Raw result type: {type(result)}")
    print(f"Keys: {list(result.keys())}")
    print()
    
    # Analyze each field
    for key, array in result.items():
        print(f"📊 {key:12}:")
        print(f"  Type: {type(array)} / {array.dtype if hasattr(array, 'dtype') else 'N/A'}")
        print(f"  Shape: {array.shape if hasattr(array, 'shape') else len(array)}")
        print(f"  Sample: {array[:2] if len(array) > 1 else array}")
        if key in ['opens', 'highs', 'lows', 'closes', 'volumes', 'turnovers']:
            print(f"  Scaled (÷1e8): {array[:2] / 1e8 if len(array) > 1 else array / 1e8}")
        print()
    
    return result

def analyze_schema_compatibility():
    """Analyze schema compatibility and future-proofness."""
    print("🔍 SCHEMA COMPATIBILITY ANALYSIS")
    print("=" * 60)
    
    # Create sample data
    trades_data = create_test_data()
    trades = [AggTrade(data) for data in trades_data]
    python_bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
    
    if not python_bars:
        print("No bars generated for analysis")
        return
    
    bar = python_bars[0]
    
    print("📋 PYTHON OUTPUT SCHEMA:")
    print("-" * 30)
    
    schema_analysis = {}
    for key, value in bar.items():
        value_type = type(value)
        
        # Analyze constraints and ranges
        analysis = {
            'python_type': value_type.__name__,
            'value': str(value),
            'nullable': value is None,
        }
        
        if key in ['open_time', 'close_time']:
            analysis.update({
                'format': 'Unix timestamp (milliseconds)',
                'range': 'int64 (1970-2106)',
                'timezone': 'UTC assumed',
                'future_proof': 'Until 2106, then needs int128'
            })
        elif key in ['open', 'high', 'low', 'close']:
            analysis.update({
                'format': 'Decimal with arbitrary precision',
                'range': 'Unlimited precision',
                'currency': 'Assumed USD (no metadata)',
                'future_proof': 'Excellent - handles any precision'
            })
        elif key in ['volume', 'turnover']:
            analysis.update({
                'format': 'Decimal with arbitrary precision',
                'range': 'Unlimited precision',
                'unit': 'Assumed base currency units',
                'future_proof': 'Excellent - handles any precision'
            })
        elif key == 'trade_count':
            analysis.update({
                'format': 'Integer count',
                'range': 'Limited by int64',
                'future_proof': 'Good until 9.2 quintillion trades/bar'
            })
        elif key in ['first_id', 'last_id']:
            analysis.update({
                'format': 'Binance trade ID (int64)',
                'range': 'Exchange-specific',
                'future_proof': 'Depends on exchange ID scheme'
            })
            
        schema_analysis[key] = analysis
        
        print(f"  {key:12}: {analysis['python_type']:10} | {analysis.get('format', 'Unknown format')}")
    
    print()
    
    # Convert to DataFrame to check pandas compatibility
    print("📊 PANDAS COMPATIBILITY:")
    print("-" * 30)
    
    try:
        df = pd.DataFrame(python_bars)
        print(f"✅ Successfully created DataFrame with {len(df)} rows")
        print("Column dtypes:")
        for col, dtype in df.dtypes.items():
            print(f"  {col:12}: {dtype}")
        print()
        
        # Memory usage
        memory_bytes = df.memory_usage(deep=True).sum()
        print(f"Memory usage: {memory_bytes:,} bytes ({memory_bytes/1024/1024:.2f} MB)")
        print()
        
    except Exception as e:
        print(f"❌ Pandas conversion failed: {e}")
    
    # Check Parquet compatibility
    print("📁 PARQUET COMPATIBILITY:")
    print("-" * 30)
    
    try:
        # Convert Decimal to float for Parquet (limitation)
        parquet_data = []
        for bar in python_bars:
            converted_bar = {}
            for key, value in bar.items():
                if isinstance(value, Decimal):
                    converted_bar[key] = float(value)
                else:
                    converted_bar[key] = value
            parquet_data.append(converted_bar)
        
        df_parquet = pd.DataFrame(parquet_data)
        
        # Create PyArrow table
        table = pa.Table.from_pandas(df_parquet)
        print(f"✅ PyArrow table created successfully")
        print("PyArrow schema:")
        print(table.schema)
        print()
        
        # Test Parquet write/read
        temp_file = Path("temp_rangebar_test.parquet")
        try:
            pq.write_table(table, temp_file)
            read_table = pq.read_table(temp_file)
            print(f"✅ Parquet round-trip successful")
            print(f"Original rows: {len(df_parquet)}, Read rows: {len(read_table)}")
            temp_file.unlink()  # Clean up
        except Exception as e:
            print(f"❌ Parquet I/O failed: {e}")
            
    except Exception as e:
        print(f"❌ PyArrow conversion failed: {e}")
    
    return schema_analysis

def analyze_future_proofness():
    """Analyze future-proofness and extensibility."""
    print("🔮 FUTURE-PROOFNESS ANALYSIS")
    print("=" * 60)
    
    issues = []
    recommendations = []
    
    print("🎯 CURRENT LIMITATIONS:")
    print("-" * 30)
    
    # Timestamp limitations
    print("1. ⏰ TIMESTAMP LIMITATIONS:")
    print("   - Current: int64 milliseconds (Unix timestamp)")
    print("   - Range: 1970-01-01 to 2106-02-07")
    print("   - Issue: Will overflow in 2106")
    print("   - Impact: 🟡 Medium (81 years away)")
    issues.append("Timestamp will overflow in 2106")
    recommendations.append("Consider int128 or alternative timestamp format")
    print()
    
    # Precision limitations  
    print("2. 💰 PRICE PRECISION:")
    print("   - Python: Decimal (unlimited precision) ✅")
    print("   - Rust: i64 scaled by 1e8 (8 decimal places)")
    print("   - Range: ±92,233,720,368.54775807")
    print("   - Issue: Fixed 8-decimal precision in Rust")
    print("   - Impact: 🟡 Medium (sufficient for current crypto prices)")
    issues.append("Rust implementation limited to 8 decimal places")
    recommendations.append("Consider configurable precision or higher bit depth")
    print()
    
    # Exchange-specific data
    print("3. 🏪 EXCHANGE DEPENDENCY:")
    print("   - Current: Binance-specific trade IDs and formats")
    print("   - Issue: Not portable to other exchanges")
    print("   - Impact: 🟠 High (limits reusability)")
    issues.append("Schema tied to Binance-specific fields")
    recommendations.append("Add exchange identifier and standardize optional fields")
    print()
    
    # Missing metadata
    print("4. 📝 MISSING METADATA:")
    print("   - No symbol/instrument identifier in output")
    print("   - No exchange identifier")
    print("   - No timezone information")
    print("   - No version/schema information")
    print("   - Impact: 🟠 High (data provenance unclear)")
    issues.append("Missing essential metadata in output")
    recommendations.append("Add metadata fields for symbol, exchange, timezone, schema version")
    print()
    
    # Data type inconsistencies
    print("5. 🔄 TYPE INCONSISTENCIES:")
    print("   - Python: Decimal types (arbitrary precision)")
    print("   - Rust: i64 fixed-point (8 decimals)")  
    print("   - Parquet: float64 (precision loss)")
    print("   - Impact: 🟡 Medium (precision varies by implementation)")
    issues.append("Inconsistent precision across implementations")
    recommendations.append("Standardize on consistent precision strategy")
    print()
    
    print("💡 EXTENSIBILITY ANALYSIS:")
    print("-" * 30)
    
    print("✅ EXTENSIBLE ASPECTS:")
    print("   - Dictionary/JSON structure allows new fields")
    print("   - Threshold parameters configurable")
    print("   - Algorithm logic is exchange-agnostic")
    print("   - Python API allows custom processing")
    print()
    
    print("❌ RIGID ASPECTS:")
    print("   - Fixed field names (not configurable)")
    print("   - Rust fixed-point precision hardcoded")  
    print("   - Binance-specific field assumptions")
    print("   - No schema versioning")
    print()
    
    return {
        'issues': issues,
        'recommendations': recommendations
    }

def propose_future_proof_schema():
    """Propose a more future-proof schema."""
    print("🚀 PROPOSED FUTURE-PROOF SCHEMA")
    print("=" * 60)
    
    schema = {
        "metadata": {
            "schema_version": "2.0.0",
            "created_timestamp": "2025-09-09T22:00:00Z",
            "generator": "rangebar-2.1.0",
            "algorithm": "non-lookahead-range-bars",
            "parameters": {
                "threshold_bps": 8000,
                "precision_decimals": 8
            }
        },
        "instrument": {
            "symbol": "BTCUSDT",
            "exchange": "binance",
            "market_type": "um_futures",
            "base_currency": "BTC", 
            "quote_currency": "USDT"
        },
        "bars": [
            {
                "bar_id": "uuid-or-sequence",
                "timespan": {
                    "open_time": "1640995200000",  # Consider ISO8601 strings
                    "close_time": "1640995202000",
                    "timezone": "UTC"
                },
                "ohlcv": {
                    "open": "50000.12345678",      # String for precision
                    "high": "50400.00000000", 
                    "low": "50000.12345678",
                    "close": "50400.00000000",
                    "volume": "4.50000000",
                    "turnover": "251801.23456789"
                },
                "statistics": {
                    "trade_count": 3,
                    "first_trade_id": "1",
                    "last_trade_id": "3",
                    "buyer_volume": "2.25000000",  # Additional metrics
                    "seller_volume": "2.25000000"
                },
                "thresholds": {
                    "upper_threshold": "50400.00000000",
                    "lower_threshold": "49600.00000000",
                    "breach_type": "upper"  # which threshold was hit
                }
            }
        ]
    }
    
    print("📋 PROPOSED IMPROVEMENTS:")
    print(json.dumps(schema, indent=2))
    print()
    
    print("🔧 KEY IMPROVEMENTS:")
    print("- ✅ Schema versioning for future compatibility")  
    print("- ✅ Complete metadata and provenance")
    print("- ✅ String-based precision (no loss)")
    print("- ✅ Exchange-agnostic design")
    print("- ✅ Extensible nested structure") 
    print("- ✅ Additional statistics and context")
    print("- ✅ Timezone and timestamp clarity")
    print()
    
    return schema

def main():
    """Run comprehensive output format analysis."""
    print("🔬 RANGEBAR OUTPUT FORMAT ANALYSIS")
    print("=" * 80)
    print()
    
    # Analyze current implementations
    python_bars = analyze_python_output()
    rust_result = analyze_rust_output()
    
    # Schema compatibility
    schema_info = analyze_schema_compatibility() 
    
    # Future-proofness analysis
    future_analysis = analyze_future_proofness()
    
    # Proposed improvements
    proposed_schema = propose_future_proof_schema()
    
    # Summary
    print("📊 EXECUTIVE SUMMARY")
    print("=" * 60)
    print(f"✅ Current format works for immediate needs")
    print(f"🟡 {len(future_analysis['issues'])} future-proofness issues identified")
    print(f"💡 {len(future_analysis['recommendations'])} recommendations proposed")
    print()
    print("🎯 PRIORITY IMPROVEMENTS:")
    print("1. Add metadata and schema versioning")
    print("2. Standardize precision across implementations") 
    print("3. Make exchange-agnostic")
    print("4. Add timezone and symbol context")
    print()
    
    return {
        'python_bars': python_bars,
        'rust_result': rust_result,
        'schema_info': schema_info,
        'future_analysis': future_analysis,
        'proposed_schema': proposed_schema
    }

if __name__ == "__main__":
    analysis = main()