//! Chart layout and spacing management for range bar visualization

use crate::data::RangeBarData;
use crate::errors::{Result, VisualizationError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Chart layout configuration and calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartLayout {
    /// Total chart width in pixels
    pub width: u32,
    /// Total chart height in pixels  
    pub height: u32,
    /// Margin configuration
    pub margins: Margins,
    /// Price axis configuration
    pub price_axis: AxisConfig,
    /// Time axis configuration (special handling for range bars)
    pub time_axis: TimeAxisConfig,
    /// Whether to show volume panel below main chart
    pub show_volume_panel: bool,
    /// Volume panel height ratio (0.0 to 1.0 of remaining space)
    pub volume_panel_ratio: f64,
}

impl Default for ChartLayout {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            margins: Margins::default(),
            price_axis: AxisConfig::default(),
            time_axis: TimeAxisConfig::default(),
            show_volume_panel: false,
            volume_panel_ratio: 0.25,
        }
    }
}

impl ChartLayout {
    /// Calculate the main chart area after accounting for margins and panels
    pub fn chart_area(&self) -> (u32, u32, u32, u32) {
        let left = self.margins.left;
        let right = self.width - self.margins.right;
        let top = self.margins.top;
        
        let bottom = if self.show_volume_panel {
            let total_content_height = self.height - self.margins.top - self.margins.bottom;
            let main_chart_height = (total_content_height as f64 * (1.0 - self.volume_panel_ratio)) as u32;
            self.margins.top + main_chart_height
        } else {
            self.height - self.margins.bottom
        };
        
        (left, top, right, bottom)
    }
    
    /// Calculate volume panel area if enabled
    pub fn volume_area(&self) -> Option<(u32, u32, u32, u32)> {
        if !self.show_volume_panel {
            return None;
        }
        
        let left = self.margins.left;
        let right = self.width - self.margins.right;
        
        let total_content_height = self.height - self.margins.top - self.margins.bottom;
        let main_chart_height = (total_content_height as f64 * (1.0 - self.volume_panel_ratio)) as u32;
        let top = self.margins.top + main_chart_height + 10; // 10px gap
        let bottom = self.height - self.margins.bottom;
        
        Some((left, top, right, bottom))
    }
    
    /// Calculate chart area dimensions
    pub fn chart_dimensions(&self) -> (u32, u32) {
        let (left, top, right, bottom) = self.chart_area();
        (right - left, bottom - top)
    }
}

/// Margin configuration for chart layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            left: 80,   // Space for price labels
            right: 40,  // Small buffer
            top: 40,    // Space for title
            bottom: 60, // Space for time labels
        }
    }
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// Whether to show the axis
    pub show: bool,
    /// Number of tick marks
    pub tick_count: usize,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Label format string
    pub label_format: String,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            show: true,
            tick_count: 10,
            show_grid: true,
            label_format: "{:.2}".to_string(),
        }
    }
}

/// Time axis configuration specific to range bars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAxisConfig {
    /// Base axis configuration
    pub base: AxisConfig,
    /// Time scale calculation method
    pub scale: TimeScale,
    /// Whether to show time gaps visually
    pub show_time_gaps: bool,
    /// Minimum gap threshold to display (in minutes)
    pub min_gap_minutes: i64,
    /// Maximum gap compression ratio
    pub max_gap_compression: f64,
}

impl Default for TimeAxisConfig {
    fn default() -> Self {
        Self {
            base: AxisConfig {
                label_format: "%H:%M".to_string(),
                ..AxisConfig::default()
            },
            scale: TimeScale::NonUniform,
            show_time_gaps: true,
            min_gap_minutes: 5,
            max_gap_compression: 0.5,
        }
    }
}

/// Time scale calculation methods for range bars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeScale {
    /// Uniform spacing regardless of actual time gaps
    Uniform,
    /// Non-uniform spacing that reflects actual time gaps
    NonUniform,
    /// Compressed time gaps beyond a threshold
    Compressed,
}

/// Time position calculator for range bar non-uniform spacing
pub struct TimePositionCalculator {
    /// Chart start and end positions (in chart coordinates)
    pub chart_start: f64,
    pub chart_end: f64,
    /// Time range of the data
    pub time_start: DateTime<Utc>,
    pub time_end: DateTime<Utc>,
    /// Calculated positions for each bar
    positions: Vec<f64>,
    /// Time scale configuration
    scale_config: TimeScale,
}

impl TimePositionCalculator {
    /// Create new position calculator for given data and layout
    pub fn new(
        bars: &[RangeBarData],
        chart_start: f64,
        chart_end: f64,
        scale_config: TimeScale,
    ) -> Result<Self> {
        if bars.is_empty() {
            return Err(VisualizationError::InvalidData {
                message: "Cannot calculate positions for empty data".to_string(),
            });
        }
        
        let time_start = bars.first().unwrap().open_time;
        let time_end = bars.last().unwrap().close_time;
        
        let mut calculator = Self {
            chart_start,
            chart_end,
            time_start,
            time_end,
            positions: Vec::new(),
            scale_config,
        };
        
        calculator.calculate_positions(bars)?;
        Ok(calculator)
    }
    
    /// Calculate positions for all bars based on the time scale method
    fn calculate_positions(&mut self, bars: &[RangeBarData]) -> Result<()> {
        self.positions.clear();
        self.positions.reserve(bars.len());
        
        match self.scale_config {
            TimeScale::Uniform => self.calculate_uniform_positions(bars),
            TimeScale::NonUniform => self.calculate_non_uniform_positions(bars),
            TimeScale::Compressed => self.calculate_compressed_positions(bars),
        }
    }
    
    /// Calculate uniform spacing (traditional approach)
    fn calculate_uniform_positions(&mut self, bars: &[RangeBarData]) -> Result<()> {
        let chart_width = self.chart_end - self.chart_start;
        let spacing = chart_width / (bars.len() as f64);
        
        for i in 0..bars.len() {
            let position = self.chart_start + (i as f64 + 0.5) * spacing;
            self.positions.push(position);
        }
        
        Ok(())
    }
    
    /// Calculate non-uniform spacing based on actual time gaps
    fn calculate_non_uniform_positions(&mut self, bars: &[RangeBarData]) -> Result<()> {
        if bars.len() == 1 {
            self.positions.push((self.chart_start + self.chart_end) / 2.0);
            return Ok(());
        }
        
        let total_time_ms = (self.time_end - self.time_start).num_milliseconds() as f64;
        let chart_width = self.chart_end - self.chart_start;
        
        // Calculate cumulative time positions
        let mut cumulative_time = 0.0;
        
        for i in 0..bars.len() {
            if i == 0 {
                // First bar at start position
                self.positions.push(self.chart_start);
            } else {
                // Time elapsed from start to this bar's open time
                let elapsed_ms = (bars[i].open_time - self.time_start).num_milliseconds() as f64;
                let position = self.chart_start + (elapsed_ms / total_time_ms) * chart_width;
                self.positions.push(position);
            }
        }
        
        Ok(())
    }
    
    /// Calculate compressed spacing for large time gaps
    fn calculate_compressed_positions(&mut self, bars: &[RangeBarData]) -> Result<()> {
        // First calculate non-uniform positions
        self.calculate_non_uniform_positions(bars)?;
        
        // Then compress large gaps
        if bars.len() <= 1 {
            return Ok(());
        }
        
        let chart_width = self.chart_end - self.chart_start;
        let min_spacing = chart_width / (bars.len() as f64 * 2.0); // Minimum spacing
        
        // Identify large gaps and compress them
        for i in 1..self.positions.len() {
            let gap = self.positions[i] - self.positions[i-1];
            if gap > min_spacing * 3.0 {
                // Compress this gap
                let compressed_gap = min_spacing * 2.0;
                let reduction = gap - compressed_gap;
                
                // Shift all subsequent positions left
                for j in i..self.positions.len() {
                    self.positions[j] -= reduction;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the chart position for a specific bar index
    pub fn position(&self, index: usize) -> Option<f64> {
        self.positions.get(index).copied()
    }
    
    /// Get all positions
    pub fn positions(&self) -> &[f64] {
        &self.positions
    }
    
    /// Calculate bar width based on available space and neighboring bars
    pub fn bar_width(&self, index: usize, base_width_ratio: f64) -> f64 {
        if self.positions.len() <= 1 {
            return (self.chart_end - self.chart_start) * base_width_ratio;
        }
        
        let available_space = if index == 0 {
            // First bar: space to next bar
            if self.positions.len() > 1 {
                self.positions[1] - self.positions[0]
            } else {
                self.chart_end - self.chart_start
            }
        } else if index == self.positions.len() - 1 {
            // Last bar: space from previous bar
            self.positions[index] - self.positions[index - 1]
        } else {
            // Middle bar: half space to each neighbor
            let left_space = self.positions[index] - self.positions[index - 1];
            let right_space = self.positions[index + 1] - self.positions[index];
            (left_space + right_space) / 2.0
        };
        
        available_space * base_width_ratio
    }
}