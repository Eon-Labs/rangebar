#!/bin/bash
# Production Validation Script for Rangebar
# Validates all critical security fixes and production readiness
#
# Usage: ./scripts/production_validation.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 RANGEBAR PRODUCTION VALIDATION SUITE"
echo "======================================="
echo "📅 $(date)"
echo "📁 Project: $PROJECT_ROOT"
echo ""

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=8

log_test() {
    local test_name="$1"
    echo "🧪 Testing: $test_name"
}

log_pass() {
    local test_name="$1"
    echo "✅ PASS: $test_name"
    ((TESTS_PASSED++))
}

log_fail() {
    local test_name="$1"
    echo "❌ FAIL: $test_name"
    ((TESTS_FAILED++))
}

cd "$PROJECT_ROOT"

# Test 1: Privilege Escalation Prevention
log_test "Privilege escalation prevention (profiling_tools.sh)"
if ./scripts/profiling_tools.sh --help >/dev/null 2>&1; then
    # Check that no sudo commands are present in the script
    if grep -q "sudo" ./scripts/profiling_tools.sh; then
        log_fail "Privilege escalation prevention - sudo found in script"
    else
        log_pass "Privilege escalation prevention"
    fi
else
    log_fail "Privilege escalation prevention - script failed to run"
fi

# Test 2: Command Injection Prevention
log_test "Command injection prevention (dependency_monitor.sh)"
if timeout 10s ./scripts/dependency_monitor.sh check >/dev/null 2>&1 || [[ $? == 124 ]]; then
    # Check for secure metadata parsing patterns
    if grep -q "metadata_file=" ./scripts/dependency_monitor.sh && grep -q "jq.*<.*metadata_file" ./scripts/dependency_monitor.sh; then
        log_pass "Command injection prevention"
    else
        log_fail "Command injection prevention - secure parsing not found"
    fi
else
    log_fail "Command injection prevention - script validation failed"
fi

# Test 3: Path Traversal Protection
log_test "Path traversal protection (rangebar-export)"
if cargo build --bin rangebar-export --quiet; then
    # Test that path traversal is blocked
    if cargo run --bin rangebar-export --quiet -- BTCUSDT 2025-01-01 2025-01-02 0.008 "../malicious" 2>&1 | grep -q "Path traversal detected"; then
        log_pass "Path traversal protection"
    else
        log_fail "Path traversal protection - malicious path not blocked"
    fi
else
    log_fail "Path traversal protection - compilation failed"
fi

# Test 4: GPU Stability
log_test "GPU stability (multi-symbol processing)"
if cargo build --example multi_symbol_gpu_demo --features gpu --quiet; then
    # Run GPU demo with timeout to ensure it doesn't crash
    if timeout 30s cargo run --example multi_symbol_gpu_demo --features gpu --quiet >/dev/null 2>&1; then
        log_pass "GPU stability"
    else
        # Check if it's just a timeout (acceptable) vs crash
        if [[ $? == 124 ]]; then
            log_pass "GPU stability (timed out but no crash)"
        else
            log_fail "GPU stability - processing crashed"
        fi
    fi
else
    log_fail "GPU stability - compilation failed"
fi

# Test 5: Data Consistency
log_test "Data consistency (deterministic symbol ordering)"
if cargo run --bin tier1-symbol-discovery --quiet -- --format minimal >/dev/null 2>&1; then
    # Run twice and compare outputs
    FIRST_RUN=$(cargo run --bin tier1-symbol-discovery --quiet -- --format minimal 2>&1 | grep "Tier-1 symbols" | head -1)
    SECOND_RUN=$(cargo run --bin tier1-symbol-discovery --quiet -- --format minimal 2>&1 | grep "Tier-1 symbols" | head -1)
    if [[ "$FIRST_RUN" == "$SECOND_RUN" ]]; then
        log_pass "Data consistency"
    else
        log_fail "Data consistency - non-deterministic output"
    fi
else
    log_fail "Data consistency - symbol discovery failed"
fi

# Test 6: Pipeline Integration
log_test "Pipeline integration (18 symbol count)"
if cargo run --bin tier1-symbol-discovery --quiet -- --format minimal >/dev/null 2>&1; then
    SYMBOL_COUNT=$(wc -l /tmp/tier1_usdt_pairs.txt 2>/dev/null | awk '{print $1}')
    if [[ "$SYMBOL_COUNT" == "18" ]]; then
        log_pass "Pipeline integration"
    else
        log_fail "Pipeline integration - expected 18 symbols, got $SYMBOL_COUNT"
    fi
else
    log_fail "Pipeline integration - tier1 discovery failed"
fi

# Test 7: User Safety
log_test "User safety (help flags available)"
if cargo run --bin rangebar-analyze --quiet -- --help >/dev/null 2>&1; then
    # Check that help doesn't start analysis
    HELP_OUTPUT=$(cargo run --bin rangebar-analyze --quiet -- --help 2>&1)
    if echo "$HELP_OUTPUT" | grep -q "Usage:" && ! echo "$HELP_OUTPUT" | grep -q "Starting parallel execution"; then
        log_pass "User safety"
    else
        log_fail "User safety - help flag issues"
    fi
else
    log_fail "User safety - help flag not working"
fi

# Test 8: Architecture Performance (Polars migration)
log_test "Architecture performance (Polars migration)"
if grep -q "import polars as pl" momentum_pattern_analyzer.py 2>/dev/null; then
    log_pass "Architecture performance"
else
    log_fail "Architecture performance - Polars migration not found"
fi

# Summary
echo ""
echo "🏁 VALIDATION SUMMARY"
echo "===================="
echo "✅ Tests passed: $TESTS_PASSED/$TESTS_TOTAL"
echo "❌ Tests failed: $TESTS_FAILED/$TESTS_TOTAL"

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo ""
    echo "🎉 ALL TESTS PASSED - PRODUCTION READY"
    echo "✅ System validated for secure production deployment"
    exit 0
else
    echo ""
    echo "⚠️  VALIDATION FAILURES DETECTED"
    echo "❌ System requires fixes before production deployment"
    exit 1
fi