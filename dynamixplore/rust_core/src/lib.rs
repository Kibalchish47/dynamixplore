// This is the root of the Rust crate and the main entry point for the PyO3 module.
// It declares all the sub-modules and registers their public `#[pyclass]` structs
// with the Python interpreter.

// Declare the modules corresponding to the other files in `src/`.
pub mod entropy;
pub mod integrators;
pub mod lyapunov;
pub mod stats;

#[cfg(test)]
mod unit_test;

use pyo3::prelude::*;

#[pymodule]
mod dynamixplore {
    use pyo3::prelude::*;

    /// # DynamiXplore Rust Core (`_core`)
    ///
    /// This Python module, written in Rust, provides the high-performance computational
    /// backend for the DynamiXplore library.
    #[pymodule]
    mod _core {
        // Use statements to bring the public classes from each module into scope.
        #[rustfmt::skip]
        #[pymodule_export]
        use crate::integrators::{
            // --- Register Solver Classes ---
            Euler, Rk4, Rk45,
            // --- Register Parameter Data Classes ---
            AdaptiveParams, ExplicitParams, ImplicitParams, 
        };

        // --- Register Analysis Tool Classes ---
        #[pymodule_export]
        use crate::{entropy::Entropy, lyapunov::Lyapunov, stats::Stats};
    }
}
