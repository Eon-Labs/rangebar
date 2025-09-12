#!/bin/bash
# Automated performance monitoring for rangebar algorithm
#
# Usage: ./scripts/benchmark_runner.sh [baseline|compare|continuous]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCHMARK_DATA_DIR="$PROJECT_ROOT/benchmark_data"
BASELINE_FILE="$BENCHMARK_DATA_DIR/baseline.json"

# Ensure benchmark data directory exists
mkdir -p "$BENCHMARK_DATA_DIR"

# Performance targets from user requirements
TARGET_1M_TICKS_MS=100      # 1M ticks should complete in < 100ms
TARGET_1B_TICKS_SEC=30      # 1B ticks should complete in < 30 seconds

log_info() {
    echo "🔍 [BENCH] $1"
}

log_success() {
    echo "✅ [BENCH] $1"  
}

log_warn() {
    echo "⚠️ [BENCH] $1"
}

log_error() {
    echo "❌ [BENCH] $1"
}

run_benchmark() {
    local output_file="$1"
    log_info "Running performance benchmarks..."
    
    cd "$PROJECT_ROOT"
    cargo bench --bench rangebar_bench -- --output-format json | tee "$output_file"
}

create_baseline() {
    log_info "Creating performance baseline..."
    run_benchmark "$BASELINE_FILE"
    log_success "Baseline created at $BASELINE_FILE"
}

compare_with_baseline() {
    if [[ ! -f "$BASELINE_FILE" ]]; then
        log_error "No baseline found. Run './scripts/benchmark_runner.sh baseline' first."
        exit 1
    fi
    
    local current_file="$BENCHMARK_DATA_DIR/current_$(date +%Y%m%d_%H%M%S).json"
    
    log_info "Running benchmarks and comparing with baseline..."
    run_benchmark "$current_file"
    
    # Simple performance regression detection
    log_info "Performance comparison:"
    echo "📊 Baseline: $BASELINE_FILE"
    echo "📊 Current:  $current_file"
    
    # TODO: Add more sophisticated comparison logic
    log_warn "Detailed comparison analysis requires additional tooling"
}

validate_performance_targets() {
    log_info "Validating performance targets..."
    
    # Run quick performance check
    cd "$PROJECT_ROOT"
    local bench_output=$(cargo bench --bench rangebar_bench -- --quick 2>&1 | grep -E "(process_trades/1000000|ns/iter)")
    
    if [[ -n "$bench_output" ]]; then
        log_info "Performance validation results:"
        echo "$bench_output"
        
        # Basic validation (simplified)
        log_warn "Automated target validation needs implementation"
        log_info "Target: 1M ticks < 100ms (100,000,000 ns)"
        log_info "Target: 1B ticks < 30s (30,000,000,000 ns)"
    else
        log_error "Could not extract performance metrics"
    fi
}

continuous_monitoring() {
    log_info "Starting continuous performance monitoring..."
    
    while true; do
        log_info "Running performance check at $(date)"
        validate_performance_targets
        
        # Sleep for 1 hour
        log_info "Next check in 1 hour..."
        sleep 3600
    done
}

show_help() {
    echo "Rangebar Performance Monitoring"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  baseline    Create performance baseline for future comparisons"
    echo "  compare     Compare current performance with baseline"
    echo "  validate    Validate performance against targets (1M ticks < 100ms)"
    echo "  continuous  Run continuous performance monitoring"
    echo "  help        Show this help message"
    echo ""
    echo "Performance Targets:"
    echo "  • 1M ticks processing: < 100ms"
    echo "  • 1B ticks processing: < 30 seconds"
    echo ""
    echo "Examples:"
    echo "  $0 baseline              # Create initial baseline"
    echo "  $0 validate              # Quick performance check"
    echo "  $0 compare               # Compare with baseline"
}

# Main command handling
case "${1:-help}" in
    "baseline")
        create_baseline
        ;;
    "compare")
        compare_with_baseline
        ;;
    "validate")
        validate_performance_targets
        ;;
    "continuous")
        continuous_monitoring
        ;;
    "help"|*)
        show_help
        ;;
esac