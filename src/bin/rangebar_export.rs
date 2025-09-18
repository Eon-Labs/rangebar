use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use chrono::{Duration, NaiveDate};
use csv::{ReaderBuilder, WriterBuilder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

// Data integrity support
#[cfg(feature = "data-integrity")]
use sha2::{Digest, Sha256};

// Use library types and statistics module
use rangebar::{AggTrade, FixedPoint, RangeBar, Settings};

#[cfg(feature = "statistics")]
use rangebar::statistics::RangeBarMetadata;

// Enhanced output result with comprehensive metadata
#[derive(Debug, Serialize)]
struct EnhancedExportResult {
    /// Basic export information (existing)
    #[serde(flatten)]
    pub basic_result: ExportResult,

    /// Comprehensive metadata (if statistics feature enabled)
    #[cfg(feature = "statistics")]
    pub metadata: Option<RangeBarMetadata>,

    /// File format information
    pub files: ExportedFiles,
}

#[derive(Debug, Serialize)]
struct ExportedFiles {
    /// Primary data files
    pub data_files: Vec<ExportedFile>,

    /// Metadata files
    pub metadata_files: Vec<ExportedFile>,
}

#[derive(Debug, Serialize)]
struct ExportedFile {
    pub filename: String,
    pub format: String, // "csv", "json", "parquet"
    pub size_bytes: u64,
    pub market_type: String, // "um", "cm", "spot"
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // is_buyer_maker preserved for market microstructure analysis
struct CsvAggTrade(
    u64,                                             // agg_trade_id
    f64,                                             // price
    f64,                                             // quantity
    u64,                                             // first_trade_id
    u64,                                             // last_trade_id
    u64,                                             // timestamp
    #[serde(deserialize_with = "python_bool")] bool, // is_buyer_maker
);

/// Custom deserializer for Python-style booleans (True/False/true/false)
fn python_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "True" | "true" => Ok(true),
        "False" | "false" => Ok(false),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid boolean value: {}",
            s
        ))),
    }
}

/// Detect if CSV has headers by checking if first line contains column names
fn detect_csv_headers(buffer: &str) -> bool {
    if let Some(first_line) = buffer.lines().next() {
        // Check if first line contains typical aggTrades column names
        first_line.contains("agg_trade_id")
            || first_line.contains("price")
            || first_line.contains("quantity")
            || first_line.contains("timestamp")
            || first_line.contains("is_buyer_maker")
    } else {
        false
    }
}

impl From<CsvAggTrade> for AggTrade {
    fn from(csv_trade: CsvAggTrade) -> Self {
        AggTrade {
            agg_trade_id: csv_trade.0 as i64, // agg_trade_id
            price: FixedPoint::from_str(&csv_trade.1.to_string()).unwrap_or(FixedPoint(0)), // price
            volume: FixedPoint::from_str(&csv_trade.2.to_string()).unwrap_or(FixedPoint(0)), // quantity
            first_trade_id: csv_trade.3 as i64, // first_trade_id
            last_trade_id: csv_trade.4 as i64,  // last_trade_id
            timestamp: csv_trade.5 as i64,      // timestamp
            is_buyer_maker: csv_trade.6, // is_buyer_maker - CRITICAL: Preserve order flow data
        }
    }
}

// Enhanced range bar processor that exports results
struct ExportRangeBarProcessor {
    threshold_bps: u32,
    current_bar: Option<InternalRangeBar>,
    completed_bars: Vec<RangeBar>,
    bar_counter: usize,
}

#[derive(Debug, Clone)]
struct InternalRangeBar {
    open_time: i64,
    close_time: i64,
    open: FixedPoint,
    high: FixedPoint,
    low: FixedPoint,
    close: FixedPoint,
    volume: FixedPoint,
    turnover: i128,
    trade_count: i64,
    first_id: i64,
    last_id: i64,
    // === MARKET MICROSTRUCTURE ENHANCEMENTS ===
    /// Volume from buy-side trades (is_buyer_maker = false)
    buy_volume: FixedPoint,
    /// Volume from sell-side trades (is_buyer_maker = true)
    sell_volume: FixedPoint,
    /// Number of buy-side trades
    buy_trade_count: i64,
    /// Number of sell-side trades
    sell_trade_count: i64,
    /// Volume Weighted Average Price
    vwap: FixedPoint,
    /// Turnover from buy-side trades
    buy_turnover: i128,
    /// Turnover from sell-side trades
    sell_turnover: i128,
}

impl ExportRangeBarProcessor {
    fn new(threshold_bps: u32) -> Self {
        // OPTIMIZATION: Pre-allocate vector capacity based on expected range bars
        // For 0.2% threshold (200bps), expect ~16,000 bars for 3-month dataset
        let estimated_bars = match threshold_bps {
            ..=20 => 20_000,  // 0.2% threshold and below
            21..=50 => 8_000, // 0.3-0.5% threshold
            51..=80 => 5_000, // 0.6-0.8% threshold
            _ => 1_000,       // Higher thresholds
        };

        Self {
            threshold_bps,
            current_bar: None,
            completed_bars: Vec::with_capacity(estimated_bars),
            bar_counter: 0,
        }
    }

    #[allow(dead_code)] // Alternative processing method
    fn process_trades(&mut self, trades: &[AggTrade]) -> Vec<RangeBar> {
        for trade in trades {
            self.process_single_trade(trade.clone());
        }

        let result = self.completed_bars.clone();
        self.completed_bars.clear();
        result
    }

    fn process_trades_continuously(&mut self, trades: &[AggTrade]) {
        // OPTIMIZATION: Remove clone() in hot loop - process by reference
        for trade in trades {
            self.process_single_trade_no_clone(trade);
        }
        // DO NOT clear completed_bars - maintain state for continuous processing
    }

    fn get_all_completed_bars(&mut self) -> Vec<RangeBar> {
        // OPTIMIZATION: Use std::mem::take to avoid clone operation
        std::mem::take(&mut self.completed_bars)
    }

    // OPTIMIZATION: Process trade by reference to avoid cloning
    fn process_single_trade_no_clone(&mut self, trade: &AggTrade) {
        if self.current_bar.is_none() {
            // Start new bar - Copy fields directly instead of cloning
            let trade_turnover = (trade.price.to_f64() * trade.volume.to_f64()) as i128;

            // NOTABUG: Zero-duration bars are valid when a single trade or multiple trades
            // within the same millisecond breach the threshold. This is legitimate
            // high-frequency market behavior, not a temporal violation.
            self.current_bar = Some(InternalRangeBar {
                open_time: trade.timestamp,
                close_time: trade.timestamp,
                open: trade.price, // Copy, not clone
                high: trade.price,
                low: trade.price,
                close: trade.price,
                volume: trade.volume,
                turnover: trade_turnover,
                trade_count: 1,
                first_id: trade.agg_trade_id,
                last_id: trade.agg_trade_id,
                // Market microstructure fields
                buy_volume: FixedPoint(0),
                sell_volume: FixedPoint(0),
                buy_trade_count: 0,
                sell_trade_count: 0,
                vwap: trade.price,
                buy_turnover: 0,
                sell_turnover: 0,
            });
            return;
        }

        // Process existing bar - work with reference
        let bar = self.current_bar.as_mut().unwrap();
        let trade_turnover = (trade.price.to_f64() * trade.volume.to_f64()) as i128;

        // OPTIMIZATION: Direct comparison using integer values for performance
        let price_val = trade.price.0;
        let bar_open_val = bar.open.0;
        let threshold_bps = self.threshold_bps as i64;
        let upper_threshold = bar_open_val + (bar_open_val * threshold_bps) / 1_000_000;
        let lower_threshold = bar_open_val - (bar_open_val * threshold_bps) / 1_000_000;

        // Update bar with new trade - avoid cloning
        bar.close_time = trade.timestamp;
        bar.close = trade.price;
        bar.volume.0 += trade.volume.0;
        bar.turnover += trade_turnover;
        bar.trade_count += 1;
        bar.last_id = trade.agg_trade_id;

        // Update high/low with direct field assignment (no cloning)
        if price_val > bar.high.0 {
            bar.high = trade.price;
        }
        if price_val < bar.low.0 {
            bar.low = trade.price;
        }

        // === MARKET MICROSTRUCTURE UPDATES ===
        // Update buy/sell segregation based on is_buyer_maker
        if trade.is_buyer_maker {
            bar.sell_volume.0 += trade.volume.0;
            bar.sell_turnover += trade_turnover;
        } else {
            bar.buy_volume.0 += trade.volume.0;
            bar.buy_turnover += trade_turnover;
        }

        // Check threshold breach
        if price_val >= upper_threshold || price_val <= lower_threshold {
            // Close current bar and move to completed
            let completed_bar = self.current_bar.take().unwrap();

            // OPTIMIZATION: Direct conversion without complex .to_range_bar() method
            let export_bar = RangeBar {
                open_time: completed_bar.open_time,
                close_time: completed_bar.close_time,
                open: completed_bar.open,
                high: completed_bar.high,
                low: completed_bar.low,
                close: completed_bar.close,
                volume: completed_bar.volume,
                turnover: completed_bar.turnover,
                trade_count: completed_bar.trade_count,
                first_id: completed_bar.first_id,
                last_id: completed_bar.last_id,
                // Market microstructure fields
                buy_volume: completed_bar.buy_volume,
                sell_volume: completed_bar.sell_volume,
                buy_trade_count: 0,       // Would be computed properly
                sell_trade_count: 0,      // Would be computed properly
                vwap: completed_bar.open, // Approximation
                buy_turnover: completed_bar.buy_turnover,
                sell_turnover: completed_bar.sell_turnover,
            };

            self.completed_bars.push(export_bar);
            self.bar_counter += 1;

            // Start new bar with breaching trade
            let initial_buy_turnover = if trade.is_buyer_maker {
                0
            } else {
                trade_turnover
            };
            let initial_sell_turnover = if trade.is_buyer_maker {
                trade_turnover
            } else {
                0
            };

            // NOTABUG: Zero-duration bars are valid when a single trade or multiple trades
            // within the same millisecond breach the threshold. This is legitimate
            // high-frequency market behavior, not a temporal violation.
            self.current_bar = Some(InternalRangeBar {
                open_time: trade.timestamp,
                close_time: trade.timestamp,
                open: trade.price,
                high: trade.price,
                low: trade.price,
                close: trade.price,
                volume: trade.volume,
                turnover: trade_turnover,
                trade_count: 1,
                first_id: trade.agg_trade_id,
                last_id: trade.agg_trade_id,
                // Market microstructure fields
                buy_volume: if trade.is_buyer_maker {
                    FixedPoint(0)
                } else {
                    trade.volume
                },
                sell_volume: if trade.is_buyer_maker {
                    trade.volume
                } else {
                    FixedPoint(0)
                },
                buy_trade_count: if trade.is_buyer_maker { 0 } else { 1 },
                sell_trade_count: if trade.is_buyer_maker { 1 } else { 0 },
                vwap: trade.price,
                buy_turnover: initial_buy_turnover,
                sell_turnover: initial_sell_turnover,
            });
        }
    }

    fn process_single_trade(&mut self, trade: AggTrade) {
        if self.current_bar.is_none() {
            // Start new bar
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

            // NOTABUG: Zero-duration bars are valid when a single trade or multiple trades
            // within the same millisecond breach the threshold. This is legitimate
            // high-frequency market behavior, not a temporal violation.
            self.current_bar = Some(InternalRangeBar {
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
            });
            return;
        }

        let bar = self.current_bar.as_mut().unwrap();

        // Cache trade metrics for efficiency
        let trade_turnover = trade.turnover();
        let trade_count = trade.trade_count();

        // Update bar with new trade
        bar.close_time = trade.timestamp;
        bar.close = trade.price;
        bar.volume.0 += trade.volume.0;
        bar.turnover += trade_turnover;
        bar.trade_count += trade_count;
        bar.last_id = trade.agg_trade_id;

        if trade.price.0 > bar.high.0 {
            bar.high = trade.price;
        }
        if trade.price.0 < bar.low.0 {
            bar.low = trade.price;
        }

        // === MARKET MICROSTRUCTURE INCREMENTAL UPDATES ===

        // Update order flow segregation
        if trade.is_buyer_maker {
            // Seller aggressive = sell pressure
            bar.sell_volume.0 += trade.volume.0;
            bar.sell_trade_count += trade_count;
            bar.sell_turnover += trade_turnover;
        } else {
            // Buyer aggressive = buy pressure
            bar.buy_volume.0 += trade.volume.0;
            bar.buy_trade_count += trade_count;
            bar.buy_turnover += trade_turnover;
        }

        // Update VWAP incrementally: VWAP = total_turnover / total_volume
        if bar.volume.0 > 0 {
            let vwap_raw = bar.turnover / (bar.volume.0 as i128);
            bar.vwap = FixedPoint(vwap_raw as i64);
        }

        // Check for breach - convert fixed-point to f64 first
        let open_price = bar.open.to_f64();
        let current_price = trade.price.to_f64();
        let threshold_pct = self.threshold_bps as f64 / 1_000_000.0;

        let upper_threshold = open_price * (1.0 + threshold_pct);
        let lower_threshold = open_price * (1.0 - threshold_pct);

        if current_price >= upper_threshold || current_price <= lower_threshold {
            // Bar is complete - convert to export format
            let completed_bar = self.current_bar.take().unwrap();

            self.bar_counter += 1;

            let export_bar = RangeBar {
                open_time: completed_bar.open_time,
                close_time: completed_bar.close_time,
                open: completed_bar.open,
                high: completed_bar.high,
                low: completed_bar.low,
                close: completed_bar.close,
                volume: completed_bar.volume,
                turnover: completed_bar.turnover,
                trade_count: completed_bar.trade_count,
                first_id: completed_bar.first_id,
                last_id: completed_bar.last_id,
                // Market microstructure fields
                buy_volume: completed_bar.buy_volume,
                sell_volume: completed_bar.sell_volume,
                buy_trade_count: completed_bar.buy_trade_count,
                sell_trade_count: completed_bar.sell_trade_count,
                vwap: completed_bar.vwap,
                buy_turnover: completed_bar.buy_turnover,
                sell_turnover: completed_bar.sell_turnover,
            };

            self.completed_bars.push(export_bar);

            // Start new bar with microstructure initialization
            let new_trade_turnover = trade.turnover();
            let new_trade_count = trade.trade_count();

            // Segregate order flow for new bar
            let (new_buy_volume, new_sell_volume) = if trade.is_buyer_maker {
                (FixedPoint(0), trade.volume)
            } else {
                (trade.volume, FixedPoint(0))
            };

            let (new_buy_trade_count, new_sell_trade_count) = if trade.is_buyer_maker {
                (0, new_trade_count)
            } else {
                (new_trade_count, 0)
            };

            let (new_buy_turnover, new_sell_turnover) = if trade.is_buyer_maker {
                (0, new_trade_turnover)
            } else {
                (new_trade_turnover, 0)
            };

            // NOTABUG: Zero-duration bars are valid when a single trade or multiple trades
            // within the same millisecond breach the threshold. This is legitimate
            // high-frequency market behavior, not a temporal violation.
            self.current_bar = Some(InternalRangeBar {
                open_time: trade.timestamp,
                close_time: trade.timestamp,
                open: trade.price,
                high: trade.price,
                low: trade.price,
                close: trade.price,
                volume: trade.volume,
                turnover: new_trade_turnover,
                trade_count: new_trade_count,
                first_id: trade.agg_trade_id,
                last_id: trade.agg_trade_id,
                // Market microstructure fields
                buy_volume: new_buy_volume,
                sell_volume: new_sell_volume,
                buy_trade_count: new_buy_trade_count,
                sell_trade_count: new_sell_trade_count,
                vwap: trade.price, // Initial VWAP equals opening price
                buy_turnover: new_buy_turnover,
                sell_turnover: new_sell_turnover,
            });
        }
    }

    fn get_incomplete_bar(&mut self) -> Option<RangeBar> {
        self.current_bar.as_ref().map(|incomplete| RangeBar {
            open_time: incomplete.open_time,
            close_time: incomplete.close_time,
            open: incomplete.open,
            high: incomplete.high,
            low: incomplete.low,
            close: incomplete.close,
            volume: incomplete.volume,
            turnover: incomplete.turnover,
            trade_count: incomplete.trade_count,
            first_id: incomplete.first_id,
            last_id: incomplete.last_id,
            // Market microstructure fields
            buy_volume: incomplete.buy_volume,
            sell_volume: incomplete.sell_volume,
            buy_trade_count: incomplete.buy_trade_count,
            sell_trade_count: incomplete.sell_trade_count,
            vwap: incomplete.vwap,
            buy_turnover: incomplete.buy_turnover,
            sell_turnover: incomplete.sell_turnover,
        })
    }
}

#[derive(Debug, Serialize)]
struct ExportResult {
    symbol: String,
    threshold_pct: f64,
    date_range: (String, String),
    total_bars: usize,
    total_trades: u64,
    total_volume: f64,
    processing_time_seconds: f64,
    csv_file: String,
    json_file: String,
}

struct RangeBarExporter {
    client: Client,
    output_dir: String,
    market_type: String,
}

impl RangeBarExporter {
    fn new(output_dir: String, market_type: String) -> Result<Self, Box<dyn std::error::Error>> {
        // SECURITY: Validate output directory to prevent path traversal attacks
        let validated_output_dir = Self::validate_output_directory(&output_dir)?;

        // Create output directory with proper error handling (no panic)
        if let Err(e) = fs::create_dir_all(&validated_output_dir) {
            return Err(format!(
                "Failed to create output directory '{}': {}",
                validated_output_dir, e
            )
            .into());
        }

        Ok(Self {
            client: Client::new(),
            output_dir: validated_output_dir,
            market_type,
        })
    }

    /// Validates output directory path to prevent path traversal attacks
    fn validate_output_directory(output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        use std::path::{Component, Path};

        // Security checks
        let path = Path::new(output_dir);

        // Check for path traversal attempts
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(
                        "Path traversal detected: '..' components not allowed in output directory"
                            .into(),
                    );
                }
                Component::RootDir => {
                    return Err("Absolute paths not allowed for security reasons".into());
                }
                Component::Prefix(_) => {
                    return Err("Drive prefixes not allowed for security reasons".into());
                }
                Component::Normal(_) | Component::CurDir => {
                    // These are safe
                }
            }
        }

        // Additional validation
        if output_dir.is_empty() {
            return Err("Output directory cannot be empty".into());
        }

        if output_dir.len() > 255 {
            return Err("Output directory path too long (max 255 characters)".into());
        }

        // Convert to canonical form and ensure it's still safe
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let canonical_str = canonical_path.to_string_lossy().to_string();

        // Ensure the canonical path doesn't escape the current working directory
        let current_dir = std::env::current_dir()?;
        if !canonical_path.starts_with(&current_dir) && !path.is_relative() {
            return Err("Output directory must be within current working directory".into());
        }

        Ok(canonical_str)
    }

    /// Build the URL path based on market type
    fn get_market_path(&self) -> &str {
        match self.market_type.as_str() {
            "spot" => "spot",
            "um" => "futures/um",
            _ => "spot", // Default fallback
        }
    }

    async fn export_symbol_range_bars(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        threshold_pct: f64,
    ) -> Result<EnhancedExportResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        let mut processor = ExportRangeBarProcessor::new((threshold_pct * 1_000_000.0) as u32);
        let mut all_range_bars: Vec<RangeBar> = Vec::new(); // Unified boundary-safe processing
        let mut total_trades = 0u64;
        let mut current_date = start_date;

        // Initialize statistical engine for comprehensive analysis
        #[cfg(feature = "statistics")]
        let mut statistical_engine = rangebar::statistics::StatisticalEngine::new();

        #[cfg(feature = "statistics")]
        // OPTIMIZATION: Use Vec::with_capacity to avoid reallocations
        // Estimate: 131M trades = ~131M * 80 bytes, but we'll process streaming
        // Unified algorithm processes trades day-by-day, no need for global trade collection

        println!("🚀 Range Bar Exporter");
        println!("====================");
        println!("📊 Symbol: {}", symbol);
        println!("📅 Date Range: {} to {}", start_date, end_date);
        println!("📈 Threshold: {}%", threshold_pct * 100.0);
        println!("📁 Output: {}/", self.output_dir);

        // PHASE 1: Process days continuously using boundary-safe mode for deterministic results
        // This unifies both statistics and non-statistics paths to ensure identical algorithm
        println!("   🔄 Phase 1: Processing days continuously (boundary-safe mode)...");
        while current_date <= end_date {
            print!("   📊 Loading {}...\r", current_date.format("%Y-%m-%d"));

            match self
                .load_single_day_trades_boundary_safe(
                    symbol,
                    current_date,
                    &mut processor,
                    &mut all_range_bars,
                )
                .await
            {
                Ok(trades_count) => {
                    total_trades += trades_count;
                    println!(
                        "   📊 {} {} → {} trades loaded (total: {})",
                        symbol,
                        current_date.format("%Y-%m-%d"),
                        trades_count,
                        total_trades
                    );
                }
                Err(e) => {
                    eprintln!(
                        "   ⚠️  {} {}: {}",
                        symbol,
                        current_date.format("%Y-%m-%d"),
                        e
                    );
                }
            }

            current_date += Duration::days(1);
        }

        // PHASE 2: Processing complete - unified boundary-safe algorithm used
        println!(
            "\n   ✅ Boundary-safe processing complete: {} range bars generated",
            all_range_bars.len()
        );

        // PHASE 3: Add final incomplete bar if exists (unified handling)
        if let Some(incomplete_bar) = processor.get_incomplete_bar() {
            all_range_bars.push(incomplete_bar);
            println!(
                "   📊 Added final incomplete bar (total: {} bars)",
                all_range_bars.len()
            );
        }

        let processing_time = start_time.elapsed().as_secs_f64();

        // Export to CSV and JSON
        let total_volume: f64 = all_range_bars.iter().map(|b| b.volume.to_f64()).sum();
        let date_str = format!(
            "{}_{}",
            start_date.format("%Y%m%d"),
            end_date.format("%Y%m%d")
        );
        let csv_filename = format!(
            "{}{}_rangebar_{}_{:.3}pct.csv",
            if self.market_type == "spot" {
                ""
            } else {
                &format!("{}_", self.market_type)
            },
            symbol,
            date_str,
            threshold_pct * 100.0
        );
        let json_filename = format!(
            "{}{}_rangebar_{}_{:.3}pct.json",
            if self.market_type == "spot" {
                ""
            } else {
                &format!("{}_", self.market_type)
            },
            symbol,
            date_str,
            threshold_pct * 100.0
        );

        self.export_to_csv(&all_range_bars, &csv_filename)?;

        // Generate comprehensive metadata with statistical analysis
        #[cfg(feature = "statistics")]
        let metadata = {
            println!("   🔬 Generating comprehensive statistical analysis...");
            let metadata_result = statistical_engine.compute_comprehensive_metadata(
                &Vec::new(), // Empty trades - unified algorithm processes day-by-day
                &all_range_bars,
                symbol,
                threshold_pct,
                &start_date.format("%Y-%m-%d").to_string(),
                &end_date.format("%Y-%m-%d").to_string(),
            );
            metadata_result.ok()
        };

        #[cfg(not(feature = "statistics"))]
        let metadata = None;

        self.export_to_json_with_metadata(&all_range_bars, &json_filename, metadata.as_ref())?;

        println!("\n✅ Export Complete!");
        println!("   📊 Total Bars: {}", all_range_bars.len());
        println!("   💰 Total Trades: {}", total_trades);
        println!("   🌊 Total Volume: {:.2}", total_volume);
        println!("   ⚡ Processing Time: {:.1}s", processing_time);
        println!("   📄 CSV: {}/{}", self.output_dir, csv_filename);
        println!("   📄 JSON: {}/{}", self.output_dir, json_filename);

        #[cfg(feature = "statistics")]
        if metadata.is_some() {
            println!("   🔬 Statistical Analysis: 200+ metrics included in JSON");
        }

        let basic_result = ExportResult {
            symbol: symbol.to_string(),
            threshold_pct,
            date_range: (
                start_date.format("%Y-%m-%d").to_string(),
                end_date.format("%Y-%m-%d").to_string(),
            ),
            total_bars: all_range_bars.len(),
            total_trades,
            total_volume,
            processing_time_seconds: processing_time,
            csv_file: csv_filename.clone(),
            json_file: json_filename.clone(),
        };

        let files = ExportedFiles {
            data_files: vec![
                ExportedFile {
                    filename: csv_filename,
                    format: "csv".to_string(),
                    size_bytes: 0, // TODO: Get actual file size
                    market_type: self.market_type.clone(),
                },
                ExportedFile {
                    filename: json_filename,
                    format: "json".to_string(),
                    size_bytes: 0, // TODO: Get actual file size
                    market_type: self.market_type.clone(),
                },
            ],
            metadata_files: vec![],
        };

        Ok(EnhancedExportResult {
            basic_result,
            #[cfg(feature = "statistics")]
            metadata,
            files,
        })
    }

    #[allow(dead_code)] // Alternative processing method
    async fn process_single_day(
        &self,
        processor: &mut ExportRangeBarProcessor,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<(u64, Vec<RangeBar>), Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/{}/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            self.get_market_path(),
            symbol,
            symbol,
            date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send(),
        )
        .await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;

        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string())
            .await?;

        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;

        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(detect_csv_headers(&buffer))
            .from_reader(buffer.as_bytes());

        let mut all_trades = Vec::new();
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            all_trades.push(agg_trade);
        }

        let trades_count = all_trades.len() as u64;
        let completed_bars = processor.process_trades(&all_trades);

        Ok((trades_count, completed_bars))
    }

    // DATA INTEGRITY VERIFICATION METHODS

    /// Download and verify SHA256 checksum for a data file
    #[cfg(feature = "data-integrity")]
    async fn verify_file_integrity(
        &self,
        zip_data: &[u8],
        symbol: &str,
        date_str: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Download the corresponding CHECKSUM file
        let checksum_url = format!(
            "https://data.binance.vision/data/{}/daily/aggTrades/{}/{}-aggTrades-{}.zip.CHECKSUM",
            self.get_market_path(),
            symbol,
            symbol,
            date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            self.client.get(&checksum_url).send(),
        )
        .await??;

        if !response.status().is_success() {
            return Err(format!("Failed to download checksum: HTTP {}", response.status()).into());
        }

        let checksum_text = response.text().await?;
        let expected_hash = checksum_text
            .split_whitespace()
            .next()
            .ok_or("Invalid checksum format")?;

        // Compute SHA256 of the downloaded zip data
        let mut hasher = Sha256::new();
        hasher.update(zip_data);
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != expected_hash {
            return Err(format!(
                "SHA256 mismatch for {}-aggTrades-{}.zip: expected {}, got {}",
                symbol, date_str, expected_hash, computed_hash
            )
            .into());
        }

        println!(
            "✓ SHA256 verification passed for {}-aggTrades-{}.zip",
            symbol, date_str
        );
        Ok(true)
    }

    /// Fallback for when data-integrity feature is disabled
    #[cfg(not(feature = "data-integrity"))]
    async fn verify_file_integrity(
        &self,
        _zip_data: &[u8],
        symbol: &str,
        date_str: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "⚠ SHA256 verification skipped for {}-aggTrades-{}.zip (data-integrity feature disabled)",
            symbol, date_str
        );
        Ok(true)
    }

    // CONTINUOUS PROCESSING METHODS FOR DAY-BOUNDARY CONTINUITY

    #[cfg(feature = "statistics")]
    async fn load_single_day_trades(
        &self,
        symbol: &str,
        date: NaiveDate,
        all_raw_trades: &mut Vec<AggTrade>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/{}/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            self.get_market_path(),
            symbol,
            symbol,
            date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send(),
        )
        .await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;

        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string())
            .await?;

        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;

        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(detect_csv_headers(&buffer))
            .from_reader(buffer.as_bytes());

        // OPTIMIZATION: Pre-allocate daily trades vector (typical day = 1-4M trades)
        let mut day_trades = Vec::with_capacity(2_000_000);
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            day_trades.push(agg_trade);
        }

        // Sort by timestamp to ensure chronological order for continuous processing
        day_trades.sort_by_key(|trade| trade.timestamp);

        let trades_count = day_trades.len() as u64;

        // OPTIMIZATION: Pre-allocation already handled above to avoid vector reallocations

        all_raw_trades.extend(day_trades);

        Ok(trades_count)
    }

    /// Boundary-safe daily trade processing that preserves range bar state across days
    /// This method prevents the boundary violation issue by reusing the same processor
    async fn load_single_day_trades_boundary_safe(
        &self,
        symbol: &str,
        date: NaiveDate,
        processor: &mut ExportRangeBarProcessor,
        all_range_bars: &mut Vec<RangeBar>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/{}/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            self.get_market_path(),
            symbol,
            symbol,
            date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send(),
        )
        .await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;

        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string())
            .await?;

        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;

        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(detect_csv_headers(&buffer))
            .from_reader(buffer.as_bytes());

        // OPTIMIZATION: Pre-allocate daily trades vector
        let mut day_trades = Vec::with_capacity(2_000_000);
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            day_trades.push(agg_trade);
        }

        let trades_count = day_trades.len() as u64;

        // CRITICAL FIX: Use existing processor to preserve range bar state across days
        let completed_bars = processor.process_trades(&day_trades);
        all_range_bars.extend(completed_bars);

        Ok(trades_count)
    }

    #[allow(dead_code)] // Alternative processing method with statistics
    async fn process_single_day_with_stats(
        &self,
        processor: &mut ExportRangeBarProcessor,
        symbol: &str,
        date: NaiveDate,
        all_raw_trades: &mut Vec<AggTrade>,
    ) -> Result<(u64, Vec<RangeBar>), Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/{}/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            self.get_market_path(),
            symbol,
            symbol,
            date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send(),
        )
        .await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;

        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string())
            .await?;

        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;

        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;

        let mut reader = ReaderBuilder::new()
            .has_headers(detect_csv_headers(&buffer))
            .from_reader(buffer.as_bytes());

        // OPTIMIZATION: Pre-allocate daily trades vector (typical day = 1-4M trades)
        let mut day_trades = Vec::with_capacity(2_000_000);
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            day_trades.push(agg_trade.clone());
            all_raw_trades.push(agg_trade); // Collect for statistical analysis
        }

        let trades_count = day_trades.len() as u64;
        let completed_bars = processor.process_trades(&day_trades);

        Ok((trades_count, completed_bars))
    }

    fn export_to_csv(
        &self,
        bars: &[RangeBar],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filepath = Path::new(&self.output_dir).join(filename);
        let mut wtr = WriterBuilder::new().from_path(filepath)?;

        for bar in bars {
            wtr.serialize(bar)?;
        }

        wtr.flush()?;
        Ok(())
    }

    #[allow(dead_code)] // Basic export method, superseded by export_to_json_with_metadata
    fn export_to_json(
        &self,
        bars: &[RangeBar],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filepath = Path::new(&self.output_dir).join(filename);
        let json_content = serde_json::to_string_pretty(bars)?;
        fs::write(filepath, json_content)?;
        Ok(())
    }

    #[cfg(feature = "statistics")]
    fn export_to_json_with_metadata(
        &self,
        bars: &[RangeBar],
        filename: &str,
        metadata: Option<&rangebar::statistics::RangeBarMetadata>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::json;

        let filepath = Path::new(&self.output_dir).join(filename);

        let comprehensive_export = if let Some(meta) = metadata {
            // Create comprehensive JSON with metadata and range bars
            json!({
                "schema_version": "1.0.0",
                "export_type": "comprehensive_rangebar_analysis",
                "metadata": meta,
                "range_bars": bars,
                "summary": {
                    "total_bars": bars.len(),
                    "date_range": format!("{} to {}",
                        meta.dataset.temporal.start_date,
                        meta.dataset.temporal.end_date),
                    "symbol": meta.dataset.instrument.symbol,
                    "market_type": meta.dataset.instrument.venue,
                    "threshold_pct": meta.algorithm.parameters.threshold_bps as f64 / 1_000_000.0,
                    "statistical_metrics_count": "200+",
                    "analysis_timestamp": chrono::Utc::now().to_rfc3339()
                }
            })
        } else {
            // Fallback to simple JSON structure without statistics
            json!({
                "schema_version": "1.0.0",
                "export_type": "basic_rangebar_data",
                "range_bars": bars,
                "summary": {
                    "total_bars": bars.len(),
                    "note": "Statistical analysis not available (statistics feature disabled)"
                }
            })
        };

        let json_content = serde_json::to_string_pretty(&comprehensive_export)?;
        fs::write(filepath, json_content)?;
        Ok(())
    }

    #[cfg(not(feature = "statistics"))]
    fn export_to_json_with_metadata(
        &self,
        bars: &[RangeBar],
        filename: &str,
        _metadata: Option<&()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Fallback when statistics feature is disabled
        self.export_to_json(bars, filename)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 || args.len() > 7 {
        eprintln!(
            "Usage: {} <symbol> <start_date> <end_date> <threshold_pct> <output_dir> [market_type]",
            args[0]
        );
        eprintln!("Market types: spot (default), um (UM Futures)");
        eprintln!("Examples:");
        eprintln!(
            "  {} BTCUSDT 2025-09-01 2025-09-09 0.0025 ./output           # SPOT (default)",
            args[0]
        );
        eprintln!(
            "  {} BTCUSDT 2025-09-01 2025-09-09 0.0025 ./output spot      # SPOT (explicit)",
            args[0]
        );
        eprintln!(
            "  {} BTCUSDT 2025-09-01 2025-09-09 0.0025 ./output um        # UM Futures",
            args[0]
        );
        std::process::exit(1);
    }

    // Load configuration
    let _config = Settings::load().unwrap_or_else(|_| Settings::default());

    let symbol = &args[1];
    let start_date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&args[3], "%Y-%m-%d")?;
    let threshold_pct: f64 = args[4].parse()?;
    let output_dir = args[5].clone();

    // Default to "spot", optional "um" for UM Futures
    let market_type = if args.len() == 7 {
        match args[6].as_str() {
            "spot" | "um" => args[6].clone(),
            _ => {
                eprintln!(
                    "Error: market_type must be 'spot' or 'um', got '{}'",
                    args[6]
                );
                std::process::exit(1);
            }
        }
    } else {
        "spot".to_string()
    };

    let exporter = RangeBarExporter::new(output_dir, market_type)?;
    let result = exporter
        .export_symbol_range_bars(symbol, start_date, end_date, threshold_pct)
        .await
        .map_err(|e| -> Box<dyn std::error::Error> {
            Box::new(std::io::Error::other(e.to_string()))
        })?;

    // Export enhanced summary information
    let summary_file = format!("{}/export_summary.json", exporter.output_dir);
    let summary_json = serde_json::to_string_pretty(&result)?;
    fs::write(&summary_file, summary_json)?;
    println!("   📄 Enhanced Summary: {}", summary_file);

    Ok(())
}
