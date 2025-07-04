// This is the root of the Rust library crate. It defines the Python module.

use pyo3::prelude::*;
mod integrators;

// This function defines the Python module.
// The name of the function (`dx_core`) determines the name of the module in Python.
#[pymodule]
fn dx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // We add our Rust function to the Python module.
    // `wrap_pyfunction!` creates the necessary boilerplate to expose it.
    m.add_function(wrap_pyfunction!(integrators::solve_rk45, m)?)?;
    Ok(())
}
