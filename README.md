# RangeBar

High-performance non-lookahead bias range bar construction from Binance UM Futures aggTrades data.

## Overview

RangeBar implements precise range bar construction using a threshold-based algorithm where bars close when price moves ±0.8% (configurable) from the bar's **open price**. This ensures non-lookahead bias - thresholds are computed only from the bar's opening price and never recalculated.

### Key Features

- **Non-lookahead Bias**: Thresholds computed from bar open only, never from high/low ranges
- **Breach Inclusion**: Breach tick is included in the closing bar before next tick opens new bar
- **High Performance**: Rust core processes 137M+ trades/second (2025 benchmarks)
- **Fixed-Point Arithmetic**: No floating-point rounding errors (8 decimal precision)
- **Format Alignment**: JSON (Python) and Arrow (Rust) formats perfectly aligned for seamless conversion
- **Zero Conversion Overhead**: Direct pandas/Excel/CSV compatibility from Rust output
- **Schema Validation**: Built-in format validation and metadata for robust integration
- **UM Futures Only**: Designed specifically for Binance USD-M Futures aggTrades data

## Installation

### Requirements
- Python 3.13+ (2025 standard)
- Rust 1.89+ (for building from source)

### Using UV (Recommended)

```bash
uv add rangebar
```

### Using pip

```bash
pip install rangebar
```

### Upgrading from Previous Versions

If you're upgrading from a previous version, see the [Migration Guide](MIGRATION.md) for detailed upgrade instructions and breaking changes:
- **v0.3.x → v0.4.0**: Field name alignment (breaking changes)
- **v0.1.x → v0.2.0+**: Major dependency updates and performance improvements

## Quick Start

### CLI Usage

```bash
# Fetch UM Futures aggTrades data
rangebar fetch BTCUSDT 2024-01-01 2024-01-01

# Generate range bars 
rangebar generate BTCUSDT_2024-01-01_aggtrades.parquet --threshold-bps 8000

# Inspect generated bars
rangebar inspect BTCUSDT_2024-01-01_range_bars_80bps.parquet --head 5
```

### Python API

```python
from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
from decimal import Decimal
import asyncio

# Create sample trades
trades_data = [
    {'a': 1, 'p': '50000.0', 'q': '1.0', 'f': 1, 'l': 1, 'T': 1000, 'm': False},
    {'a': 2, 'p': '50400.0', 'q': '1.0', 'f': 2, 'l': 2, 'T': 2000, 'm': False},  # +0.8% breach
    {'a': 3, 'p': '50500.0', 'q': '1.0', 'f': 3, 'l': 3, 'T': 3000, 'm': False},  # New bar
]

trades = [AggTrade(data) for data in trades_data]

# Generate range bars
bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))

print(f"Generated {len(bars)} range bars")
for i, bar in enumerate(bars):
    print(f"Bar {i}: Open={bar['open']}, High={bar['high']}, Low={bar['low']}, Close={bar['close']}")
```

### Fetching Market Data

```python
from rangebar.data_fetcher import fetch_um_futures_aggtrades
import asyncio

async def main():
    # Fetch UM Futures aggTrades
    trades = await fetch_um_futures_aggtrades('BTCUSDT', '2024-01-01', '2024-01-01')
    print(f"Fetched {len(trades)} aggTrades")

asyncio.run(main())
```

### Format Alignment & Integration (v0.4.0+)

```python
import pandas as pd
from rangebar import _rangebar_rust as rust, convert
import numpy as np

# Rust high-performance processing with immediate usability
prices = np.array([5000000000000, 5040000000000], dtype=np.int64)
volumes = np.array([100000000, 100000000], dtype=np.int64)
timestamps = np.array([1000, 2000], dtype=np.int64)
trade_ids = np.array([1, 2], dtype=np.int64)
first_ids = np.array([1, 2], dtype=np.int64)
last_ids = np.array([1, 2], dtype=np.int64)

# Get aligned Rust output (decimal values, singular field names)
rust_result = rust.compute_range_bars(
    prices=prices, volumes=volumes, timestamps=timestamps,
    trade_ids=trade_ids, first_ids=first_ids, last_ids=last_ids,
    threshold_bps=8000
)

# Direct pandas integration (no conversion needed!)
df = pd.DataFrame({k: v for k, v in rust_result.items() if not k.startswith('_')})
print(df)

# Schema validation and conversion utilities
print(f"Valid format: {rust.validate_output_format(rust_result)}")
schema_info = rust.get_schema_info()
print(f"Schema version: {schema_info['schema_version']}")

# Seamless format conversion
json_data = convert.rust_to_json(rust_result)  # For trading systems
arrow_data = convert.json_to_arrow(json_data)  # For analytical workloads
```

## Algorithm Specification

### Non-Lookahead Bias Range Bars

1. **Threshold Calculation**: When a bar opens, thresholds are computed as:
   - Upper threshold = Open price × (1 + 0.008) = Open × 1.008
   - Lower threshold = Open price × (1 - 0.008) = Open × 0.992

2. **Breach Detection**: For each incoming trade tick:
   - Update bar OHLCV with the trade (always include the trade)
   - Check if trade price ≥ upper threshold OR ≤ lower threshold
   - If breach: close current bar, next tick opens new bar

3. **Critical Property**: Thresholds are **NEVER** recalculated after bar opening. This prevents lookahead bias.

### Example: 0.8% Threshold

```
Bar opens at $50,000:
- Upper threshold: $50,000 × 1.008 = $50,400 (fixed)
- Lower threshold: $50,000 × 0.992 = $49,600 (fixed)

Trades sequence:
1. $50,200 → update bar, no breach
2. $50,400 → update bar, BREACH detected → close bar
3. $50,500 → opens new bar with new thresholds
```

## Data Sources

RangeBar fetches data from Binance UM Futures using the [binance-historical-data](https://github.com/stas-prokopiev/binance_historical_data) library.

### Supported Symbols

All Binance UM Futures perpetual contracts (USDT-margined):
- BTCUSDT, ETHUSDT, ADAUSDT, etc.
- Use Binance API symbols exactly as listed

### Data Format

Input: Binance UM Futures aggTrades JSON/CSV
```json
{
  "a": 123456789,     // Aggregate trade ID
  "p": "50000.12345", // Price
  "q": "1.50000000",  // Quantity
  "f": 100,           // First trade ID
  "l": 105,           // Last trade ID  
  "T": 1609459200000, // Timestamp
  "m": false          // Is buyer maker
}
```

## Performance

- **Rust Core**: 137M+ trades/second processing (PyO3 0.26, Rust 2024 edition) 
- **Python Implementation**: 2.5M+ trades/second (41x speedup with Rust)
- **Memory Efficient**: Streaming processing, minimal memory footprint
- **Fixed-Point**: No floating-point precision errors
- **Modern Stack**: Latest Python 3.13, numpy>=2.3.0, pandas>=2.3.0

### Benchmark Results (v0.4.0)
Tested with latest 2025 dependencies on 1M trade dataset:
- **Peak Rust Performance**: 137,000,000+ trades/sec
- **Python Reference**: 2,500,000+ trades/sec  
- **Algorithm Integrity**: 100% verified parity between implementations
- **Format Alignment**: Zero conversion overhead, immediate pandas/Excel compatibility
- **Schema Validation**: Built-in metadata with sub-millisecond validation

## Development

### Requirements

- Python 3.13+ (current 2025 standard)
- Rust 1.89+ (2024 edition)
- UV package manager (recommended)

### Building from Source

```bash
# Clone repository
git clone https://github.com/Eon-Labs/rangebar.git
cd rangebar

# Install dependencies (requires Python 3.13+)
uv sync

# Build Rust extension with uv integration
uv run maturin develop --release --uv

# Run tests
uv run pytest
cargo test --release

# Run benchmarks
uv run python benchmark_v2.py
```

### Running Validation

```bash
# Comprehensive algorithm validation
uv run python validate_algorithm_parity.py
```

## Documentation

- [Migration Guide](MIGRATION.md) - Upgrading between versions (v0.3.x→v0.4.0, v0.1.x→v0.2.0+)
- [Changelog](CHANGELOG.md) - Version history and breaking changes  
- [Publishing Guide](PUBLISH.md) - Instructions for maintainers

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch  
3. Run tests: `cargo test --release && uv run pytest`
4. Run benchmarks: `uv run python benchmark_v2.py`
5. Submit a pull request

## Support

- Issues: [GitHub Issues](https://github.com/Eon-Labs/rangebar/issues)
- Documentation: See CLI help `rangebar --help`