//! Boolean Tensor Extraction Testing
//!
//! Tests the specific tensor slicing and extraction operations that are
//! failing in our extract_bool_value function.

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
fn test_extract_bool_value_simulation() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing extract_bool_value function simulation");

    // Simulate the exact scenario from our extract_bool_value function
    let bool_data = vec![true, false, true, false, true, false];
    let tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(bool_data.clone(), [6, 1]),
        &device,
    );

    println!("✅ Created [6,1] boolean tensor");
    println!("🔍 Original data: {:?}", bool_data);

    // Test extraction for each symbol (like our loop in complete_breached_bars)
    for symbol_idx in 0..6 {
        println!("\n🔍 Testing symbol_idx {}", symbol_idx);

        // This is exactly what our extract_bool_value function does
        let slice = tensor.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        println!("   Slice range: [{}..{}, 0..1]", symbol_idx, symbol_idx + 1);
        println!("   Slice shape: {:?}", slice.shape());

        let data = slice.to_data();
        match data.as_slice::<bool>() {
            Ok(bool_slice) => {
                println!("   Extracted slice: {:?}", bool_slice);

                if !bool_slice.is_empty() {
                    let extracted_value = bool_slice[0];
                    let expected_value = bool_data[symbol_idx];

                    println!("   Extracted value: {} (expected: {})", extracted_value, expected_value);

                    if extracted_value == expected_value {
                        println!("   ✅ Symbol {} extraction: CORRECT", symbol_idx);
                    } else {
                        println!("   ❌ Symbol {} extraction: WRONG", symbol_idx);
                    }
                } else {
                    println!("   ❌ Symbol {} extraction: EMPTY SLICE", symbol_idx);
                }
            }
            Err(e) => {
                println!("   ❌ Symbol {} extraction: FAILED with error {:?}", symbol_idx, e);
            }
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_comparison_result_extraction() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing extraction from comparison operation results");

    // Create scenario that should produce known boolean results
    let prices = vec![50.0, 60.0, 40.0, 55.0, 45.0];
    let thresholds = vec![55.0, 55.0, 55.0, 55.0, 55.0];

    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices.clone(), [5, 1]),
        &device,
    );

    let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(thresholds.clone(), [5, 1]),
        &device,
    );

    // Perform comparison (this is what happens in detect_parallel_breaches)
    let breach_tensor = price_tensor.greater_equal(threshold_tensor);

    println!("🔍 Testing extraction from comparison results:");
    for i in 0..5 {
        println!("   Price {}: {} >= {} = {}", i, prices[i], thresholds[i], prices[i] >= thresholds[i]);
    }

    // Test individual element extraction (like our extract_bool_value)
    for symbol_idx in 0..5 {
        let slice = breach_tensor.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        let data = slice.to_data();

        match data.as_slice::<bool>() {
            Ok(bool_slice) => {
                if !bool_slice.is_empty() {
                    let extracted = bool_slice[0];
                    let expected = prices[symbol_idx] >= thresholds[symbol_idx];

                    println!("🔍 Symbol {}: extracted={}, expected={}", symbol_idx, extracted, expected);

                    if extracted != expected {
                        println!("❌ Comparison extraction mismatch at symbol {}", symbol_idx);
                    }
                } else {
                    println!("❌ Empty slice for symbol {}", symbol_idx);
                }
            }
            Err(e) => {
                println!("❌ Extraction failed for symbol {}: {:?}", symbol_idx, e);
            }
        }
    }

    // Also test full tensor extraction for comparison
    let full_data = breach_tensor.to_data();
    match full_data.as_slice::<bool>() {
        Ok(full_results) => {
            println!("🔍 Full tensor extraction: {:?}", full_results);
            let expected = [false, true, false, false, false];
            println!("🔍 Expected results: {:?}", expected);

            if full_results == expected {
                println!("✅ Full tensor extraction: CORRECT");
            } else {
                println!("❌ Full tensor extraction: WRONG");
            }
        }
        Err(e) => {
            println!("❌ Full tensor extraction failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_active_bars_scenario() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing active_bars scenario (our exact use case)");

    // Simulate our active_bars tensor creation and usage
    let num_symbols = 18; // MAX_TIER1_SYMBOLS
    let active_data = vec![true; num_symbols];

    let active_bars = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(active_data.clone(), [num_symbols, 1]),
        &device,
    );

    println!("✅ Created active_bars tensor [18,1] with all true");

    // Test .any() operation (this was working)
    let any_result = active_bars.clone().any();
    let any_value = any_result.into_scalar();
    println!("🔍 Active bars .any(): {} (expected: >0)", any_value);

    // Test individual symbol extraction (this was failing)
    for symbol_idx in 0..3 { // Test first 3 symbols
        let slice = active_bars.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        let data = slice.to_data();

        match data.as_slice::<bool>() {
            Ok(bool_slice) => {
                if !bool_slice.is_empty() {
                    let value = bool_slice[0];
                    println!("🔍 Symbol {} active: {} (expected: true)", symbol_idx, value);

                    if !value {
                        println!("❌ Symbol {} should be active but shows false", symbol_idx);
                    }
                } else {
                    println!("❌ Empty slice for symbol {}", symbol_idx);
                }
            }
            Err(e) => {
                println!("❌ Active bars extraction failed for symbol {}: {:?}", symbol_idx, e);
            }
        }
    }

    // Test scenario where some bars become inactive
    let mixed_data = vec![
        true, false, true, false, true, false,
        true, false, true, false, true, false,
        true, false, true, false, true, false
    ];

    let mixed_bars = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(mixed_data.clone(), [18, 1]),
        &device,
    );

    println!("\n🔍 Testing mixed active/inactive scenario:");
    println!("   Data pattern: alternating true/false");

    // Test any() on mixed data
    let mixed_any = mixed_bars.clone().any();
    let mixed_any_value = mixed_any.into_scalar();
    println!("🔍 Mixed bars .any(): {} (expected: >0)", mixed_any_value);

    // Test individual extractions
    for symbol_idx in 0..6 {
        let slice = mixed_bars.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        let data = slice.to_data();

        match data.as_slice::<bool>() {
            Ok(bool_slice) => {
                if !bool_slice.is_empty() {
                    let value = bool_slice[0];
                    let expected = mixed_data[symbol_idx];
                    println!("🔍 Symbol {} mixed: {} (expected: {})", symbol_idx, value, expected);

                    if value != expected {
                        println!("❌ Mixed scenario mismatch at symbol {}", symbol_idx);
                    }
                }
            }
            Err(e) => {
                println!("❌ Mixed bars extraction failed for symbol {}: {:?}", symbol_idx, e);
            }
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_cpu_gpu_conversion_roundtrip() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing CPU→GPU→CPU conversion roundtrip");

    // Original CPU data
    let original_cpu_data = vec![true, false, true, false, true, false, true, false, true];
    println!("🔍 Original CPU data: {:?}", original_cpu_data);

    // CPU → GPU
    let gpu_tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(original_cpu_data.clone(), [9, 1]),
        &device,
    );

    println!("✅ Converted to GPU tensor [9,1]");

    // GPU → CPU (full extraction)
    let gpu_data = gpu_tensor.to_data();
    match gpu_data.as_slice::<bool>() {
        Ok(extracted_cpu_data) => {
            println!("🔍 Extracted CPU data: {:?}", extracted_cpu_data);

            if extracted_cpu_data == original_cpu_data {
                println!("✅ Full roundtrip: PASSED");
            } else {
                println!("❌ Full roundtrip: FAILED");
                println!("   Original: {:?}", original_cpu_data);
                println!("   Extracted: {:?}", extracted_cpu_data);
            }

            // Test element-by-element extraction (slice-based)
            let mut all_slices_correct = true;
            for i in 0..9 {
                let slice = gpu_tensor.clone().slice([i..i + 1, 0..1]);
                let slice_data = slice.to_data();

                match slice_data.as_slice::<bool>() {
                    Ok(slice_result) => {
                        if !slice_result.is_empty() {
                            let slice_value = slice_result[0];
                            let expected_value = original_cpu_data[i];

                            if slice_value != expected_value {
                                println!("❌ Slice extraction mismatch at index {}: got {}, expected {}",
                                    i, slice_value, expected_value);
                                all_slices_correct = false;
                            }
                        } else {
                            println!("❌ Empty slice at index {}", i);
                            all_slices_correct = false;
                        }
                    }
                    Err(e) => {
                        println!("❌ Slice extraction failed at index {}: {:?}", i, e);
                        all_slices_correct = false;
                    }
                }
            }

            if all_slices_correct {
                println!("✅ Slice-based extraction: PASSED");
            } else {
                println!("❌ Slice-based extraction: FAILED");
            }

        }
        Err(e) => {
            println!("❌ GPU→CPU conversion failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_zero_vs_nonzero_boolean_extraction() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing edge cases: all false vs mixed boolean extraction");

    // Test all false scenario
    let all_false_data = vec![false; 5];
    let false_tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(all_false_data.clone(), [5, 1]),
        &device,
    );

    println!("🔍 Testing all false tensor:");
    let false_extracted = false_tensor.to_data();
    match false_extracted.as_slice::<bool>() {
        Ok(results) => {
            println!("   Extracted: {:?}", results);
            if results == all_false_data {
                println!("   ✅ All false extraction: CORRECT");
            } else {
                println!("   ❌ All false extraction: WRONG");
            }
        }
        Err(e) => {
            println!("   ❌ All false extraction failed: {:?}", e);
        }
    }

    // Test all true scenario
    let all_true_data = vec![true; 5];
    let true_tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(all_true_data.clone(), [5, 1]),
        &device,
    );

    println!("🔍 Testing all true tensor:");
    let true_extracted = true_tensor.to_data();
    match true_extracted.as_slice::<bool>() {
        Ok(results) => {
            println!("   Extracted: {:?}", results);
            if results == all_true_data {
                println!("   ✅ All true extraction: CORRECT");
            } else {
                println!("   ❌ All true extraction: WRONG");
            }
        }
        Err(e) => {
            println!("   ❌ All true extraction failed: {:?}", e);
        }
    }

    // Test single true scenario
    let single_true_data = vec![false, false, true, false, false];
    let single_tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(single_true_data.clone(), [5, 1]),
        &device,
    );

    println!("🔍 Testing single true tensor:");
    let single_extracted = single_tensor.to_data();
    match single_extracted.as_slice::<bool>() {
        Ok(results) => {
            println!("   Extracted: {:?}", results);
            if results == single_true_data {
                println!("   ✅ Single true extraction: CORRECT");
            } else {
                println!("   ❌ Single true extraction: WRONG");
            }
        }
        Err(e) => {
            println!("   ❌ Single true extraction failed: {:?}", e);
        }
    }
}