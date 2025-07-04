// This is the root of the Rust library crate. It defines the Python module.

use pyo3::prelude::*;
mod integrators;

// This function defines the Python module.
// The name of the function (`dx_core`) determines the name of the module in Python.
#[pymodule]
fn dx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // We add our Rust functions to the Python module, with clear explicit/implicit names.
    m.add_function(wrap_pyfunction!(integrators::solve_rk45_explicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_rk45_implicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_rk4_explicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_rk4_implicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_euler_explicit, m)?)?;
    m.add_function(wrap_pyfunction!(integrators::solve_euler_implicit, m)?)?;
    Ok(())
}