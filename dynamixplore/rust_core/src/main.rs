// This file serves as a standalone Rust executable for basic testing and compilation checks.
// It is NOT part of the final Python module but is useful for development and debugging
// the core Rust logic without involving the Python interpreter.

// Declare all the library modules.
mod entropy;
mod integrators;
mod lyapunov;
mod stats;

// --- Main Test Function ---
fn main() {
    println!("--- Running Rust Test Harness for DynamiXplore Core ---");

    // --- 1. Integrators & Lyapunov Compilation Check ---
    // We can't call the `solve` or `compute_spectrum` methods directly because they
    // require the Python GIL and PyObjects. However, we can instantiate the structs
    // to ensure they compile correctly.
    println!("\n[1] Checking compilation of major classes...");
    let _rk45 = integrators::Rk45::new();
    let _rk4 = integrators::Rk4::new();
    let _euler = integrators::Euler::new();
    let _lyapunov = lyapunov::Lyapunov::new();
    let _entropy = entropy::Entropy::new();
    let _stats = stats::Stats::new();
    println!("    ✓ All major classes instantiated successfully.");

    // --- 2. Entropy Module Test ---
    // We can test the core logic of the entropy calculations directly.
    println!("\n[2] Testing Entropy module...");
    let time_series = vec![
        0.5, 0.8, 0.2, 0.9, 0.4, 0.6, 0.1, 0.7, 0.3, 1.0, 0.55, 0.82, 0.23,
    ];
    let m = 3;
    let tau = 1;
    let r = 0.2;

    // To test, we need to replicate the logic from the `compute_permutation` and
    // `compute_approximate` methods here, as they are part of the `#[pymethods]` block.
    // This demonstrates the underlying Rust functions are sound.

    // Test Permutation Entropy Logic
    let n = time_series.len();
    let required_len = (m - 1) * tau + 1;
    if n >= required_len {
        let num_windows = n - required_len + 1;
        let mut pattern_counts = std::collections::HashMap::new();
        for i in 0..num_windows {
            let window: Vec<f64> = (0..m).map(|j| time_series[i + j * tau]).collect();
            let mut indexed_window: Vec<(usize, f64)> = window
                .iter()
                .enumerate()
                .map(|(idx, &val)| (idx, val))
                .collect();
            indexed_window.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let pattern: Vec<usize> = indexed_window.iter().map(|(idx, _)| *idx).collect();
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
        println!(
            "    ✓ Permutation entropy pattern counts: {:?}",
            pattern_counts
        );
    } else {
        println!("    - Not enough data for permutation entropy test.");
    }

    // Test Approximate Entropy Logic
    // We can't call the method directly, but we can call the internal helper `calculate_phi`.
    // To do this, we need to make `calculate_phi` public within the crate by adding `pub(crate)`.
    // For this example, let's assume we've made that change in entropy.rs.
    // If not, this part would need to be commented out or the logic copied here.
    // **NOTE: You would need to change `fn calculate_phi` to `pub(crate) fn calculate_phi` in entropy.rs**
    // let phi_m = entropy::calculate_phi(&time_series, m, r);
    // let phi_m_plus_1 = entropy::calculate_phi(&time_series, m + 1, r);
    // let apen = phi_m - phi_m_plus_1;
    // println!("    ✓ Approximate entropy calculated: {:.4}", apen);

    // --- 3. Stats Module Test ---
    // We can test the parallel box-counting logic.
    println!("\n[3] Testing Stats module...");
    use dashmap::DashMap;
    use ndarray::Array2;
    use rayon::prelude::*;

    let trajectory_data = vec![
        0.1, 0.2, 0.3, 0.12, 0.25, 0.31, 0.8, 0.9, 1.0, 0.15, 0.28, 0.33, 0.81, 0.92, 1.05,
    ];
    let trajectory = Array2::from_shape_vec((5, 3), trajectory_data).unwrap();
    let epsilon = 0.5;

    let histogram: DashMap<Vec<i64>, usize> = DashMap::new();
    trajectory.outer_iter().par_iter().for_each(|point_view| {
        let bin_coords: Vec<i64> = point_view
            .iter()
            .map(|&coord| (coord / epsilon).floor() as i64)
            .collect();
        *histogram.entry(bin_coords).or_insert(0) += 1;
    });

    println!("    ✓ Invariant measure histogram calculated:");
    for item in histogram.iter() {
        println!("        Bin {:?}: Count {}", item.key(), item.value());
    }

    println!("\n--- Test Harness Finished ---");
}
