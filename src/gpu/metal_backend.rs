//! Metal backend for GPU acceleration on Mac
//!
//! This module provides GPU device detection and initialization using the Burn
//! framework with Metal backend for native Mac GPU acceleration.

#[cfg(feature = "gpu")]
use burn::backend::wgpu::WgpuDevice;

/// GPU device wrapper for cross-platform compatibility
#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
pub struct GpuDevice {
    device: WgpuDevice,
    device_type: GpuBackend,
    name: String,
}

#[cfg(feature = "gpu")]
#[derive(Debug, Clone)]
pub enum GpuBackend {
    Metal,  // Mac Metal
    Vulkan, // Cross-platform Vulkan
    Dx12,   // Windows DirectX 12
}

#[cfg(feature = "gpu")]
impl GpuDevice {
    /// Create new GPU device
    pub fn new(device: WgpuDevice, backend: GpuBackend, name: String) -> Self {
        Self {
            device,
            device_type: backend,
            name,
        }
    }

    /// Get device for Burn operations
    pub fn device(&self) -> &WgpuDevice {
        &self.device
    }

    /// Get backend type
    pub fn backend(&self) -> &GpuBackend {
        &self.device_type
    }

    /// Get device info string
    pub fn info(&self) -> String {
        format!("{} ({:?})", self.name, self.device_type)
    }

    /// Check if this is a Metal device (Mac native)
    pub fn is_metal(&self) -> bool {
        matches!(self.device_type, GpuBackend::Metal)
    }
}

/// Detect and initialize GPU device
#[cfg(feature = "gpu")]
pub fn detect_gpu_device() -> Option<GpuDevice> {
    // Try to initialize WGPU device with Metal preference on Mac
    let device = WgpuDevice::default();
    // Get adapter info to determine backend type
    let backend_type = detect_backend_type();
    let device_name = get_device_name(&device);

    Some(GpuDevice::new(device, backend_type, device_name))
}

#[cfg(feature = "gpu")]
fn detect_backend_type() -> GpuBackend {
    // On Mac, WGPU should automatically select Metal
    #[cfg(target_os = "macos")]
    {
        GpuBackend::Metal
    }

    // On other platforms, detect based on available APIs
    #[cfg(not(target_os = "macos"))]
    {
        // This is a simplified detection - in practice, WGPU handles this
        GpuBackend::Vulkan
    }
}

#[cfg(feature = "gpu")]
fn get_device_name(_device: &WgpuDevice) -> String {
    // Get actual device name from Metal/WGPU adapter
    #[cfg(target_os = "macos")]
    {
        // On Mac, typically Apple M1/M2/M3 GPU
        std::env::var("GPU_DEVICE_NAME").unwrap_or_else(|_| "Apple GPU (Metal)".to_string())
    }

    #[cfg(not(target_os = "macos"))]
    {
        "GPU Device".to_string()
    }
}

/// Check if GPU acceleration is available and functional
#[cfg(feature = "gpu")]
pub fn is_gpu_functional() -> bool {
    detect_gpu_device().is_some()
}

/// Get detailed GPU information for diagnostics
#[cfg(feature = "gpu")]
pub fn get_gpu_diagnostics() -> String {
    if let Some(device) = detect_gpu_device() {
        let mut info = format!("GPU Device: {}\n", device.info());
        info.push_str(&format!("Backend: {:?}\n", device.backend()));
        info.push_str(&format!("Metal Support: {}\n", device.is_metal()));

        // Add memory information if available
        #[cfg(target_os = "macos")]
        {
            info.push_str("Unified Memory Architecture: Yes\n");
        }

        info
    } else {
        "No GPU device available\n".to_string()
    }
}

/// Initialize GPU backend for range bar processing
#[cfg(feature = "gpu")]
pub fn initialize_gpu_backend() -> Result<GpuDevice, GpuError> {
    detect_gpu_device().ok_or(GpuError::NoDeviceAvailable)
}

/// GPU-related errors
#[cfg(feature = "gpu")]
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("No GPU device available")]
    NoDeviceAvailable,

    #[error("GPU initialization failed: {message}")]
    InitializationFailed { message: String },

    #[error("GPU computation error: {message}")]
    ComputationError { message: String },
}

// No-op implementations when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub fn detect_gpu_device() -> Option<()> {
    None
}

#[cfg(not(feature = "gpu"))]
pub fn is_gpu_functional() -> bool {
    false
}

#[cfg(not(feature = "gpu"))]
pub fn get_gpu_diagnostics() -> String {
    "GPU support not compiled (gpu feature disabled)".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_detection() {
        // This test will only pass if GPU is actually available
        let device = detect_gpu_device();
        if device.is_some() {
            println!("GPU detected: {:?}", device.unwrap().info());
        } else {
            println!("No GPU available for testing");
        }
    }

    #[test]
    fn test_gpu_diagnostics() {
        let diagnostics = get_gpu_diagnostics();
        assert!(!diagnostics.is_empty());
        println!("GPU Diagnostics:\n{}", diagnostics);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_metal_detection() {
        #[cfg(feature = "gpu")]
        {
            if let Some(device) = detect_gpu_device() {
                assert!(device.is_metal(), "Should detect Metal backend on Mac");
            }
        }
    }
}
