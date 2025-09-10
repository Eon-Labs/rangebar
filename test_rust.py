#!/usr/bin/env python3
"""
Test script for the Rust range bar implementation.
This tests the core algorithm without requiring the full Python extension build.
"""

import subprocess
import json
import sys
from pathlib import Path

def test_rust_core():
    """Test the Rust implementation directly"""
    print("Testing Rust core algorithm...")
    
    # Setup environment to include Rust
    import os
    env = os.environ.copy()
    # Use standard Rust installation path
    env['PATH'] = f"{Path.home()}/.cargo/bin:{env.get('PATH', '')}"
    
    # Run Rust tests
    result = subprocess.run(
        ["cargo", "test", "--verbose"], 
        cwd=Path(__file__).parent,
        capture_output=True,
        text=True,
        env=env
    )
    
    print(f"Cargo test exit code: {result.returncode}")
    if result.stdout:
        print("STDOUT:")
        print(result.stdout)
    if result.stderr:
        print("STDERR:")
        print(result.stderr)
    
    return result.returncode == 0

def test_algorithm_properties():
    """Test core algorithm properties using a simple Rust binary"""
    # Create a simple test binary
    test_binary = '''
use rangebar_rust::fixed_point::FixedPoint;
use rangebar_rust::range_bars::RangeBarProcessor;
use rangebar_rust::types::AggTrade;

fn main() {
    // Test basic threshold calculation
    let price = FixedPoint::from_str("50000.0").unwrap();
    let (upper, lower) = price.compute_range_thresholds(8000); // 0.8%
    
    println!("Price: {}", price);
    println!("Upper threshold: {}", upper);
    println!("Lower threshold: {}", lower);
    
    // Test should show:
    // Price: 50000.00000000
    // Upper threshold: 50400.00000000  
    // Lower threshold: 49600.00000000
    
    // Test simple range bar processing
    let mut processor = RangeBarProcessor::new(8000);
    
    let trades = vec![
        AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1000,
        },
        AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("50400.0").unwrap(),  // Exact breach
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 2,
            last_trade_id: 2,
            timestamp: 2000,
        },
    ];
    
    match processor.process_trades(&trades) {
        Ok(bars) => {
            println!("Generated {} bars", bars.len());
            for (i, bar) in bars.iter().enumerate() {
                println!("Bar {}: open={}, high={}, low={}, close={}", 
                         i, bar.open, bar.high, bar.low, bar.close);
            }
        }
        Err(e) => {
            println!("Error processing trades: {}", e);
        }
    }
}
    '''
    
    # For now, just run the existing tests
    return test_rust_core()

if __name__ == "__main__":
    success = test_algorithm_properties()
    if success:
        print("✅ All Rust tests passed!")
        print("\nNext steps:")
        print("1. Install maturin: cargo install maturin")
        print("2. Build Python extension: maturin develop --release")
        print("3. Test Python integration")
    else:
        print("❌ Rust tests failed!")
        sys.exit(1)