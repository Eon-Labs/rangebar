# rangebar

[![Crates.io](https://img.shields.io/crates/v/rangebar)](https://crates.io/crates/rangebar)
[![Downloads](https://img.shields.io/crates/d/rangebar)](https://crates.io/crates/rangebar)
[![Documentation](https://docs.rs/rangebar/badge.svg)](https://docs.rs/rangebar)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/Eon-Labs/rangebar/workflows/CI/badge.svg)](https://github.com/Eon-Labs/rangebar/actions)

High-performance non-lookahead bias range bar construction for Binance UM Futures data.

> [!NOTE]
> This crate processes **137M+ trades/second** using fixed-point arithmetic for financial-grade precision.

> [!WARNING]
> Range bars use **non-lookahead bias** - thresholds are computed only from bar opening prices, never from evolving high/low ranges.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rangebar = "0.5.0"
```

## Basic Usage

```rust
use rangebar::{RangeBarProcessor, AggTrade, FixedPoint};

// Create processor with 0.8% threshold (80 basis points)
let mut processor = RangeBarProcessor::new(80);

// Create sample trade data
let trade = AggTrade {
    agg_trade_id: 123456789,
    price: FixedPoint::from_str("50000.12345").unwrap(),
    volume: FixedPoint::from_str("1.50000000").unwrap(),
    first_trade_id: 100,
    last_trade_id: 105,
    timestamp: 1609459200000,
    is_buyer_maker: false,
};

// Process trades into range bars
let trades = vec![trade];
let bars = processor.process_trades(&trades).unwrap();

for bar in bars {
    println!("Bar: O={} H={} L={} C={} V={}",
             bar.open, bar.high, bar.low, bar.close, bar.volume);
}
```

## Algorithm

Range bars close when price moves ±threshold% from the bar's **opening price**:

1. **Non-lookahead bias**: Thresholds computed only from bar open price
2. **Breach inclusion**: Breaching trade included in closing bar
3. **Fixed thresholds**: Never recalculated during bar lifetime

## Features

- **`statistics`** (default): Comprehensive statistical analysis via Polars
- **`data-integrity`** (default): Data validation and checksums
- **`arrow-support`**: Apache Arrow and Parquet export
- **`python-bindings`**: PyO3 Python bindings (optional)

## Performance

- **137M+ trades/second** processing (2025 benchmarks)
- **Fixed-point arithmetic** (no floating-point errors)
- **Memory efficient** streaming processing
- **Zero-copy** design where possible

## Data Source

Designed for [Binance UM Futures](https://binance-docs.github.io/apidocs/futures/en/) aggTrades data:

```rust
// Sample aggTrade format
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

## License

MIT license. See [LICENSE](LICENSE) for details.
