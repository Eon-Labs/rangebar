//! API request models

#[cfg(feature = "api")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use utoipa::ToSchema;
#[cfg(feature = "api")]
use validator::Validate;

#[cfg(feature = "api")]
use crate::types::{AggTrade, RangeBar};

/// Request to generate range bars from trade data
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct GenerateRangeBarsRequest {
    /// Trading symbol (e.g., "BTCUSDT")
    #[validate(length(min = 3, max = 20))]
    pub symbol: String,

    /// Range threshold as percentage (0.008 = 0.8%)
    #[validate(range(min = 0.0001, max = 0.1))]
    pub threshold_pct: f64,

    /// Aggregated trades to process
    #[validate(length(min = 1, max = 1000000))]
    pub trades: Vec<AggTrade>,
}

/// Request to compute statistics on range bars
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ComputeStatisticsRequest {
    /// Range bars to analyze
    #[validate(length(min = 1))]
    pub bars: Vec<RangeBar>,

    /// Optional configuration for statistics computation
    pub config: Option<StatisticsConfig>,
}

/// Configuration for statistics computation
#[cfg(feature = "api")]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsConfig {
    /// Include distribution analysis
    #[serde(default = "default_true")]
    pub include_distributions: bool,

    /// Include performance metrics
    #[serde(default = "default_true")]
    pub include_performance_metrics: bool,

    /// Use parallel computation
    #[serde(default = "default_true")]
    pub parallel_computation: bool,
}

#[cfg(feature = "api")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "api")]
impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            include_distributions: true,
            include_performance_metrics: true,
            parallel_computation: true,
        }
    }
}
