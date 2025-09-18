use rangebar::{RangeBarProcessor, AggTrade, FixedPoint};

fn create_test_trade(id: i64, price: &str, volume: &str, timestamp: i64) -> AggTrade {
    AggTrade {
        agg_trade_id: id,
        price: FixedPoint::from_str(price).unwrap(),
        volume: FixedPoint::from_str(volume).unwrap(),
        first_trade_id: id,
        last_trade_id: id,
        timestamp,
        is_buyer_maker: false,
    }
}

fn main() {
    println!("Testing basic range bar logic...");
    
    let mut processor = RangeBarProcessor::new(10); // 0.1% = 10 bps
    
    // Create trades similar to our test data
    let trades = vec![
        create_test_trade(1, "50014.00859087", "0.12019569", 1756710002083),
        create_test_trade(2, "50163.87750994", "1.01283708", 1756710005113), // ~0.3% increase
        create_test_trade(3, "50032.44128269", "0.69397094", 1756710008770),
    ];
    
    println!("Processing {} trades with 10 bps threshold...", trades.len());
    
    match processor.process_trades(&trades) {
        Ok(bars) => {
            println!("Generated {} range bars:", bars.len());
            for (i, bar) in bars.iter().enumerate() {
                println!("  Bar {}: O={} H={} L={} C={}", 
                    i+1, bar.open, bar.high, bar.low, bar.close);
            }
        }
        Err(e) => {
            println!("Error processing trades: {:?}", e);
        }
    }
}
