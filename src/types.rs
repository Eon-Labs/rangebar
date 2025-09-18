//! Type definitions for range bar processing

use crate::fixed_point::FixedPoint;
use serde::{Deserialize, Serialize};

/// Aggregate trade data from Binance UM Futures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
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

    /// Whether buyer is market maker (true = sell pressure, false = buy pressure)
    /// Critical for order flow analysis and market microstructure
    pub is_buyer_maker: bool,
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

/// Range bar with OHLCV data and market microstructure enhancements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
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

    // === MARKET MICROSTRUCTURE ENHANCEMENTS ===
    /// Volume from buy-side trades (is_buyer_maker = false)
    /// Represents aggressive buying pressure
    pub buy_volume: FixedPoint,

    /// Volume from sell-side trades (is_buyer_maker = true)
    /// Represents aggressive selling pressure
    pub sell_volume: FixedPoint,

    /// Number of buy-side trades (aggressive buying)
    pub buy_trade_count: i64,

    /// Number of sell-side trades (aggressive selling)
    pub sell_trade_count: i64,

    /// Volume Weighted Average Price for the bar
    /// Calculated incrementally as: sum(price * volume) / sum(volume)
    pub vwap: FixedPoint,

    /// Turnover from buy-side trades (buy pressure)
    pub buy_turnover: i128,

    /// Turnover from sell-side trades (sell pressure)
    pub sell_turnover: i128,
}

impl RangeBar {
    /// Create new range bar from opening trade
    pub fn new(trade: &AggTrade) -> Self {
        let trade_turnover = trade.turnover();
        let trade_count = trade.trade_count();

        // Segregate order flow based on is_buyer_maker
        let (buy_volume, sell_volume) = if trade.is_buyer_maker {
            (FixedPoint(0), trade.volume) // Seller aggressive = sell pressure
        } else {
            (trade.volume, FixedPoint(0)) // Buyer aggressive = buy pressure
        };

        let (buy_trade_count, sell_trade_count) = if trade.is_buyer_maker {
            (0, trade_count)
        } else {
            (trade_count, 0)
        };

        let (buy_turnover, sell_turnover) = if trade.is_buyer_maker {
            (0, trade_turnover)
        } else {
            (trade_turnover, 0)
        };

        Self {
            open_time: trade.timestamp,
            close_time: trade.timestamp,
            open: trade.price,
            high: trade.price,
            low: trade.price,
            close: trade.price,
            volume: trade.volume,
            turnover: trade_turnover,
            trade_count,
            first_id: trade.agg_trade_id,
            last_id: trade.agg_trade_id,
            // Market microstructure fields
            buy_volume,
            sell_volume,
            buy_trade_count,
            sell_trade_count,
            vwap: trade.price, // Initial VWAP equals opening price
            buy_turnover,
            sell_turnover,
        }
    }

    /// Update bar with new trade data (always call before checking breach)
    /// Maintains market microstructure metrics incrementally
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

        // Cache trade metrics for efficiency
        let trade_turnover = trade.turnover();
        let trade_count = trade.trade_count();

        // Update total volume and trade count
        self.volume = FixedPoint(self.volume.0 + trade.volume.0);
        self.turnover += trade_turnover;
        self.trade_count += trade_count;

        // === MARKET MICROSTRUCTURE INCREMENTAL UPDATES ===

        // Update order flow segregation
        if trade.is_buyer_maker {
            // Seller aggressive = sell pressure
            self.sell_volume = FixedPoint(self.sell_volume.0 + trade.volume.0);
            self.sell_trade_count += trade_count;
            self.sell_turnover += trade_turnover;
        } else {
            // Buyer aggressive = buy pressure
            self.buy_volume = FixedPoint(self.buy_volume.0 + trade.volume.0);
            self.buy_trade_count += trade_count;
            self.buy_turnover += trade_turnover;
        }

        // Update VWAP incrementally: VWAP = total_turnover / total_volume
        // Using integer arithmetic to maintain precision
        if self.volume.0 > 0 {
            // Calculate VWAP: turnover / volume, but maintain FixedPoint precision
            // turnover is in (price * volume) units, volume is in volume units
            // VWAP should be in price units
            let vwap_raw = self.turnover / (self.volume.0 as i128);
            self.vwap = FixedPoint(vwap_raw as i64);
        }
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
    pub fn is_breach(
        &self,
        price: FixedPoint,
        upper_threshold: FixedPoint,
        lower_threshold: FixedPoint,
    ) -> bool {
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
            is_buyer_maker: false, // Buy pressure (taker buying from maker)
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
            is_buyer_maker: true, // Sell pressure (taker selling to maker)
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
            is_buyer_maker: false, // Buy pressure
        };

        let mut bar = RangeBar::new(&trade1);

        let trade2 = AggTrade {
            agg_trade_id: 12346,
            price: FixedPoint::from_str("50100.0").unwrap(),
            volume: FixedPoint::from_str("2.0").unwrap(),
            first_trade_id: 101,
            last_trade_id: 101,
            timestamp: 1640995201000,
            is_buyer_maker: true, // Sell pressure
        };

        bar.update_with_trade(&trade2);

        assert_eq!(bar.open.to_string(), "50000.00000000");
        assert_eq!(bar.high.to_string(), "50100.00000000");
        assert_eq!(bar.low.to_string(), "50000.00000000");
        assert_eq!(bar.close.to_string(), "50100.00000000");
        assert_eq!(bar.volume.to_string(), "3.00000000");
        assert_eq!(bar.trade_count, 2);
    }

    #[test]
    fn test_microstructure_segregation() {
        // Create buy trade (is_buyer_maker = false)
        let buy_trade = AggTrade {
            agg_trade_id: 1,
            price: FixedPoint::from_str("50000.0").unwrap(),
            volume: FixedPoint::from_str("1.5").unwrap(),
            first_trade_id: 1,
            last_trade_id: 1,
            timestamp: 1640995200000,
            is_buyer_maker: false, // Buy pressure (taker buying from maker)
        };

        let mut bar = RangeBar::new(&buy_trade);

        // Create sell trade (is_buyer_maker = true)
        let sell_trade = AggTrade {
            agg_trade_id: 2,
            price: FixedPoint::from_str("50050.0").unwrap(),
            volume: FixedPoint::from_str("2.5").unwrap(),
            first_trade_id: 2,
            last_trade_id: 3, // Multiple trades aggregated
            timestamp: 1640995201000,
            is_buyer_maker: true, // Sell pressure (taker selling to maker)
        };

        bar.update_with_trade(&sell_trade);

        // Verify order flow segregation
        assert_eq!(bar.buy_volume.to_string(), "1.50000000"); // Only first trade
        assert_eq!(bar.sell_volume.to_string(), "2.50000000"); // Only second trade
        assert_eq!(bar.buy_trade_count, 1); // First trade count
        assert_eq!(bar.sell_trade_count, 2); // Second trade count (3 - 2 + 1 = 2)

        // Verify totals
        assert_eq!(bar.volume.to_string(), "4.00000000"); // 1.5 + 2.5
        assert_eq!(bar.trade_count, 3); // 1 + 2

        // Verify VWAP calculation
        // VWAP = (50000 * 1.5 + 50050 * 2.5) / 4.0 = (75000 + 125125) / 4.0 = 50031.25
        assert_eq!(bar.vwap.to_string(), "50031.25000000");

        println!("✅ Microstructure segregation test passed:");
        println!(
            "   Buy volume: {}, Sell volume: {}",
            bar.buy_volume.to_string(),
            bar.sell_volume.to_string()
        );
        println!(
            "   Buy trades: {}, Sell trades: {}",
            bar.buy_trade_count, bar.sell_trade_count
        );
        println!("   VWAP: {}", bar.vwap.to_string());
    }
}
