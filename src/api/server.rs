//! HTTP server setup and routing

#[cfg(feature = "api")]
use axum::{
    Router,
    routing::{get, post},
};
#[cfg(feature = "api")]
use tower::ServiceBuilder;
#[cfg(feature = "api")]
use tower_http::{cors::CorsLayer, trace::TraceLayer};
#[cfg(feature = "api")]
use utoipa::OpenApi;

#[cfg(feature = "api")]
use crate::api::{
    handlers::{generate_range_bars, stream_range_bars},
    models::{
        ComputeStatisticsRequest, ErrorResponse, GenerateRangeBarsRequest, ProcessingStats,
        RangeBarsResponse, StatisticsConfig, StatisticsResponse,
    },
};

/// OpenAPI documentation
#[cfg(feature = "api")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::rangebar::generate_range_bars,
        crate::api::handlers::rangebar::stream_range_bars,
    ),
    components(
        schemas(
            GenerateRangeBarsRequest,
            RangeBarsResponse,
            ProcessingStats,
            ComputeStatisticsRequest,
            StatisticsConfig,
            StatisticsResponse,
            ErrorResponse,
            crate::types::AggTrade,
            crate::types::RangeBar,
        )
    ),
    tags(
        (name = "Range Bars", description = "Range bar generation and processing"),
        (name = "Statistics", description = "Statistical analysis"),
        (name = "System", description = "System health and monitoring"),
    )
)]
pub struct ApiDoc;

/// Create the main application router
#[cfg(feature = "api")]
pub fn create_app() -> Router {
    // Create the main API router
    let api_routes = Router::new()
        .route("/rangebar/generate", post(generate_range_bars))
        .route("/rangebar/stream", get(stream_range_bars))
        // Health endpoint
        .route("/health", get(crate::api::handlers::health_check))
        // Statistics endpoints
        .route(
            "/statistics/compute",
            post(crate::api::handlers::compute_statistics),
        )
        .route(
            "/statistics/metrics",
            get(crate::api::handlers::get_available_metrics),
        )
        // Export endpoints
        .route(
            "/export/:format",
            get(crate::api::handlers::export_range_bars),
        )
        .route(
            "/export/formats",
            get(crate::api::handlers::get_export_formats),
        )
        // Symbols endpoints
        .route(
            "/symbols/tier1",
            get(crate::api::handlers::get_tier1_symbols_endpoint),
        )
        .route(
            "/symbols/tier1/usdt",
            get(crate::api::handlers::get_tier1_usdt_pairs_endpoint),
        )
        .route(
            "/symbols/:symbol/tier1",
            get(crate::api::handlers::check_tier1_symbol),
        )
        .route(
            "/symbols/:symbol",
            get(crate::api::handlers::get_symbol_info),
        );

    // Add middleware
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Combine routes with Swagger UI
    // TODO: Add SwaggerUI integration - currently disabled due to type compatibility issues
    // SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())

    Router::new().nest("/api/v1", api_routes).layer(middleware)
}

/// Start the HTTP server
#[cfg(feature = "api")]
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting Rangebar API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
