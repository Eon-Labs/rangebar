//! RESTful API module for range bar processing
//!
//! This module provides OpenAPI 3.1.1 compliant REST endpoints for:
//! - Range bar generation from trade data
//! - Streaming range bar processing
//! - Statistical analysis
//! - Data export functionality
//!
//! ## Features
//! - 🚀 High-performance async processing with Axum
//! - 📊 OpenAPI documentation with Swagger UI
//! - 🔒 Input validation and error handling
//! - ⚡ Streaming-first architecture for memory efficiency

#[cfg(feature = "api")]
pub mod handlers;
// #[cfg(feature = "api")]
// pub mod middleware; // TODO: Implement middleware when needed
#[cfg(feature = "api")]
pub mod models;
#[cfg(feature = "api")]
pub mod server;

#[cfg(feature = "api")]
pub use server::create_app;
