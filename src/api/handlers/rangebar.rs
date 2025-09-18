//! Range bar generation handlers

#[cfg(feature = "api")]
use axum::{extract::Query, http::StatusCode, response::Json};
#[cfg(feature = "api")]
use serde::Deserialize;
#[cfg(feature = "api")]
use std::time::Instant;
#[cfg(feature = "api")]
use utoipa::IntoParams;
#[cfg(feature = "api")]
use validator::Validate;

#[cfg(feature = "api")]
use crate::{
    api::models::{ErrorResponse, GenerateRangeBarsRequest, ProcessingStats, RangeBarsResponse},
    range_bars::RangeBarProcessor,
};

/// Generate range bars from trade data
#[cfg(feature = "api")]
#[utoipa::path(
    post,
    path = "/api/v1/rangebar/generate",
    request_body = GenerateRangeBarsRequest,
    responses(
        (status = 200, description = "Range bars generated successfully", body = RangeBarsResponse),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 422, description = "Processing error", body = ErrorResponse)
    ),
    tag = "Range Bars"
)]
pub async fn generate_range_bars(
    Json(request): Json<GenerateRangeBarsRequest>,
) -> Result<Json<RangeBarsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: format!("Input validation failed: {}", validation_errors),
                details: None,
                request_id: Some(uuid::Uuid::new_v4()),
            }),
        ));
    }

    let start_time = Instant::now();

    // Convert threshold percentage to basis points for processor
    let threshold_bp = (request.threshold_pct * 1_000_000.0) as u32;

    // Create range bar processor
    let mut processor = RangeBarProcessor::new(threshold_bp);

    // Process trades into range bars
    let range_bars = match processor.process_trades(&request.trades) {
        Ok(bars) => bars,
        Err(e) => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse {
                    error: "PROCESSING_ERROR".to_string(),
                    message: format!("Failed to process trades: {}", e),
                    details: None,
                    request_id: Some(uuid::Uuid::new_v4()),
                }),
            ));
        }
    };

    let processing_time = start_time.elapsed();

    let response = RangeBarsResponse {
        symbol: request.symbol,
        threshold_pct: request.threshold_pct,
        bars: range_bars.clone(),
        processing_stats: ProcessingStats {
            trades_processed: request.trades.len() as u64,
            bars_generated: range_bars.len() as u32,
            processing_time_ms: processing_time.as_millis() as u64,
            memory_used_bytes: None, // Would require memory profiling integration
        },
    };

    Ok(Json(response))
}

/// WebSocket streaming parameters
#[cfg(feature = "api")]
#[derive(Debug, Deserialize, Validate, IntoParams)]
pub struct StreamParams {
    /// Trading symbol
    #[validate(length(min = 3, max = 20))]
    pub symbol: String,
    /// Threshold percentage
    #[validate(range(min = 0.0001, max = 0.1))]
    pub threshold_pct: f64,
}

/// WebSocket streaming endpoint (placeholder)
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/rangebar/stream",
    params(StreamParams),
    responses(
        (status = 101, description = "WebSocket connection established"),
        (status = 400, description = "Invalid parameters", body = ErrorResponse)
    ),
    tag = "Range Bars"
)]
pub async fn stream_range_bars(
    Query(params): Query<StreamParams>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    // Validate parameters
    if let Err(validation_errors) = params.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: format!("Parameter validation failed: {}", validation_errors),
                details: None,
                request_id: Some(uuid::Uuid::new_v4()),
            }),
        ));
    }

    // TODO: Implement WebSocket streaming
    // This would require WebSocket connection handling and real-time trade data feed
    Ok("WebSocket streaming endpoint - implementation pending".to_string())
}
