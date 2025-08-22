// This is the root of the Rust crate and the main entry point for the PyO3 module.
// It declares all the sub-modules and registers their public `#[pyclass]` structs
// with the Python interpreter.

// Declare the modules corresponding to the other files in `src/`.
mod entropy;
mod integrators;
mod lyapunov;
mod stats;

// Use statements to bring the public classes from each module into scope.
use entropy::Entropy;
use integrators::{AdaptiveParams, Euler, ExplicitParams, ImplicitParams, Rk4, Rk45};
use lyapunov::Lyapunov;
use stats::Stats;

use pyo3::prelude::*;

/// # DynamiXplore Rust Core (`_core`)
///
/// This Python module, written in Rust, provides the high-performance computational
/// backend for the DynamiXplore library.
#[pymodule]
// FIX: The name inside the pymodule macro MUST match the `[lib]` name in Cargo.toml.
// This creates the `PyInit__core` function that Python looks for.
fn _core(_py: Python, m: &PyModule) -> PyResult<()> {
    // --- Register Solver Classes ---
    m.add_class::<Rk45>()?;
    m.add_class::<Rk4>()?;
    m.add_class::<Euler>()?;

    // --- Register Parameter Data Classes ---
    m.add_class::<ExplicitParams>()?;
    m.add_class::<ImplicitParams>()?;
    m.add_class::<AdaptiveParams>()?;

    // --- Register Analysis Tool Classes ---
    m.add_class::<Lyapunov>()?;
    m.add_class::<Entropy>()?;
    m.add_class::<Stats>()?;

    Ok(())
}
