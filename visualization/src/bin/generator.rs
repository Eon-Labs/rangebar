//! Sample chart generator for visual verification
//! 
//! This binary generates test range bar charts to verify our visualization implementation.
//! It creates multiple variations to test different aspects of the charting system.

use rangebar_visualization::{
    data::{DataPreprocessor, RangeBarData},
    export::{export_png, ExportConfig, quick_export_sample},
    layouts::{ChartLayout, TimeScale},
    styles::{RangeBarStyle, ColorScheme},
    errors::Result,
};
use std::path::Path;
use std::fs;

fn main() -> Result<()> {
    println!("🎨 Range Bar Chart Generator");
    println!("============================");
    
    // Create output directory
    let output_dir = Path::new("visualization/output");
    fs::create_dir_all(output_dir)?;
    
    // Generate different test charts
    generate_basic_samples(&output_dir)?;
    generate_style_variations(&output_dir)?;
    generate_scale_comparisons(&output_dir)?;
    generate_data_variations(&output_dir)?;
    
    println!("\n✅ Chart generation completed!");
    println!("📁 Check the 'visualization/output' directory for generated PNG files");
    println!("🔍 Review each chart to verify visual appearance against expectations");
    
    Ok(())
}

/// Generate basic sample charts with different bar counts
fn generate_basic_samples(output_dir: &Path) -> Result<()> {
    println!("\n📊 Generating basic sample charts...");
    
    let test_cases = [
        (10, "01_basic_10_bars"),
        (50, "02_basic_50_bars"),
        (100, "03_basic_100_bars"),
        (500, "04_basic_500_bars"),
    ];
    
    for &(bar_count, filename) in &test_cases {
        let output_path = output_dir.join(format!("{}.png", filename));
        let title = format!("Basic Range Bars - {} Bars", bar_count);
        
        print!("  Generating {} bars... ", bar_count);
        quick_export_sample(&output_path, bar_count, &title)?;
        println!("✅ {}", output_path.display());
    }
    
    Ok(())
}

/// Generate charts with different color schemes and styles
fn generate_style_variations(output_dir: &Path) -> Result<()> {
    println!("\n🎨 Generating style variations...");
    
    let preprocessor = DataPreprocessor::default();
    let sample_data = preprocessor.generate_sample_data(100);
    
    let style_configs = [
        (ColorScheme::traditional(), "05_style_traditional"),
        (ColorScheme::dark(), "06_style_dark"),
        (ColorScheme::professional(), "07_style_professional"),
        (ColorScheme::high_contrast(), "08_style_high_contrast"),
    ];
    
    for (color_scheme, filename) in style_configs {
        let output_path = output_dir.join(format!("{}.png", filename));
        
        let mut style = RangeBarStyle::range_bar_optimized();
        style.colors = color_scheme;
        
        let config = ExportConfig {
            title: format!("Style Test - {}", filename),
            symbol: "STYLETEST".to_string(),
            style,
            ..Default::default()
        };
        
        print!("  Generating {}... ", filename);
        export_png(&sample_data, &output_path, Some(config))?;
        println!("✅ {}", output_path.display());
    }
    
    Ok(())
}

/// Generate charts comparing different time scaling methods
fn generate_scale_comparisons(output_dir: &Path) -> Result<()> {
    println!("\n⏱️  Generating time scale comparisons...");
    
    let preprocessor = DataPreprocessor::default();
    let sample_data = preprocessor.generate_sample_data(80);
    
    let scale_configs = [
        (TimeScale::Uniform, "09_scale_uniform"),
        (TimeScale::NonUniform, "10_scale_non_uniform"),
        (TimeScale::Compressed, "11_scale_compressed"),
    ];
    
    for (time_scale, filename) in scale_configs {
        let output_path = output_dir.join(format!("{}.png", filename));
        
        let config = ExportConfig {
            title: format!("Time Scale - {:?}", time_scale),
            symbol: "SCALETEST".to_string(),
            time_scale,
            ..Default::default()
        };
        
        print!("  Generating {:?} scale... ", filename);
        export_png(&sample_data, &output_path, Some(config))?;
        println!("✅ {}", output_path.display());
    }
    
    Ok(())
}

/// Generate charts with different data characteristics
fn generate_data_variations(output_dir: &Path) -> Result<()> {
    println!("\n📈 Generating data variation tests...");
    
    let preprocessor = DataPreprocessor::default();
    
    // Test edge cases
    let edge_cases = [
        (1, "12_single_bar"),
        (2, "13_two_bars"),
        (5, "14_five_bars"),
    ];
    
    for &(bar_count, filename) in &edge_cases {
        let output_path = output_dir.join(format!("{}.png", filename));
        let data = preprocessor.generate_sample_data(bar_count);
        
        let config = ExportConfig {
            title: format!("Edge Case - {} Bar(s)", bar_count),
            symbol: "EDGETEST".to_string(),
            ..Default::default()
        };
        
        print!("  Generating {} bar(s)... ", bar_count);
        export_png(&data, &output_path, Some(config))?;
        println!("✅ {}", output_path.display());
    }
    
    // Test with different chart dimensions
    let dimension_tests = [
        ((1920, 1080), "15_full_hd"),
        ((1280, 720), "16_hd"),
        ((800, 600), "17_small"),
        ((3840, 2160), "18_4k"),
    ];
    
    let test_data = preprocessor.generate_sample_data(150);
    
    for ((width, height), filename) in dimension_tests {
        let output_path = output_dir.join(format!("{}.png", filename));
        
        let mut layout = ChartLayout::default();
        layout.width = width;
        layout.height = height;
        
        let config = ExportConfig {
            title: format!("Resolution Test - {}x{}", width, height),
            symbol: "RESTEST".to_string(),
            layout,
            ..Default::default()
        };
        
        print!("  Generating {}x{}... ", width, height);
        export_png(&test_data, &output_path, Some(config))?;
        println!("✅ {}", output_path.display());
    }
    
    Ok(())
}

/// Print analysis of generated files for verification
fn print_verification_guide() {
    println!("\n🔍 VISUAL VERIFICATION GUIDE");
    println!("============================");
    println!();
    println!("Review each generated PNG file and verify:");
    println!();
    println!("📊 BASIC SAMPLES (01-04):");
    println!("  ✓ Bars are properly formed with body + wicks");
    println!("  ✓ Green bars have close > open, red bars have close < open");
    println!("  ✓ Chart scaling looks appropriate");
    println!("  ✓ No overlapping or missing bars");
    println!();
    println!("🎨 STYLE VARIATIONS (05-08):");
    println!("  ✓ Colors match expected schemes");
    println!("  ✓ Background and text are readable");
    println!("  ✓ Grid lines are visible but not overwhelming");
    println!();
    println!("⏱️  TIME SCALE TESTS (09-11):");
    println!("  ✓ Uniform: bars evenly spaced");
    println!("  ✓ Non-uniform: variable spacing based on time gaps");
    println!("  ✓ Compressed: large gaps are visually compressed");
    println!();
    println!("📈 DATA VARIATIONS (12-18):");
    println!("  ✓ Edge cases render without errors");
    println!("  ✓ Different resolutions maintain quality");
    println!("  ✓ Single bars don't crash the system");
    println!();
    println!("❌ COMMON ISSUES TO WATCH FOR:");
    println!("  • Bars appearing as lines instead of rectangles");
    println!("  • Incorrect color mapping (bullish/bearish)");
    println!("  • Missing wicks or body components");
    println!("  • Text overlapping or being cut off");
    println!("  • Axis labels not matching data ranges");
}