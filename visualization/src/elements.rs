//! Range bar drawing elements and series

use crate::data::RangeBarData;
use crate::styles::RangeBarStyle;
use crate::layouts::TimePositionCalculator;
use crate::errors::{Result, VisualizationError};
use plotters::prelude::*;

/// A single range bar visual element optimized for plotters
pub struct RangeBarElement {
    /// Center X position in chart coordinates
    pub x_center: f64,
    /// Bar width in chart coordinates
    pub width: f64,
    /// Price data
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    /// Visual properties
    pub is_bullish: bool,
    pub volume: f64,
    /// Style override (if any)
    pub style_override: Option<RangeBarElementStyle>,
}

/// Individual bar style overrides
#[derive(Debug, Clone)]
pub struct RangeBarElementStyle {
    pub fill_color: Option<RGBColor>,
    pub border_color: Option<RGBColor>,
    pub border_width: Option<f32>,
    pub highlight: bool,
}

impl RangeBarElement {
    /// Create a new range bar element from data and position
    pub fn new(
        data: &RangeBarData,
        x_center: f64,
        width: f64,
    ) -> Self {
        Self {
            x_center,
            width,
            open: data.open,
            high: data.high,
            low: data.low,
            close: data.close,
            is_bullish: data.is_bullish(),
            volume: data.volume,
            style_override: None,
        }
    }
    
    /// Set style override for this specific bar
    pub fn with_style_override(mut self, style: RangeBarElementStyle) -> Self {
        self.style_override = Some(style);
        self
    }
    
    /// Calculate the body rectangle coordinates (open to close)
    pub fn body_rect(&self) -> (f64, f64, f64, f64) {
        let half_width = self.width / 2.0;
        let left = self.x_center - half_width;
        let right = self.x_center + half_width;
        
        let body_top = self.open.max(self.close);
        let body_bottom = self.open.min(self.close);
        
        (left, body_top, right, body_bottom)
    }
    
    /// Calculate the wick coordinates (high/low extensions)
    pub fn wick_coordinates(&self) -> (((f64, f64), (f64, f64)), ((f64, f64), (f64, f64))) {
        let body_top = self.open.max(self.close);
        let body_bottom = self.open.min(self.close);
        
        // Upper wick from body top to high
        let upper_wick = ((self.x_center, body_top), (self.x_center, self.high));
        // Lower wick from low to body bottom  
        let lower_wick = ((self.x_center, self.low), (self.x_center, body_bottom));
        
        (upper_wick, lower_wick)
    }
}

// Drawable implementation removed - we'll draw directly using plotters API

/// Series of range bars for plotting
pub struct RangeBarSeries {
    /// Individual bar elements
    pub elements: Vec<RangeBarElement>,
    /// Global style configuration
    pub style: RangeBarStyle,
    /// Metadata about the series
    pub metadata: SeriesMetadata,
}

/// Metadata about a range bar series
#[derive(Debug, Clone)]
pub struct SeriesMetadata {
    pub symbol: String,
    pub timeframe: String,
    pub threshold_bps: u32,
    pub total_bars: usize,
    pub time_span_hours: f64,
    pub price_range: (f64, f64),
    pub volume_range: (f64, f64),
}

impl RangeBarSeries {
    /// Create a new range bar series from data
    pub fn new(
        data: &[RangeBarData],
        calculator: &TimePositionCalculator,
        style: RangeBarStyle,
    ) -> Result<Self> {
        if data.is_empty() {
            return Err(VisualizationError::InvalidData {
                message: "Cannot create series from empty data".to_string(),
            });
        }
        
        let mut elements = Vec::with_capacity(data.len());
        
        // Create individual bar elements
        for (i, bar_data) in data.iter().enumerate() {
            let x_center = calculator.position(i)
                .ok_or_else(|| VisualizationError::LayoutError {
                    message: format!("No position calculated for bar {}", i),
                })?;
                
            let width = calculator.bar_width(i, style.bar_width_ratio);
            
            let element = RangeBarElement::new(bar_data, x_center, width);
            elements.push(element);
        }
        
        // Calculate metadata
        let metadata = Self::calculate_metadata(data);
        
        Ok(Self {
            elements,
            style,
            metadata,
        })
    }
    
    /// Calculate series metadata
    fn calculate_metadata(data: &[RangeBarData]) -> SeriesMetadata {
        let price_range = data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |acc, bar| {
            (acc.0.min(bar.low), acc.1.max(bar.high))
        });
        
        let volume_range = data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |acc, bar| {
            (acc.0.min(bar.volume), acc.1.max(bar.volume))
        });
        
        let time_span_hours = if data.len() > 1 {
            let duration = data.last().unwrap().close_time - data.first().unwrap().open_time;
            duration.num_milliseconds() as f64 / (1000.0 * 60.0 * 60.0)
        } else {
            0.0
        };
        
        SeriesMetadata {
            symbol: "UNKNOWN".to_string(), // Will be set by caller
            timeframe: "Range".to_string(),
            threshold_bps: 8000, // Default, will be set by caller
            total_bars: data.len(),
            time_span_hours,
            price_range,
            volume_range,
        }
    }
    
    /// Get all drawing data for the series (for use by export functions)
    pub fn get_all_drawing_data(&self) -> Vec<RangeBarDrawingData> {
        self.elements.iter()
            .map(|element| self.get_element_drawing_data(element))
            .collect()
    }
    
    /// Get drawing data for a single range bar element (for use by export functions)
    pub fn get_element_drawing_data(
        &self,
        element: &RangeBarElement,
    ) -> RangeBarDrawingData {
        // Get colors for this bar
        let fill_color = element.style_override
            .as_ref()
            .and_then(|s| s.fill_color)
            .unwrap_or_else(|| self.style.bar_color_with_opacity(element.is_bullish));
            
        let border_color = element.style_override
            .as_ref()
            .and_then(|s| s.border_color)
            .unwrap_or_else(|| self.style.colors.border);
        
        // Calculate coordinates
        let (left, body_top, right, body_bottom) = element.body_rect();
        let (upper_wick, lower_wick) = element.wick_coordinates();
        
        RangeBarDrawingData {
            body_rect: (left, body_top, right, body_bottom),
            upper_wick,
            lower_wick,
            fill_color,
            border_color,
            show_border: self.style.show_borders,
            border_width: self.style.border_width,
        }
    }
    
    /// Update series metadata
    pub fn set_metadata(&mut self, symbol: String, threshold_bps: u32) {
        self.metadata.symbol = symbol;
        self.metadata.threshold_bps = threshold_bps;
    }
    
    /// Get price range for axis scaling
    pub fn price_range(&self) -> (f64, f64) {
        self.metadata.price_range
    }
    
    /// Get time range for axis scaling
    pub fn time_range(&self) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
        if self.elements.is_empty() {
            let now = chrono::Utc::now();
            return (now, now);
        }
        
        // For range bars, we need to extract time info from somewhere
        // This is a simplification - in real usage we'd pass time data separately
        let now = chrono::Utc::now();
        let duration = chrono::Duration::hours(self.metadata.time_span_hours as i64);
        (now - duration, now)
    }
    
    /// Highlight specific bars (e.g., for user interaction)
    pub fn highlight_bars(&mut self, indices: &[usize]) {
        for &index in indices {
            if let Some(element) = self.elements.get_mut(index) {
                element.style_override = Some(RangeBarElementStyle {
                    fill_color: Some(self.style.colors.highlight),
                    border_color: Some(self.style.colors.border),
                    border_width: Some(2.0),
                    highlight: true,
                });
            }
        }
    }
    
    /// Clear all highlights
    pub fn clear_highlights(&mut self) {
        for element in &mut self.elements {
            element.style_override = None;
        }
    }
    
    /// Get summary statistics for the series
    pub fn statistics(&self) -> SeriesStatistics {
        if self.elements.is_empty() {
            return SeriesStatistics::default();
        }
        
        let mut stats = SeriesStatistics::default();
        
        for element in &self.elements {
            let movement = element.close - element.open;
            
            if movement > 0.0 {
                stats.bullish_bars += 1;
                stats.total_bullish_movement += movement;
            } else if movement < 0.0 {
                stats.bearish_bars += 1;
                stats.total_bearish_movement += movement.abs();
            } else {
                stats.doji_bars += 1;
            }
            
            stats.total_volume += element.volume;
            stats.average_range += element.high - element.low;
        }
        
        stats.total_bars = self.elements.len();
        stats.average_range /= self.elements.len() as f64;
        
        stats
    }
}

/// Statistical summary of a range bar series
#[derive(Debug, Default)]
pub struct SeriesStatistics {
    pub total_bars: usize,
    pub bullish_bars: usize,
    pub bearish_bars: usize,
    pub doji_bars: usize,
    pub total_volume: f64,
    pub total_bullish_movement: f64,
    pub total_bearish_movement: f64,
    pub average_range: f64,
}

/// Drawing data for a single range bar element
#[derive(Debug)]
pub struct RangeBarDrawingData {
    pub body_rect: (f64, f64, f64, f64), // left, top, right, bottom
    pub upper_wick: ((f64, f64), (f64, f64)), // start, end points
    pub lower_wick: ((f64, f64), (f64, f64)), // start, end points  
    pub fill_color: RGBColor,
    pub border_color: RGBColor,
    pub show_border: bool,
    pub border_width: f32,
}