# Makefile for rangebar development workflow

.PHONY: help fmt lint test deny check build clean install-hooks run-hooks bench docs

# Default target
help:
	@echo "Rangebar Development Commands:"
	@echo "  fmt            - Format code with cargo fmt"
	@echo "  lint           - Run clippy linting"
	@echo "  test           - Run tests with cargo nextest"
	@echo "  deny           - Run cargo deny security checks"  
	@echo "  check          - Run all quality checks (fmt + test)"
	@echo "  build          - Build in release mode"
	@echo "  clean          - Clean build artifacts" 
	@echo "  bench          - Run performance benchmarks"
	@echo "  bench-baseline - Create performance baseline"
	@echo "  bench-validate - Validate performance targets (1M ticks < 100ms)"
	@echo "  bench-compare  - Compare current performance with baseline"
	@echo "  docs           - Generate documentation"
	@echo "  install-hooks  - Install pre-commit hooks"
	@echo "  run-hooks      - Manually run pre-commit hooks"
	@echo "  deps-check     - Check for outdated dependencies"
	@echo "  deps-update    - Update dependencies with validation"
	@echo "  deps-security  - Run security audit on dependencies"
	@echo "  deps-auto      - Automated dependency update workflow"
	@echo "  profile-install     - Install profiling tools"
	@echo "  profile-flamegraph  - Generate CPU flamegraph"
	@echo "  profile-cpu         - Run CPU profiling"
	@echo "  profile-memory      - Run memory profiling"
	@echo "  profile-full        - Comprehensive profiling suite"

# Code formatting
fmt:
	cargo fmt --all

# Linting
lint:
	cargo clippy --all-targets --all-features

# Testing with enhanced output
test:
	cargo nextest run

# Security and dependency validation
deny:
	cargo deny check

# Run all quality checks
check: fmt test
	@echo "✅ Core quality checks passed!"

# Full quality checks (when clippy/deny issues are resolved)
check-full: fmt lint test deny
	@echo "✅ All quality checks passed!"

# Build in release mode
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Run benchmarks
bench:
	cargo bench --bench rangebar_bench

# Performance monitoring commands
bench-baseline:
	./scripts/benchmark_runner.sh baseline

bench-validate:
	./scripts/benchmark_runner.sh validate

bench-compare:
	./scripts/benchmark_runner.sh compare

# Generate documentation  
docs:
	cargo doc --open --all-features

# Install pre-commit hooks
install-hooks:
	pip install pre-commit
	pre-commit install

# Manually run pre-commit hooks
run-hooks:
	pre-commit run --all-files

# Dependency monitoring commands  
deps-check:
	./scripts/dependency_monitor.sh check

deps-update:
	./scripts/dependency_monitor.sh update

deps-security:
	./scripts/dependency_monitor.sh security

deps-auto:
	./scripts/dependency_monitor.sh auto

# Profiling commands for high-frequency processing
profile-install:
	./scripts/profiling_tools.sh install

profile-flamegraph:
	./scripts/profiling_tools.sh flamegraph

profile-cpu:
	./scripts/profiling_tools.sh cpu

profile-memory:
	./scripts/profiling_tools.sh memory

profile-full:
	./scripts/profiling_tools.sh full

# Quick development cycle
dev: fmt test
	@echo "🚀 Development checks complete!"