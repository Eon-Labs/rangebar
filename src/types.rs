//! Type definitions for range bar processing

use crate::fixed_point::FixedPoint;

/// Aggregate trade data from Binance UM Futures
#[derive(Debug, Clone)]
pub struct AggTrade {
    /// Aggregate trade ID  
    pub agg_trade_id: i64,
    
    /// Price as fixed-point integer
    pub price: FixedPoint,
    
    /// Volume as fixed-point integer
    pub volume: FixedPoint,
    
    /// First trade ID in aggregation
    pub first_trade_id: i64,
    
    /// Last trade ID in aggregation  
    pub last_trade_id: i64,
    
    /// Timestamp in milliseconds
    pub timestamp: i64,
}

impl AggTrade {
    /// Number of individual trades aggregated
    pub fn trade_count(&self) -> i64 {
        self.last_trade_id - self.first_trade_id + 1
    }
    
    /// Turnover (price * volume) as i128 to prevent overflow
    pub fn turnover(&self) -> i128 {
        (self.price.0 as i128) * (self.volume.0 as i128)
    }
}

/// Range bar with OHLCV data
#[derive(Debug, Clone)]
pub struct RangeBar {
    /// Opening timestamp (first trade)
    pub open_time: i64,
    
    /// Closing timestamp (last trade)
    pub close_time: i64,
    
    /// Opening price (first trade price)
    pub open: FixedPoint,
    
    /// Highest price in bar
    pub high: FixedPoint,
    
    /// Lowest price in bar  
    pub low: FixedPoint,
    
    /// Closing price (breach trade price)
    pub close: FixedPoint,
    
    /// Total volume
    pub volume: FixedPoint,
    
    /// Total turnover (sum of price * volume)
    pub turnover: i128,
    
    /// Number of trades
    pub trade_count: i64,
    
    /// First aggregate trade ID
    pub first_id: i64,
    
    /// Last aggregate trade ID  
    pub last_id: i64,
}

impl RangeBar {
    /// Create new range bar from opening trade
    pub fn new(trade: &AggTrade) -> Self {
        Self {
            open_time: trade.timestamp,
            close_time: trade.timestamp,
            open: trade.price,
            high: trade.price,
            low: trade.price,
            close: trade.price,
            volume: trade.volume,
            turnover: trade.turnover(),
            trade_count: trade.trade_count(),
            first_id: trade.agg_trade_id,
            last_id: trade.agg_trade_id,
        }
    }
    
    /// Update bar with new trade data (always call before checking breach)
    pub fn update_with_trade(&mut self, trade: &AggTrade) {
        // Update price extremes
        if trade.price > self.high {
            self.high = trade.price;
        }
        if trade.price < self.low {
            self.low = trade.price;
        }
        
        // Update closing data
        self.close = trade.price;
        self.close_time = trade.timestamp;
        self.last_id = trade.agg_trade_id;
        
        // Update volume and trade count
        self.volume = FixedPoint(self.volume.0 + trade.volume.0);
        self.turnover += trade.turnover();
        self.trade_count += trade.trade_count();
    }
    
    /// Check if price breaches the range thresholds
    /// 
    /// # Arguments
    /// 
    /// * `price` - Current price to check
    /// * `upper_threshold` - Upper breach threshold (from bar open)
    /// * `lower_threshold` - Lower breach threshold (from bar open)
    /// 
    /// # Returns
    /// 
    /// `true` if price breaches either threshold
    pub fn is_breach(&self, price: FixedPoint, upper_threshold: FixedPoint, lower_threshold: FixedPoint) -> bool {
        price >= upper_threshold || price <= lower_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point::FixedPoint;
    
    #[test]
    fn test_agg_trade_creation() {
        let trade = AggTrade {
            agg_trade_id: 12345,
            price: FixedPoint::from_str("50000.12345678").unwrap(),
            volume: FixedPoint::from_str("1.5").unwrap(),
            first_trade_id: 100,
            last_trade_id: 102,
            timestamp: 1640995200000,
        };
        
        assert_eq!(trade.trade_count(), 3); // 102 - 100 + 1
        assert!(trade.turnover() > 0);
    }
    
    #[test]
    fn test_range_bar_creation() {
        let trade = AggTrade {
            agg_trade_id: 12345,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 100,
            last_trade_id: 100,
            timestamp: 1640995200000,
        };
        
        let bar = RangeBar::new(&trade);
        assert_eq!(bar.open, trade.price);
        assert_eq!(bar.high, trade.price);
        assert_eq!(bar.low, trade.price);
        assert_eq!(bar.close, trade.price);
    }
    
    #[test]
    fn test_range_bar_update() {
        let trade1 = AggTrade {
            agg_trade_id: 12345,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.0").unwrap(),
            first_trade_id: 100,
            last_trade_id: 100,
            timestamp: 1640995200000,
        };
        
        let mut bar = RangeBar::new(&trade1);
        
        let trade2 = AggTrade {
            agg_trade_id: 12346,
            price: FixedPoint::from_str("50100.0").unwrap(),
            volume: FixedPoint::from_str("2.0").unwrap(),
            first_trade_id: 101,
            last_trade_id: 101,
            timestamp: 1640995201000,
        };
        
        bar.update_with_trade(&trade2);
        
        assert_eq!(bar.open.to_string(), "50000.00000000");
        assert_eq!(bar.high.to_string(), "50100.00000000");
        assert_eq!(bar.low.to_string(), "50000.00000000");
        assert_eq!(bar.close.to_string(), "50100.00000000");
        assert_eq!(bar.volume.to_string(), "3.00000000");
        assert_eq!(bar.trade_count, 2);
    }
}