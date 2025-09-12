# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Non-lookahead bias range bar construction from Binance UM Futures aggTrades data.

**Core Algorithm**: Range bars close when price moves ±0.8% from the bar's OPEN price (not from high/low range).

**Architecture**: Rust core for performance (processes 1B+ ticks) with Python wrapper for data orchestration.

## Key Commands

### Development
```bash
# Build Rust core
maturin develop --release

# Run all tests
pytest tests/ -v

# Run Rust tests
cargo test

# Type checking
mypy src/rangebar/

# Linting
ruff check src/
```

### Data Operations
```bash
# Fetch UM futures aggTrades data
python -m rangebar.data_fetcher --symbol BTCUSDT --start 2024-01-01 --end 2024-01-02

# Convert raw data to Parquet
python -m rangebar.data_processor --input data/um_futures --output data/parquet

# Generate range bars
python -m rangebar.cli build --input data/parquet/BTCUSDT --pct 0.008 --output data/bars
```

## Architecture

### Data Pipeline
1. **Data Fetching**: `binance_historical_data` → Raw CSV/ZIP files
2. **Preprocessing**: CSV → Parquet with schema validation  
3. **Computation**: Rust core processes Parquet → Range bars
4. **Output**: Structured bar data (OHLCV format)

### Performance Tiers
- **Python + Decimal**: Reference accuracy (slow)
- **Rust + Fixed-point**: Production speed (100-1000x faster)

## Critical Algorithm Invariants

### Non-Lookahead Guarantee
```python
# CORRECT: Thresholds computed from bar's OPEN only
upper_breach = bar_open * 1.008
lower_breach = bar_open * 0.992

# Thresholds remain FIXED for entire bar lifetime
# Current tick price compared against these fixed thresholds
```

### Bar Construction Sequence
1. Bar opens at tick price
2. Compute fixed thresholds from open: `±0.8%`
3. For each subsequent tick:
   - Update `high` = max(high, tick_price)
   - Update `low` = min(low, tick_price) 
   - Update `volume` += tick_volume
   - Check: `tick_price >= upper_breach OR tick_price <= lower_breach`
4. If breach: Include breach tick in bar, close bar, next tick opens new bar

### Critical Validation Logic
**Breach Consistency Rule**: If `high >= upper_threshold` OR `low <= lower_threshold`, then `close` must also breach the same threshold.

**Rationale**: The breaching trade that triggers bar closure becomes the close price. If high/low breach but close doesn't, it indicates the breaching trade was not properly included in the closing bar.

**Validation**: `(high_breach → close_breach) AND (low_breach → close_breach)`

### Data Source Requirements
- **Source**: https://github.com/stas-prokopiev/binance_historical_data
- **Asset Class**: `"um"` (USD-M Futures) **ONLY**
- **Data Type**: `"aggTrades"` **ONLY**
- **Not Used**: Spot markets, CM Futures, other data types

## Project Structure

```
rangebar/
├── CLAUDE.md                    # This file
├── Cargo.toml                   # Rust configuration  
├── pyproject.toml               # Python/maturin config
├── docs/
│   ├── planning/               # YAML planning docs
│   └── architecture/           # Technical specifications
├── src/
│   ├── lib.rs                  # Rust entry point
│   ├── range_bars.rs           # Core algorithm (Rust)
│   └── rangebar/               # Python package
├── tests/                      # Test suites
└── data/                       # Data storage
```

## Common Issues

### Build Issues
- Ensure Rust toolchain installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install maturin: `pip install maturin`
- If PyO3 errors: Check Python version compatibility (3.8+)

### Data Issues  
- Verify UM futures (not spot) data: Check `asset_class="um"` parameter
- Sort aggTrades by `(timestamp, aggTradeId)` for deterministic processing
- Validate schema: Required fields `[a, p, q, f, l, T, m]`

### Algorithm Issues
- Ensure thresholds computed from bar OPEN, not evolving high/low
- Verify breach tick included in closing bar (not excluded)
- Check defer_open mechanism: next tick after breach opens new bar

## Testing

### Critical Test Categories
1. **Non-lookahead validation**: Thresholds from prior state only
2. **Edge cases**: Exact threshold hits, large gaps, first tick
3. **Performance**: 1M ticks < 100ms, 1B ticks < 30 seconds
4. **Data integrity**: UM futures schema compliance

### Running Tests
```bash
# Full test suite
pytest tests/ -v --cov=rangebar

# Non-lookahead specific tests
pytest tests/test_non_lookahead.py -v

# Performance benchmarks
pytest benchmarks/ -v
```