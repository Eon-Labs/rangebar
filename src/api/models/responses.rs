//! API response models

#[cfg(feature = "api")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use utoipa::ToSchema;
#[cfg(feature = "api")]
use uuid::Uuid;

#[cfg(feature = "api")]
use crate::types::RangeBar;

/// Health check response
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Tier-1 symbols response
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Tier1SymbolsResponse {
    /// List of Tier-1 symbols
    pub symbols: Vec<String>,
    /// Number of symbols
    pub count: usize,
    /// USDT perpetual pairs
    pub usdt_pairs: Vec<String>,
}

/// Range bars generation response
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RangeBarsResponse {
    /// Trading symbol
    pub symbol: String,
    /// Threshold in basis points used
    pub threshold_bps: u32,
    /// Generated range bars
    pub bars: Vec<RangeBar>,
    /// Processing statistics
    pub processing_stats: ProcessingStats,
}

/// Processing statistics
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProcessingStats {
    /// Number of trades processed
    pub trades_processed: u64,
    /// Number of bars generated
    pub bars_generated: u32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Memory used in bytes
    pub memory_used_bytes: Option<u64>,
}

/// Statistics computation response
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsResponse {
    /// Statistical summary results
    pub summary: serde_json::Value,
    /// Market data statistics
    pub market_data: Option<serde_json::Value>,
    /// Range bar specific statistics
    pub range_bars: Option<serde_json::Value>,
    /// Distribution analysis
    pub distributions: Option<serde_json::Value>,
    /// Performance metrics
    pub performance: Option<serde_json::Value>,
    /// Computation time in milliseconds
    pub computation_time_ms: u64,
}

/// Standard error response
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error context
    pub details: Option<serde_json::Value>,
    /// Request ID for tracking
    pub request_id: Option<Uuid>,
}
