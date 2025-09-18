//! API route handlers
//!
//! This module contains the HTTP handlers for all API endpoints.
//! Each handler corresponds to a specific endpoint in the OpenAPI specification.

#[cfg(feature = "api")]
pub mod export;
#[cfg(feature = "api")]
pub mod health;
#[cfg(feature = "api")]
pub mod rangebar;
#[cfg(feature = "api")]
pub mod statistics;
#[cfg(feature = "api")]
pub mod symbols;

#[cfg(feature = "api")]
pub use export::*;
#[cfg(feature = "api")]
pub use health::*;
#[cfg(feature = "api")]
pub use rangebar::*;
#[cfg(feature = "api")]
pub use statistics::*;
#[cfg(feature = "api")]
pub use symbols::*;
