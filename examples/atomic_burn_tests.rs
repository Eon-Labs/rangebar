//! Atomic Burn Framework Tests - Standalone Executable
//!
//! Ultra-minimal tests to isolate the exact failure point in GPU vs CPU
//! range bar generation (6 GPU bars vs 1500 CPU bars).

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
        println!("❌ GPU feature not enabled. Run with: cargo run --example atomic_burn_tests --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        println!("🧪 **ATOMIC BURN FRAMEWORK DIAGNOSTIC TESTS**");
        println!("═══════════════════════════════════════════════");
        println!("Goal: Isolate why GPU generates 6 bars vs CPU's 1500 bars\n");

        let device = WgpuDevice::default();
        println!("✅ GPU device initialized: {:?}\n", device);

        // Run atomic tests in order of complexity
        test_level_1_single_value(&device)?;
        test_level_2_threshold_calculation()?;
        test_level_3_multi_element_chain(&device)?;
        test_level_4_range_bar_scenario(&device)?;
        test_level_5_edge_cases(&device)?;

        println!("\n🎯 **ATOMIC DIAGNOSTIC COMPLETE**");
        println!("═══════════════════════════════════");
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_level_1_single_value(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **LEVEL 1: Single Value Comparison (Most Atomic)**");
    println!("────────────────────────────────────────────────────");

    // Values that absolutely MUST trigger breach
    let price = 52000.0f32;           // Clearly above threshold
    let upper_threshold = 51156.0f32; // Range bar threshold

    println!("Testing: {:.1} >= {:.1} (MUST be true)", price, upper_threshold);

    // Create minimal [1,1] tensors
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![price], [1, 1]),
        device
    );
    let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper_threshold], [1, 1]),
        device
    );

    // Test comparison operation
    let result_tensor = price_tensor.greater_equal(threshold_tensor);

    // Test extraction using our .any() workaround
    let extracted_value = result_tensor.slice([0..1, 0..1]).any().into_scalar();

    println!("GPU Result: {} (expected: >0)", extracted_value);

    // CPU verification
    let cpu_result = price >= upper_threshold;
    println!("CPU Result: {} (reference)", cpu_result);

    if extracted_value > 0 {
        println!("✅ LEVEL 1: PASSED - Basic comparison works\n");
    } else {
        println!("❌ LEVEL 1: FAILED - Fundamental comparison broken!");
        println!("   {} >= {} should be true on GPU!", price, upper_threshold);
        return Err("ATOMIC FAILURE: Basic comparison failed".into());
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_level_2_threshold_calculation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **LEVEL 2: Threshold Calculation Precision**");
    println!("─────────────────────────────────────────────");

    // Exact values from our range bar implementation
    let open_price = 50750.0f32;
    let threshold_bps = 8000u32;

    // CPU calculation (reference)
    let cpu_multiplier = (threshold_bps as f32) / 1_000_000.0;
    let cpu_upper = open_price * (1.0 + cpu_multiplier);
    let cpu_lower = open_price * (1.0 - cpu_multiplier);

    println!("Open: {:.6}", open_price);
    println!("BPS: {}", threshold_bps);
    println!("CPU Multiplier: {:.9}", cpu_multiplier);
    println!("CPU Upper: {:.6}", cpu_upper);
    println!("CPU Lower: {:.6}", cpu_lower);

    // Expected exact values
    let expected_upper = 51156.0f32;
    let expected_lower = 50344.0f32;

    println!("Expected Upper: {:.6}", expected_upper);
    println!("Expected Lower: {:.6}", expected_lower);

    // Verify calculations match exactly
    let upper_diff = (cpu_upper - expected_upper).abs();
    let lower_diff = (cpu_lower - expected_lower).abs();

    if upper_diff < 0.001 && lower_diff < 0.001 {
        println!("✅ LEVEL 2: PASSED - Threshold calculation verified\n");
    } else {
        println!("❌ LEVEL 2: FAILED - Threshold calculation mismatch!");
        println!("   Upper diff: {:.6}", upper_diff);
        println!("   Lower diff: {:.6}", lower_diff);
        return Err("Threshold calculation precision error".into());
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_level_3_multi_element_chain(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **LEVEL 3: Multi-Element Comparison Chain**");
    println!("────────────────────────────────────────────");

    // Known values with predictable outcomes
    let prices = vec![50750.0, 52000.0, 49000.0]; // no breach, upper breach, lower breach
    let upper_thresholds = vec![51156.0; 3];
    let lower_thresholds = vec![50344.0; 3];

    // Expected results
    let expected_upper = [false, true, false];
    let expected_lower = [false, false, true];
    let expected_any = [false, true, true];

    println!("Prices: {:?}", prices);
    println!("Upper thresholds: {:?}", upper_thresholds);
    println!("Lower thresholds: {:?}", lower_thresholds);
    println!("Expected ANY breach: {:?}", expected_any);

    // Create GPU tensors
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices.clone(), [3, 1]),
        device
    );
    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(upper_thresholds, [3, 1]),
        device
    );
    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(lower_thresholds, [3, 1]),
        device
    );

    // Test the full comparison chain
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);
    let any_breach = upper_breach.clone().bool_or(lower_breach.clone());

    // Extract results using .any() per element
    let mut gpu_results = Vec::new();
    for i in 0..3 {
        let slice_result = any_breach.clone().slice([i..i+1, 0..1]).any().into_scalar() > 0;
        gpu_results.push(slice_result);
    }

    println!("GPU Results: {:?}", gpu_results);

    // Verify each element
    let mut all_correct = true;
    for i in 0..3 {
        let expected = expected_any[i];
        let actual = gpu_results[i];

        if expected != actual {
            println!("❌ MISMATCH at index {}: price={:.1}, expected={}, actual={}",
                i, prices[i], expected, actual);

            // Debug individual comparisons
            let upper_slice = upper_breach.clone().slice([i..i+1, 0..1]).any().into_scalar() > 0;
            let lower_slice = lower_breach.clone().slice([i..i+1, 0..1]).any().into_scalar() > 0;
            println!("   Upper breach: {} (expected: {})", upper_slice, expected_upper[i]);
            println!("   Lower breach: {} (expected: {})", lower_slice, expected_lower[i]);
            all_correct = false;
        } else {
            println!("✅ Element {} correct", i);
        }
    }

    if all_correct {
        println!("✅ LEVEL 3: PASSED - Multi-element chain works\n");
    } else {
        println!("❌ LEVEL 3: FAILED - Multi-element comparison chain broken!");
        return Err("Multi-element chain verification failed".into());
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_level_4_range_bar_scenario(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **LEVEL 4: Exact Range Bar Breach Scenario**");
    println!("─────────────────────────────────────────────");

    // Simulate the exact sequence that should generate a range bar
    let open_price = 50750.0f32;
    let breach_price = 52000.0f32;  // Definite upper breach

    let threshold_bps = 8000u32;
    let threshold_multiplier = (threshold_bps as f32) / 1_000_000.0;
    let upper_threshold = open_price * (1.0 + threshold_multiplier);
    let lower_threshold = open_price * (1.0 - threshold_multiplier);

    println!("Open: {:.1}", open_price);
    println!("Breach: {:.1}", breach_price);
    println!("Upper threshold: {:.1}", upper_threshold);
    println!("Lower threshold: {:.1}", lower_threshold);

    // This should definitely breach
    let should_breach_upper = breach_price >= upper_threshold;
    let should_breach_lower = breach_price <= lower_threshold;
    let should_breach_any = should_breach_upper || should_breach_lower;

    println!("CPU: upper={}, lower={}, any={}", should_breach_upper, should_breach_lower, should_breach_any);

    // Test on GPU
    let breach_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![breach_price], [1, 1]),
        device
    );
    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![upper_threshold], [1, 1]),
        device
    );
    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![lower_threshold], [1, 1]),
        device
    );

    let gpu_upper_breach = breach_tensor.clone().greater_equal(upper_tensor).slice([0..1, 0..1]).any().into_scalar() > 0;
    let gpu_lower_breach = breach_tensor.lower_equal(lower_tensor).slice([0..1, 0..1]).any().into_scalar() > 0;
    let gpu_any_breach = gpu_upper_breach || gpu_lower_breach;

    println!("GPU: upper={}, lower={}, any={}", gpu_upper_breach, gpu_lower_breach, gpu_any_breach);

    if gpu_upper_breach == should_breach_upper &&
       gpu_lower_breach == should_breach_lower &&
       gpu_any_breach == should_breach_any {
        println!("✅ LEVEL 4: PASSED - Range bar scenario verified\n");
    } else {
        println!("❌ LEVEL 4: FAILED - Range bar breach detection broken!");
        println!("   Upper mismatch: GPU={}, CPU={}", gpu_upper_breach, should_breach_upper);
        println!("   Lower mismatch: GPU={}, CPU={}", gpu_lower_breach, should_breach_lower);
        println!("   Any mismatch: GPU={}, CPU={}", gpu_any_breach, should_breach_any);
        return Err("Range bar scenario verification failed".into());
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_level_5_edge_cases(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **LEVEL 5: Edge Case Precision Test**");
    println!("──────────────────────────────────────");

    let threshold = 51156.0f32;

    let edge_cases = [
        (51155.9, false, "just below"),
        (51156.0, true,  "exact threshold"),
        (51156.1, true,  "just above"),
        (52000.0, true,  "clearly above"),
    ];

    let mut all_passed = true;

    for (price, expected, description) in edge_cases {
        println!("Testing {}: {:.1} >= {:.1} (expected: {})", description, price, threshold, expected);

        let price_tensor = Tensor::<TestBackend, 2>::from_data(
            TensorData::new(vec![price as f32], [1, 1]),
            device
        );
        let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
            TensorData::new(vec![threshold], [1, 1]),
            device
        );

        let result = price_tensor.greater_equal(threshold_tensor).slice([0..1, 0..1]).any().into_scalar() > 0;

        println!("   GPU result: {}", result);

        if result != expected {
            println!("   ❌ EDGE CASE FAILURE: {} >= {} should be {}, got {}", price, threshold, expected, result);
            all_passed = false;
        } else {
            println!("   ✅ Edge case correct");
        }
    }

    if all_passed {
        println!("✅ LEVEL 5: PASSED - Edge cases verified\n");
    } else {
        println!("❌ LEVEL 5: FAILED - Edge case precision issues!");
        return Err("Edge case verification failed".into());
    }

    Ok(())
}

#[cfg(not(feature = "gpu"))]
fn test_level_1_single_value(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_level_2_threshold_calculation() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_level_3_multi_element_chain(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_level_4_range_bar_scenario(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_level_5_edge_cases(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }