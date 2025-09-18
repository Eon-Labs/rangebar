# Contributing to Rangebar

Thank you for your interest in contributing to the rangebar crate! This document provides guidelines for contributing to this high-performance range bar construction library.

## 🎯 Project Overview

Rangebar is a pure Rust implementation of non-lookahead bias range bar construction for cryptocurrency trading data. The library processes 137M+ trades/second with guaranteed temporal integrity for financial backtesting.

## 🛠️ Development Setup

### Prerequisites

- Rust 1.85+ (specified in `rust-version` field)
- Git for version control

### Local Development

```bash
# Clone the repository
git clone https://github.com/Eon-Labs/rangebar.git
cd rangebar

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code (mandatory)
cargo fmt

# Check for linting issues
cargo clippy --all-targets --all-features -- -D warnings
```

## 📋 Code Quality Standards

### Mandatory Requirements

Before submitting any changes, ensure:

1. **Code formatting**: `cargo fmt --check` passes
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings` passes
3. **Tests**: `cargo test` passes with 100% success rate
4. **Documentation**: All public APIs have rustdoc comments

### Code Style

- Follow standard Rust conventions
- Use `#![deny(missing_docs)]` compliance for public APIs
- Maintain fixed-point arithmetic precision (no f64 in core algorithms)
- Preserve non-lookahead bias principles in all algorithm changes

## 🧪 Testing Guidelines

### Test Categories

1. **Unit Tests**: Located in `src/` modules with `#[cfg(test)]`
2. **Integration Tests**: Located in `tests/` directory
3. **Benchmarks**: Located in `benches/` directory
4. **Examples**: Located in `examples/` directory

### Testing Requirements

- All new features must include comprehensive tests
- Algorithm changes require both unit and integration tests
- Performance-critical code must include benchmarks
- Zero-duration bar handling must be tested (see NOTABUG comments)

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_zero_duration_bars_are_valid

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## 🏗️ Architecture Guidelines

### Core Principles

1. **Non-lookahead Bias**: Thresholds computed from bar open price only
2. **Fixed-point Arithmetic**: No floating-point precision errors
3. **Zero-copy Processing**: Minimize allocations in hot paths
4. **Temporal Integrity**: Maintain strict chronological ordering

### Module Organization

```
src/
├── lib.rs              # Public API and re-exports
├── types.rs            # Core data structures
├── fixed_point.rs      # Fixed-point arithmetic
├── range_bars.rs       # Core algorithm implementation
├── tier1.rs            # Tier-1 symbol functionality
└── bin/                # Binary executables
    ├── rangebar_export.rs
    ├── tier1_symbol_discovery.rs
    └── parallel_tier1_analysis.rs
```

## 📝 Contribution Types

### Bug Fixes

1. Create an issue describing the bug
2. Include minimal reproduction case
3. Submit PR with fix and test
4. Ensure no performance regression

### New Features

1. Discuss feature in an issue first
2. Maintain backward compatibility
3. Include comprehensive documentation
4. Add examples demonstrating usage

### Performance Improvements

1. Include benchmarks proving improvement
2. Verify algorithm correctness unchanged
3. Test on realistic data volumes (1M+ trades)
4. Document performance characteristics

### Algorithm Changes

⚠️ **Special Requirements for Algorithm Changes**:

- Must preserve non-lookahead bias
- Must maintain fixed-point precision
- Requires extensive testing with real market data
- Must include adversarial test cases
- Performance must match or exceed current implementation

## 🔍 Code Review Process

### Pull Request Requirements

1. **Clear Description**: Explain the change and motivation
2. **Test Coverage**: Include tests for all changes
3. **Documentation**: Update relevant documentation
4. **Performance**: Verify no performance regression
5. **Examples**: Update examples if API changes

### Review Criteria

- Algorithm correctness and temporal integrity
- Code quality and Rust best practices
- Test coverage and edge case handling
- Performance characteristics
- Documentation completeness

## 📊 Performance Standards

### Benchmarks

Current performance targets:
- **Range Bar Processing**: 137M+ trades/second
- **Memory Usage**: Minimal allocations in hot paths
- **Latency**: Sub-microsecond per trade processing

### Testing Performance

```bash
# Run performance benchmarks
cargo bench

# Profile with specific datasets
cargo run --release --bin rangebar-export -- BTCUSDT 2024-01-01 2024-01-02 0.008 ./output
```

## 🚨 Security Considerations

- No credentials or API keys in code
- Validate all external inputs
- Use fixed-point arithmetic to prevent precision attacks
- Maintain audit trail for financial calculations

## 📋 Issue Reporting

### Bug Reports

Include:
- Rust version and platform
- Minimal reproduction case
- Expected vs actual behavior
- Performance impact if applicable

### Feature Requests

Include:
- Use case description
- API design proposal
- Performance considerations
- Backward compatibility impact

## 🎉 Recognition

Contributors will be:
- Listed in `Cargo.toml` authors for significant contributions
- Acknowledged in release notes
- Credited in documentation for major features

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and community support
- **Documentation**: https://docs.rs/rangebar

## 📄 License

By contributing to rangebar, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to rangebar! 🚀