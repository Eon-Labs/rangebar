//! API data models and DTOs
//!
//! This module contains request/response models that correspond to the OpenAPI 3.1.1 specification.
//! All models implement Serialize/Deserialize for JSON API compatibility.

#[cfg(feature = "api")]
pub mod requests;
#[cfg(feature = "api")]
pub mod responses;

#[cfg(feature = "api")]
pub use requests::*;
#[cfg(feature = "api")]
pub use responses::*;
