//! Range Bar Visualization Library
//! 
//! Provides comprehensive visualization capabilities for range bar data including:
//! - Static chart generation with PNG/SVG export
//! - Interactive real-time charts with egui
//! - Statistical analysis overlays
//! - Comparison with traditional candlestick charts
//! 
//! # Architecture
//! 
//! This library is designed with clear separation of concerns:
//! - `elements`: Custom drawing elements for range bars
//! - `layouts`: Chart layout and spacing management  
//! - `styles`: Color schemes and visual styling
//! - `data`: Data preparation and transformation
//! - `export`: Static image export functionality
//! - `interactive`: Real-time interactive charts
//! - `analysis`: Statistical visualization overlays

pub mod elements;
pub mod layouts;
pub mod styles;
pub mod data;
pub mod export;
pub mod interactive;
pub mod analysis;
pub mod errors;

// Re-export key types for convenience
pub use elements::{RangeBarElement, RangeBarSeries};
pub use layouts::{ChartLayout, TimeScale};
pub use styles::{RangeBarStyle, ColorScheme};
pub use data::{RangeBarData, DataPreprocessor};
pub use export::{PngExporter, SvgExporter, ChartExporter};
pub use errors::{VisualizationError, Result};

/// Version of the visualization library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default chart dimensions for export
pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;

/// Default DPI for high-quality exports
pub const DEFAULT_DPI: u32 = 300;