//! Data preparation and transformation for range bar visualization

use crate::errors::{Result, VisualizationError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json;

/// Range bar data point optimized for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeBarData {
    /// Opening timestamp  
    pub open_time: DateTime<Utc>,
    /// Closing timestamp
    pub close_time: DateTime<Utc>,
    /// Opening price
    pub open: f64,
    /// Highest price in bar
    pub high: f64,
    /// Lowest price in bar  
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Total volume
    pub volume: f64,
    /// Total turnover
    pub turnover: f64,
    /// Number of trades
    pub trade_count: i64,
    /// Bar duration in milliseconds (for spacing calculations)
    pub duration_ms: i64,
    /// Price range that triggered bar closure
    pub price_range: f64,
}

impl RangeBarData {
    /// Create from dictionary data (from rangebar core)
    pub fn from_dict(data: &HashMap<String, Vec<f64>>, index: usize) -> Result<Self> {
        let get_value = |key: &str| -> Result<f64> {
            data.get(key)
                .and_then(|v| v.get(index))
                .copied()
                .ok_or_else(|| VisualizationError::InvalidData {
                    message: format!("Missing or invalid data for key: {}", key),
                })
        };

        let open_time_ms = get_value("open_time")? as i64;
        let close_time_ms = get_value("close_time")? as i64;
        
        let open_time = DateTime::from_timestamp_millis(open_time_ms)
            .ok_or_else(|| VisualizationError::InvalidData {
                message: format!("Invalid timestamp: {}", open_time_ms),
            })?;
            
        let close_time = DateTime::from_timestamp_millis(close_time_ms)
            .ok_or_else(|| VisualizationError::InvalidData {
                message: format!("Invalid timestamp: {}", close_time_ms),
            })?;

        let open = get_value("open")?;
        let high = get_value("high")?;
        let low = get_value("low")?;
        let close = get_value("close")?;
        
        Ok(RangeBarData {
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume: get_value("volume")?,
            turnover: get_value("turnover")?,
            trade_count: get_value("trade_count")? as i64,
            duration_ms: close_time_ms - open_time_ms,
            price_range: high - low,
        })
    }
    
    /// Calculate the price movement percentage from open to close
    pub fn price_movement_pct(&self) -> f64 {
        if self.open == 0.0 {
            0.0
        } else {
            ((self.close - self.open) / self.open) * 100.0
        }
    }
    
    /// Check if this is a bullish (up) bar
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
    
    /// Calculate the body size (close - open) as percentage of total range
    pub fn body_ratio(&self) -> f64 {
        if self.price_range == 0.0 {
            1.0
        } else {
            (self.close - self.open).abs() / self.price_range
        }
    }
}

/// Data preprocessor for converting range bar data into visualization-ready format
pub struct DataPreprocessor {
    /// Minimum time gap to consider significant (for spacing calculations)
    pub min_time_gap_ms: i64,
    /// Maximum number of bars to display (for performance)
    pub max_bars: Option<usize>,
}

impl Default for DataPreprocessor {
    fn default() -> Self {
        Self {
            min_time_gap_ms: 1000, // 1 second
            max_bars: Some(10000),  // Limit for performance
        }
    }
}

impl DataPreprocessor {
    /// Convert raw range bar data dictionary to visualization data
    pub fn process_dict_data(&self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<RangeBarData>> {
        // Determine the number of bars
        let bar_count = data.get("open_time")
            .ok_or_else(|| VisualizationError::InvalidData {
                message: "Missing 'open_time' data".to_string(),
            })?
            .len();
            
        if bar_count == 0 {
            return Ok(Vec::new());
        }
        
        // Apply max bars limit if specified
        let actual_count = self.max_bars.map_or(bar_count, |max| bar_count.min(max));
        
        let mut bars = Vec::with_capacity(actual_count);
        
        for i in 0..actual_count {
            let bar = RangeBarData::from_dict(data, i)?;
            bars.push(bar);
        }
        
        // Sort by open time to ensure proper ordering
        bars.sort_by(|a, b| a.open_time.cmp(&b.open_time));
        
        Ok(bars)
    }
    
    /// Calculate spacing factors for non-uniform time distribution
    pub fn calculate_spacing_factors(&self, bars: &[RangeBarData]) -> Vec<f64> {
        if bars.len() <= 1 {
            return vec![1.0; bars.len()];
        }
        
        let mut factors = Vec::with_capacity(bars.len());
        factors.push(1.0); // First bar gets default spacing
        
        for i in 1..bars.len() {
            let time_gap = bars[i].open_time.timestamp_millis() - bars[i-1].close_time.timestamp_millis();
            
            // Calculate spacing factor based on time gap
            let factor = if time_gap > self.min_time_gap_ms {
                // Larger gaps get more spacing (up to 3x)
                1.0 + (time_gap as f64 / (self.min_time_gap_ms as f64 * 10.0)).min(2.0)
            } else {
                1.0 // Standard spacing
            };
            
            factors.push(factor);
        }
        
        factors
    }
    
    /// Load authentic CCXT USDⓈ-M Perpetuals range bar data
    pub fn load_authentic_data(&self) -> Vec<RangeBarData> {
        use std::path::Path;
        use std::fs;
        
        // Path to authentic range bar data - check multiple possible locations
        let possible_paths = [
            Path::new("data/authentic_range_bars.json"),
            Path::new("visualization/data/authentic_range_bars.json"),
            Path::new("../data/authentic_range_bars.json"),
            Path::new("./data/authentic_range_bars.json"),
        ];
        
        let data_path = possible_paths.iter().find(|p| p.exists()).cloned();
        
        let data_path = match data_path {
            Some(path) => path,
            None => {
                eprintln!("⚠️  Authentic data not found in any of the expected locations, generating fallback sample");
                return self.generate_fallback_sample_data(50);
            }
        };
        
        match fs::read_to_string(data_path) {
            Ok(json_content) => {
                match serde_json::from_str::<Vec<serde_json::Value>>(&json_content) {
                    Ok(json_bars) => {
                        let mut authentic_bars = Vec::new();
                        
                        for json_bar in json_bars {
                            if let Ok(bar) = self.parse_authentic_bar(&json_bar) {
                                authentic_bars.push(bar);
                            }
                        }
                        
                        println!("✅ Loaded {} authentic CCXT USDⓈ-M Perpetuals range bars", authentic_bars.len());
                        authentic_bars
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to parse authentic data JSON: {}", e);
                        self.generate_fallback_sample_data(50)
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to read authentic data file: {}", e);
                self.generate_fallback_sample_data(50)
            }
        }
    }
    
    /// Parse a single authentic range bar from JSON
    fn parse_authentic_bar(&self, json: &serde_json::Value) -> std::result::Result<RangeBarData, String> {
        use chrono::{DateTime, Utc};
        
        let open_time_str = json["open_time"].as_str().ok_or("Missing open_time")?;
        let close_time_str = json["close_time"].as_str().ok_or("Missing close_time")?;
        
        let open_time: DateTime<Utc> = open_time_str.parse().map_err(|e| format!("Invalid open_time: {}", e))?;
        let close_time: DateTime<Utc> = close_time_str.parse().map_err(|e| format!("Invalid close_time: {}", e))?;
        
        Ok(RangeBarData {
            open_time,
            close_time,
            open: json["open"].as_f64().ok_or("Missing open")?,
            high: json["high"].as_f64().ok_or("Missing high")?,
            low: json["low"].as_f64().ok_or("Missing low")?,
            close: json["close"].as_f64().ok_or("Missing close")?,
            volume: json["volume"].as_f64().ok_or("Missing volume")?,
            turnover: json["turnover"].as_f64().ok_or("Missing turnover")?,
            trade_count: json["trade_count"].as_i64().ok_or("Missing trade_count")?,
            duration_ms: json["duration_ms"].as_i64().ok_or("Missing duration_ms")?,
            price_range: json["price_range"].as_f64().ok_or("Missing price_range")?,
        })
    }
    
    /// Generate sample range bar data for testing (now loads authentic data by default)
    pub fn generate_sample_data(&self, _count: usize) -> Vec<RangeBarData> {
        // Always prioritize authentic data over synthetic
        self.load_authentic_data()
    }
    
    /// Generate fallback sample data when authentic data is unavailable
    fn generate_fallback_sample_data(&self, count: usize) -> Vec<RangeBarData> {
        use chrono::Duration;
        
        eprintln!("⚠️  Using fallback synthetic data - fetch authentic data first!");
        
        let mut bars = Vec::with_capacity(count);
        let mut current_time = Utc::now() - Duration::hours(count as i64);
        let mut current_price = 111000.0; // Realistic BTC price
        
        for i in 0..count {
            // More realistic BTC price movements
            let trend = (i as f64 / count as f64 - 0.5) * 2000.0; // Overall trend
            let volatility = (rand_like(i) - 0.5) * 800.0; // Random volatility
            current_price += trend / count as f64 + volatility;
            
            let open = current_price;
            // Range bars: exactly 0.8% movement triggers close
            let threshold = open * 0.008;
            let direction = if rand_like(i * 2) > 0.5 { 1.0 } else { -1.0 };
            let close = open + direction * threshold;
            
            let high = open.max(close) + rand_like(i * 3) * threshold * 0.5;
            let low = open.min(close) - rand_like(i * 4) * threshold * 0.5;
            
            // Variable durations like real range bars
            let duration = Duration::minutes((30.0 + rand_like(i * 5) * 300.0) as i64);
            let close_time = current_time + duration;
            
            bars.push(RangeBarData {
                open_time: current_time,
                close_time,
                open,
                high,
                low,
                close,
                volume: 10000.0 + rand_like(i * 6) * 50000.0,
                turnover: (open + close) / 2.0 * (10000.0 + rand_like(i * 6) * 50000.0),
                trade_count: (50.0 + rand_like(i * 7) * 500.0) as i64,
                duration_ms: duration.num_milliseconds(),
                price_range: high - low,
            });
            
            current_time = close_time + Duration::minutes((rand_like(i * 8) * 120.0) as i64);
            current_price = close;
        }
        
        bars
    }
}

/// Simple deterministic pseudo-random function for testing
fn rand_like(seed: usize) -> f64 {
    let x = (seed.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7fff_ffff;
    (x as f64) / (0x7fff_ffff as f64)
}