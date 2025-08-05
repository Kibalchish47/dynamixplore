// This is the root of the Rust library crate. It defines the Python module.

use pyo3::prelude::*;

// We now declare three modules: integrators, entropy, and stats.
// Rust will look for `integrators.rs`, `entropy.rs`, and `stats.rs` in the same directory.
mod integrators;
mod entropy;
mod stats;

// This function defines the Python module.
// The name of the function (`dx_core`) determines the name of the module in Python.
#[pymodule]
fn dx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // --- Add Integrator Functions ---
    // Note the change from `solve_rk45_explicit` to `solve_rk45_adaptive`.
    m.add_function(wrap_pyfunction!(integrators::solve_rk45_adaptive, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_rk4_explicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_rk4_implicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_euler_explicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_euler_implicit, m)?)?;

    // --- Add Entropy and Stats Functions ---
    m.add_function(wrap_pyfunction!(entropy::compute_permutation_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(stats::compute_invariant_measure, m)?)?;
    
    Ok(())
}
