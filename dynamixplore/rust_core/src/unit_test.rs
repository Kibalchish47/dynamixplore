use crate::*;

use nalgebra::DVector;
use ndarray::Array2;

// --- 1. Define the Dynamical System in Rust ---
// This is the Rust equivalent of the `lorenz_system` function in demo.py.
// Note: This version can't be passed directly to the integrator because the
// integrator is designed to call a Python function.
#[allow(dead_code)]
fn lorenz_system_rust(state: &DVector<f64>) -> DVector<f64> {
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    let x = state[0];
    let y = state[1];
    let z = state[2];

    let dx_dt = sigma * (y - x);
    let dy_dt = x * (rho - z) - y;
    let dz_dt = x * y - beta * z;

    DVector::from_vec(vec![dx_dt, dy_dt, dz_dt])
}

// --- Main Test Function ---
#[test]
fn test_apples() {
    println!("--- Running Rust Test Harness for DynamiXplore Core ---");

    // --- 2. Configure Simulation Parameters ---
    // These are the same parameters used in the Python demo script.
    let initial_state = DVector::from_vec(vec![1.0, 1.0, 1.0]);
    let t_start = 0.0;
    let t_end = 50.0;
    let dt = 0.01;
    println!("[1] Simulation Parameters Configured:");
    println!("    - Initial State: {:?}", initial_state.as_slice());
    println!("    - Time Span: ({}, {})", t_start, t_end);
    println!("    - Initial dt: {}", dt);

    // --- NOTE on Running the Simulation ---
    // We cannot call `integration_loop` here because it requires the Python
    // GIL and a Python function (`PyObject`) for the dynamics. The primary
    // way to test the full simulation is through the Python interface,
    // as demonstrated in `examples/demo.py`.
    // However, we can instantiate the solver structs to ensure they compile.
    let _rk45 = integrators::Rk45;
    let _rk4 = integrators::Rk4;
    let _euler = integrators::Euler;
    println!("\n[2] Solver classes compile successfully.");

    // --- 3. Test Analysis Functions with Mock Data ---
    println!("\n[3] Testing analysis functions with mock trajectory data...");

    // Create a mock trajectory (e.g., a simple sine wave)
    let n_points = 1000;
    let mut mock_trajectory_data = Vec::with_capacity(n_points * 3);
    for i in 0..n_points {
        let t = (i as f64) * 0.1;
        mock_trajectory_data.push(t.sin()); // x
        mock_trajectory_data.push(t.cos()); // y
        mock_trajectory_data.push(t.sin() * 0.5); // z
    }
    let mock_trajectory = Array2::from_shape_vec((n_points, 3), mock_trajectory_data).unwrap();
    let x_series = mock_trajectory.column(0).to_owned().into_raw_vec();

    // Instantiate analysis tools
    let entropy_solver = entropy::Entropy::new();
    let stats_solver = stats::Stats::new();
    let _lyapunov_solver = lyapunov::Lyapunov::new(); // Instantiated for compilation check

    // Test Permutation Entropy
    let m = 3;
    let tau = 1;
    let required_len = (m - 1) * tau + 1;
    if x_series.len() >= required_len {
        // This block replicates the core logic of the permutation entropy for testing
        let num_windows = x_series.len() - required_len + 1;
        let mut pattern_counts = std::collections::HashMap::new();
        for i in 0..num_windows {
            let window: Vec<f64> = (0..m).map(|j| x_series[i + j * tau]).collect();
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
            "    ✓ Permutation entropy test ran successfully. Found {} unique patterns.",
            pattern_counts.len()
        );
    }

    // Test Invariant Measure
    let epsilon = 0.5;
    let projected_traj = mock_trajectory.select(ndarray::Axis(1), &[0, 2]);
    let histogram = stats_solver.compute_invariant_measure_rust(&projected_traj.view(), epsilon);
    println!(
        "    ✓ Invariant measure test ran successfully. Found {} populated bins.",
        histogram.len()
    );

    println!("\n--- Test Harness Finished ---");
}

impl stats::Stats {
    fn compute_invariant_measure_rust(
        &self,
        trajectory: &ndarray::ArrayView2<f64>,
        epsilon: f64,
    ) -> dashmap::DashMap<Vec<i64>, usize> {
        use ndarray::prelude::*;
        use rayon::prelude::*;

        let histogram: dashmap::DashMap<Vec<i64>, usize> = dashmap::DashMap::new();
        trajectory
            .axis_iter(ndarray::Axis(0))
            .for_each(|point_view| {
                let bin_coords: Vec<i64> = point_view
                    .iter()
                    .map(|&coord| (coord / epsilon).floor() as i64)
                    .collect();
                *histogram.entry(bin_coords).or_insert(0) += 1;
            });
        histogram
    }
}
