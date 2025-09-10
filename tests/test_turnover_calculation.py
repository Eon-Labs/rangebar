#!/usr/bin/env python3
"""
Regression tests for turnover calculation to prevent overflow bugs.

This test specifically guards against the i128→i64 overflow bug that was
fixed in v0.2.2, where turnover values were producing negative/incorrect
results due to integer overflow in the Rust implementation.
"""

import pytest
import numpy as np
from decimal import Decimal
import sys
import os

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
import rangebar._rangebar_rust as rust


class TestTurnoverCalculation:
    """Test turnover calculation accuracy and overflow prevention."""
    
    @pytest.fixture
    def test_trades(self):
        """Standard test trades with known turnover values."""
        return [
            {'a': 1, 'p': '50000.12345678', 'q': '1.50000000', 'f': 1, 'l': 1, 'T': 1640995200000, 'm': False},
            {'a': 2, 'p': '50200.87654321', 'q': '2.25000000', 'f': 2, 'l': 2, 'T': 1640995201000, 'm': True},
            {'a': 3, 'p': '50400.00000000', 'q': '0.75000000', 'f': 3, 'l': 3, 'T': 1640995202000, 'm': False},
            {'a': 4, 'p': '50500.99999999', 'q': '3.12345678', 'f': 4, 'l': 4, 'T': 1640995203000, 'm': True},
            {'a': 5, 'p': '50300.50000000', 'q': '100.00000000', 'f': 5, 'l': 5, 'T': 1640995204000, 'm': False},
            {'a': 6, 'p': '49700.25000000', 'q': '0.12345678', 'f': 6, 'l': 10, 'T': 1640995205000, 'm': True},
        ]
    
    @pytest.fixture 
    def expected_turnovers(self):
        """Expected turnover values calculated manually."""
        return [
            383489.84825414,  # Bar 1: trades 1-4
            5036185.83283020  # Bar 2: trades 5-6
        ]
    
    def test_python_turnover_calculation(self, test_trades, expected_turnovers):
        """Test Python implementation produces expected turnover values."""
        trades = [AggTrade(data) for data in test_trades]
        python_bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
        
        assert len(python_bars) == 2, "Should produce exactly 2 range bars"
        
        for i, (bar, expected) in enumerate(zip(python_bars, expected_turnovers)):
            actual = float(bar['turnover'])
            assert abs(actual - expected) < 1e-6, f"Bar {i+1} turnover mismatch: expected {expected}, got {actual}"
    
    def test_rust_turnover_calculation(self, test_trades, expected_turnovers):
        """Test Rust implementation produces expected turnover values."""
        # Convert to Rust format
        prices = np.array([int(float(t['p']) * 1e8) for t in test_trades], dtype=np.int64)
        volumes = np.array([int(float(t['q']) * 1e8) for t in test_trades], dtype=np.int64)
        timestamps = np.array([t['T'] for t in test_trades], dtype=np.int64)
        trade_ids = np.array([t['a'] for t in test_trades], dtype=np.int64)
        first_ids = np.array([t['f'] for t in test_trades], dtype=np.int64)
        last_ids = np.array([t['l'] for t in test_trades], dtype=np.int64)
        
        rust_result = rust.compute_range_bars(
            prices=prices,
            volumes=volumes,
            timestamps=timestamps,
            trade_ids=trade_ids,
            first_ids=first_ids,
            last_ids=last_ids,
            threshold_bps=8000
        )
        
        assert len(rust_result['turnover']) == 2, "Should produce exactly 2 range bars"
        
        for i, expected in enumerate(expected_turnovers):
            actual = rust_result['turnover'][i]  # Now returns decimal values directly
            assert abs(actual - expected) < 1e-6, f"Bar {i+1} turnover mismatch: expected {expected}, got {actual}"
    
    def test_python_rust_turnover_consistency(self, test_trades):
        """Test that Python and Rust implementations produce identical turnover values."""
        # Python implementation
        trades = [AggTrade(data) for data in test_trades]
        python_bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
        
        # Rust implementation  
        prices = np.array([int(float(t['p']) * 1e8) for t in test_trades], dtype=np.int64)
        volumes = np.array([int(float(t['q']) * 1e8) for t in test_trades], dtype=np.int64)
        timestamps = np.array([t['T'] for t in test_trades], dtype=np.int64)
        trade_ids = np.array([t['a'] for t in test_trades], dtype=np.int64)
        first_ids = np.array([t['f'] for t in test_trades], dtype=np.int64)
        last_ids = np.array([t['l'] for t in test_trades], dtype=np.int64)
        
        rust_result = rust.compute_range_bars(
            prices=prices,
            volumes=volumes,
            timestamps=timestamps,
            trade_ids=trade_ids,
            first_ids=first_ids,
            last_ids=last_ids,
            threshold_bps=8000
        )
        
        assert len(python_bars) == len(rust_result['turnover']), "Bar count mismatch between implementations"
        
        for i in range(len(python_bars)):
            python_turnover = float(python_bars[i]['turnover'])
            rust_turnover = rust_result['turnover'][i]  # Now returns decimal values directly
            diff = abs(python_turnover - rust_turnover)
            
            assert diff < 1e-6, f"Bar {i+1}: Python={python_turnover}, Rust={rust_turnover}, diff={diff}"
    
    def test_no_negative_turnovers(self, test_trades):
        """Regression test: ensure turnovers are never negative (overflow bug)."""
        prices = np.array([int(float(t['p']) * 1e8) for t in test_trades], dtype=np.int64)
        volumes = np.array([int(float(t['q']) * 1e8) for t in test_trades], dtype=np.int64)
        timestamps = np.array([t['T'] for t in test_trades], dtype=np.int64)
        trade_ids = np.array([t['a'] for t in test_trades], dtype=np.int64)
        first_ids = np.array([t['f'] for t in test_trades], dtype=np.int64)
        last_ids = np.array([t['l'] for t in test_trades], dtype=np.int64)
        
        rust_result = rust.compute_range_bars(
            prices=prices,
            volumes=volumes,
            timestamps=timestamps,
            trade_ids=trade_ids,
            first_ids=first_ids,
            last_ids=last_ids,
            threshold_bps=8000
        )
        
        for i, turnover in enumerate(rust_result['turnover']):
            assert turnover >= 0, f"Bar {i+1} has negative turnover: {turnover} (indicates overflow bug)"
    
    def test_large_volume_turnover(self):
        """Test turnover calculation with large volumes to stress-test overflow prevention."""
        large_trades = [
            {'a': 1, 'p': '50000.0', 'q': '1000.0', 'f': 1, 'l': 1, 'T': 1640995200000, 'm': False},
            {'a': 2, 'p': '50400.0', 'q': '2000.0', 'f': 2, 'l': 2, 'T': 1640995201000, 'm': True},  # Breach
            {'a': 3, 'p': '50000.0', 'q': '500.0', 'f': 3, 'l': 3, 'T': 1640995202000, 'm': False},
        ]
        
        prices = np.array([int(float(t['p']) * 1e8) for t in large_trades], dtype=np.int64)
        volumes = np.array([int(float(t['q']) * 1e8) for t in large_trades], dtype=np.int64)
        timestamps = np.array([t['T'] for t in large_trades], dtype=np.int64)
        trade_ids = np.array([t['a'] for t in large_trades], dtype=np.int64)
        first_ids = np.array([t['f'] for t in large_trades], dtype=np.int64)
        last_ids = np.array([t['l'] for t in large_trades], dtype=np.int64)
        
        rust_result = rust.compute_range_bars(
            prices=prices,
            volumes=volumes,
            timestamps=timestamps,
            trade_ids=trade_ids,
            first_ids=first_ids,
            last_ids=last_ids,
            threshold_bps=8000
        )
        
        # Expected: Bar 1 = 50000*1000 + 50400*2000 = 151.4M, Bar 2 = 50000*500 = 25M  
        expected_bar1 = 50000 * 1000 + 50400 * 2000  # 151,400,000
        expected_bar2 = 50000 * 500  # 25,000,000
        
        actual_bar1 = rust_result['turnover'][0]  # Now returns decimal values directly
        actual_bar2 = rust_result['turnover'][1]  # Now returns decimal values directly
        
        assert abs(actual_bar1 - expected_bar1) < 1e-6, f"Large volume bar 1: expected {expected_bar1}, got {actual_bar1}"
        assert abs(actual_bar2 - expected_bar2) < 1e-6, f"Large volume bar 2: expected {expected_bar2}, got {actual_bar2}"
        
        # Ensure no overflow
        assert actual_bar1 > 0, f"Large volume turnover should be positive, got {actual_bar1}"
        assert actual_bar2 > 0, f"Large volume turnover should be positive, got {actual_bar2}"
    
    def test_output_format_alignment(self, test_trades):
        """Test that Rust output has correct format alignment and metadata."""
        prices = np.array([int(float(t['p']) * 1e8) for t in test_trades], dtype=np.int64)
        volumes = np.array([int(float(t['q']) * 1e8) for t in test_trades], dtype=np.int64)
        timestamps = np.array([t['T'] for t in test_trades], dtype=np.int64)
        trade_ids = np.array([t['a'] for t in test_trades], dtype=np.int64)
        first_ids = np.array([t['f'] for t in test_trades], dtype=np.int64)
        last_ids = np.array([t['l'] for t in test_trades], dtype=np.int64)
        
        rust_result = rust.compute_range_bars(
            prices=prices,
            volumes=volumes,
            timestamps=timestamps,
            trade_ids=trade_ids,
            first_ids=first_ids,
            last_ids=last_ids,
            threshold_bps=8000
        )
        
        # Test canonical field names (singular)
        expected_fields = [
            'open_time', 'close_time', 'open', 'high', 'low', 'close',
            'volume', 'turnover', 'trade_count', 'first_id', 'last_id'
        ]
        
        for field in expected_fields:
            assert field in rust_result, f"Missing required field: {field}"
        
        # Test metadata presence and content
        assert '_metadata' in rust_result, "Missing _metadata field"
        metadata = rust_result['_metadata']
        
        required_meta_fields = ['schema_version', 'format_version', 'algorithm', 'source']
        for field in required_meta_fields:
            assert field in metadata, f"Missing metadata field: {field}"
        
        assert metadata['source'] == 'rust', "Source should be 'rust'"
        assert metadata['algorithm'] == 'non-lookahead-range-bars', "Algorithm should be correct"
        
        # Test format validation function
        is_valid = rust.validate_output_format(rust_result)
        assert is_valid, "Output should be valid according to format validation"
    
    def test_schema_info_access(self):
        """Test that schema information is accessible from Rust."""
        schema_info = rust.get_schema_info()
        
        # Check schema structure
        assert 'schema_version' in schema_info, "Missing schema_version"
        assert 'format_version' in schema_info, "Missing format_version"
        assert 'field_names' in schema_info, "Missing field_names"
        assert 'fields' in schema_info, "Missing field details"
        
        # Check field names are correct and in order
        field_names = schema_info['field_names']
        expected_order = [
            'open_time', 'close_time', 'open', 'high', 'low', 'close',
            'volume', 'turnover', 'trade_count', 'first_id', 'last_id'
        ]
        assert field_names == expected_order, f"Field order mismatch: {field_names}"
        
        # Check field details
        fields = schema_info['fields']
        for field_name in expected_order:
            assert field_name in fields, f"Missing field details for {field_name}"
            field_info = fields[field_name]
            assert 'rust_type' in field_info, f"Missing rust_type for {field_name}"
            assert 'numpy_dtype' in field_info, f"Missing numpy_dtype for {field_name}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])