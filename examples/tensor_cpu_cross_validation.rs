//! Tensor vs CPU Cross-Validation Tests
//!
//! Reconciles the discrepancy where CPU logic correctly identifies breaches
//! but GPU tensor extraction returns false. Tests each operation step-by-step.


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
        println!("❌ GPU feature not enabled. Run with: cargo run --example tensor_cpu_cross_validation --features gpu");
        return Ok(());
    }

    #[cfg(feature = "gpu")]
    {
        println!("🔬 **TENSOR vs CPU CROSS-VALIDATION**");
        println!("═══════════════════════════════════════");
        println!("Goal: Reconcile CPU logic vs tensor extraction discrepancy\\n");

        let device = WgpuDevice::default();
        println!("✅ GPU device initialized\\n");

        // Use EXACT values from the debug output where CPU shows breach but GPU doesn't
        test_exact_debug_scenario(&device)?;
        test_tensor_value_extraction(&device)?;
        test_step_by_step_comparison(&device)?;
        test_boolean_tensor_contents(&device)?;
        test_cross_reference_validation(&device)?;

        println!("\\n🎯 **CROSS-VALIDATION COMPLETE**");
        println!("═══════════════════════════════════");
    }

    Ok(())
}

#[cfg(feature = "gpu")]
fn test_exact_debug_scenario(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **TEST 1: Exact Debug Scenario Reproduction**");
    println!("─────────────────────────────────────────────");

    // Values from actual debug output where CPU shows breach but GPU doesn't
    let price = 49938.49f32;
    let upper_threshold = 51104.57f32;
    let lower_threshold = 50293.38f32;

    println!("Debug scenario: price={:.2}, upper={:.2}, lower={:.2}", price, upper_threshold, lower_threshold);

    // CPU calculation (what works)
    let cpu_upper_breach = price >= upper_threshold;
    let cpu_lower_breach = price <= lower_threshold;
    let cpu_any_breach = cpu_upper_breach || cpu_lower_breach;

    println!("CPU results: upper={}, lower={}, any={}", cpu_upper_breach, cpu_lower_breach, cpu_any_breach);

    // GPU tensor calculation (what fails)
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![price], [1, 1]),
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

    // Individual comparisons
    let upper_breach_tensor = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach_tensor = price_tensor.clone().lower_equal(lower_tensor);

    // Extract using .any() (current approach)
    let gpu_upper_breach = upper_breach_tensor.clone().slice([0..1, 0..1]).any().into_scalar() > 0;
    let gpu_lower_breach = lower_breach_tensor.clone().slice([0..1, 0..1]).any().into_scalar() > 0;
    let gpu_any_breach = gpu_upper_breach || gpu_lower_breach;

    println!("GPU results: upper={}, lower={}, any={}", gpu_upper_breach, gpu_lower_breach, gpu_any_breach);

    // Compare results
    if cpu_any_breach != gpu_any_breach {
        println!("❌ DISCREPANCY CONFIRMED: CPU={}, GPU={}", cpu_any_breach, gpu_any_breach);
        println!("   Upper: CPU={}, GPU={}", cpu_upper_breach, gpu_upper_breach);
        println!("   Lower: CPU={}, GPU={}", cpu_lower_breach, gpu_lower_breach);

        // Additional debugging - extract raw tensor values
        println!("\\n🔍 Extracting raw tensor contents...");

        // Try to extract the boolean tensor contents directly
        let upper_bool_data = upper_breach_tensor.to_data();
        let lower_bool_data = lower_breach_tensor.to_data();

        println!("   Upper boolean tensor: {:?}", upper_bool_data);
        println!("   Lower boolean tensor: {:?}", lower_bool_data);

    } else {
        println!("✅ CPU and GPU agree: both={}", cpu_any_breach);
    }

    println!();
    Ok(())
}

#[cfg(feature = "gpu")]
fn test_tensor_value_extraction(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **TEST 2: Tensor Value Extraction Verification**");
    println!("─────────────────────────────────────────────────");

    // Create tensors with known values
    let test_prices = vec![49938.49, 52000.0, 48000.0];
    let test_upper_thresholds = vec![51104.57, 51104.57, 51104.57];
    let test_lower_thresholds = vec![50293.38, 50293.38, 50293.38];

    println!("Input values:");
    println!("  Prices: {:?}", test_prices);
    println!("  Upper thresholds: {:?}", test_upper_thresholds);
    println!("  Lower thresholds: {:?}", test_lower_thresholds);

    // Create tensors
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_prices.clone(), [3, 1]),
        device
    );
    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_upper_thresholds.clone(), [3, 1]),
        device
    );
    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(test_lower_thresholds.clone(), [3, 1]),
        device
    );

    // Extract values back from tensors to verify they match
    let extracted_prices = price_tensor.to_data();
    let extracted_upper = upper_tensor.to_data();
    let extracted_lower = lower_tensor.to_data();

    println!("\\nExtracted from tensors:");
    println!("  Prices: {:?}", extracted_prices);
    println!("  Upper: {:?}", extracted_upper);
    println!("  Lower: {:?}", extracted_lower);

    // Verify extraction matches input
    let prices_match = match extracted_prices.as_slice::<f32>() {
        Ok(slice) => slice == test_prices.as_slice(),
        Err(_) => false,
    };

    println!("\\nValue preservation: {}", if prices_match { "✅ PASS" } else { "❌ FAIL" });

    println!();
    Ok(())
}

#[cfg(feature = "gpu")]
fn test_step_by_step_comparison(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **TEST 3: Step-by-Step Comparison Operations**");
    println!("────────────────────────────────────────────────");

    // Use a clear breach case
    let price = 52000.0f32;
    let threshold = 51000.0f32;

    println!("Clear breach case: {} >= {} (should be TRUE)", price, threshold);

    // CPU verification
    let cpu_result = price >= threshold;
    println!("CPU result: {}", cpu_result);

    // Step 1: Create tensors
    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![price], [1, 1]),
        device
    );
    let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(vec![threshold], [1, 1]),
        device
    );

    println!("\\nStep 1: Tensors created");

    // Step 2: Comparison operation
    let comparison_tensor = price_tensor.greater_equal(threshold_tensor);
    println!("Step 2: Comparison tensor created");

    // Step 3: Extract tensor data to see what's inside
    let comparison_data = comparison_tensor.clone().to_data();
    println!("Step 3: Comparison tensor contains: {:?}", comparison_data);

    // Step 4: Apply slice operation
    let sliced_tensor = comparison_tensor.clone().slice([0..1, 0..1]);
    let sliced_data = sliced_tensor.clone().to_data();
    println!("Step 4: Sliced tensor contains: {:?}", sliced_data);

    // Step 5: Apply .any() operation
    let any_result = sliced_tensor.any().into_scalar();
    println!("Step 5: .any() result: {} (type: scalar)", any_result);

    // Step 6: Convert to boolean
    let boolean_result = any_result > 0;
    println!("Step 6: Boolean conversion: {}", boolean_result);

    // Final comparison
    if cpu_result == boolean_result {
        println!("\\n✅ STEP-BY-STEP: CPU and GPU agree: {}", cpu_result);
    } else {
        println!("\\n❌ STEP-BY-STEP: CPU={}, GPU={}", cpu_result, boolean_result);
    }

    println!();
    Ok(())
}

#[cfg(feature = "gpu")]
fn test_boolean_tensor_contents(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **TEST 4: Boolean Tensor Contents Analysis**");
    println!("─────────────────────────────────────────────");

    // Test multiple cases to see pattern
    let test_cases = [
        (50000.0, 51000.0, "clear pass"),
        (52000.0, 51000.0, "clear breach"),
        (51000.0, 51000.0, "exact equality"),
    ];

    for (price, threshold, description) in test_cases {
        println!("\\nCase: {} - {} >= {}", description, price, threshold);

        let price_tensor = Tensor::<TestBackend, 2>::from_data(
            TensorData::new(vec![price], [1, 1]),
            device
        );
        let threshold_tensor = Tensor::<TestBackend, 2>::from_data(
            TensorData::new(vec![threshold], [1, 1]),
            device
        );

        let comparison = price_tensor.greater_equal(threshold_tensor);
        let comparison_data = comparison.clone().to_data();

        let cpu_expected = price >= threshold;
        let gpu_any = comparison.clone().slice([0..1, 0..1]).any().into_scalar() > 0;

        println!("  CPU expected: {}", cpu_expected);
        println!("  Tensor data: {:?}", comparison_data);
        println!("  GPU .any(): {}", gpu_any);
        println!("  Match: {}", cpu_expected == gpu_any);
    }

    println!();
    Ok(())
}

#[cfg(feature = "gpu")]
fn test_cross_reference_validation(device: &WgpuDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 **TEST 5: Cross-Reference Validation**");
    println!("───────────────────────────────────────────");

    // Use the exact multi-symbol scenario from the actual processing
    let symbol_count = 3;
    let prices = vec![49938.49, 52000.0, 48000.0];  // Mix of breach/no-breach
    let open_prices = vec![50750.0, 50750.0, 50750.0];  // Same open for all
    let threshold_bps = 8000u32;

    println!("Multi-symbol scenario:");
    println!("  Symbol count: {}", symbol_count);
    println!("  Prices: {:?}", prices);
    println!("  Open prices: {:?}", open_prices);
    println!("  Threshold: {}bps", threshold_bps);

    // CPU calculation for reference
    let threshold_multiplier = (threshold_bps as f32) / 1_000_000.0;
    println!("\\nCPU calculations:");

    for i in 0..symbol_count {
        let open = open_prices[i];
        let price = prices[i];
        let upper_threshold = open * (1.0 + threshold_multiplier);
        let lower_threshold = open * (1.0 - threshold_multiplier);

        let cpu_upper = price >= upper_threshold;
        let cpu_lower = price <= lower_threshold;
        let cpu_any = cpu_upper || cpu_lower;

        println!("  Symbol {}: price={:.2}, upper={:.2}, lower={:.2}", i, price, upper_threshold, lower_threshold);
        println!("    CPU: upper={}, lower={}, any={}", cpu_upper, cpu_lower, cpu_any);
    }

    // GPU tensor calculation
    println!("\\nGPU tensor calculations:");

    let price_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(prices, [symbol_count, 1]),
        device
    );

    // Calculate thresholds the same way
    let mut upper_thresholds = Vec::new();
    let mut lower_thresholds = Vec::new();

    for open in &open_prices {
        upper_thresholds.push(open * (1.0 + threshold_multiplier));
        lower_thresholds.push(open * (1.0 - threshold_multiplier));
    }

    let upper_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(upper_thresholds, [symbol_count, 1]),
        device
    );
    let lower_tensor = Tensor::<TestBackend, 2>::from_data(
        TensorData::new(lower_thresholds, [symbol_count, 1]),
        device
    );

    // Perform comparisons
    let upper_breach = price_tensor.clone().greater_equal(upper_tensor);
    let lower_breach = price_tensor.lower_equal(lower_tensor);

    // Extract per-symbol results
    for i in 0..symbol_count {
        let gpu_upper = upper_breach.clone().slice([i..i+1, 0..1]).any().into_scalar() > 0;
        let gpu_lower = lower_breach.clone().slice([i..i+1, 0..1]).any().into_scalar() > 0;
        let gpu_any = gpu_upper || gpu_lower;

        println!("  Symbol {}: GPU: upper={}, lower={}, any={}", i, gpu_upper, gpu_lower, gpu_any);
    }

    println!();
    Ok(())
}

#[cfg(not(feature = "gpu"))]
fn test_exact_debug_scenario(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_tensor_value_extraction(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_step_by_step_comparison(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_boolean_tensor_contents(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "gpu"))]
fn test_cross_reference_validation(_: &()) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }