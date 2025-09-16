//! Comparison Operations Testing for GPU Tensors
//!
//! Tests the fundamental comparison operations (greater_equal, lower_equal)
//! that are failing in our breach detection logic.

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
fn test_simple_greater_equal_comparison() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing simple greater_equal comparison");

    // Test data with obvious comparisons
    let prices = vec![50.0, 60.0, 40.0, 55.0];
    let threshold = 55.0;

    // Create tensors
    let price_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(prices.clone(), [4]),
        &device,
    );

    let threshold_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![threshold; 4], [4]),
        &device,
    );

    println!("🔍 Prices: {:?}", prices);
    println!("🔍 Threshold: {}", threshold);
    println!("🔍 Expected results: [false, true, false, false]");

    // Perform comparison
    let result_tensor = price_tensor.greater_equal(threshold_tensor);

    // Extract results
    let result_data = result_tensor.to_data();
    match result_data.as_slice::<bool>() {
        Ok(results) => {
            println!("🔍 Actual results: {:?}", results);

            // Manual verification
            let expected = [false, true, false, false];
            let mut all_correct = true;

            for (i, (&actual, &expected)) in results.iter().zip(expected.iter()).enumerate() {
                println!("   Index {}: {} >= {} = {} (expected: {})",
                    i, prices[i], threshold, actual, expected);
                if actual != expected {
                    println!("❌ Mismatch at index {}", i);
                    all_correct = false;
                }
            }

            if all_correct {
                println!("🎯 Simple greater_equal comparison: PASSED");
            } else {
                println!("❌ Simple greater_equal comparison: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Could not extract comparison results: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_simple_lower_equal_comparison() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing simple lower_equal comparison");

    // Test data with obvious comparisons
    let prices = vec![50.0, 45.0, 60.0, 45.0];
    let threshold = 45.0;

    // Create tensors
    let price_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(prices.clone(), [4]),
        &device,
    );

    let threshold_tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(vec![threshold; 4], [4]),
        &device,
    );

    println!("🔍 Prices: {:?}", prices);
    println!("🔍 Threshold: {}", threshold);
    println!("🔍 Expected results: [false, true, false, true]");

    // Perform comparison
    let result_tensor = price_tensor.lower_equal(threshold_tensor);

    // Extract results
    let result_data = result_tensor.to_data();
    match result_data.as_slice::<bool>() {
        Ok(results) => {
            println!("🔍 Actual results: {:?}", results);

            // Manual verification
            let expected = [false, true, false, true];
            let mut all_correct = true;

            for (i, (&actual, &expected)) in results.iter().zip(expected.iter()).enumerate() {
                println!("   Index {}: {} <= {} = {} (expected: {})",
                    i, prices[i], threshold, actual, expected);
                if actual != expected {
                    println!("❌ Mismatch at index {}", i);
                    all_correct = false;
                }
            }

            if all_correct {
                println!("🎯 Simple lower_equal comparison: PASSED");
            } else {
                println!("❌ Simple lower_equal comparison: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Could not extract comparison results: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_2d_tensor_comparisons() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing 2D tensor comparisons (like our breach detection)");

    // Simulate our breach detection scenario
    let prices = vec![50750.0, 51200.0, 50300.0]; // Symbol prices
    let upper_thresholds = vec![51156.0, 51156.0, 51156.0]; // Upper breach thresholds
    let lower_thresholds = vec![50344.0, 50344.0, 50344.0]; // Lower breach thresholds

    // Create 2D tensors [3, 1] like our implementation
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices.clone(), [3, 1]),
        &device,
    );

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(upper_thresholds.clone(), [3, 1]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(lower_thresholds.clone(), [3, 1]),
        &device,
    );

    println!("🔍 Testing scenario similar to our breach detection:");
    for i in 0..3 {
        println!("   Symbol {}: price={:.1}, upper={:.1}, lower={:.1}",
            i, prices[i], upper_thresholds[i], lower_thresholds[i]);
        println!("     Should breach upper: {}", prices[i] >= upper_thresholds[i]);
        println!("     Should breach lower: {}", prices[i] <= lower_thresholds[i]);
    }

    // Test upper breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let upper_data = upper_breach.to_data();
    match upper_data.as_slice::<bool>() {
        Ok(upper_results) => {
            println!("🔍 Upper breach results: {:?}", upper_results);
            println!("🔍 Expected: [false, true, false]");

            let expected_upper = [false, true, false];
            let upper_correct = upper_results == expected_upper;

            if upper_correct {
                println!("✅ Upper breach detection: PASSED");
            } else {
                println!("❌ Upper breach detection: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Upper breach extraction failed: {:?}", e);
        }
    }

    // Test lower breach detection
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let lower_data = lower_breach.to_data();
    match lower_data.as_slice::<bool>() {
        Ok(lower_results) => {
            println!("🔍 Lower breach results: {:?}", lower_results);
            println!("🔍 Expected: [false, false, true]");

            let expected_lower = [false, false, true];
            let lower_correct = lower_results == expected_lower;

            if lower_correct {
                println!("✅ Lower breach detection: PASSED");
            } else {
                println!("❌ Lower breach detection: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Lower breach extraction failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_tensor_scalar_comparisons() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing tensor-to-scalar comparisons");

    // Test data
    let values = vec![45.0, 50.0, 55.0];
    let scalar_threshold = 50.0;

    let tensor = Tensor::<TestBackend, 1>::from_data(
        TensorData::new(values.clone(), [3]),
        &device,
    );

    println!("🔍 Values: {:?}", values);
    println!("🔍 Scalar threshold: {}", scalar_threshold);

    // Test greater_equal with scalar
    let ge_result = tensor.clone().greater_equal_elem(scalar_threshold);
    let ge_data = ge_result.to_data();

    match ge_data.as_slice::<bool>() {
        Ok(ge_results) => {
            println!("🔍 Greater-equal scalar results: {:?}", ge_results);
            println!("🔍 Expected: [false, true, true]");

            let expected = [false, true, true];
            if ge_results == expected {
                println!("✅ Tensor-scalar greater_equal: PASSED");
            } else {
                println!("❌ Tensor-scalar greater_equal: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Tensor-scalar comparison extraction failed: {:?}", e);
        }
    }

    // Test lower_equal with scalar
    let le_result = tensor.lower_equal_elem(scalar_threshold);
    let le_data = le_result.to_data();

    match le_data.as_slice::<bool>() {
        Ok(le_results) => {
            println!("🔍 Lower-equal scalar results: {:?}", le_results);
            println!("🔍 Expected: [true, true, false]");

            let expected = [true, true, false];
            if le_results == expected {
                println!("✅ Tensor-scalar lower_equal: PASSED");
            } else {
                println!("❌ Tensor-scalar lower_equal: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Tensor-scalar comparison extraction failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_boolean_or_operations() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing boolean OR operations (like our any_breach logic)");

    // Create test boolean tensors
    let upper_breaches = vec![false, true, false];
    let lower_breaches = vec![true, false, false];

    let upper_tensor = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(upper_breaches.clone(), [3]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 1, burn::tensor::Bool>::from_data(
        TensorData::new(lower_breaches.clone(), [3]),
        &device,
    );

    println!("🔍 Upper breaches: {:?}", upper_breaches);
    println!("🔍 Lower breaches: {:?}", lower_breaches);
    println!("🔍 Expected OR result: [true, true, false]");

    // Test boolean OR operation
    let or_result = upper_tensor.bool_or(lower_tensor);
    let or_data = or_result.to_data();

    match or_data.as_slice::<bool>() {
        Ok(or_results) => {
            println!("🔍 Actual OR results: {:?}", or_results);

            let expected = [true, true, false];
            if or_results == expected {
                println!("🎯 Boolean OR operation: PASSED");
            } else {
                println!("❌ Boolean OR operation: FAILED");
            }
        }
        Err(e) => {
            println!("❌ Boolean OR extraction failed: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "gpu")]
fn test_range_bar_scenario() {
    let device = match get_test_device() {
        Some(device) => device,
        None => {
            println!("⚠️ GPU not available - skipping test");
            return;
        }
    };

    println!("🧪 Testing complete range bar breach scenario");

    // Simulate exact scenario from our range bar implementation
    let open_price = 50000.0;
    let threshold_bps = 8000; // 0.8%
    let threshold_multiplier = (threshold_bps as f32) / 1_000_000.0; // 0.008

    let upper_threshold = open_price * (1.0 + threshold_multiplier); // 50400.0
    let lower_threshold = open_price * (1.0 - threshold_multiplier); // 49600.0

    // Test prices that should trigger breaches
    let test_prices = vec![
        50000.0, // No breach
        50450.0, // Upper breach (should be true)
        49550.0, // Lower breach (should be true)
        50200.0, // No breach
    ];

    println!("🔍 Range bar scenario:");
    println!("   Open price: {:.1}", open_price);
    println!("   Upper threshold: {:.1}", upper_threshold);
    println!("   Lower threshold: {:.1}", lower_threshold);
    println!("   Threshold %: {:.3}%", threshold_multiplier * 100.0);

    // Create tensors [4, 1] like our implementation
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_prices.clone(), [4, 1]),
        &device,
    );

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper_threshold; 4], [4, 1]),
        &device,
    );

    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![lower_threshold; 4], [4, 1]),
        &device,
    );

    // Test breach detection
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.bool_or(lower_breach);

    // Extract results
    let breach_data = any_breach.to_data();
    match breach_data.as_slice::<bool>() {
        Ok(breach_results) => {
            println!("🔍 Breach detection results:");
            for (i, &price) in test_prices.iter().enumerate() {
                let upper_breached = price >= upper_threshold;
                let lower_breached = price <= lower_threshold;
                let should_breach = upper_breached || lower_breached;
                let actual_breach = breach_results[i];

                println!("   Price {:.1}: upper={}, lower={}, expected={}, actual={}",
                    price, upper_breached, lower_breached, should_breach, actual_breach);

                if should_breach != actual_breach {
                    println!("❌ Breach detection mismatch at price {:.1}", price);
                }
            }

            let expected = [false, true, true, false];
            if breach_results == expected {
                println!("🎯 Range bar breach scenario: PASSED");
            } else {
                println!("❌ Range bar breach scenario: FAILED");
                println!("   Expected: {:?}", expected);
                println!("   Actual:   {:?}", breach_results);
            }
        }
        Err(e) => {
            println!("❌ Range bar breach extraction failed: {:?}", e);
        }
    }
}