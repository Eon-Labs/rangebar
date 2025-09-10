#!/usr/bin/env python3
"""
Conversion utilities between JSON and Arrow formats.

This module provides seamless conversion between row-oriented (JSON) and
column-oriented (Arrow) rangebar formats using the shared schema definition.
"""

from typing import Dict, List, Any, Union
import numpy as np
from .schema import RANGEBAR_FIELDS, FIELD_NAMES, get_field_names, validate_field_names

__version__ = "1.0.0"


def json_to_arrow(json_data: List[Dict[str, Any]]) -> Dict[str, np.ndarray]:
    """
    Convert JSON rangebar format to Arrow-compatible columnar format.
    
    Args:
        json_data: List of rangebar dictionaries (row-oriented)
        
    Returns:
        Dictionary with field names as keys and numpy arrays as values (columnar)
        
    Raises:
        ValueError: If input data doesn't match canonical schema
    """
    if not json_data:
        # Return empty arrays with correct field names
        return {field_name: np.array([], dtype=_get_numpy_dtype(field_name)) 
                for field_name in FIELD_NAMES}
    
    # Validate input schema
    input_fields = list(json_data[0].keys())
    if not validate_field_names(input_fields):
        expected = FIELD_NAMES
        raise ValueError(f"Input fields {input_fields} don't match canonical schema {expected}")
    
    # Convert to columnar format
    result = {}
    for field_name in FIELD_NAMES:
        values = [row[field_name] for row in json_data]
        dtype = _get_numpy_dtype(field_name)
        
        # Handle different data types appropriately
        if field_name in ['open_time', 'close_time', 'trade_count', 'first_id', 'last_id']:
            result[field_name] = np.array(values, dtype=dtype)
        else:  # decimal fields
            result[field_name] = np.array([float(v) for v in values], dtype=dtype)
    
    return result


def arrow_to_json(arrow_data: Dict[str, np.ndarray]) -> List[Dict[str, Any]]:
    """
    Convert Arrow columnar format to JSON rangebar format.
    
    Args:
        arrow_data: Dictionary with field names as keys and numpy arrays as values
        
    Returns:
        List of rangebar dictionaries (row-oriented)
        
    Raises:
        ValueError: If input data doesn't match canonical schema
    """
    if not arrow_data:
        return []
    
    # Validate input schema
    input_fields = list(arrow_data.keys())
    if not validate_field_names(input_fields):
        expected = FIELD_NAMES
        raise ValueError(f"Input fields {input_fields} don't match canonical schema {expected}")
    
    # Validate all arrays have same length
    lengths = [len(arr) for arr in arrow_data.values()]
    if not all(length == lengths[0] for length in lengths):
        raise ValueError(f"All arrays must have same length, got: {dict(zip(input_fields, lengths))}")
    
    if lengths[0] == 0:
        return []
    
    # Convert to row format
    result = []
    for i in range(lengths[0]):
        row = {}
        for field_name in FIELD_NAMES:
            value = arrow_data[field_name][i]
            
            # Convert numpy types to Python native types
            if field_name in ['open_time', 'close_time', 'trade_count', 'first_id', 'last_id']:
                row[field_name] = int(value)
            else:  # decimal fields
                row[field_name] = float(value)
        
        result.append(row)
    
    return result


def normalize_rust_output(rust_output: Dict[str, np.ndarray]) -> Dict[str, np.ndarray]:
    """
    Normalize Rust output to ensure field name consistency and proper types.
    
    Args:
        rust_output: Raw dictionary from Rust compute_range_bars function
        
    Returns:
        Normalized dictionary with canonical field names and types
        
    Raises:
        ValueError: If required fields are missing
    """
    # Check if we have all required fields
    missing_fields = set(FIELD_NAMES) - set(rust_output.keys())
    if missing_fields:
        raise ValueError(f"Missing required fields: {missing_fields}")
    
    # Ensure proper field ordering and types
    result = {}
    for field_name in FIELD_NAMES:
        if field_name not in rust_output:
            continue
            
        array = rust_output[field_name]
        dtype = _get_numpy_dtype(field_name)
        
        # Ensure proper numpy array with correct dtype
        if not isinstance(array, np.ndarray):
            array = np.array(array)
        
        # Convert to proper dtype if needed
        if array.dtype != dtype:
            result[field_name] = array.astype(dtype)
        else:
            result[field_name] = array
    
    return result


def _get_numpy_dtype(field_name: str) -> np.dtype:
    """Get appropriate numpy dtype for field."""
    field_def = next(f for f in RANGEBAR_FIELDS if f.name == field_name)
    
    if field_def.type == "timestamp_ms":
        return np.int64
    elif field_def.type == "int64":
        return np.int64
    elif field_def.type == "decimal":
        return np.float64
    else:
        raise ValueError(f"Unknown field type: {field_def.type}")


def validate_rangebar_data(data: Union[List[Dict[str, Any]], Dict[str, np.ndarray]], 
                          format_type: str = "auto") -> bool:
    """
    Validate rangebar data against canonical schema.
    
    Args:
        data: Rangebar data in JSON or Arrow format
        format_type: "json", "arrow", or "auto" to detect
        
    Returns:
        True if data is valid
        
    Raises:
        ValueError: If data is invalid with detailed error message
    """
    if format_type == "auto":
        format_type = "json" if isinstance(data, list) else "arrow"
    
    if format_type == "json":
        if not isinstance(data, list):
            raise ValueError("JSON format must be a list of dictionaries")
        
        if not data:  # Empty is valid
            return True
            
        # Check first row for field validation
        if not isinstance(data[0], dict):
            raise ValueError("JSON format must contain dictionaries")
        
        return validate_field_names(list(data[0].keys()))
    
    elif format_type == "arrow":
        if not isinstance(data, dict):
            raise ValueError("Arrow format must be a dictionary")
        
        if not data:  # Empty is valid
            return True
            
        # Validate field names
        if not validate_field_names(list(data.keys())):
            return False
        
        # Validate array lengths are consistent
        lengths = [len(arr) for arr in data.values()]
        if not all(length == lengths[0] for length in lengths):
            raise ValueError(f"Inconsistent array lengths: {dict(zip(data.keys(), lengths))}")
        
        return True
    
    else:
        raise ValueError(f"Unknown format_type: {format_type}")


# Convenience functions for common operations
def rust_to_json(rust_output: Dict[str, np.ndarray]) -> List[Dict[str, Any]]:
    """Convert Rust output directly to JSON format."""
    normalized = normalize_rust_output(rust_output)
    return arrow_to_json(normalized)


def json_to_rust_compatible(json_data: List[Dict[str, Any]]) -> Dict[str, np.ndarray]:
    """Convert JSON data to Rust-compatible Arrow format."""
    arrow_data = json_to_arrow(json_data)
    return normalize_rust_output(arrow_data)


if __name__ == "__main__":
    import json
    
    # Test conversion utilities
    print("🔄 RANGEBAR CONVERSION UTILITIES")
    print("=" * 40)
    
    # Test data
    test_json = [
        {
            "open_time": 1000,
            "close_time": 2000,
            "open": 50000.0,
            "high": 50200.0,
            "low": 49800.0,
            "close": 50100.0,
            "volume": 10.5,
            "turnover": 525000.0,
            "trade_count": 25,
            "first_id": 1001,
            "last_id": 1025
        }
    ]
    
    print("Original JSON:")
    print(json.dumps(test_json[0], indent=2))
    
    # JSON -> Arrow
    arrow_data = json_to_arrow(test_json)
    print("\nConverted to Arrow (first few values):")
    for field, array in arrow_data.items():
        print(f"  {field}: {array[0]} (dtype: {array.dtype})")
    
    # Arrow -> JSON
    json_back = arrow_to_json(arrow_data)
    print("\nConverted back to JSON:")
    print(json.dumps(json_back[0], indent=2))
    
    # Validation
    print("\nValidation results:")
    print(f"JSON format valid: {validate_rangebar_data(test_json, 'json')}")
    print(f"Arrow format valid: {validate_rangebar_data(arrow_data, 'arrow')}")
    
    print("\n✅ Conversion utilities working correctly!")