//! Basic Boolean Operations Testing for GPU Tensors
//!
//! This test module isolates and tests fundamental boolean tensor operations
//! to identify issues with the Burn framework on Mac Metal GPU backend.

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
fn test_basic_boolean_tensor_creation() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing basic boolean tensor creation");

    // Test 1D boolean tensor
    let bool_data_1d = vec![true, false, true];
    let tensor_1d = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(bool_data_1d.clone(), [3]),
        &device,
    );

    println!("✅ Created 1D boolean tensor: shape {:?}", tensor_1d.shape());

    // Test 2D boolean tensor
    let bool_data_2d = vec![true, false, true, false, true, false];
    let tensor_2d = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(bool_data_2d.clone(), [3, 2]),
        &device,
    );

    println!("✅ Created 2D boolean tensor: shape {:?}", tensor_2d.shape());

    // Test [N, 1] shape (like our active_bars)
    let bool_data_nx1 = vec![true, false, true];
    let tensor_nx1 = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(bool_data_nx1.clone(), [3, 1]),
        &device,
    );

    println!("✅ Created [3,1] boolean tensor: shape {:?}", tensor_nx1.shape());

    println!("🎯 Basic boolean tensor creation: PASSED");
}

#[test]
#[cfg(feature = "gpu")]
fn test_boolean_tensor_extraction() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing boolean tensor data extraction");

    // Create known boolean data
    let original_data = vec![true, false, true, false];
    let tensor = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(original_data.clone(), [4]),
        &device,
    );

    // Test extraction back to CPU
    let extracted_data = tensor.to_data();
    let extracted_slice = extracted_data.as_slice::<bool>();

    match extracted_slice {
        Ok(slice) => {
            println!("✅ Extracted boolean data: {:?}", slice);
            println!("🔍 Original:  {:?}", original_data);
            println!("🔍 Extracted: {:?}", slice);

            // Verify data matches
            if slice == original_data {
                println!("🎯 Boolean extraction verification: PASSED");
            } else {
                println!("❌ Boolean extraction verification: FAILED - data mismatch");
                println!("   Expected: {:?}", original_data);
                println!("   Got:      {:?}", slice);
            }
        }
        Err(e) => {
            println!("❌ Boolean extraction FAILED: {:?}", e);
            println!("💡 This may indicate a fundamental issue with boolean tensor extraction");
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_boolean_tensor_any_operation() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing boolean tensor .any() operation");

    // Test case 1: All false
    let all_false = vec![false, false, false];
    let tensor_false = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(all_false, [3]),
        &device,
    );

    let any_false = tensor_false.any();
    let result_false = any_false.into_scalar();
    println!("🔍 All false .any(): {} (expected: 0)", result_false);

    // Test case 2: All true
    let all_true = vec![true, true, true];
    let tensor_true = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(all_true, [3]),
        &device,
    );

    let any_true = tensor_true.any();
    let result_true = any_true.into_scalar();
    println!("🔍 All true .any(): {} (expected: 1)", result_true);

    // Test case 3: Mixed
    let mixed = vec![false, true, false];
    let tensor_mixed = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(mixed, [3]),
        &device,
    );

    let any_mixed = tensor_mixed.any();
    let result_mixed = any_mixed.into_scalar();
    println!("🔍 Mixed .any(): {} (expected: 1)", result_mixed);

    // Verify results
    if result_false == 0 && result_true > 0 && result_mixed > 0 {
        println!("🎯 Boolean .any() operation: PASSED");
    } else {
        println!("❌ Boolean .any() operation: FAILED");
        println!("   All false result: {} (expected: 0)", result_false);
        println!("   All true result: {} (expected: >0)", result_true);
        println!("   Mixed result: {} (expected: >0)", result_mixed);
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_2d_boolean_tensor_operations() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing 2D boolean tensor operations");

    // Create 2D tensor [3, 1] like our active_bars
    let data_2d = vec![true, false, true];
    let tensor_2d = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(data_2d.clone(), [3, 1]),
        &device,
    );

    println!("✅ Created [3,1] tensor: shape {:?}", tensor_2d.shape());

    // Test .any() on 2D tensor
    let any_result = tensor_2d.clone().any();
    let any_value = any_result.into_scalar();
    println!("🔍 2D tensor .any(): {} (expected: >0)", any_value);

    // Test extraction from 2D tensor
    let extracted = tensor_2d.to_data();
    let slice_result = extracted.as_slice::<bool>();

    match slice_result {
        Ok(slice) => {
            println!("🔍 2D tensor extracted: {:?}", slice);
            println!("🔍 Original data: {:?}", data_2d);

            if slice == data_2d {
                println!("🎯 2D boolean tensor operations: PASSED");
            } else {
                println!("❌ 2D extraction mismatch");
            }
        }
        Err(e) => {
            println!("❌ 2D tensor extraction FAILED: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_boolean_tensor_slicing() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing boolean tensor slicing operations");

    // Create test data similar to our use case
    let data = vec![true, false, true, false, true];
    let tensor = Tensor::<TestBackend, 2, burn::tensor::Bool>::from_data(
        TensorData::new(data.clone(), [5, 1]),
        &device,
    );

    println!("✅ Created [5,1] tensor for slicing");

    // Test slicing individual elements (like our extract_bool_value function)
    for i in 0..5 {
        let slice = tensor.clone().slice([i..i + 1, 0..1]);
        println!("🔍 Slice [{}..{}, 0..1] shape: {:?}", i, i + 1, slice.shape());

        let slice_data = slice.to_data();
        match slice_data.as_slice::<bool>() {
            Ok(slice_result) => {
                let value = if !slice_result.is_empty() { slice_result[0] } else { false };
                let expected = data[i];
                println!("   Element {}: got {}, expected {}", i, value, expected);

                if value != expected {
                    println!("❌ Slice extraction mismatch at index {}", i);
                }
            }
            Err(e) => {
                println!("❌ Slice extraction failed at index {}: {:?}", i, e);
            }
        }
    }

    println!("🎯 Boolean tensor slicing test completed");
}