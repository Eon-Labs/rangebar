# Migration Guide

This guide helps you upgrade between RangeBar versions.

## v0.3.x → v0.4.0: Format Alignment

### 🚨 Breaking Changes

#### Field Names Changed (Breaking)
Output field names changed from **plural** to **singular** for consistency:

| v0.3.x (Old) | v0.4.0 (New) | 
|-------------|-------------|
| `opens` → | `open` |
| `highs` → | `high` |
| `lows` → | `low` |
| `closes` → | `close` |
| `volumes` → | `volume` |
| `turnovers` → | `turnover` |
| `trade_counts` → | `trade_count` |
| `first_ids` → | `first_id` |
| `last_ids` → | `last_id` |

**Timestamps remain unchanged**: `open_time`, `close_time`

#### Migration Steps

**1. Update field access in your code:**

```python
# ❌ v0.3.x (Old)
rust_result = rust.compute_range_bars(...)
first_price = rust_result['opens'][0]
first_volume = rust_result['volumes'][0]

# ✅ v0.4.0 (New) 
rust_result = rust.compute_range_bars(...)
first_price = rust_result['open'][0]
first_volume = rust_result['volume'][0]
```

**2. Update pandas DataFrame creation:**

```python
# ❌ v0.3.x (Old)
df = pd.DataFrame({
    'open': rust_result['opens'],
    'volume': rust_result['volumes']
})

# ✅ v0.4.0 (New) - Direct compatibility!
df = pd.DataFrame({k: v for k, v in rust_result.items() if not k.startswith('_')})
```

### 🎯 New Features

#### Zero Conversion Overhead
- **Direct pandas integration**: No manual conversion needed
- **Excel/CSV ready**: Immediate export compatibility
- **Schema validation**: Built-in format checking
- **Metadata included**: Automatic schema information

#### Format Alignment Benefits
```python
import pandas as pd
from rangebar import _rangebar_rust as rust, convert

# Get aligned output with metadata
result = rust.compute_range_bars(...)

# Direct pandas usage (no conversion!)
df = pd.DataFrame({k: v for k, v in result.items() if not k.startswith('_')})
df.to_csv('rangebar_output.csv')  # Works immediately

# Schema validation
print(f"Valid: {rust.validate_output_format(result)}")
print(f"Schema: {rust.get_schema_info()['schema_version']}")

# Seamless format conversion
json_data = convert.rust_to_json(result)
arrow_data = convert.json_to_arrow(json_data)
```

#### Conversion Utilities
New `rangebar.convert` module provides seamless format conversion:

```python
from rangebar import convert

# JSON ↔ Arrow conversion
arrow_data = convert.json_to_arrow(python_bars)
json_data = convert.arrow_to_json(arrow_data)

# Validation
is_valid = convert.validate_rangebar_data(data, 'auto')

# Convenience functions
json_from_rust = convert.rust_to_json(rust_output)
arrow_compatible = convert.json_to_rust_compatible(json_data)
```

### 🔄 Upgrade Instructions

```bash
# Upgrade to v0.4.0
uv add "rangebar>=0.4.0"  # or pip install --upgrade "rangebar>=0.4.0"
```

**Test your upgrade:**
```python
import rangebar
print(f"Version: {rangebar.__version__}")  # Should be 0.4.0+

# Test new format alignment
from rangebar import _rangebar_rust as rust
import numpy as np

prices = np.array([5000000000000, 5040000000000], dtype=np.int64)  
volumes = np.array([100000000, 100000000], dtype=np.int64)
timestamps = np.array([1000, 2000], dtype=np.int64)
trade_ids = np.array([1, 2], dtype=np.int64)
first_ids = np.array([1, 2], dtype=np.int64) 
last_ids = np.array([1, 2], dtype=np.int64)

result = rust.compute_range_bars(
    prices=prices, volumes=volumes, timestamps=timestamps,
    trade_ids=trade_ids, first_ids=first_ids, last_ids=last_ids,
    threshold_bps=8000
)

# Verify singular field names
print("Field names:", [k for k in result.keys() if not k.startswith('_')])
print(f"First open: {result['open'][0]}")  # Note: 'open' not 'opens'
print("✅ v0.4.0 format alignment working!")
```

---

## v0.1.x → v0.2.0: Dependency Updates

This guide helps you upgrade from RangeBar v0.1.x to v0.2.0 with the latest 2025 dependencies.

## 🚨 Breaking Changes

### Python Version Requirement
- **v0.1.x**: Python 3.12+
- **v0.2.0**: Python 3.13+ ⚠️

**Action Required**: Upgrade your Python environment to 3.13+

```bash
# Check your Python version
python --version

# If < 3.13, upgrade your Python installation
# Using UV (recommended):
uv python install 3.13
uv python pin 3.13
```

### Dependency Versions
Major dependency updates that may affect your environment:

| Package | v0.1.x | v0.2.0 | Impact |
|---------|--------|--------|---------|
| numpy | >=1.24.0 | >=2.3.0 | Major version bump |
| pandas | >=2.0.0 | >=2.3.0 | Minor updates |
| pyarrow | >=12.0.0 | >=21.0.0 | Major version bump |
| httpx | >=0.24.0 | >=0.28.0 | Minor updates |

## 📈 Performance Improvements

### Massive Speed Increase
- **v0.1.x**: ~2.5M trades/second
- **v0.2.0**: **137M+ trades/second** (54x faster!)

Your existing code will automatically benefit from these improvements with no changes required.

## 🔄 Upgrade Steps

### 1. Update Python Environment

```bash
# Check current version
python --version

# If using UV (recommended)
uv python install 3.13
cd your-project/
uv python pin 3.13
```

### 2. Update RangeBar

```bash
# Using UV
uv add "rangebar>=0.2.0"

# Using pip
pip install --upgrade "rangebar>=0.2.0"
```

### 3. Verify Installation

```python
import rangebar
import numpy as np

print(f"RangeBar: {rangebar.__version__}")
print(f"NumPy: {np.__version__}")  # Should be 2.3+

# Test basic functionality
from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
from decimal import Decimal

# Your existing code should work unchanged
trades_data = [
    {'a': 1, 'p': '50000.0', 'q': '1.0', 'f': 1, 'l': 1, 'T': 1000, 'm': False},
    {'a': 2, 'p': '50400.0', 'q': '1.0', 'f': 2, 'l': 2, 'T': 2000, 'm': False},
]

trades = [AggTrade(data) for data in trades_data]
bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))

print(f"✅ Generated {len(bars)} bars - upgrade successful!")
```

## 🔧 Code Compatibility

### ✅ No Code Changes Required

All existing RangeBar v0.1.x code is **100% compatible** with v0.2.0:

- **Python API**: No changes to function signatures or behavior
- **CLI Commands**: All commands work identically
- **Algorithm**: Same non-lookahead bias algorithm, just much faster
- **Data Formats**: Same input/output formats

### Example: Your Existing Code Still Works

```python
# This v0.1.x code works unchanged in v0.2.0
import asyncio
from rangebar.data_fetcher import fetch_um_futures_aggtrades
from rangebar.range_bars import iter_range_bars_from_aggtrades
from decimal import Decimal

async def process_data():
    # Fetch data (same API)
    trades = await fetch_um_futures_aggtrades('BTCUSDT', '2024-01-01', '2024-01-01')
    
    # Generate bars (same API, 54x faster!)
    bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
    
    print(f"Processed {len(trades)} trades → {len(bars)} bars")

asyncio.run(process_data())
```

## 🚀 Performance Benefits

### Before vs After
```python
import time
from rangebar.range_bars import iter_range_bars_from_aggtrades

# Your existing code gets automatic performance boost
start = time.time()
bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
duration = time.time() - start

# v0.1.x: ~0.4 seconds for 1M trades
# v0.2.0: ~0.007 seconds for 1M trades (54x faster!)
```

## 🐛 Troubleshooting

### Common Issues

#### 1. Python Version Error
```
error: externally-managed-environment
```

**Solution**: Upgrade to Python 3.13+
```bash
uv python install 3.13
uv sync
```

#### 2. NumPy Compatibility
```
ImportError: numpy compatibility issue
```

**Solution**: Clear environment and reinstall
```bash
uv sync --reinstall
```

#### 3. Dependency Conflicts
```
No solution found when resolving dependencies
```

**Solution**: Use exact version specification
```bash
uv add "rangebar==0.2.0"
```

## 📊 Validation

### Benchmark Your Upgrade
Run this script to verify performance improvements:

```python
import time
import numpy as np
from rangebar.range_bars import iter_range_bars_from_aggtrades, AggTrade
from decimal import Decimal

# Generate test data
trades_data = [
    {'a': i, 'p': f'{50000 + i*0.1}', 'q': '1.0', 'f': i, 'l': i, 'T': 1000+i, 'm': False}
    for i in range(10000)
]
trades = [AggTrade(data) for data in trades_data]

# Benchmark
start = time.perf_counter()
bars = list(iter_range_bars_from_aggtrades(trades, pct=Decimal('0.008')))
duration = time.perf_counter() - start

trades_per_sec = len(trades) / duration
print(f"✅ Performance: {trades_per_sec:,.0f} trades/sec")
print(f"✅ Expected v0.2.0: 2M+ trades/sec (Python), 100M+ trades/sec (Rust)")
```

## 🎯 Summary

**Upgrading to v0.2.0:**
- ✅ **Massive performance boost** (54x faster)
- ✅ **No code changes** required
- ✅ **Latest 2025 dependencies**
- ⚠️ **Python 3.13+ required**
- ⚠️ **Environment update needed**

The upgrade is straightforward but requires updating your Python environment to 3.13+. Once upgraded, you'll get dramatic performance improvements with zero code changes.

## 🆘 Support

If you encounter issues during migration:

1. Check your Python version: `python --version`
2. Verify dependencies: `uv tree` or `pip list`
3. Create a fresh environment: `uv sync --reinstall`
4. Report issues: [GitHub Issues](https://github.com/eonlabs/rangebar/issues)