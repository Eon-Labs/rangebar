//! GPU acceleration module for range bar processing using Mac Metal
//!
//! This module provides GPU-accelerated implementations of range bar construction
//! and statistical analysis using the Burn framework with Metal backend on Mac.
//!
//! # Features
//!
//! - **Metal Backend**: Native Mac GPU acceleration via Metal API
//! - **Cross-platform**: WGPU backend for other platforms
//! - **Automatic Fallback**: Falls back to CPU if GPU unavailable
//! - **Tensor Operations**: Vectorized range bar algorithms
//! - **Multi-symbol Batch**: Process multiple symbols simultaneously
//!
//! # Example
//!
//! ```rust,no_run
//! use rangebar::gpu::{GpuRangeBarProcessor, detect_gpu_device};
//!
//! if let Some(device) = detect_gpu_device() {
//!     let processor = GpuRangeBarProcessor::new(device, 8000, None);
//!     // GPU-accelerated processing
//! } else {
//!     // Fall back to CPU processing
//! }
//! ```

#[cfg(feature = "gpu")]
pub mod metal_backend;

#[cfg(feature = "gpu")]
pub mod range_bars_gpu;

#[cfg(feature = "gpu")]
pub mod multi_symbol;

#[cfg(feature = "gpu")]
pub mod benchmarks;

#[cfg(feature = "gpu")]
pub mod multi_symbol_tests;

// Re-export main types when GPU feature is enabled
#[cfg(feature = "gpu")]
pub use range_bars_gpu::{GpuProcessingError, GpuRangeBarProcessor};

#[cfg(feature = "gpu")]
pub use metal_backend::{GpuBackend, GpuDevice, detect_gpu_device};

#[cfg(feature = "gpu")]
pub use multi_symbol::{
    MAX_TIER1_SYMBOLS, MAX_TRADES_PER_SYMBOL, MultiSymbolGpuError, MultiSymbolGpuProcessor,
};

/// GPU feature detection and initialization
#[cfg(feature = "gpu")]
pub fn is_gpu_available() -> bool {
    detect_gpu_device().is_some()
}

/// Get GPU device info string for diagnostics
#[cfg(feature = "gpu")]
pub fn get_gpu_info() -> String {
    if let Some(device) = detect_gpu_device() {
        device.info()
    } else {
        "No GPU device available".to_string()
    }
}

// No-op implementations when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub fn is_gpu_available() -> bool {
    false
}

#[cfg(not(feature = "gpu"))]
pub fn get_gpu_info() -> String {
    "GPU support not compiled (gpu feature disabled)".to_string()
}
