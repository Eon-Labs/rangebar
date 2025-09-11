//! Visual styling and color schemes for range bar charts

use plotters::style::RGBColor;

/// Color scheme for range bar visualization
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Color for bullish (up) bars
    pub bullish: RGBColor,
    /// Color for bearish (down) bars  
    pub bearish: RGBColor,
    /// Background color
    pub background: RGBColor,
    /// Grid line color
    pub grid: RGBColor,
    /// Text color
    pub text: RGBColor,
    /// Border color for bars
    pub border: RGBColor,
    /// Highlight color for selected bars
    pub highlight: RGBColor,
}

impl ColorScheme {
    /// Traditional green/red color scheme
    pub fn traditional() -> Self {
        Self {
            bullish: RGBColor(34, 139, 34),   // Forest Green
            bearish: RGBColor(220, 20, 60),   // Crimson
            background: RGBColor(248, 248, 255), // Ghost White
            grid: RGBColor(192, 192, 192),    // Silver
            text: RGBColor(47, 79, 79),       // Dark Slate Gray
            border: RGBColor(105, 105, 105),  // Dim Gray
            highlight: RGBColor(255, 215, 0), // Gold
        }
    }
    
    /// Dark theme color scheme
    pub fn dark() -> Self {
        Self {
            bullish: RGBColor(0, 200, 83),    // Bright Green
            bearish: RGBColor(255, 77, 77),   // Bright Red
            background: RGBColor(21, 23, 25), // Very Dark Gray
            grid: RGBColor(64, 68, 75),       // Dark Gray
            text: RGBColor(208, 210, 214),    // Light Gray
            border: RGBColor(128, 128, 128),  // Gray
            highlight: RGBColor(255, 193, 7), // Amber
        }
    }
    
    /// Professional blue theme
    pub fn professional() -> Self {
        Self {
            bullish: RGBColor(70, 130, 180),  // Steel Blue
            bearish: RGBColor(178, 34, 34),   // Fire Brick
            background: RGBColor(240, 248, 255), // Alice Blue
            grid: RGBColor(176, 196, 222),    // Light Steel Blue
            text: RGBColor(25, 25, 112),      // Midnight Blue
            border: RGBColor(119, 136, 153),  // Light Slate Gray
            highlight: RGBColor(255, 140, 0), // Dark Orange
        }
    }
    
    /// High contrast color scheme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            bullish: RGBColor(0, 255, 0),     // Lime
            bearish: RGBColor(255, 0, 0),     // Red
            background: RGBColor(255, 255, 255), // White
            grid: RGBColor(0, 0, 0),          // Black
            text: RGBColor(0, 0, 0),          // Black
            border: RGBColor(0, 0, 0),        // Black
            highlight: RGBColor(255, 255, 0), // Yellow
        }
    }
}

/// Visual styling configuration for range bars
#[derive(Debug, Clone)]
pub struct RangeBarStyle {
    /// Color scheme to use
    pub colors: ColorScheme,
    /// Bar width relative to available space (0.1 to 1.0)
    pub bar_width_ratio: f64,
    /// Border width in pixels
    pub border_width: f32,
    /// Whether to show borders around bars
    pub show_borders: bool,
    /// Opacity for bar fills (0.0 to 1.0)
    pub fill_opacity: f64,
    /// Whether to show volume as bar width variation
    pub volume_weighted_width: bool,
    /// Grid line style
    pub grid_style: GridStyle,
    /// Font size for labels
    pub font_size: u32,
}

impl Default for RangeBarStyle {
    fn default() -> Self {
        Self {
            colors: ColorScheme::traditional(),
            bar_width_ratio: 0.8,
            border_width: 1.0,
            show_borders: true,
            fill_opacity: 0.8,
            volume_weighted_width: false,
            grid_style: GridStyle::default(),
            font_size: 12,
        }
    }
}

impl RangeBarStyle {
    /// Create a style optimized for range bars (vs candlesticks)
    pub fn range_bar_optimized() -> Self {
        Self {
            colors: ColorScheme::professional(),
            bar_width_ratio: 0.9,  // Slightly wider since range bars have irregular spacing
            border_width: 0.5,     // Thinner borders to reduce visual noise
            show_borders: true,
            fill_opacity: 0.7,     // Slightly more transparent to see overlaps
            volume_weighted_width: true, // Show volume information
            grid_style: GridStyle::minimal(),
            font_size: 11,
        }
    }
    
    /// Get the appropriate color for a bar based on price movement
    pub fn bar_color(&self, is_bullish: bool) -> RGBColor {
        if is_bullish {
            self.colors.bullish
        } else {
            self.colors.bearish
        }
    }
    
    /// Get color with opacity applied
    pub fn bar_color_with_opacity(&self, is_bullish: bool) -> RGBColor {
        let base_color = self.bar_color(is_bullish);
        self.apply_opacity(base_color, self.fill_opacity)
    }
    
    /// Apply opacity to a color
    fn apply_opacity(&self, color: RGBColor, opacity: f64) -> RGBColor {
        let opacity = opacity.clamp(0.0, 1.0);
        let bg = self.colors.background;
        
        // Alpha blend with background
        let r = (color.0 as f64 * opacity + bg.0 as f64 * (1.0 - opacity)) as u8;
        let g = (color.1 as f64 * opacity + bg.1 as f64 * (1.0 - opacity)) as u8;
        let b = (color.2 as f64 * opacity + bg.2 as f64 * (1.0 - opacity)) as u8;
        
        RGBColor(r, g, b)
    }
}

/// Grid line styling options
#[derive(Debug, Clone)]
pub struct GridStyle {
    /// Whether to show horizontal grid lines
    pub show_horizontal: bool,
    /// Whether to show vertical grid lines
    pub show_vertical: bool,
    /// Grid line width
    pub line_width: f32,
    /// Grid line dash pattern (empty for solid lines)
    pub dash_pattern: Vec<f32>,
    /// Grid opacity
    pub opacity: f64,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            show_horizontal: true,
            show_vertical: true,
            line_width: 0.5,
            dash_pattern: vec![2.0, 2.0], // Dashed lines
            opacity: 0.3,
        }
    }
}

impl GridStyle {
    /// Minimal grid style for clean charts
    pub fn minimal() -> Self {
        Self {
            show_horizontal: true,
            show_vertical: false, // Range bars have irregular time spacing
            line_width: 0.3,
            dash_pattern: vec![1.0, 3.0], // More subtle dashes
            opacity: 0.2,
        }
    }
    
    /// No grid lines
    pub fn none() -> Self {
        Self {
            show_horizontal: false,
            show_vertical: false,
            line_width: 0.0,
            dash_pattern: vec![],
            opacity: 0.0,
        }
    }
}