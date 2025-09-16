//! GPU Tensor Operations Debugging Tool
//!
//! Isolated test to debug the exact tensor operations failing
//! in our GPU range bar implementation.

#[cfg(feature = "gpu")]
use burn::{
    backend::wgpu::{Wgpu, WgpuDevice},
    tensor::{Tensor, TensorData},
};

#[cfg(feature = "gpu")]
type TestBackend = Wgpu;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "gpu"))]
    {
        println!("❌ GPU feature not enabled. Run with: cargo run --example gpu_tensor_debug --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        println!("🧪 GPU Tensor Operations Debug Tool");
        println!("═══════════════════════════════════════");

        let device = WgpuDevice::default();
        println!("✅ GPU device initialized: {:?}", device);

        test_basic_tensor_operations(&device)?;
        test_boolean_operations(&device)?;
        test_comparison_operations(&device)?;
        test_breach_detection_scenario(&device)?;

        println!("\n🎯 GPU tensor debugging completed");
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_basic_tensor_operations(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing basic tensor operations:");

    // Test 1: Create and extract f32 tensor
    let float_data = vec![1.0, 2.0, 3.0, 4.0];
    let float_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(float_data.clone(), [4]),
        device,
    );

    let extracted_float = float_tensor.to_data();
    match extracted_float.as_slice::<f32>() {
        Ok(slice) => {
            println!("✅ Float tensor: created and extracted");
            println!("   Original: {:?}", float_data);
            println!("   Extracted: {:?}", slice);
        }
        Err(e) => {
            println!("❌ Float tensor extraction failed: {:?}", e);
        }
    }

    // Test 2: Create and extract boolean tensor
    let bool_data = vec![true, false, true, false];
    let bool_tensor = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(bool_data.clone(), [4]),
        device,
    );

    let extracted_bool = bool_tensor.to_data();
    match extracted_bool.as_slice::<bool>() {
        Ok(slice) => {
            println!("✅ Boolean tensor: created and extracted");
            println!("   Original: {:?}", bool_data);
            println!("   Extracted: {:?}", slice);
        }
        Err(e) => {
            println!("❌ Boolean tensor extraction failed: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_boolean_operations(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing boolean operations:");

    // Test boolean tensor .any() operation
    let test_cases = vec![
        (vec![false, false, false], "all false", 0),
        (vec![true, true, true], "all true", 1),
        (vec![false, true, false], "mixed", 1),
    ];

    for (data, description, expected) in test_cases {
        let tensor = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
            TensorData::new(data.clone(), [3]),
            device,
        );

        let any_result = tensor.any();
        let scalar_result = any_result.into_scalar();

        println!("   {} .any(): {} (expected: {})", description, scalar_result, expected);

        if (expected == 0 && scalar_result == 0) || (expected > 0 && scalar_result > 0) {
            println!("   ✅ Correct");
        } else {
            println!("   ❌ Wrong");
        }
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_comparison_operations(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing comparison operations:");

    // Test simple greater_equal comparison
    let prices = vec![50.0, 60.0, 40.0, 55.0];
    let threshold = 55.0;

    let price_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(prices.clone(), [4]),
        device,
    );

    // Test tensor vs scalar comparison
    let result_scalar = price_tensor.clone().greater_equal_elem(threshold);
    match result_scalar.to_data().as_slice::<bool>() {
        Ok(results) => {
            println!("   Tensor vs scalar >= {}: {:?}", threshold, results);
            let expected = [false, true, false, false];
            if results == expected {
                println!("   ✅ Tensor vs scalar: correct");
            } else {
                println!("   ❌ Tensor vs scalar: wrong (expected {:?})", expected);
            }
        }
        Err(e) => {
            println!("   ❌ Tensor vs scalar comparison failed: {:?}", e);
        }
    }

    // Test tensor vs tensor comparison
    let threshold_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![threshold; 4], [4]),
        device,
    );

    let result_tensor = price_tensor.greater_equal(threshold_tensor);
    match result_tensor.to_data().as_slice::<bool>() {
        Ok(results) => {
            println!("   Tensor vs tensor >= {}: {:?}", threshold, results);
            let expected = [false, true, false, false];
            if results == expected {
                println!("   ✅ Tensor vs tensor: correct");
            } else {
                println!("   ❌ Tensor vs tensor: wrong (expected {:?})", expected);
            }
        }
        Err(e) => {
            println!("   ❌ Tensor vs tensor comparison failed: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_breach_detection_scenario(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing exact breach detection scenario:");

    // Use exact values from our range bar implementation
    let open_price = 50750.0;
    let threshold_bps = 8000; // 0.8%
    let threshold_multiplier = (threshold_bps as f32) / 1_000_000.0; // 0.008

    let upper_threshold = open_price * (1.0 + threshold_multiplier); // 51156.0
    let lower_threshold = open_price * (1.0 - threshold_multiplier); // 50344.0

    println!("   Open: {:.1}", open_price);
    println!("   Upper threshold: {:.1}", upper_threshold);
    println!("   Lower threshold: {:.1}", lower_threshold);
    println!("   Threshold %: {:.6}%", threshold_multiplier * 100.0);

    // Test with 1D tensors first
    println!("\n   Testing 1D tensors:");
    test_breach_1d(device, open_price, upper_threshold, lower_threshold)?;

    // Test with 2D tensors [N,1] like our implementation
    println!("\n   Testing 2D tensors [N,1]:");
    test_breach_2d(device, open_price, upper_threshold, lower_threshold)?;

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_breach_1d(
    device: &WgpuDevice,
    open: f32,
    upper: f32,
    lower: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_prices = vec![
        open,      // No breach
        upper + 50.0, // Upper breach
        lower - 50.0, // Lower breach
        open + 100.0, // No breach (within threshold)
    ];

    let price_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(test_prices.clone(), [4]),
        device,
    );

    let upper_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![upper; 4], [4]),
        device,
    );

    let lower_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![lower; 4], [4]),
        device,
    );

    // Perform breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.bool_or(lower_breach);

    // Extract results
    match any_breach.to_data().as_slice::<bool>() {
        Ok(results) => {
            println!("      Prices: {:?}", test_prices);
            println!("      Breaches: {:?}", results);

            // Manual verification
            for (i, &price) in test_prices.iter().enumerate() {
                let should_breach = price >= upper || price <= lower;
                let detected_breach = results[i];
                println!("      Price {:.1}: should={}, detected={}", price, should_breach, detected_breach);

                if should_breach != detected_breach {
                    println!("      ❌ Mismatch at price {:.1}", price);
                }
            }
        }
        Err(e) => {
            println!("      ❌ 1D breach detection failed: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_breach_2d(
    device: &WgpuDevice,
    open: f32,
    upper: f32,
    lower: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_prices = vec![
        open,      // No breach
        upper + 50.0, // Upper breach
        lower - 50.0, // Lower breach
    ];

    // Create 2D tensors [3,1]
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_prices.clone(), [3, 1]),
        device,
    );

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper; 3], [3, 1]),
        device,
    );

    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![lower; 3], [3, 1]),
        device,
    );

    // Perform breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.bool_or(lower_breach);

    // Test full extraction
    match any_breach.to_data().as_slice::<bool>() {
        Ok(results) => {
            println!("      2D full extraction: {:?}", results);

            // Test individual slice extraction (like our extract_bool_value)
            for i in 0..3 {
                let slice = any_breach.clone().slice([i..i + 1, 0..1]);
                match slice.to_data().as_slice::<bool>() {
                    Ok(slice_data) => {
                        if !slice_data.is_empty() {
                            let slice_value = slice_data[0];
                            let full_value = results[i];
                            println!("      Element {}: full={}, slice={}", i, full_value, slice_value);

                            if slice_value != full_value {
                                println!("      ❌ Slice extraction mismatch at element {}", i);
                            }
                        } else {
                            println!("      ❌ Empty slice for element {}", i);
                        }
                    }
                    Err(e) => {
                        println!("      ❌ Slice extraction failed for element {}: {:?}", i, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("      ❌ 2D breach detection failed: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "gpu"))]
fn test_basic_tensor_operations(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_boolean_operations(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_comparison_operations(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_breach_detection_scenario(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }