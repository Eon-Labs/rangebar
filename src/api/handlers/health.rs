//! Health check handler

#[cfg(feature = "api")]
use axum::response::Json;
#[cfg(feature = "api")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "api")]
use crate::api::models::HealthResponse;

/// Health check endpoint
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "System"
)]
pub async fn health_check() -> Json<HealthResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::VERSION.to_string(),
        uptime_seconds: uptime,
    })
}
