//! Statistical analysis overlays for range bar charts

use crate::data::RangeBarData;
use crate::errors::Result;

/// Statistical analysis calculator for range bar data
pub struct RangeBarAnalyzer {
    /// Window size for moving statistics
    pub window_size: usize,
}

impl Default for RangeBarAnalyzer {
    fn default() -> Self {
        Self { window_size: 20 }
    }
}

impl RangeBarAnalyzer {
    /// Calculate moving average of closing prices
    pub fn moving_average(&self, data: &[RangeBarData]) -> Vec<f64> {
        if data.len() < self.window_size {
            return data.iter().map(|d| d.close).collect();
        }
        
        let mut averages = Vec::new();
        
        for i in 0..data.len() {
            let start = if i >= self.window_size { i - self.window_size + 1 } else { 0 };
            let sum: f64 = data[start..=i].iter().map(|d| d.close).sum();
            let count = i - start + 1;
            averages.push(sum / count as f64);
        }
        
        averages
    }
    
    /// Calculate range bar duration statistics
    pub fn duration_statistics(&self, data: &[RangeBarData]) -> DurationStats {
        if data.is_empty() {
            return DurationStats::default();
        }
        
        let durations: Vec<i64> = data.iter().map(|d| d.duration_ms).collect();
        let total = durations.iter().sum::<i64>() as f64;
        let count = durations.len() as f64;
        let mean = total / count;
        
        let variance = durations.iter()
            .map(|&d| (d as f64 - mean).powi(2))
            .sum::<f64>() / count;
        
        DurationStats {
            mean_ms: mean,
            std_dev_ms: variance.sqrt(),
            min_ms: *durations.iter().min().unwrap() as f64,
            max_ms: *durations.iter().max().unwrap() as f64,
        }
    }
}

/// Duration statistics for range bars
#[derive(Debug, Default)]
pub struct DurationStats {
    pub mean_ms: f64,
    pub std_dev_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}