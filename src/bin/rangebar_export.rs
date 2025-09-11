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
use sha2::{Sha256, Digest};

// Use library types and statistics module
use rangebar_rust::{AggTrade, RangeBar, FixedPoint, RangeBarProcessor};

#[cfg(feature = "statistics")]
use rangebar_rust::statistics::{StatisticalEngine, StatisticalConfig, RangeBarMetadata};

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

// Market-type aware range bar for enhanced output
#[derive(Debug, Clone, Serialize)]
struct MarketAwareRangeBar {
    bar_id: usize,
    open_time: i64,
    close_time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    duration_minutes: f64,
    price_move_pct: f64,
    market_type: String,
    symbol: String,
    threshold_pct: f64,
}

#[derive(Debug, Deserialize)]
struct CsvAggTrade {
    price: f64,
    quantity: f64,
    transact_time: u64,
}

impl From<CsvAggTrade> for AggTrade {
    fn from(csv_trade: CsvAggTrade) -> Self {
        AggTrade {
            agg_trade_id: 0, // CSV doesn't have this field - set to 0
            price: FixedPoint::from_str(&csv_trade.price.to_string()).unwrap_or(FixedPoint(0)),
            volume: FixedPoint::from_str(&csv_trade.quantity.to_string()).unwrap_or(FixedPoint(0)),
            first_trade_id: 0, // CSV doesn't have this field - set to 0
            last_trade_id: 0, // CSV doesn't have this field - set to 0
            timestamp: csv_trade.transact_time as i64,
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
}

impl ExportRangeBarProcessor {
    fn new(threshold_bps: u32) -> Self {
        Self {
            threshold_bps,
            current_bar: None,
            completed_bars: Vec::new(),
            bar_counter: 0,
        }
    }
    
    fn process_trades(&mut self, trades: &[AggTrade]) -> Vec<RangeBar> {
        for trade in trades {
            self.process_single_trade(trade.clone());
        }
        
        let result = self.completed_bars.clone();
        self.completed_bars.clear();
        result
    }
    
    fn process_trades_continuously(&mut self, trades: &[AggTrade]) {
        for trade in trades {
            self.process_single_trade(trade.clone());
        }
        // DO NOT clear completed_bars - maintain state for continuous processing
    }
    
    fn get_all_completed_bars(&mut self) -> Vec<RangeBar> {
        let result = self.completed_bars.clone();
        self.completed_bars.clear();
        result
    }
    
    
    fn process_single_trade(&mut self, trade: AggTrade) {
        if self.current_bar.is_none() {
            // Start new bar
            self.current_bar = Some(InternalRangeBar {
                open_time: trade.timestamp,
                close_time: trade.timestamp,
                open: trade.price.clone(),
                high: trade.price.clone(),
                low: trade.price.clone(),
                close: trade.price.clone(),
                volume: trade.volume.clone(),
                turnover: trade.turnover(),
                trade_count: trade.trade_count(),
                first_id: trade.agg_trade_id,
                last_id: trade.agg_trade_id,
            });
            return;
        }
        
        let bar = self.current_bar.as_mut().unwrap();
        
        // Update bar with new trade
        bar.close_time = trade.timestamp;
        bar.close = trade.price.clone();
        bar.volume.0 += trade.volume.0;
        bar.turnover += trade.turnover();
        bar.trade_count += trade.trade_count();
        bar.last_id = trade.agg_trade_id;
        
        if trade.price.0 > bar.high.0 {
            bar.high = trade.price.clone();
        }
        if trade.price.0 < bar.low.0 {
            bar.low = trade.price.clone();
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
            };
            
            self.completed_bars.push(export_bar);
            
            // Start new bar
            self.current_bar = Some(InternalRangeBar {
                open_time: trade.timestamp,
                close_time: trade.timestamp,
                open: trade.price.clone(),
                high: trade.price.clone(),
                low: trade.price.clone(),
                close: trade.price.clone(),
                volume: trade.volume.clone(),
                turnover: trade.turnover(),
                trade_count: trade.trade_count(),
                first_id: trade.agg_trade_id,
                last_id: trade.agg_trade_id,
            });
        }
    }
    
    fn get_incomplete_bar(&mut self) -> Option<RangeBar> {
        if let Some(incomplete) = &self.current_bar {
            Some(RangeBar {
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
            })
        } else {
            None
        }
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
}

impl RangeBarExporter {
    fn new(output_dir: String) -> Self {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&output_dir).unwrap();
        
        Self {
            client: Client::new(),
            output_dir,
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
        let mut all_range_bars = Vec::new();
        let mut total_trades = 0u64;
        let mut current_date = start_date;
        
        // Initialize statistical engine for comprehensive analysis
        #[cfg(feature = "statistics")]
        let mut statistical_engine = rangebar_rust::statistics::StatisticalEngine::new();
        
        #[cfg(feature = "statistics")]
        let mut all_raw_trades = Vec::new(); // Collect raw trades for statistical analysis

        println!("🚀 Range Bar Exporter");
        println!("====================");
        println!("📊 Symbol: {}", symbol);
        println!("📅 Date Range: {} to {}", start_date, end_date);
        println!("📈 Threshold: {}%", threshold_pct * 100.0);
        println!("📁 Output: {}/", self.output_dir);

        // PHASE 1: Collect all trades chronologically for continuous processing
        println!("   🔄 Phase 1: Collecting trades chronologically...");
        while current_date <= end_date {
            print!("   📊 Loading {}...\r", current_date.format("%Y-%m-%d"));
            
            #[cfg(feature = "statistics")]
            match self.load_single_day_trades(symbol, current_date, &mut all_raw_trades).await {
                Ok(trades_count) => {
                    total_trades += trades_count;
                    println!("   📊 {} {} → {} trades loaded (total: {})",
                        symbol, current_date.format("%Y-%m-%d"), 
                        trades_count, total_trades);
                }
                Err(e) => {
                    eprintln!("   ⚠️  {} {}: {}", symbol, current_date.format("%Y-%m-%d"), e);
                }
            }
            
            #[cfg(not(feature = "statistics"))]
            match self.load_single_day_trades_simple(symbol, current_date, threshold_pct, &mut all_range_bars).await {
                Ok(trades_count) => {
                    total_trades += trades_count;
                    println!("   📊 {} {} → {} trades loaded (total: {})",
                        symbol, current_date.format("%Y-%m-%d"), 
                        trades_count, total_trades);
                }
                Err(e) => {
                    eprintln!("   ⚠️  {} {}: {}", symbol, current_date.format("%Y-%m-%d"), e);
                }
            }
            
            current_date += Duration::days(1);
        }
        
        // PHASE 2: Process all trades continuously (maintaining state across day boundaries)
        #[cfg(feature = "statistics")]
        {
            println!("\n   🔄 Phase 2: Processing {} trades continuously...", total_trades);
            processor.process_trades_continuously(&all_raw_trades);
            all_range_bars = processor.get_all_completed_bars();
            println!("   ✅ Continuous processing complete: {} range bars generated", all_range_bars.len());
        }

        // PHASE 3: Add incomplete bar if exists (final bar may be incomplete)
        if let Some(incomplete_bar) = processor.get_incomplete_bar() {
            all_range_bars.push(incomplete_bar);
            println!("   📊 Added final incomplete bar (total: {} bars)", all_range_bars.len());
        }

        let processing_time = start_time.elapsed().as_secs_f64();

        // Export to CSV and JSON
        let total_volume: f64 = all_range_bars.iter().map(|b| b.volume.to_f64()).sum();
        let date_str = format!("{}_{}", start_date.format("%Y%m%d"), end_date.format("%Y%m%d"));
        let csv_filename = format!("um_{}_rangebar_{}_{:.3}pct.csv", symbol, date_str, threshold_pct * 100.0);
        let json_filename = format!("um_{}_rangebar_{}_{:.3}pct.json", symbol, date_str, threshold_pct * 100.0);
        
        self.export_to_csv(&all_range_bars, &csv_filename)?;
        
        // Generate comprehensive metadata with statistical analysis
        #[cfg(feature = "statistics")]
        let metadata = {
            println!("   🔬 Generating comprehensive statistical analysis...");
            let metadata_result = statistical_engine.compute_comprehensive_metadata(
                &all_raw_trades,
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
            date_range: (start_date.format("%Y-%m-%d").to_string(), end_date.format("%Y-%m-%d").to_string()),
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
                    market_type: "um".to_string(),
                },
                ExportedFile {
                    filename: json_filename,
                    format: "json".to_string(),
                    size_bytes: 0, // TODO: Get actual file size  
                    market_type: "um".to_string(),
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

    async fn process_single_day(
        &self,
        processor: &mut ExportRangeBarProcessor,
        symbol: &str,
        date: NaiveDate,
    ) -> Result<(u64, Vec<RangeBar>), Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/futures/um/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            symbol, symbol, date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;
        
        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string()).await?;
        
        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;
        
        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;
        
        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
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
            "https://data.binance.vision/data/futures/um/daily/aggTrades/{}/{}-aggTrades-{}.zip.CHECKSUM",
            symbol, symbol, date_str
        );
        
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            self.client.get(&checksum_url).send()
        ).await??;
        
        if !response.status().is_success() {
            return Err(format!("Failed to download checksum: HTTP {}", response.status()).into());
        }
        
        let checksum_text = response.text().await?;
        let expected_hash = checksum_text.split_whitespace().next()
            .ok_or("Invalid checksum format")?;
        
        // Compute SHA256 of the downloaded zip data
        let mut hasher = Sha256::new();
        hasher.update(zip_data);
        let computed_hash = format!("{:x}", hasher.finalize());
        
        if computed_hash != expected_hash {
            return Err(format!(
                "SHA256 mismatch for {}-aggTrades-{}.zip: expected {}, got {}",
                symbol, date_str, expected_hash, computed_hash
            ).into());
        }
        
        println!("✓ SHA256 verification passed for {}-aggTrades-{}.zip", symbol, date_str);
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
        println!("⚠ SHA256 verification skipped for {}-aggTrades-{}.zip (data-integrity feature disabled)", symbol, date_str);
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
            "https://data.binance.vision/data/futures/um/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            symbol, symbol, date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;
        
        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string()).await?;
        
        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;
        
        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;
        
        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(buffer.as_bytes());

        let mut day_trades = Vec::new();
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            day_trades.push(agg_trade);
        }

        // Sort by timestamp to ensure chronological order for continuous processing
        day_trades.sort_by_key(|trade| trade.timestamp);
        
        let trades_count = day_trades.len() as u64;
        all_raw_trades.extend(day_trades);

        Ok(trades_count)
    }
    
    #[cfg(not(feature = "statistics"))]
    async fn load_single_day_trades_simple(
        &self,
        symbol: &str,
        date: NaiveDate,
        threshold_pct: f64,
        all_range_bars: &mut Vec<RangeBar>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Fallback for when statistics feature is disabled
        // This method maintains the old day-by-day processing for compatibility
        let mut temp_processor = ExportRangeBarProcessor::new((threshold_pct * 1_000_000.0) as u32);
        
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/futures/um/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            symbol, symbol, date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;
        
        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string()).await?;
        
        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;
        
        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;
        
        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(buffer.as_bytes());

        let mut day_trades = Vec::new();
        for result in reader.deserialize() {
            let csv_trade: CsvAggTrade = result?;
            let agg_trade: AggTrade = csv_trade.into();
            day_trades.push(agg_trade);
        }

        let trades_count = day_trades.len() as u64;
        let completed_bars = temp_processor.process_trades(&day_trades);
        all_range_bars.extend(completed_bars);

        Ok(trades_count)
    }

    async fn process_single_day_with_stats(
        &self,
        processor: &mut ExportRangeBarProcessor,
        symbol: &str,
        date: NaiveDate,
        all_raw_trades: &mut Vec<AggTrade>,
    ) -> Result<(u64, Vec<RangeBar>), Box<dyn std::error::Error + Send + Sync>> {
        let date_str = date.format("%Y-%m-%d");
        let url = format!(
            "https://data.binance.vision/data/futures/um/daily/aggTrades/{}/{}-aggTrades-{}.zip",
            symbol, symbol, date_str
        );

        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let zip_bytes = response.bytes().await?;
        
        // Verify data integrity using SHA256 checksum
        self.verify_file_integrity(&zip_bytes, symbol, &date_str.to_string()).await?;
        
        let cursor = Cursor::new(zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;
        
        let csv_filename = format!("{}-aggTrades-{}.csv", symbol, date_str);
        let mut csv_file = archive.by_name(&csv_filename)?;
        
        let mut buffer = String::with_capacity(8 * 1024 * 1024);
        csv_file.read_to_string(&mut buffer)?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(buffer.as_bytes());

        let mut day_trades = Vec::new();
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

    fn export_to_csv(&self, bars: &[RangeBar], filename: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filepath = Path::new(&self.output_dir).join(filename);
        let mut wtr = WriterBuilder::new().from_path(filepath)?;
        
        for bar in bars {
            wtr.serialize(bar)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn export_to_json(&self, bars: &[RangeBar], filename: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        metadata: Option<&rangebar_rust::statistics::RangeBarMetadata>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::{json, Value};
        
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
        _metadata: Option<&()>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Fallback when statistics feature is disabled
        self.export_to_json(bars, filename)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        eprintln!("Usage: {} <symbol> <start_date> <end_date> <threshold_pct> <output_dir>", args[0]);
        eprintln!("Example: {} BTCUSDT 2025-09-01 2025-09-09 0.008 ./output", args[0]);
        std::process::exit(1);
    }

    let symbol = &args[1];
    let start_date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
    let end_date = NaiveDate::parse_from_str(&args[3], "%Y-%m-%d")?;
    let threshold_pct: f64 = args[4].parse()?;
    let output_dir = args[5].clone();

    let exporter = RangeBarExporter::new(output_dir);
    let result = exporter.export_symbol_range_bars(symbol, start_date, end_date, threshold_pct).await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) })?;

    // Export enhanced summary information
    let summary_file = format!("{}/export_summary.json", exporter.output_dir);
    let summary_json = serde_json::to_string_pretty(&result)?;
    fs::write(&summary_file, summary_json)?;
    println!("   📄 Enhanced Summary: {}", summary_file);

    Ok(())
}