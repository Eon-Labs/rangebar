//! Chart export functionality for static image generation

use crate::data::{RangeBarData, DataPreprocessor};
use crate::elements::RangeBarSeries;
use crate::layouts::{ChartLayout, TimePositionCalculator, TimeScale};
use crate::styles::RangeBarStyle;
use crate::errors::{Result, VisualizationError};
use plotters::prelude::*;
use std::path::Path;

/// Trait for chart export backends
pub trait ChartExporter {
    /// Export a range bar chart to the specified path
    fn export_chart(
        &self,
        data: &[RangeBarData],
        output_path: &Path,
        config: &ExportConfig,
    ) -> Result<()>;
    
    /// Get the file extension for this export format
    fn file_extension(&self) -> &'static str;
}

/// Configuration for chart export
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Chart layout configuration
    pub layout: ChartLayout,
    /// Visual style configuration
    pub style: RangeBarStyle,
    /// Chart title
    pub title: String,
    /// Symbol name (e.g., "BTCUSDT")
    pub symbol: String,
    /// Threshold in basis points
    pub threshold_bps: u32,
    /// Time scale method
    pub time_scale: TimeScale,
    /// Whether to include volume panel
    pub include_volume: bool,
    /// Whether to include statistical overlays
    pub include_statistics: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            layout: ChartLayout::default(),
            style: RangeBarStyle::range_bar_optimized(),
            title: "Range Bar Chart".to_string(),
            symbol: "UNKNOWN".to_string(),
            threshold_bps: 8000,
            time_scale: TimeScale::NonUniform,
            include_volume: false,
            include_statistics: false,
        }
    }
}

/// PNG export implementation
pub struct PngExporter;

impl ChartExporter for PngExporter {
    fn export_chart(
        &self,
        data: &[RangeBarData],
        output_path: &Path,
        config: &ExportConfig,
    ) -> Result<()> {
        // Create drawing backend
        let root = BitMapBackend::new(output_path, (config.layout.width, config.layout.height))
            .into_drawing_area();
        
        root.fill(&config.style.colors.background)?;
        
        // Calculate chart area
        let (chart_left, chart_top, chart_right, chart_bottom) = config.layout.chart_area();
        let chart_area = root.margin(
            chart_top,
            config.layout.margins.right, 
            config.layout.height - chart_bottom,
            chart_left
        );
        
        // Calculate price range with some padding
        if data.is_empty() {
            return self.export_empty_chart(&root, config);
        }
        
        let price_range = self.calculate_price_range(data);
        let price_padding = (price_range.1 - price_range.0) * 0.05; // 5% padding
        let price_min = price_range.0 - price_padding;
        let price_max = price_range.1 + price_padding;
        
        // Create time position calculator
        let calculator = TimePositionCalculator::new(
            data,
            0.0,
            (chart_right - chart_left) as f64,
            config.time_scale.clone(),
        )?;
        
        // Create chart context
        let mut chart = ChartBuilder::on(&chart_area)
            .caption(&config.title, ("Arial", 24).into_font().color(&config.style.colors.text))
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(
                0.0..(chart_right - chart_left) as f64,
                price_min..price_max
            )?;
        
        // Configure chart mesh/grid
        chart.configure_mesh()
            .x_desc("Bar Index")
            .y_desc("Price")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.2}", y))
            .axis_desc_style(("Arial", 12).into_font().color(&config.style.colors.text))
            .label_style(("Arial", 10).into_font().color(&config.style.colors.text))
            .draw()?;
        
        // Create range bar series
        let mut series = RangeBarSeries::new(data, &calculator, config.style.clone())?;
        series.set_metadata(config.symbol.clone(), config.threshold_bps);
        
        // Draw range bars manually using plotters API
        let drawing_data = series.get_all_drawing_data();
        for bar_data in drawing_data {
            // Draw body rectangle
            let (left, top, right, bottom) = bar_data.body_rect;
            let body_rect = Rectangle::new([(left, bottom), (right, top)], bar_data.fill_color.filled());
            chart.draw_series(std::iter::once(body_rect)).map_err(|e| VisualizationError::RenderingError { message: format!("Failed to draw bar body: {}", e) })?;
            
            // Draw border if enabled
            if bar_data.show_border {
                let border_rect = Rectangle::new([(left, bottom), (right, top)], bar_data.border_color.stroke_width(bar_data.border_width as u32));
                chart.draw_series(std::iter::once(border_rect)).map_err(|e| VisualizationError::RenderingError { message: format!("Failed to draw bar border: {}", e) })?;
            }
            
            // Draw wicks
            let ((uw_x1, uw_y1), (uw_x2, uw_y2)) = bar_data.upper_wick;
            let ((lw_x1, lw_y1), (lw_x2, lw_y2)) = bar_data.lower_wick;
            
            // Upper wick
            if uw_y2 > uw_y1 {
                let upper_wick = PathElement::new(vec![(uw_x1, uw_y1), (uw_x2, uw_y2)], bar_data.border_color.stroke_width(1));
                chart.draw_series(std::iter::once(upper_wick)).map_err(|e| VisualizationError::RenderingError { message: format!("Failed to draw upper wick: {}", e) })?;
            }
            
            // Lower wick
            if lw_y1 < lw_y2 {
                let lower_wick = PathElement::new(vec![(lw_x1, lw_y1), (lw_x2, lw_y2)], bar_data.border_color.stroke_width(1));
                chart.draw_series(std::iter::once(lower_wick)).map_err(|e| VisualizationError::RenderingError { message: format!("Failed to draw lower wick: {}", e) })?;
            }
        }
        
        // Add statistics if enabled (temporarily commented out to avoid compilation issues)
        // if config.include_statistics {
        //     self.add_statistics_overlay(&mut chart, &series, config)?;
        // }
        
        // Finalize the chart
        root.present()?;
        
        Ok(())
    }
    
    fn file_extension(&self) -> &'static str {
        "png"
    }
}

impl PngExporter {
    /// Export an empty chart with appropriate message
    fn export_empty_chart<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        config: &ExportConfig,
    ) -> Result<()> 
    where
        DB::ErrorType: 'static,
    {
        let text_style = ("Arial", 24).into_font().color(&config.style.colors.text);
        
        root.draw(&Text::new(
            "No Data Available",
            (config.layout.width as i32 / 2, config.layout.height as i32 / 2),
            text_style,
        ))?;
        
        root.present()?;
        Ok(())
    }
    
    /// Calculate the price range for the chart
    fn calculate_price_range(&self, data: &[RangeBarData]) -> (f64, f64) {
        data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |acc, bar| {
            (acc.0.min(bar.low), acc.1.max(bar.high))
        })
    }
    
    /// Add statistical overlay to the chart
    fn add_statistics_overlay<DB: DrawingBackend, CT: CoordTranslate>(
        &self,
        chart: &mut ChartContext<DB, CT>,
        series: &RangeBarSeries,
        config: &ExportConfig,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let stats = series.statistics();
        
        // Create statistics text
        let stats_text = format!(
            "Bars: {} | Bullish: {} | Bearish: {} | Avg Range: {:.2}",
            stats.total_bars,
            stats.bullish_bars,
            stats.bearish_bars,
            stats.average_range
        );
        
        // Position statistics in top-right corner (simplified)
        // Note: This is a basic implementation - real positioning would need coordinate transformation
        println!("Statistics: {}", stats_text); // For now, just print
        
        Ok(())
    }
}

/// SVG export implementation
pub struct SvgExporter;

impl ChartExporter for SvgExporter {
    fn export_chart(
        &self,
        data: &[RangeBarData],
        output_path: &Path,
        config: &ExportConfig,
    ) -> Result<()> {
        // Create SVG backend
        let root = SVGBackend::new(output_path, (config.layout.width, config.layout.height))
            .into_drawing_area();
        
        // Similar implementation to PNG but with SVG backend
        // For now, delegate to PNG logic with different backend
        root.fill(&config.style.colors.background)?;
        
        if data.is_empty() {
            let text_style = ("Arial", 24).into_font().color(&config.style.colors.text);
            root.draw(&Text::new(
                "No Data Available",
                (config.layout.width as i32 / 2, config.layout.height as i32 / 2),
                text_style,
            ))?;
            root.present()?;
            return Ok(());
        }
        
        // Calculate chart area and implement SVG-specific rendering
        // This is a simplified version - full implementation would mirror PNG exporter
        let (chart_left, chart_top, chart_right, chart_bottom) = config.layout.chart_area();
        let chart_area = root.margin(
            chart_top,
            config.layout.margins.right,
            config.layout.height - chart_bottom,
            chart_left
        );
        
        // Basic SVG chart implementation
        let price_range = data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |acc, bar| {
            (acc.0.min(bar.low), acc.1.max(bar.high))
        });
        
        let price_padding = (price_range.1 - price_range.0) * 0.05;
        let price_min = price_range.0 - price_padding;
        let price_max = price_range.1 + price_padding;
        
        let mut chart = ChartBuilder::on(&chart_area)
            .caption(&config.title, ("Arial", 24))
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(
                0.0..(chart_right - chart_left) as f64,
                price_min..price_max
            )?;
        
        chart.configure_mesh()
            .x_desc("Bar Index")
            .y_desc("Price")
            .draw()?;
        
        root.present()?;
        Ok(())
    }
    
    fn file_extension(&self) -> &'static str {
        "svg"
    }
}

/// Convenience function to export a range bar chart to PNG
pub fn export_png<P: AsRef<Path>>(
    data: &[RangeBarData],
    output_path: P,
    config: Option<ExportConfig>,
) -> Result<()> {
    let exporter = PngExporter;
    let config = config.unwrap_or_default();
    exporter.export_chart(data, output_path.as_ref(), &config)
}

/// Convenience function to export a range bar chart to SVG
pub fn export_svg<P: AsRef<Path>>(
    data: &[RangeBarData],
    output_path: P,
    config: Option<ExportConfig>,
) -> Result<()> {
    let exporter = SvgExporter;
    let config = config.unwrap_or_default();
    exporter.export_chart(data, output_path.as_ref(), &config)
}

/// Quick export function for testing and development
pub fn quick_export_sample<P: AsRef<Path>>(
    output_path: P,
    bar_count: usize,
    title: &str,
) -> Result<()> {
    let preprocessor = DataPreprocessor::default();
    let sample_data = preprocessor.generate_sample_data(bar_count);
    
    let config = ExportConfig {
        title: title.to_string(),
        symbol: "SAMPLE".to_string(),
        threshold_bps: 8000,
        ..Default::default()
    };
    
    export_png(&sample_data, output_path, Some(config))
}