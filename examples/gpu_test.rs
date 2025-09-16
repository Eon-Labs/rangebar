//! Simple GPU test to validate Metal backend detection

#[cfg(feature = "gpu")]
use rangebar::gpu::{is_gpu_available, get_gpu_info};

fn main() {
    println!("🔍 GPU Detection Test");
    println!("{}", "=".repeat(50));

    #[cfg(feature = "gpu")]
    {
        println!("GPU Available: {}", is_gpu_available());
        println!("GPU Info:\n{}", get_gpu_info());

        if is_gpu_available() {
            println!("✅ GPU acceleration is available!");

            // Test basic GPU device detection
            if let Some(device) = rangebar::gpu::detect_gpu_device() {
                println!("Device Details:");
                println!("  - Name: {}", device.info());
                println!("  - Metal Backend: {}", device.is_metal());
            }
        } else {
            println!("❌ GPU acceleration is not available");
            println!("This could be normal on systems without compatible GPU or in CI environments");
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU support not compiled (gpu feature disabled)");
        println!("Run with: cargo run --features gpu --example gpu_test");
    }
}