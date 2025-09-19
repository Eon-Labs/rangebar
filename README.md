# rangebar

[![Crates.io](https://img.shields.io/crates/v/rangebar)](https://crates.io/crates/rangebar)
[![Documentation](https://docs.rs/rangebar/badge.svg)](https://docs.rs/rangebar)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Non-lookahead range bar construction for cryptocurrency trading with temporal integrity guarantees.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rangebar = "0.5"
```

## Usage

```rust
use rangebar::{RangeBarProcessor, AggTrade, FixedPoint};

// Create processor with 25 basis points threshold
let mut processor = RangeBarProcessor::new(25);

// Create sample trade
let trade = AggTrade {
    agg_trade_id: 1,
    price: FixedPoint::from_str("50000.0").unwrap(),
    volume: FixedPoint::from_str("1.0").unwrap(),
    first_trade_id: 1,
    last_trade_id: 1,
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

Range bars close when price moves ±threshold basis points from the bar's **opening price**:

1. **Non-lookahead bias**: Thresholds computed only from bar open price
2. **Breach inclusion**: Breaching trade included in closing bar
3. **Fixed thresholds**: Never recalculated during bar lifetime

## Features

- Non-lookahead bias range bar construction
- Fixed-point arithmetic for precision
- Streaming and batch processing modes
- Tier-1 cryptocurrency symbol discovery

## License

MIT license. See [LICENSE](LICENSE) for details.
