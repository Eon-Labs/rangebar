# Changelog

All notable changes to RangeBar will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2025-09-18

### Removed
- Legacy streaming implementations: `streaming_processor.rs`, `streaming_processor_optimized.rs`, `streaming_abstraction.rs`
- Legacy test infrastructure: `optimized_streaming_validation.rs`
- Legacy dual-run validation test from `production_streaming_validation.rs`

### Changed
- Architecture consolidation: Batch (`ExportRangeBarProcessor`) + Production V2 (`StreamingProcessor`)
- Cross-year performance test updated for batch vs V2 comparison only
- Memory comparison demo simplified to batch processing demonstration
- Build and test infrastructure updated for simplified architecture

### Fixed
- Production Streaming V2 final incomplete bar handling: automatic dispatch when trade channel closes
- Algorithmic consistency verification: identical bar counts across batch and V2 implementations
- Bounded memory guarantee maintained: <172MB peak usage for infinite streaming capability

## [0.5.0] - 2025-09-17

### Added
- Introduced `config` module (`src/config/*`) with `Settings` aggregator for TOML, environment, and CLI merging.
- Added CSV streaming ingestion (`src/csv_streaming.rs`) and new `StreamingRangeBarProcessor` for bounded Tokio pipelines.
- Exposed streaming statistics support (`src/streaming_stats.rs`) alongside `StreamingStatsSummary` re-exports.
- Published OpenAPI contract (`api/openapi.yaml`) and `rangebar_api` binary scaffold.
- Added regression suites under `tests/` and `test_streaming/` plus reusable fixtures in `examples/`.

### Changed
- Reworked `RangeBarProcessor` internals to validate ordering, expose incomplete-bar option, and supply export-oriented processor variant.
- Expanded statistical engine to emit metadata, performance, and validation summaries compatible with streaming collectors.
- Updated CLI binaries (`tier1_symbol_discovery`, `rangebar_export`, `parallel_tier1_analysis`) to load shared configuration and API metadata.
- Moved audit and benchmark documentation into `docs/` hierarchy; migrated premium dashboard assets to archival directory.
- Updated README default threshold guidance and dependency usage to align with new processor defaults.

### Removed
- Deleted legacy root-level audit markdown files replaced by `docs/*` versions.
- Removed obsolete generated dashboard artifacts from `output/premium_analysis/tradability_analysis/`.

## [0.4.0] - 2025-09-09

### 🔗 Format Alignment & Interoperability
- **MAJOR**: Aligned JSON (Python) and Arrow (Rust) formats for seamless conversion
- **Field Names**: Standardized to singular form (e.g., `opens` → `open`, `turnovers` → `turnover`)
- **Shared Schema**: Added `rangebar.schema` module with canonical field definitions  
- **Zero Conversion**: Rust output now requires minimal Python-side processing

### 🛠️ Rust-Level Improvements
- **Format Alignment Module**: New `format_alignment.rs` with consistent output helpers
- **Metadata Integration**: Automatic schema metadata in all Rust outputs
- **Validation Functions**: Built-in format validation (`validate_output_format`)
- **Schema Introspection**: Runtime schema information access (`get_schema_info`)

### 🔄 Conversion Utilities
- **NEW**: `rangebar.convert` module for seamless JSON ↔ Arrow conversion
- **Functions**: `json_to_arrow()`, `arrow_to_json()`, `normalize_rust_output()`
- **Validation**: `validate_rangebar_data()` with automatic format detection
- **Compatibility**: Handle both legacy plural and new singular field names

### 📊 Enhanced Integration
- **Direct Pandas**: Create DataFrames directly from Rust output (no conversion needed)
- **Excel/CSV Ready**: Immediate export compatibility with external tools
- **API Consistency**: Uniform field naming across all output formats  
- **Schema Versioning**: Version-aware format compatibility checking

### 🧪 Testing & Validation
- **Updated Tests**: All test files updated for new singular field names
- **Format Validation**: New tests for metadata presence and schema consistency
- **Integration Tests**: Updated usability tests showing v0.4.0 improvements
- **Regression Protection**: Maintain backward compatibility detection

### 🚀 Performance & Architecture  
- **Simplified Output**: Consolidated array creation in Rust reduces code complexity
- **Metadata Overhead**: Negligible performance impact from built-in metadata
- **Memory Efficiency**: Optimized vector pre-allocation in alignment helpers

### 📚 API Changes
- **BREAKING**: Field names changed from plural to singular (migration required)
- **NEW**: `rangebar.convert` module available in public API
- **NEW**: `rangebar.schema` module available in public API  
- **ENHANCED**: Rust functions now include `get_schema_info()` and `validate_output_format()`

### 🔧 Migration Guide
- **Field Names**: Update code accessing `opens` → `open`, `turnovers` → `turnover`, etc.
- **Metadata**: Output now includes `_metadata` field with schema information
- **Validation**: Use `convert.validate_rangebar_data()` for format checking
- **Conversion**: Use `convert` module functions for format transformations

## [0.3.0] - 2025-09-09

### 🚀 Major Usability Improvements
- **BREAKING**: Rust output now returns decimal values instead of raw scaled integers
- **User-Friendly API**: All price/volume/turnover values now immediately usable (no manual scaling)
- **Excel/CSV Ready**: Direct pandas, Excel, and CSV compatibility out of the box
- **Zero Learning Curve**: No need to understand internal 1e8 scaling factor

### 🔧 Technical Changes
- Added `to_f64()` method to `FixedPoint` for decimal conversion
- Modified Rust output format: `opens`, `highs`, `lows`, `closes`, `volumes`, `turnovers` now return `f64` arrays
- Updated turnover calculation to handle i128→f64 conversion with correct 1e16 scaling
- Maintained full precision compatibility with Python implementation

### 📊 API Breaking Changes
- **Before**: `result["opens"][0]` → `5000012345678` (required ÷1e8)
- **After**: `result["opens"][0]` → `50000.12345678` (immediately usable)
- **Migration**: Remove any manual `/1e8` scaling in user code

### ✅ Integration Benefits
- Direct pandas DataFrame creation without conversion
- Excel/CSV export works immediately  
- Trading system integration ready
- Data analysis tools compatible
- Matches Python implementation user experience

### 🧪 Testing
- Updated regression test suite for new decimal format
- Maintained sub-microsecond precision consistency with Python
- Verified integration scenarios with real-world data

## [0.2.2] - 2025-09-09

### 🚨 Critical Bug Fixes
- **CRITICAL**: Fixed integer overflow in Rust turnover calculation that produced negative/incorrect values
- Applied data-first debugging protocol to identify root cause: i128→i64 cast overflow in `src/lib.rs:162`
- Solution: Scale down i128 turnover by 1e8 before casting to maintain consistent precision
- Impact: All previous Rust-calculated turnovers were incorrect due to overflow

### 🧪 Testing & Validation
- Added comprehensive regression test suite (`tests/test_turnover_calculation.py`)
- Verified Python-Rust output consistency (sub-nanosecond precision match)
- Validated real-world BTCUSDT turnover values match expected calculations
- Added overflow prevention tests for large volume scenarios

### 🔍 Security Audit
- Conducted systematic overflow audit of entire Rust codebase
- Identified potential overflow in `FixedPoint::from_str` for extreme prices (low priority)
- Confirmed threshold calculations are safe for all realistic price ranges

### 📊 Data Integrity
- Before fix: Turnover values like `-20,242,847,901 USDT` (impossible)
- After fix: Correct values like `383,489.85 USDT` for ~7.6 BTC at 50K price
- Turnovers now correctly represent `price × volume` as per Binance specification

## [0.2.1] - 2025-09-09

### 🐛 Bug Fixes
- Fixed version string reporting in `__init__.py` (now correctly shows 0.2.1)
- Added comprehensive migration guide and changelog

### 📚 Documentation
- **NEW**: [MIGRATION.md](MIGRATION.md) - Complete upgrade guide from v0.1.x
- **NEW**: [CHANGELOG.md](CHANGELOG.md) - Version history and breaking changes
- **NEW**: [benchmark_v2.py](benchmark_v2.py) - Performance verification script
- Updated all documentation with verified performance metrics (137M+ trades/sec)

### ✅ Verified Performance
All performance claims verified through independent testing:
- **Peak Rust Performance**: 137M+ trades/second
- **Python Reference**: 2.5M+ trades/second  
- **41x speedup** with Rust implementation

## [0.2.0] - 2025-09-09

### 🚀 Major Performance Improvements
- **MASSIVE Performance Boost**: Peak performance increased from 2.5M to **137M+ trades/second** (54x improvement)
- Rust implementation now processes 100M+ trades/second consistently with latest dependencies

### 🔧 Updated Dependencies (2025 Latest)
#### Rust Dependencies
- **PyO3**: Updated to 0.26.0 (latest Python bindings)
- **numpy**: Updated to 0.26.0 (latest numpy integration)
- **rayon**: Updated to 1.11.0 (latest parallel processing)
- **thiserror**: Updated to 2.0.0 (latest error handling)
- **Rust Edition**: Updated to 2024 (current for 2025)

#### Python Dependencies
- **Python**: Minimum requirement raised to 3.13+ (2025 standard)
- **numpy**: Updated to >=2.3.0 (latest stable)
- **pandas**: Updated to >=2.3.0 (latest stable)
- **pyarrow**: Updated to >=21.0.0 (latest columnar processing)
- **httpx**: Updated to >=0.28.0 (latest async HTTP)

### 🛠️ Breaking Changes
- **Python Version**: Minimum Python version raised from 3.12+ to 3.13+
- **Dependency Versions**: All dependencies updated to 2025 latest versions
- May require updating development environments to Python 3.13+

### ✨ Enhancements
- Fixed PyO3 0.26 API compatibility issues
- Updated deprecated `allow_threads` to `detach` method
- Added repository metadata to Cargo.toml (fixes build warnings)
- Comprehensive benchmark suite added for performance verification

### 🧪 Testing
- Full algorithm parity maintained between Python and Rust implementations
- Comprehensive benchmarks verify performance across 10K to 1M trade datasets
- All edge cases validated with latest dependency stack

### 📚 Documentation
- Updated README with latest dependency versions and performance metrics
- Added migration guide for upgrading from v0.1.x
- Updated installation requirements and development setup

## [0.1.1] - 2025-09-09

### 🐛 Bug Fixes
- Fixed Python source code inclusion in PyPI wheel
- Added proper maturin configuration for mixed Python/Rust projects

### 📦 Packaging
- Improved wheel building with `python-source = "src"` configuration
- Better MANIFEST.in for comprehensive file inclusion

## [0.1.0] - 2025-09-09

### 🎉 Initial Release
- **Core Algorithm**: Non-lookahead bias range bar construction
- **High Performance**: Rust core with Python bindings
- **Data Integration**: Direct Binance UM Futures aggTrades fetching
- **CLI Tools**: Complete command-line interface
- **Python API**: Easy-to-use Python interface
- **Fixed-Point Arithmetic**: Precise decimal calculations without floating-point errors

### 🔧 Features
- Range bars with configurable thresholds (default 0.8%)
- Breach tick inclusion in closing bars
- Deterministic, reproducible results
- Support for historical data fetching
- Parquet format support for efficient storage

### 📋 Dependencies
- Python 3.12+ support
- PyO3 0.22.x for Python-Rust bindings
- Modern Python data stack (numpy, pandas, pyarrow)

[0.2.0]: https://github.com/Eon-Labs/rangebar/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Eon-Labs/rangebar/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Eon-Labs/rangebar/releases/tag/v0.1.0
