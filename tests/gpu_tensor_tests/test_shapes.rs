//! Tensor Shape and Broadcasting Testing
//!
//! Tests different tensor shapes and their interactions to identify
//! potential shape-related issues in our GPU operations.

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
fn test_tensor_shape_variations() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing various tensor shapes");

    let test_data = vec![1.0, 2.0, 3.0, 4.0];

    // Test different shapes with same data
    let shapes_to_test = vec![
        ([4], "1D: [4]"),
        ([4, 1], "2D: [4,1]"),
        ([1, 4], "2D: [1,4]"),
        ([2, 2], "2D: [2,2]"),
    ];

    for (shape, description) in shapes_to_test {
        match shape {
            [dim1] => {
                let tensor = Tensor::<TestBackend, 1>::from_data(
                    TensorData::new(test_data.clone(), [dim1]),
                    &device,
                );
                println!("✅ Created tensor {}: shape {:?}", description, tensor.shape());

                // Test extraction
                let extracted = tensor.to_data();
                match extracted.as_slice::<f32>() {
                    Ok(data) => {
                        println!("   Extracted data: {:?}", data);
                    }
                    Err(e) => {
                        println!("   ❌ Extraction failed: {:?}", e);
                    }
                }
            }
            [dim1, dim2] => {
                let tensor = Tensor::<TestBackend, 2>::from_data(
                    TensorData::new(test_data.clone(), [dim1, dim2]),
                    &device,
                );
                println!("✅ Created tensor {}: shape {:?}", description, tensor.shape());

                // Test extraction
                let extracted = tensor.to_data();
                match extracted.as_slice::<f32>() {
                    Ok(data) => {
                        println!("   Extracted data: {:?}", data);
                    }
                    Err(e) => {
                        println!("   ❌ Extraction failed: {:?}", e);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_shape_compatibility_operations() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing operations between different shapes");

    // Test data
    let data_4 = vec![10.0, 20.0, 30.0, 40.0];
    let data_scalar = vec![15.0];

    // Create tensors with different shapes
    let tensor_4x1 = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(data_4.clone(), [4, 1]),
        &device,
    );

    let tensor_1x4 = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(data_4.clone(), [1, 4]),
        &device,
    );

    let tensor_4_1d = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(data_4.clone(), [4]),
        &device,
    );

    let scalar_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(data_scalar.clone(), [1]),
        &device,
    );

    println!("✅ Created tensors:");
    println!("   tensor_4x1: {:?}", tensor_4x1.shape());
    println!("   tensor_1x4: {:?}", tensor_1x4.shape());
    println!("   tensor_4_1d: {:?}", tensor_4_1d.shape());
    println!("   scalar_tensor: {:?}", scalar_tensor.shape());

    // Test comparisons with scalar (element-wise)
    println!("\n🔍 Testing scalar comparisons:");
    let scalar_value = 25.0;

    // [4,1] vs scalar
    let result_4x1 = tensor_4x1.greater_equal_elem(scalar_value);
    match result_4x1.to_data().as_slice::<bool>() {
        Ok(data) => {
            println!("   [4,1] >= {}: {:?}", scalar_value, data);
            // Expected: [false, false, true, true] for [10,20,30,40] >= 25
        }
        Err(e) => {
            println!("   ❌ [4,1] comparison failed: {:?}", e);
        }
    }

    // [1,4] vs scalar
    let result_1x4 = tensor_1x4.greater_equal_elem(scalar_value);
    match result_1x4.to_data().as_slice::<bool>() {
        Ok(data) => {
            println!("   [1,4] >= {}: {:?}", scalar_value, data);
        }
        Err(e) => {
            println!("   ❌ [1,4] comparison failed: {:?}", e);
        }
    }

    // [4] vs scalar
    let result_4_1d = tensor_4_1d.greater_equal_elem(scalar_value);
    match result_4_1d.to_data().as_slice::<bool>() {
        Ok(data) => {
            println!("   [4] >= {}: {:?}", scalar_value, data);
        }
        Err(e) => {
            println!("   ❌ [4] comparison failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_tensor_broadcasting_scenarios() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing tensor broadcasting scenarios");

    // Scenario 1: [3,1] vs [3,1] (our typical case)
    let prices_data = vec![50.0, 55.0, 45.0];
    let thresholds_data = vec![52.0, 52.0, 52.0];

    let prices_3x1 = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices_data.clone(), [3, 1]),
        &device,
    );

    let thresholds_3x1 = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(thresholds_data.clone(), [3, 1]),
        &device,
    );

    println!("🔍 Testing [3,1] vs [3,1] comparison:");
    let result_3x1 = prices_3x1.greater_equal(thresholds_3x1);
    match result_3x1.to_data().as_slice::<bool>() {
        Ok(data) => {
            println!("   Prices vs thresholds: {:?}", data);
            // Expected: [false, true, false] for [50,55,45] >= [52,52,52]
            let expected = [false, true, false];
            if data == expected {
                println!("   ✅ [3,1] vs [3,1]: CORRECT");
            } else {
                println!("   ❌ [3,1] vs [3,1]: WRONG - expected {:?}", expected);
            }
        }
        Err(e) => {
            println!("   ❌ [3,1] comparison failed: {:?}", e);
        }
    }

    // Scenario 2: [3] vs [3] (flattened)
    let prices_3 = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(prices_data.clone(), [3]),
        &device,
    );

    let thresholds_3 = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(thresholds_data.clone(), [3]),
        &device,
    );

    println!("🔍 Testing [3] vs [3] comparison:");
    let result_3 = prices_3.greater_equal(thresholds_3);
    match result_3.to_data().as_slice::<bool>() {
        Ok(data) => {
            println!("   Prices vs thresholds: {:?}", data);
            let expected = [false, true, false];
            if data == expected {
                println!("   ✅ [3] vs [3]: CORRECT");
            } else {
                println!("   ❌ [3] vs [3]: WRONG - expected {:?}", expected);
            }
        }
        Err(e) => {
            println!("   ❌ [3] comparison failed: {:?}", e);
        }
    }

    // Scenario 3: Mixed shapes (if supported)
    println!("🔍 Testing mixed shape operations:");

    // Try [3,1] vs [3] - this might fail due to shape incompatibility
    // which could explain our issues
    // Note: This test is to see if Burn handles broadcasting correctly
}

#[test]
#[cfg(feature = "gpu")]
fn test_max_tier1_symbols_scenario() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing MAX_TIER1_SYMBOLS scenario (18 symbols)");

    // Simulate our exact scenario with 18 symbols
    let num_symbols = 18;
    let prices: Vec<f32> = (0..num_symbols).map(|i| 50000.0 + (i as f32) * 100.0).collect();
    let thresholds: Vec<f32> = vec![50500.0; num_symbols]; // Some should breach, some shouldn't

    println!("🔍 Creating [18,1] tensors like our implementation:");
    println!("   Price range: {:.1} to {:.1}", prices[0], prices[17]);
    println!("   Threshold: {:.1}", thresholds[0]);

    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices.clone(), [num_symbols, 1]),
        &device,
    );

    let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(thresholds.clone(), [num_symbols, 1]),
        &device,
    );

    println!("✅ Created [18,1] tensors successfully");

    // Test comparison operation
    let breach_tensor = price_tensor.greater_equal(threshold_tensor);
    println!("✅ Comparison operation completed");

    // Test full extraction
    let breach_data = breach_tensor.to_data();
    match breach_data.as_slice::<bool>() {
        Ok(results) => {
            println!("🔍 Breach results (first 10): {:?}", &results[..10]);

            // Count breaches
            let breach_count = results.iter().filter(|&&x| x).count();
            println!("🔍 Total breaches: {} out of {}", breach_count, num_symbols);

            // Verify a few specific indices
            for i in [0, 5, 10, 15, 17] {
                let price = prices[i];
                let threshold = thresholds[i];
                let should_breach = price >= threshold;
                let actual_breach = results[i];

                println!("   Symbol {}: {:.1} >= {:.1} = {} (actual: {})",
                    i, price, threshold, should_breach, actual_breach);

                if should_breach != actual_breach {
                    println!("   ❌ Mismatch at symbol {}", i);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to extract 18-symbol comparison results: {:?}", e);
        }
    }

    // Test individual slice extraction (our problematic operation)
    println!("\n🔍 Testing individual symbol extraction:");
    for symbol_idx in [0, 5, 10, 15, 17] {
        let slice = breach_tensor.clone().slice([symbol_idx..symbol_idx + 1, 0..1]);
        let slice_data = slice.to_data();

        match slice_data.as_slice::<bool>() {
            Ok(slice_result) => {
                if !slice_result.is_empty() {
                    let extracted = slice_result[0];
                    let expected = prices[symbol_idx] >= thresholds[symbol_idx];
                    println!("   Symbol {}: extracted={}, expected={}", symbol_idx, extracted, expected);

                    if extracted != expected {
                        println!("   ❌ Slice extraction wrong for symbol {}", symbol_idx);
                    }
                } else {
                    println!("   ❌ Empty slice for symbol {}", symbol_idx);
                }
            }
            Err(e) => {
                println!("   ❌ Slice extraction failed for symbol {}: {:?}", symbol_idx, e);
            }
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_reshape_operations() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing tensor reshape operations");

    // Start with 1D data
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    // Create 1D tensor
    let tensor_1d = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(data.clone(), [6]),
        &device,
    );

    println!("✅ Created 1D tensor [6]: {:?}", tensor_1d.shape());

    // Reshape to 2D
    let tensor_2d = tensor_1d.reshape([6, 1]);
    println!("✅ Reshaped to [6,1]: {:?}", tensor_2d.shape());

    // Test operations on reshaped tensor
    let comparison_result = tensor_2d.greater_equal_elem(3.5);
    match comparison_result.to_data().as_slice::<bool>() {
        Ok(results) => {
            println!("🔍 Comparison results: {:?}", results);
            // Expected: [false, false, false, true, true, true] for [1,2,3,4,5,6] >= 3.5
        }
        Err(e) => {
            println!("❌ Reshaped tensor comparison failed: {:?}", e);
        }
    }

    // Test reshape back to different 2D shape
    let tensor_2x3 = tensor_1d.reshape([2, 3]);
    println!("✅ Reshaped to [2,3]: {:?}", tensor_2x3.shape());

    // Test extraction from different shaped tensor
    let reshaped_data = tensor_2x3.to_data();
    match reshaped_data.as_slice::<f32>() {
        Ok(extracted) => {
            println!("🔍 Reshaped [2,3] data: {:?}", extracted);
            if extracted == data {
                println!("✅ Reshape preserves data order");
            } else {
                println!("❌ Reshape changed data order");
            }
        }
        Err(e) => {
            println!("❌ Reshaped tensor extraction failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_dimension_edge_cases() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing dimension edge cases");

    // Test single element tensor
    let single_element = vec![42.0];
    let tensor_single = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(single_element.clone(), [1, 1]),
        &device,
    );

    println!("✅ Created [1,1] single element tensor");

    // Test operations on single element
    let single_comparison = tensor_single.greater_equal_elem(40.0);
    match single_comparison.to_data().as_slice::<bool>() {
        Ok(result) => {
            println!("🔍 Single element comparison: {:?}", result);
            if result == [true] {
                println!("✅ Single element operation: CORRECT");
            } else {
                println!("❌ Single element operation: WRONG");
            }
        }
        Err(e) => {
            println!("❌ Single element comparison failed: {:?}", e);
        }
    }

    // Test slice extraction from single element
    let single_slice = tensor_single.slice([0..1, 0..1]);
    match single_slice.to_data().as_slice::<f32>() {
        Ok(slice_data) => {
            println!("🔍 Single element slice: {:?}", slice_data);
        }
        Err(e) => {
            println!("❌ Single element slice failed: {:?}", e);
        }
    }

    // Test empty-like scenarios (minimum valid sizes)
    let min_2d = vec![1.0, 2.0];
    let tensor_min = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(min_2d.clone(), [2, 1]),
        &device,
    );

    println!("✅ Created minimal [2,1] tensor");

    // Test slicing from minimal tensor
    for i in 0..2 {
        let slice = tensor_min.clone().slice([i..i + 1, 0..1]);
        match slice.to_data().as_slice::<f32>() {
            Ok(slice_data) => {
                println!("🔍 Minimal tensor slice {}: {:?}", i, slice_data);
            }
            Err(e) => {
                println!("❌ Minimal tensor slice {} failed: {:?}", i, e);
            }
        }
    }
}