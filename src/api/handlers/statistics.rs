//! Statistics computation handlers

#[cfg(feature = "api")]
use axum::response::Json;
#[cfg(feature = "api")]
use serde::Deserialize;
#[cfg(feature = "api")]
use uuid::Uuid;

#[cfg(feature = "api")]
use crate::api::models::{ComputeStatisticsRequest, ErrorResponse, StatisticsResponse};

/// Query parameters for statistics computation
#[cfg(feature = "api")]
#[derive(Deserialize)]
pub struct StatisticsQuery {
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub threshold: f64,
}

/// Compute statistics for range bars
#[cfg(feature = "api")]
#[utoipa::path(
    post,
    path = "/api/v1/statistics/compute",
    request_body = ComputeStatisticsRequest,
    responses(
        (status = 200, description = "Statistics computed successfully", body = StatisticsResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Statistics"
)]
pub async fn compute_statistics(
    Json(_request): Json<ComputeStatisticsRequest>,
) -> Result<Json<StatisticsResponse>, Json<ErrorResponse>> {
    // TODO: Implement statistics computation
    Err(Json(ErrorResponse {
        error: "NOT_IMPLEMENTED".to_string(),
        message: "Statistics computation not yet implemented".to_string(),
        details: Some(serde_json::Value::String(
            "This endpoint requires the statistics feature to be enabled".to_string(),
        )),
        request_id: Some(Uuid::new_v4()),
    }))
}

/// Get available statistical metrics
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/statistics/metrics",
    responses(
        (status = 200, description = "Available metrics retrieved", body = Vec<String>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Statistics"
)]
pub async fn get_available_metrics() -> Json<Vec<String>> {
    Json(vec![
        "mean".to_string(),
        "std_dev".to_string(),
        "variance".to_string(),
        "skewness".to_string(),
        "kurtosis".to_string(),
        "min".to_string(),
        "max".to_string(),
        "median".to_string(),
        "percentiles".to_string(),
    ])
}
