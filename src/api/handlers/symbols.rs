//! Symbol discovery and metadata handlers

#[cfg(feature = "api")]
use axum::response::Json;
#[cfg(feature = "api")]
use serde::Serialize;

// #[cfg(feature = "api")]
// use crate::api::models::ErrorResponse; // TODO: Use when implementing error handling
#[cfg(feature = "api")]
use crate::tier1::{get_tier1_symbols, get_tier1_usdt_pairs, is_tier1_symbol};

/// Symbol information response
#[cfg(feature = "api")]
#[derive(Serialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct SymbolInfo {
    pub symbol: String,
    pub is_tier1: bool,
    pub markets: Vec<String>,
}

/// Get all Tier-1 symbols
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/symbols/tier1",
    responses(
        (status = 200, description = "Tier-1 symbols retrieved", body = Vec<String>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Symbols"
)]
pub async fn get_tier1_symbols_endpoint() -> Json<Vec<String>> {
    Json(get_tier1_symbols())
}

/// Get Tier-1 USDT perpetual pairs
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/symbols/tier1/usdt",
    responses(
        (status = 200, description = "Tier-1 USDT pairs retrieved", body = Vec<String>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Symbols"
)]
pub async fn get_tier1_usdt_pairs_endpoint() -> Json<Vec<String>> {
    Json(get_tier1_usdt_pairs())
}

/// Check if a symbol is Tier-1
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/symbols/{symbol}/tier1",
    params(
        ("symbol" = String, Path, description = "Symbol to check")
    ),
    responses(
        (status = 200, description = "Symbol tier status", body = bool),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Symbols"
)]
pub async fn check_tier1_symbol(
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Json<bool> {
    Json(is_tier1_symbol(&symbol))
}

/// Get symbol information
#[cfg(feature = "api")]
#[utoipa::path(
    get,
    path = "/api/v1/symbols/{symbol}",
    params(
        ("symbol" = String, Path, description = "Symbol to get information for")
    ),
    responses(
        (status = 200, description = "Symbol information", body = SymbolInfo),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Symbols"
)]
pub async fn get_symbol_info(
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Json<SymbolInfo> {
    let is_tier1 = is_tier1_symbol(&symbol);
    let markets = if is_tier1 {
        vec![
            "um-usdt".to_string(),
            "um-usdc".to_string(),
            "cm".to_string(),
        ]
    } else {
        vec![] // TODO: Implement market detection for non-Tier-1 symbols
    };

    Json(SymbolInfo {
        symbol,
        is_tier1,
        markets,
    })
}
