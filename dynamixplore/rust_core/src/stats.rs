// This module is refactored to use a class-based API (`Stats`) for consistency.
// The core parallel box-counting logic remains the same.

use dashmap::DashMap;
use numpy::PyReadonlyArray2;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;

/// # Statistical Calculator
///
/// ## Purpose
/// This class provides methods for computing statistical properties of trajectories,
/// such as approximating the invariant measure of a system via multi-dimensional
/// histograms (box counting).
#[pyclass]
pub struct Stats;

#[pymethods]
impl Stats {
    #[new]
    fn new() -> Self {
        Stats
    }

    /// Approximates the invariant measure of a system by parallel box counting.
    #[pyo3(signature = (trajectory, epsilon))]
    fn compute_invariant_measure(
        &self,
        py: Python,
        trajectory: PyReadonlyArray2<f64>,
        epsilon: f64,
    ) -> PyResult<Py<PyDict>> {
        if epsilon <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Box size 'epsilon' must be positive.",
            ));
        }

        let traj_view = trajectory.as_array();
        let n_points = traj_view.nrows();
        if n_points == 0 {
            return Ok(PyDict::new(py).into());
        }

        // Use a concurrent DashMap for thread-safe histogramming.
        let histogram: DashMap<Vec<i64>, usize> = DashMap::new();

        // Iterate over the trajectory in parallel using Rayon.
        traj_view.outer_iter().par_iter().for_each(|point_view| {
            let bin_coords: Vec<i64> = point_view
                .iter()
                .map(|&coord| (coord / epsilon).floor() as i64)
                .collect();

            // Increment the count for the corresponding bin safely.
            *histogram.entry(bin_coords).or_insert(0) += 1;
        });

        // Convert the resulting DashMap to a Python dictionary.
        let result_dict = PyDict::new(py);
        for item in histogram.into_iter() {
            let key = item.key();
            let value = item.value();
            result_dict.set_item(key, value)?;
        }

        Ok(result_dict.into())
    }
}
