# Range Bar Algorithm Specification

## Executive Summary

This specification defines a **non-lookahead bias** range bar construction algorithm for Binance UM Futures aggTrades data. Range bars close when price moves ±0.8% from the bar's **OPEN** price, ensuring no future information influences historical decisions.

## Core Algorithm Definition

### Mathematical Formulation

Given:
- `threshold_bps = 80` (80 basis points = 0.8%)
- Bar opens at price `P_open`

For each bar:
```
threshold_ratio = threshold_bps / 10000  # Convert basis points to decimal
upper_breach = P_open × (1 + threshold_ratio) = P_open × 1.008
lower_breach = P_open × (1 - threshold_ratio) = P_open × 0.992
```

**Bar closes when:** `tick_price >= upper_breach OR tick_price <= lower_breach`

### Critical Properties

1. **Thresholds are FIXED**: Computed once when bar opens, never recalculated
2. **Based on OPEN only**: Not based on evolving high/low or range
3. **Breach tick included**: The tick causing the breach belongs to the closing bar
4. **Non-lookahead**: Current tick cannot influence its own breach decision

## Implementation Pseudocode

### High-Level Algorithm
```python
def iter_range_bars_from_aggtrades(trades, threshold_bps=80):
    threshold_ratio = threshold_bps / 10000  # Convert basis points to decimal
    bar = None
    defer_open = False
    
    for tick in sorted_trades:  # Sorted by (timestamp, aggTradeId)
        if defer_open:
            # Previous bar closed, this tick opens new bar
            bar = new_bar(tick)
            bar.upper_breach = bar.open * (1 + threshold_ratio)
            bar.lower_breach = bar.open * (1 - threshold_ratio)
            defer_open = False
            continue
        
        if bar is None:
            # First bar initialization
            bar = new_bar(tick)
            bar.upper_breach = bar.open * (1 + threshold_ratio)
            bar.lower_breach = bar.open * (1 - threshold_ratio)
            continue
        
        # Update bar with current tick (ALWAYS include tick first)
        bar.high = max(bar.high, tick.price)
        bar.low = min(bar.low, tick.price)
        bar.close = tick.price
        bar.volume += tick.quantity
        bar.trade_count += (tick.last_trade_id - tick.first_trade_id + 1)
        bar.close_time = tick.timestamp
        bar.last_id = tick.agg_trade_id
        
        # Check breach using FIXED thresholds (computed from open)
        if tick.price >= bar.upper_breach or tick.price <= bar.lower_breach:
            yield finalize_bar(bar)
            bar = None
            defer_open = True  # Next tick will open new bar
    
    # Yield final partial bar if exists
    if bar is not None:
        yield finalize_bar(bar)
```

### Bar Structure
```python
class RangeBar:
    # OHLCV data
    open_time: int        # Timestamp of first tick (ms)
    close_time: int       # Timestamp of last tick (ms) 
    open: Decimal         # First tick price
    high: Decimal         # Maximum price in bar
    low: Decimal          # Minimum price in bar
    close: Decimal        # Last tick price (breach tick)
    
    # Volume and trade data
    volume: Decimal       # Sum of quantities
    turnover: Decimal     # Sum of price × quantity
    trade_count: int      # Number of individual trades
    first_id: int         # First aggTradeId
    last_id: int          # Last aggTradeId
    
    # Algorithm metadata (not output)
    upper_breach: Decimal # Fixed threshold (open × 1.008)
    lower_breach: Decimal # Fixed threshold (open × 0.992)
```

## Data Source Specification

### Binance UM Futures aggTrades

**Source**: https://github.com/stas-prokopiev/binance_historical_data

**Configuration**:
```python
from binance_historical_data import BinanceDataDumper

dumper = BinanceDataDumper(
    path_dir_where_to_dump="data/um_futures",
    asset_class="um",      # USD-M Futures ONLY
    data_type="aggTrades"  # Aggregate trades ONLY
)
```

### Schema (UM Futures aggTrades)
| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `a` | int64 | Aggregate trade ID | 26129 |
| `p` | string | Price (decimal) | "0.01633102" |
| `q` | string | Quantity | "4.70443515" |
| `f` | int64 | First trade ID | 27781 |
| `l` | int64 | Last trade ID | 27781 |
| `T` | int64 | Timestamp (milliseconds) | 1498793709153 |
| `m` | bool | Was buyer the maker | true |

### Data Requirements
1. **Sorting**: Must be sorted by `(T, a)` for deterministic processing
2. **Completeness**: No gaps in trade sequence
3. **Validation**: SHA256 checksums verified
4. **Precision**: Preserve full decimal precision in price/quantity

## Non-Lookahead Validation

### What Makes It Non-Lookahead

✅ **Correct (No Lookahead)**:
1. Compute thresholds from bar's open price
2. Include current tick in bar statistics  
3. Check if current tick breaches pre-computed thresholds
4. If breach: close bar, mark next tick to open new bar

❌ **Incorrect (Has Lookahead)**:
1. Include current tick in bar statistics
2. Recompute thresholds using updated high/low  
3. Check if current tick breaches updated thresholds

### Validation Tests

```python
def test_no_lookahead_bias():
    """Verify thresholds computed from open only"""
    open_price = 50000.00
    
    # These ticks should NOT cause thresholds to change
    ticks = [
        {'price': 50000.00, 'timestamp': 1000},  # Open
        {'price': 50300.00, 'timestamp': 2000},  # +0.6% from open
        {'price': 49700.00, 'timestamp': 3000},  # -0.6% from open
        {'price': 50400.00, 'timestamp': 4000},  # +0.8% from open -> BREACH
    ]
    
    bars = list(iter_range_bars_from_aggtrades(ticks))
    
    # Thresholds should always be based on 50000.00 (open)
    assert bars[0]['close'] == 50400.00  # Breach tick included
    assert bars[0]['high'] == 50400.00   # Maximum reached
    assert bars[0]['low'] == 49700.00    # Minimum reached
```

## Performance Requirements

### Rust Implementation Targets

| Dataset Size | Target Time | Memory Usage |
|--------------|-------------|--------------|
| 1M ticks | < 100ms | < 50MB |
| 100M ticks | < 3s | < 500MB |
| 1B ticks | < 30s | < 1GB |

### Optimization Strategies

1. **Fixed-Point Arithmetic**: Use `i64` with 8 decimal precision instead of `Decimal`
2. **Zero-Copy Data Transfer**: Arrow arrays between Python and Rust
3. **Structure of Arrays**: Cache-friendly memory layout
4. **Parallel Processing**: Multiple symbols processed concurrently
5. **Streaming**: O(1) memory per output bar

### Fixed-Point Implementation
```rust
// Scale factor for 8 decimal precision
const SCALE: i64 = 100_000_000;

// Convert string price to fixed-point
fn to_fixed_point(price: &str) -> i64 {
    let decimal = price.parse::<f64>().unwrap();
    (decimal * SCALE as f64) as i64
}

// Compute thresholds without floating point
fn compute_thresholds(open: i64, basis_points: u32) -> (i64, i64) {
    let delta = (open * basis_points as i64) / 1_000_000;
    (open + delta, open - delta)  // (upper, lower)
}
```

## Edge Cases

### 1. Exact Threshold Breach
```
Open: 50000.00
Threshold: +0.8% = 50400.00
Tick: 50400.00 (exact match)
Result: Bar closes, breach tick included
```

### 2. Large Price Gap
```
Open: 50000.00  
Threshold: ±0.8% = [49600.00, 50400.00]
Tick: 51000.00 (+2% gap)
Result: Single bar closes at 51000.00 (NOT multiple bars)
```

### 3. Oscillation Without Breach
```
Open: 50000.00
Ticks: 50300.00 (+0.6%), 49700.00 (-0.6%), 50390.00 (+0.78%)
Result: No bar closes, continues building
```

### 4. First Tick Handling
```
First tick: 50000.00
Action: Initialize bar, set thresholds, continue (no breach check on open)
```

## Integration Points

### Python API
```python
from rangebar import iter_range_bars_from_aggtrades

# Read aggTrades data
trades = load_aggtrades("BTCUSDT_2024-01-01.parquet")

# Generate range bars
bars = list(iter_range_bars_from_aggtrades(trades, pct=0.008))

# Output format matches standard OHLCV
for bar in bars:
    print(f"O:{bar['open']} H:{bar['high']} L:{bar['low']} C:{bar['close']}")
```

### CLI Interface  
```bash
# Fetch data
rangebar fetch --symbol BTCUSDT --start 2024-01-01 --end 2024-01-02

# Generate bars
rangebar build --input data/parquet/BTCUSDT --pct 0.008 --output data/bars
```

## Validation Checklist

- [ ] Thresholds computed from bar open only (never recalculated)
- [ ] Breach tick included in closing bar
- [ ] Next tick after breach opens new bar  
- [ ] No future information used in decisions
- [ ] UM Futures data only (asset_class="um")
- [ ] Data sorted by (timestamp, aggTradeId)
- [ ] Performance targets met
- [ ] Memory usage within limits
- [ ] Edge cases handled correctly