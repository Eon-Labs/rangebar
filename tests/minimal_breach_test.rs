//! Minimal Breach Detection Test
//!
//! This test isolates the exact breach detection scenario that's failing
//! in our GPU implementation with simple, guaranteed-to-work data.

#[cfg(feature = "gpu")]
use burn::{
    backend::wgpu::{Wgpu, WgpuDevice},
    tensor::{Tensor, TensorData},
};

#[cfg(feature = "gpu")]
type TestBackend = Wgpu;

#[cfg(feature = "gpu")]
fn get_test_device() -> Option<WgpuDevice> {
    Some(WgpuDevice::default())
}

#[test]
#[cfg(feature = "gpu")]
fn test_minimal_breach_detection() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Minimal Breach Detection Test");
    println!("Testing the exact scenario from our GPU implementation");

    // Simple test case with guaranteed breaches
    let open_price = 100.0;
    let threshold_percent = 0.10; // 10% for easy calculation

    let upper_threshold = open_price * (1.0 + threshold_percent); // 110.0
    let lower_threshold = open_price * (1.0 - threshold_percent); // 90.0

    // Test prices with obvious breaches
    let test_prices = vec![
        100.0, // No breach (at open)
        115.0, // Upper breach (15% above)
        85.0,  // Lower breach (15% below)
        105.0, // No breach (5% above)
        95.0,  // No breach (5% below)
    ];

    println!("🔍 Test scenario:");
    println!("   Open price: {}", open_price);
    println!("   Upper threshold: {} (should breach if >= this)", upper_threshold);
    println!("   Lower threshold: {} (should breach if <= this)", lower_threshold);
    println!("   Test prices: {:?}", test_prices);

    // Expected results
    let expected_upper = [false, true, false, false, false];
    let expected_lower = [false, false, true, false, false];
    let expected_any = [false, true, true, false, false];

    println!("🔍 Expected results:");
    println!("   Upper breaches: {:?}", expected_upper);
    println!("   Lower breaches: {:?}", expected_lower);
    println!("   Any breach: {:?}", expected_any);

    // Create GPU tensors
    let price_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(test_prices.clone(), [5]),
        &device,
    );

    let upper_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![upper_threshold; 5], [5]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![lower_threshold; 5], [5]),
        &device,
    );

    println!("✅ Created GPU tensors [5]");

    // Test upper breach detection
    println!("\n🔍 Testing upper breach detection:");
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let upper_data = upper_breach.to_data();

    match upper_data.as_slice::<bool>() {
        Ok(upper_results) => {
            println!("   GPU results: {:?}", upper_results);
            println!("   Expected:    {:?}", expected_upper);

            if upper_results == expected_upper {
                println!("   ✅ Upper breach detection: PASSED");
            } else {
                println!("   ❌ Upper breach detection: FAILED");
                for (i, (&actual, &expected)) in upper_results.iter().zip(expected_upper.iter()).enumerate() {
                    if actual != expected {
                        println!("      Mismatch at index {}: {} >= {} should be {}, got {}",
                            i, test_prices[i], upper_threshold, expected, actual);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Upper breach extraction failed: {:?}", e);
        }
    }

    // Test lower breach detection
    println!("\n🔍 Testing lower breach detection:");
    let lower_breach = price_tensor.clone().lower_equal(lower_tensor);
    let lower_data = lower_breach.to_data();

    match lower_data.as_slice::<bool>() {
        Ok(lower_results) => {
            println!("   GPU results: {:?}", lower_results);
            println!("   Expected:    {:?}", expected_lower);

            if lower_results == expected_lower {
                println!("   ✅ Lower breach detection: PASSED");
            } else {
                println!("   ❌ Lower breach detection: FAILED");
                for (i, (&actual, &expected)) in lower_results.iter().zip(expected_lower.iter()).enumerate() {
                    if actual != expected {
                        println!("      Mismatch at index {}: {} <= {} should be {}, got {}",
                            i, test_prices[i], lower_threshold, expected, actual);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Lower breach extraction failed: {:?}", e);
        }
    }

    // Test combined breach detection (like our any_breach logic)
    println!("\n🔍 Testing combined breach detection:");
    let combined_breach = upper_breach.bool_or(lower_breach);
    let combined_data = combined_breach.to_data();

    match combined_data.as_slice::<bool>() {
        Ok(combined_results) => {
            println!("   GPU results: {:?}", combined_results);
            println!("   Expected:    {:?}", expected_any);

            if combined_results == expected_any {
                println!("   ✅ Combined breach detection: PASSED");
            } else {
                println!("   ❌ Combined breach detection: FAILED");
            }
        }
        Err(e) => {
            println!("   ❌ Combined breach extraction failed: {:?}", e);
        }
    }

    println!("\n🎯 Minimal breach detection test completed");
}

#[test]
#[cfg(feature = "gpu")]
fn test_2d_minimal_breach_detection() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 2D Minimal Breach Detection Test (like our implementation)");

    // Same test case but with 2D tensors [5,1]
    let open_price = 100.0;
    let threshold_percent = 0.10;

    let upper_threshold = open_price * (1.0 + threshold_percent);
    let lower_threshold = open_price * (1.0 - threshold_percent);

    let test_prices = vec![100.0, 115.0, 85.0, 105.0, 95.0];
    let expected_any = [false, true, true, false, false];

    // Create 2D tensors [5,1] like our implementation
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_prices.clone(), [5, 1]),
        &device,
    );

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper_threshold; 5], [5, 1]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![lower_threshold; 5], [5, 1]),
        &device,
    );

    println!("✅ Created 2D GPU tensors [5,1]");

    // Test breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.bool_or(lower_breach);

    // Test full extraction
    let breach_data = any_breach.to_data();
    match breach_data.as_slice::<bool>() {
        Ok(breach_results) => {
            println!("🔍 2D breach results: {:?}", breach_results);
            println!("🔍 Expected:          {:?}", expected_any);

            if breach_results == expected_any {
                println!("✅ 2D breach detection: PASSED");
            } else {
                println!("❌ 2D breach detection: FAILED");
            }
        }
        Err(e) => {
            println!("❌ 2D breach extraction failed: {:?}", e);
        }
    }

    // Test individual element extraction (like our extract_bool_value)
    println!("\n🔍 Testing individual element extraction from 2D breach tensor:");
    for i in 0..5 {
        let slice = any_breach.clone().slice([i..i + 1, 0..1]);
        let slice_data = slice.to_data();

        match slice_data.as_slice::<bool>() {
            Ok(slice_result) => {
                if !slice_result.is_empty() {
                    let extracted = slice_result[0];
                    let expected = expected_any[i];
                    println!("   Element {}: extracted={}, expected={}", i, extracted, expected);

                    if extracted != expected {
                        println!("   ❌ Slice extraction wrong for element {}", i);
                    }
                } else {
                    println!("   ❌ Empty slice for element {}", i);
                }
            }
            Err(e) => {
                println!("   ❌ Slice extraction failed for element {}: {:?}", i, e);
            }
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_exact_range_bar_values() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing with exact range bar values from our implementation");

    // Use the exact values we saw in our debug output
    let open_price = 50750.0;
    let threshold_bps = 8000; // 0.8%
    let threshold_multiplier = (threshold_bps as f32) / 1_000_000.0; // 0.008

    let upper_threshold = open_price * (1.0 + threshold_multiplier); // 51156.0
    let lower_threshold = open_price * (1.0 - threshold_multiplier); // 50344.0

    // Test with prices from our debug output plus some that should breach
    let test_prices = vec![
        50750.0,  // Open (no breach)
        50788.0,  // Small increase (no breach)
        51200.0,  // Definite upper breach
        50300.0,  // Definite lower breach
        51156.0,  // Exact upper threshold (should breach)
        50344.0,  // Exact lower threshold (should breach)
    ];

    println!("🔍 Exact range bar scenario:");
    println!("   Open: {:.1}", open_price);
    println!("   Upper threshold: {:.1}", upper_threshold);
    println!("   Lower threshold: {:.1}", lower_threshold);
    println!("   Threshold %: {:.6}%", threshold_multiplier * 100.0);

    for (i, &price) in test_prices.iter().enumerate() {
        let upper_breach = price >= upper_threshold;
        let lower_breach = price <= lower_threshold;
        let any_breach = upper_breach || lower_breach;
        println!("   Price {}: {:.1} -> upper={}, lower={}, any={}",
            i, price, upper_breach, lower_breach, any_breach);
    }

    // Create tensors
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_prices.clone(), [6, 1]),
        &device,
    );

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper_threshold; 6], [6, 1]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![lower_threshold; 6], [6, 1]),
        &device,
    );

    // Perform breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.bool_or(lower_breach);

    // Check results
    let breach_data = any_breach.to_data();
    match breach_data.as_slice::<bool>() {
        Ok(gpu_results) => {
            println!("\n🔍 GPU breach results: {:?}", gpu_results);

            // Manual calculation for verification
            let manual_results: Vec<bool> = test_prices.iter().map(|&price| {
                price >= upper_threshold || price <= lower_threshold
            }).collect();

            println!("🔍 Manual calculation: {:?}", manual_results);

            if gpu_results == manual_results {
                println!("✅ Exact range bar values: PASSED");
            } else {
                println!("❌ Exact range bar values: FAILED");
                for (i, (&gpu, &manual)) in gpu_results.iter().zip(manual_results.iter()).enumerate() {
                    if gpu != manual {
                        println!("   Mismatch at index {}: GPU={}, Manual={}", i, gpu, manual);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Exact range bar breach extraction failed: {:?}", e);
        }
    }
}