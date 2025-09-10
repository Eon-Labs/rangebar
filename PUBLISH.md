# PyPI Publication Guide

This document provides step-by-step instructions for publishing the RangeBar package to PyPI.

## Prerequisites

1. **PyPI Account**: Create accounts on both [PyPI](https://pypi.org/) and [TestPyPI](https://test.pypi.org/)
2. **API Tokens**: Generate API tokens for both PyPI and TestPyPI
3. **Development Environment**: 
   - Python 3.13+ (2025 standard)
   - Rust 1.89+ (2024 edition)
   - UV package manager
4. **Tools**: Ensure `uv`, `maturin>=1.9`, and `twine` are available

## Setup API Tokens

```bash
# Configure PyPI token
uv run python -m pip config set global.keyring-provider import

# Or set environment variables
export TWINE_USERNAME=__token__
export TWINE_PASSWORD=pypi-your-api-token-here
```

## Build Process

### 1. Clean Build Environment

```bash
# Clean previous builds
rm -rf dist/ build/ target/wheels/ .eggs/
```

### 2. Run Tests

```bash
# Run Rust tests
cargo test --release

# Run Python validation
uv run python validate_algorithm_parity.py
```

### 3. Build Package

```bash
# Build with maturin (builds both source and wheel)
uv run maturin build --release

# Or use the build script
uv run python build_package.py
```

### 4. Verify Build

```bash
# Check wheel contents
unzip -l target/wheels/rangebar-*.whl

# Test local installation
uv pip install target/wheels/rangebar-*.whl
uv run python -c "from rangebar import range_bars; print('Import successful')"
```

## Publication

### Test Publication (Recommended First)

```bash
# Upload to TestPyPI first
uv run maturin publish --repository testpypi

# Test installation from TestPyPI
uv pip install --index-url https://test.pypi.org/simple/ rangebar

# Test functionality
uv run python -c "
from rangebar.range_bars import iter_range_bars_from_aggtrades
print('TestPyPI package works!')
"
```

### Production Publication

```bash
# Upload to PyPI
uv run maturin publish

# Verify on PyPI
uv pip install rangebar
```

### Alternative: Manual Upload with Twine

```bash
# Build distributions
uv run maturin build --release

# Upload with twine
uv run twine upload target/wheels/*
```

## Post-Publication

### 1. Verify Installation

```bash
# Test installation in fresh environment
uv pip install rangebar

# Test CLI
rangebar --help

# Test Python API
uv run python -c "
import asyncio
from rangebar.data_fetcher import fetch_um_futures_aggtrades
print('Package ready for public use!')
"
```

### 2. Create GitHub Release

1. Tag the release: `git tag v0.2.0`
2. Push tag: `git push origin v0.2.0`  
3. Create GitHub release with changelog

### 3. Documentation

Update documentation with:
- Installation instructions
- Usage examples
- API reference
- Performance benchmarks

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure Rust and maturin are properly installed
2. **Import Errors**: Verify all dependencies are correctly specified
3. **Upload Errors**: Check API token permissions and network connectivity

### Debug Commands

```bash
# Check package metadata
uv run python -m pip show rangebar

# Inspect wheel contents
wheel unpack target/wheels/rangebar-*.whl

# Validate package
uv run twine check target/wheels/*
```

## Version Updates

For future releases:

1. Update version in `pyproject.toml` and `Cargo.toml`
2. Update `CHANGELOG.md` with changes
3. Run full test suite
4. Follow publication process above

## Support

- Package issues: [GitHub Issues](https://github.com/Eon-Labs/rangebar/issues)
- PyPI project page: https://pypi.org/project/rangebar/
- TestPyPI page: https://test.pypi.org/project/rangebar/