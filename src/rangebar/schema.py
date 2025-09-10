#!/usr/bin/env python3
"""
Shared schema definition for rangebar formats.

This module defines the canonical schema that both Python (JSON) and Rust (Arrow) 
implementations use, ensuring field name consistency and enabling seamless conversion.
"""

from typing import Dict, List, Any, Literal
from dataclasses import dataclass

__version__ = "1.0.0"

@dataclass
class FieldDefinition:
    """Definition of a single field in the rangebar schema."""
    name: str
    type: str
    description: str
    required: bool = True

# Canonical field definitions - shared between Python and Rust
RANGEBAR_FIELDS = [
    FieldDefinition(
        name="open_time",
        type="timestamp_ms",
        description="Bar opening timestamp in milliseconds (Unix epoch)",
        required=True
    ),
    FieldDefinition(
        name="close_time", 
        type="timestamp_ms",
        description="Bar closing timestamp in milliseconds (Unix epoch)",
        required=True
    ),
    FieldDefinition(
        name="open",
        type="decimal",
        description="Opening price of the range bar",
        required=True
    ),
    FieldDefinition(
        name="high",
        type="decimal", 
        description="Highest price within the range bar",
        required=True
    ),
    FieldDefinition(
        name="low",
        type="decimal",
        description="Lowest price within the range bar", 
        required=True
    ),
    FieldDefinition(
        name="close",
        type="decimal",
        description="Closing price of the range bar",
        required=True
    ),
    FieldDefinition(
        name="volume",
        type="decimal",
        description="Total volume traded within the range bar",
        required=True
    ),
    FieldDefinition(
        name="turnover",
        type="decimal", 
        description="Total turnover (price × volume) within the range bar",
        required=True
    ),
    FieldDefinition(
        name="trade_count",
        type="int64",
        description="Number of trades aggregated in this range bar",
        required=True
    ),
    FieldDefinition(
        name="first_id",
        type="int64",
        description="ID of the first trade in this range bar",
        required=True
    ),
    FieldDefinition(
        name="last_id",
        type="int64", 
        description="ID of the last trade in this range bar",
        required=True
    )
]

# Schema metadata
SCHEMA_METADATA = {
    "schema_version": __version__,
    "format_version": "0.4.0",
    "algorithm": "non-lookahead-range-bars",
    "field_count": len(RANGEBAR_FIELDS),
    "timestamp_unit": "milliseconds",
    "precision": "decimal"
}

def get_field_names() -> List[str]:
    """Get list of canonical field names in order."""
    return [field.name for field in RANGEBAR_FIELDS]

def get_schema_dict() -> Dict[str, Any]:
    """Get complete schema as dictionary."""
    return {
        "metadata": SCHEMA_METADATA,
        "fields": [
            {
                "name": field.name,
                "type": field.type,
                "description": field.description,
                "required": field.required
            }
            for field in RANGEBAR_FIELDS
        ]
    }

def validate_field_names(field_names: List[str]) -> bool:
    """Validate that provided field names match canonical schema."""
    canonical_names = get_field_names()
    return field_names == canonical_names

def get_json_schema() -> Dict[str, Any]:
    """Get JSON schema format for validation."""
    return {
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                field.name: {
                    "type": "string" if field.type == "decimal" else 
                           "integer" if field.type in ["int64", "timestamp_ms"] else "string",
                    "description": field.description
                }
                for field in RANGEBAR_FIELDS
            },
            "required": [field.name for field in RANGEBAR_FIELDS if field.required],
            "additionalProperties": False
        }
    }

def get_arrow_schema() -> Dict[str, Any]:
    """Get Arrow schema format for Rust integration."""
    return {
        "schema": {
            "version": SCHEMA_METADATA["schema_version"],
            "fields": [
                {
                    "name": field.name,
                    "type": field.type,
                    "nullable": not field.required,
                    "description": field.description
                }
                for field in RANGEBAR_FIELDS
            ]
        },
        "metadata": SCHEMA_METADATA
    }

# Export commonly used constants
FIELD_NAMES = get_field_names()
SCHEMA_DICT = get_schema_dict()

if __name__ == "__main__":
    import json
    print("🔗 RANGEBAR SHARED SCHEMA")
    print("=" * 40)
    print(f"Schema Version: {SCHEMA_METADATA['schema_version']}")
    print(f"Field Count: {len(RANGEBAR_FIELDS)}")
    print()
    print("Field Names:")
    for field in RANGEBAR_FIELDS:
        print(f"  {field.name:12} ({field.type})")
    print()
    print("Complete Schema:")
    print(json.dumps(get_schema_dict(), indent=2))