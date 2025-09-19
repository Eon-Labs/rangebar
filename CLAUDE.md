# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Non-lookahead bias range bar construction from Binance UM Futures aggTrades data.

**Core Algorithm**: Range bars close when price moves ±0.8% from the bar's OPEN price (not from high/low range).

**Architecture**: Pure Rust implementation for performance and reliability (processes 1B+ ticks). All components native Rust: symbol discovery, data processing, and analysis.

## Key Commands

### Development
```bash
# Build all Rust binaries
cargo build --release

# Run all tests
cargo test

# Lint Rust code
cargo clippy

# Format Rust code
cargo fmt
```

### Data Operations
```bash
# Discover Tier-1 symbols across all Binance futures markets
cargo run --bin tier1-symbol-discovery -- --format comprehensive

# Run parallel Tier-1 analysis (requires configuration)
cargo run --bin rangebar-analyze

# Export range bars for specific symbol
cargo run --bin rangebar-export

# Generate range bars from aggTrades
./target/release/rangebar BTCUSDT 2024-01-01 2024-01-02 0.008 ./output

# Validate data structure across markets and symbols
cargo run --bin data-structure-validator --features data-integrity -- --symbols BTCUSDT,ETHUSDT --start-date 2024-01-01 --end-date 2024-07-01
```

## Data Structure Validation

### Validation Tool Overview
**Purpose**: Systematic validation of Binance aggTrades data structures across Tier-1 cryptocurrency symbols for both spot and UM futures markets.

**Implementation**: `src/bin/data_structure_validator.rs`
**Configuration**: `docs/planning/data-structure-validation-plan.yml`

### Key Findings (2024 Data)

#### Market-Specific Structure Differences
**Spot Market (data.binance.vision/data/spot/)**:
- **Headers**: None
- **Columns**: Short (`a,p,q,f,l,T,m`)
- **Boolean**: `False/True` (capitalized)
- **Timestamp**: 13-digit milliseconds

**UM Futures Market (data.binance.vision/data/futures/um/)**:
- **Headers**: Present
- **Columns**: Descriptive (`agg_trade_id,price,quantity,first_trade_id,last_trade_id,transact_time,is_buyer_maker`)
- **Boolean**: `false/true` (lowercase)
- **Timestamp**: 13-digit milliseconds

**Parser**: Uses `serde` aliases and `flexible_bool()` deserializer to handle both formats automatically.

#### CSV Parsing Implications
- **Auto-Detection**: `detect_csv_headers()` function handles both formats automatically
- **Column Mapping**: Different naming conventions require market-aware parsing
- **Data Integrity**: SHA256 checksum validation available with `--features data-integrity`

### Validation Output Structure
```
output/data_structure_validation/{timestamp}_validation_run/
├── index.json                 # Validation manifest with key findings
├── validation_results.json    # Detailed per-sample results
└── structure_analysis/
    └── {SYMBOL}_structure_profile.json  # Per-symbol structure profiles
```

## Architecture

### Data Pipeline
1. **Symbol Discovery**: `tier1-symbol-discovery` → Multi-market symbol analysis
2. **Data Structure Validation**: `data-structure-validator` → Cross-market format verification
3. **Data Fetching**: `binance_historical_data` → Raw CSV/ZIP files with validated schemas
4. **Preprocessing**: CSV → Parquet with schema validation
5. **Computation**: Pure Rust processes Parquet → Range bars
6. **Analysis**: `rangebar-analyze` → Parallel Tier-1 analysis
7. **Output**: Structured bar data (OHLCV format)

### Performance Architecture
- **Pure Rust Implementation**: Production speed (100-1000x faster than reference)
- **Multi-threaded Processing**: Rayon for parallel analysis across symbols
- **Memory Efficiency**: Fixed-point arithmetic, optimized data structures

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
- **Primary Asset Class**: `"um"` (USD-M Futures) for USDT/USDC perpetuals
- **Tier-1 Analysis**: `"cm"` (Coin-M Futures) for cross-market analysis
- **Data Type**: `"aggTrades"` **ONLY**
- **Not Used**: Spot markets, other data types

### Tier-1 Instruments Definition
**Tier-1 instruments** are crypto assets that Binance respects highly enough to list across **ALL THREE** futures markets:
1. **UM Futures (USDT-margined)**: e.g., BTCUSDT, ETHUSDT
2. **UM Futures (USDC-margined)**: e.g., BTCUSDC, ETHUSDC
3. **CM Futures (Coin-margined)**: e.g., BTCUSD_PERP, ETHUSD_PERP

**Current Count**: 18 Tier-1 instruments (BTC, ETH, SOL, ADA, AVAX, etc.)
**Key Characteristic**: Multi-market availability indicates Binance's highest confidence
**Use Cases**: Cross-market extrapolative reliability analysis, settlement currency arbitrage

## Project Structure

```
rangebar/
├── CLAUDE.md                    # This file
├── Cargo.toml                   # Rust configuration
├── src/
│   ├── lib.rs                  # Rust library entry point
│   ├── range_bars.rs           # Core algorithm (Rust)
│   └── bin/                    # Rust binaries
│       ├── tier1_symbol_discovery.rs    # Symbol discovery
│       ├── parallel_tier1_analysis.rs   # Parallel analysis
│       └── rangebar_export.rs           # Range bar export
├── tests/                      # Test suites
├── output/                     # Generated analysis results
│   └── symbol_analysis/        # Multi-market symbol databases
└── scripts/                    # Legacy Python scripts (deprecated)
```

## Common Issues

### Build Issues
- Ensure Rust toolchain installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Update Rust to latest: `rustup update`
- Clean build if needed: `cargo clean && cargo build --release`

### Data Issues  
- Verify UM futures (not spot) data: Check `asset_class="um"` parameter
- Sort aggTrades by `(timestamp, aggTradeId)` for deterministic processing
- Validate schema: Required fields `[a, p, q, f, l, T, m]`

### Algorithm Issues
- Ensure thresholds computed from bar OPEN, not evolving high/low
- Verify breach tick included in closing bar (not excluded)
- Check defer_open mechanism: next tick after breach opens new bar

## Rust Binaries

### Symbol Discovery (`tier1-symbol-discovery`)
Discovers cryptocurrency symbols available across all three Binance futures markets:
- **UM Futures USDT-margined**: BTCUSDT, ETHUSDT, etc.
- **UM Futures USDC-margined**: BTCUSDC, ETHUSDC, etc.
- **CM Futures coin-margined**: BTCUSD_PERP, ETHUSD_PERP, etc.

```bash
# Generate comprehensive JSON database
cargo run --bin tier1-symbol-discovery -- --format comprehensive

# Generate minimal output for pipeline integration
cargo run --bin tier1-symbol-discovery -- --format minimal

# Include single-market symbols for analysis
cargo run --bin tier1-symbol-discovery -- --include-single-market
```

### Parallel Analysis (`rangebar-analyze`)
Executes parallel range bar analysis across all Tier-1 symbols using Rayon.

```bash
# Requires configuration: /tmp/range_bar_analysis_config.json
# Consumes symbols from: /tmp/tier1_usdt_pairs.txt (generated by tier1-symbol-discovery)
cargo run --bin rangebar-analyze
```

### Range Bar Export (`rangebar-export`)
Exports range bar data for visualization and analysis.

```bash
cargo run --bin rangebar-export -- --help
```

## Testing

### Critical Test Categories
1. **Non-lookahead validation**: Thresholds from prior state only
2. **Edge cases**: Exact threshold hits, large gaps, first tick
3. **Performance**: 1M ticks < 100ms, 1B ticks < 30 seconds
4. **Data integrity**: UM futures schema compliance

### Running Tests
```bash
# Full Rust test suite
cargo test

# Run tests with output
cargo test -- --nocapture

# Performance benchmarks
cargo bench

# Integration tests
cargo test --test integration
```

## Publishing

### Automated Publishing (2025 Best Practices)
The rangebar crate uses **GitHub Actions** with **Trusted Publishing (OIDC)** for secure, automated releases:

```bash
# Create and push release tag (triggers automation)
git tag -a v0.4.2 -m "Release v0.4.2: Your changes here"
git push origin v0.4.2
```

**Automated Pipeline:**
1. ✅ Cross-platform testing (Ubuntu + macOS)
2. ✅ Version verification (tag matches Cargo.toml)
3. ✅ Crates.io publishing via OIDC (30-min tokens)
4. ✅ GitHub release with auto-generated changelog
5. ✅ Artifact uploads (binaries)

### Crates.io Configuration
- **Version**: 0.5.0 (current release)
- **License**: MIT
- **Features**: statistics, data-integrity, arrow-support, python bindings
- **Performance**: 137M+ trades/sec range bar construction

### Security & Authentication

**Primary: Trusted Publishing (OIDC)**
- No stored secrets required
- 30-minute auto-expiring tokens
- GitHub repository verification
- Setup: crates.io → Settings → Trusted Publishers

**Fallback: API Token (Keychain)**
- **Service**: `crates.io-token`
- **Account**: `terryli`
- **Retrieval**: `security find-generic-password -a "terryli" -s "crates.io-token" -w`

### Manual Publishing (Development)
```bash
# Local publishing with keychain token
CARGO_REGISTRY_TOKEN=$(security find-generic-password -a "terryli" -s "crates.io-token" -w) cargo publish --all-features

# Dry run validation
cargo publish --dry-run --all-features
```

### GitHub Actions Workflows
- **`.github/workflows/publish.yml`**: Release automation
- **`.github/workflows/ci.yml`**: Continuous integration
