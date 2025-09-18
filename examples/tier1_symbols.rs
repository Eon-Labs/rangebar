//! Tier-1 Symbol Discovery Example
//!
//! This example demonstrates how to use the Tier-1 symbol discovery functionality
//! to identify high-confidence cryptocurrency assets available across all
//! Binance futures markets.

use rangebar::{TIER1_SYMBOLS, get_tier1_symbols, get_tier1_usdt_pairs, is_tier1_symbol};

fn main() {
    println!("🔍 Tier-1 Symbol Discovery Example");
    println!("===================================");

    // Check if specific symbols are Tier-1
    println!("\n🎯 Symbol Classification:");
    let test_symbols = vec!["BTC", "ETH", "SOL", "DOGE", "SHIB", "PEPE"];

    for symbol in test_symbols {
        let is_tier1 = is_tier1_symbol(symbol);
        let status = if is_tier1 {
            "✅ Tier-1"
        } else {
            "❌ Not Tier-1"
        };
        println!("   {} - {}", symbol, status);
    }

    // Get all Tier-1 symbols
    println!("\n📊 All Tier-1 Symbols:");
    let tier1_symbols = get_tier1_symbols();
    println!("   Total count: {} symbols", tier1_symbols.len());
    println!("   Symbols: {:?}", tier1_symbols);

    // Get USDT perpetual pairs for Tier-1 symbols
    println!("\n💰 Tier-1 USDT Perpetual Pairs:");
    let usdt_pairs = get_tier1_usdt_pairs();
    println!("   Total pairs: {}", usdt_pairs.len());
    for pair in &usdt_pairs {
        println!("   • {}", pair);
    }

    // Access the constant array directly
    println!("\n📈 Direct Constant Access:");
    println!("   TIER1_SYMBOLS.len(): {}", TIER1_SYMBOLS.len());
    println!("   First 5 symbols: {:?}", &TIER1_SYMBOLS[..5]);

    // Demonstrate multi-market availability concept
    println!("\n🌐 Multi-Market Availability:");
    println!("   Tier-1 instruments are available across ALL THREE futures markets:");
    println!("   1. UM Futures (USDT-margined): BTCUSDT, ETHUSDT, etc.");
    println!("   2. UM Futures (USDC-margined): BTCUSDC, ETHUSDC, etc.");
    println!("   3. CM Futures (Coin-margined): BTCUSD_PERP, ETHUSD_PERP, etc.");

    println!("\n💡 Use Cases:");
    println!("   • Cross-market arbitrage opportunities");
    println!("   • High-confidence asset selection for algorithms");
    println!("   • Institutional-grade liquidity analysis");
    println!("   • Risk management and portfolio construction");

    // Example: Process range bars for all Tier-1 pairs
    println!("\n🚀 Integration Example:");
    println!("   You can combine Tier-1 discovery with range bar processing:");
    println!(
        "
   use rangebar::{{RangeBarProcessor, get_tier1_usdt_pairs}};

   let tier1_pairs = get_tier1_usdt_pairs();
   for pair in tier1_pairs {{
       let mut processor = RangeBarProcessor::new(8000); // 0.8% threshold
       // Load and process trades for this pair...
   }}"
    );
}
