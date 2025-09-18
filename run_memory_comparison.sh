#!/bin/bash

# Cross-year memory comparison test runner
# Compares batch and streaming v2 (bounded memory) architectures

echo "🚀 Starting Cross-Year Memory Comparison Test"
echo "📅 $(date)"
echo "💻 Platform: $(uname -a)"
echo "🦀 Rust version: $(rustc --version)"
echo ""

# Set high memory limits to test unbounded scenarios
ulimit -v 16777216  # 16GB virtual memory limit
ulimit -m 8388608   # 8GB physical memory limit

echo "💾 Memory limits set:"
echo "  Virtual: $(ulimit -v) KB"
echo "  Physical: $(ulimit -m) KB"
echo ""

# Build with streaming-v2 feature
echo "🔧 Building with streaming-v2 feature..."
cargo build --release --features "streaming-v2" --tests

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build completed"
echo ""

# Run the comparative test
echo "🧪 Running cross-year speed comparison test..."
echo "📊 This will compare:"
echo "  1. Batch processing (ExportRangeBarProcessor)"
echo "  2. Streaming V2 (ProductionStreamingProcessor - bounded memory)"
echo ""

# Run with detailed output
RUST_LOG=debug cargo test --release --features "streaming-v2" test_cross_year_speed_comparison_oct2024_feb2025 -- --nocapture

echo ""
echo "✅ Cross-year comparison test completed"
echo "📅 $(date)"