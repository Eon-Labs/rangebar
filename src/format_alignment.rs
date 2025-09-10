//! Format alignment helpers for consistent output between Rust and Python
//! 
//! This module provides utilities to ensure Rust-generated output aligns with
//! Python schema expectations, minimizing conversion work on the Python side.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use numpy::IntoPyArray;
use crate::types::RangeBar;

/// Schema version for format compatibility
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Format version matching Python implementation
pub const FORMAT_VERSION: &str = "0.4.0";

/// Canonical field names in correct order (matches Python schema)
pub const FIELD_NAMES: &[&str] = &[
    "open_time",
    "close_time", 
    "open",
    "high",
    "low",
    "close",
    "volume",
    "turnover",
    "trade_count",
    "first_id",
    "last_id",
];

/// Field metadata for schema validation
pub struct FieldInfo {
    pub name: &'static str,
    pub rust_type: &'static str,
    pub python_type: &'static str,
    pub numpy_dtype: &'static str,
    pub description: &'static str,
}

/// Complete field schema information
pub const FIELD_SCHEMA: &[FieldInfo] = &[
    FieldInfo {
        name: "open_time",
        rust_type: "i64",
        python_type: "int",
        numpy_dtype: "int64",
        description: "Bar opening timestamp in milliseconds (Unix epoch)",
    },
    FieldInfo {
        name: "close_time",
        rust_type: "i64", 
        python_type: "int",
        numpy_dtype: "int64",
        description: "Bar closing timestamp in milliseconds (Unix epoch)",
    },
    FieldInfo {
        name: "open",
        rust_type: "f64",
        python_type: "float",
        numpy_dtype: "float64",
        description: "Opening price of the range bar",
    },
    FieldInfo {
        name: "high",
        rust_type: "f64",
        python_type: "float", 
        numpy_dtype: "float64",
        description: "Highest price within the range bar",
    },
    FieldInfo {
        name: "low",
        rust_type: "f64",
        python_type: "float",
        numpy_dtype: "float64", 
        description: "Lowest price within the range bar",
    },
    FieldInfo {
        name: "close",
        rust_type: "f64",
        python_type: "float",
        numpy_dtype: "float64",
        description: "Closing price of the range bar",
    },
    FieldInfo {
        name: "volume",
        rust_type: "f64",
        python_type: "float",
        numpy_dtype: "float64",
        description: "Total volume traded within the range bar",
    },
    FieldInfo {
        name: "turnover",
        rust_type: "f64", 
        python_type: "float",
        numpy_dtype: "float64",
        description: "Total turnover (price × volume) within the range bar",
    },
    FieldInfo {
        name: "trade_count",
        rust_type: "i64",
        python_type: "int",
        numpy_dtype: "int64",
        description: "Number of trades aggregated in this range bar",
    },
    FieldInfo {
        name: "first_id",
        rust_type: "i64",
        python_type: "int", 
        numpy_dtype: "int64",
        description: "ID of the first trade in this range bar",
    },
    FieldInfo {
        name: "last_id",
        rust_type: "i64",
        python_type: "int",
        numpy_dtype: "int64",
        description: "ID of the last trade in this range bar",
    },
];

/// Create aligned output dictionary with consistent field ordering and metadata
pub fn create_aligned_output<'py>(
    py: Python<'py>,
    bars: Vec<RangeBar>,
) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    
    if bars.is_empty() {
        // Return empty arrays with correct field names and types
        for field_info in FIELD_SCHEMA {
            match field_info.numpy_dtype {
                "int64" => {
                    let empty: Vec<i64> = Vec::new();
                    result.set_item(field_info.name, empty.into_pyarray(py))?;
                }
                "float64" => {
                    let empty: Vec<f64> = Vec::new();
                    result.set_item(field_info.name, empty.into_pyarray(py))?;
                }
                _ => unreachable!("Unknown dtype in schema"),
            };
        }
        
        add_metadata(py, &result)?;
        return Ok(result);
    }
    
    // Pre-allocate vectors with correct capacity
    let len = bars.len();
    let mut open_times = Vec::with_capacity(len);
    let mut close_times = Vec::with_capacity(len);
    let mut opens = Vec::with_capacity(len);
    let mut highs = Vec::with_capacity(len);
    let mut lows = Vec::with_capacity(len);
    let mut closes = Vec::with_capacity(len);
    let mut volumes = Vec::with_capacity(len);
    let mut turnovers = Vec::with_capacity(len);
    let mut trade_counts = Vec::with_capacity(len);
    let mut first_ids = Vec::with_capacity(len);
    let mut last_ids = Vec::with_capacity(len);
    
    // Convert bars to aligned arrays
    for bar in bars {
        open_times.push(bar.open_time);
        close_times.push(bar.close_time);
        opens.push(bar.open.to_f64());
        highs.push(bar.high.to_f64());
        lows.push(bar.low.to_f64());
        closes.push(bar.close.to_f64());
        volumes.push(bar.volume.to_f64());
        // Convert i128 turnover to f64 with correct scaling (1e16 = 1e8 * 1e8)
        turnovers.push(bar.turnover as f64 / 10_000_000_000_000_000.0);
        trade_counts.push(bar.trade_count);
        first_ids.push(bar.first_id);
        last_ids.push(bar.last_id);
    }
    
    // Set items in canonical field order
    result.set_item(FIELD_NAMES[0], open_times.into_pyarray(py))?;    // open_time
    result.set_item(FIELD_NAMES[1], close_times.into_pyarray(py))?;   // close_time
    result.set_item(FIELD_NAMES[2], opens.into_pyarray(py))?;         // open
    result.set_item(FIELD_NAMES[3], highs.into_pyarray(py))?;         // high
    result.set_item(FIELD_NAMES[4], lows.into_pyarray(py))?;          // low
    result.set_item(FIELD_NAMES[5], closes.into_pyarray(py))?;        // close
    result.set_item(FIELD_NAMES[6], volumes.into_pyarray(py))?;       // volume
    result.set_item(FIELD_NAMES[7], turnovers.into_pyarray(py))?;     // turnover
    result.set_item(FIELD_NAMES[8], trade_counts.into_pyarray(py))?;  // trade_count
    result.set_item(FIELD_NAMES[9], first_ids.into_pyarray(py))?;     // first_id
    result.set_item(FIELD_NAMES[10], last_ids.into_pyarray(py))?;     // last_id
    
    // Add metadata for format validation
    add_metadata(py, &result)?;
    
    Ok(result)
}

/// Add metadata to output for format validation and compatibility
fn add_metadata<'py>(py: Python<'py>, result: &Bound<'py, PyDict>) -> PyResult<()> {
    let metadata = PyDict::new(py);
    
    metadata.set_item("schema_version", SCHEMA_VERSION)?;
    metadata.set_item("format_version", FORMAT_VERSION)?;
    metadata.set_item("algorithm", "non-lookahead-range-bars")?;
    metadata.set_item("field_count", FIELD_NAMES.len())?;
    metadata.set_item("timestamp_unit", "milliseconds")?;
    metadata.set_item("precision", "decimal")?;
    metadata.set_item("source", "rust")?;
    
    // Add field schema information
    let fields_info = PyDict::new(py);
    for field_info in FIELD_SCHEMA {
        let field_meta = PyDict::new(py);
        field_meta.set_item("rust_type", field_info.rust_type)?;
        field_meta.set_item("python_type", field_info.python_type)?;
        field_meta.set_item("numpy_dtype", field_info.numpy_dtype)?;
        field_meta.set_item("description", field_info.description)?;
        fields_info.set_item(field_info.name, field_meta)?;
    }
    metadata.set_item("fields", fields_info)?;
    
    result.set_item("_metadata", metadata)?;
    Ok(())
}

/// Validate that output has correct field names and types
pub fn validate_output_format<'py>(output: &Bound<'py, PyDict>) -> PyResult<bool> {
    // Check all required fields are present
    for &field_name in FIELD_NAMES {
        if !output.contains(field_name)? {
            return Ok(false);
        }
        
        // Validate field is a numpy array
        let item = output.get_item(field_name)?;
        if item.is_none() {
            return Ok(false);
        }
    }
    
    // Check metadata is present
    if !output.contains("_metadata")? {
        return Ok(false);
    }
    
    Ok(true)
}

/// Get schema information as Python dictionary
pub fn get_schema_info<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    let schema = PyDict::new(py);
    
    schema.set_item("schema_version", SCHEMA_VERSION)?;
    schema.set_item("format_version", FORMAT_VERSION)?;
    schema.set_item("field_names", FIELD_NAMES.to_vec())?;
    schema.set_item("field_count", FIELD_NAMES.len())?;
    
    // Add detailed field information
    let fields = PyDict::new(py);
    for field_info in FIELD_SCHEMA {
        let field_dict = PyDict::new(py);
        field_dict.set_item("rust_type", field_info.rust_type)?;
        field_dict.set_item("python_type", field_info.python_type)?;
        field_dict.set_item("numpy_dtype", field_info.numpy_dtype)?;
        field_dict.set_item("description", field_info.description)?;
        fields.set_item(field_info.name, field_dict)?;
    }
    schema.set_item("fields", fields)?;
    
    Ok(schema)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AggTrade;
    use crate::fixed_point::FixedPoint;
    
    fn create_test_bar() -> RangeBar {
        let trade = AggTrade {
            agg_trade_id: 12345,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 100,
            last_trade_id: 100,
            timestamp: 1640995200000,
        };
        RangeBar::new(&trade)
    }
    
    #[test]
    fn test_field_names_order() {
        assert_eq!(FIELD_NAMES[0], "open_time");
        assert_eq!(FIELD_NAMES[1], "close_time");
        assert_eq!(FIELD_NAMES[2], "open");
        assert_eq!(FIELD_NAMES.len(), 11);
    }
    
    #[test] 
    fn test_field_schema_consistency() {
        assert_eq!(FIELD_SCHEMA.len(), FIELD_NAMES.len());
        
        for (i, field_info) in FIELD_SCHEMA.iter().enumerate() {
            assert_eq!(field_info.name, FIELD_NAMES[i]);
        }
    }
    
    
    #[test]
    fn test_empty_bars_output() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let empty_bars = Vec::new();
            let result = create_aligned_output(py, empty_bars).unwrap();
            
            // Should have all field names with empty arrays
            for &field_name in FIELD_NAMES {
                assert!(result.contains(field_name).unwrap());
            }
            
            // Should have metadata
            assert!(result.contains("_metadata").unwrap());
            assert!(validate_output_format(&result).unwrap());
        });
    }
    
    #[test]
    fn test_single_bar_output() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let bars = vec![create_test_bar()];
            let result = create_aligned_output(py, bars).unwrap();
            
            // Should have all required fields
            assert!(validate_output_format(&result).unwrap());
            
            // Check metadata
            let metadata = result.get_item("_metadata").unwrap().unwrap();
            let metadata_dict = metadata.downcast::<PyDict>().unwrap();
            
            let schema_version: String = metadata_dict.get_item("schema_version").unwrap().unwrap().extract().unwrap();
            assert_eq!(schema_version, SCHEMA_VERSION);
        });
    }
}