// This module is dedicated to computing statistical properties of trajectories.

// use dashmap::DashMap;
use numpy::PyReadonlyArray2;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashMap;

use ndarray::prelude::*;

/// # Statistical Calculator
///
/// ## Mathematical and Scientific Motivation
///
/// For a dynamical system, the "invariant measure" describes the long-term statistical
/// behavior of its trajectories. It tells you which regions of the state space are visited
/// most frequently. This function provides a practical way to approximate this measure by
/// creating a multi-dimensional histogram via "box counting".
///
/// ## Implementation: Parallel Box Counting
///
/// This calculation is "embarrassingly parallel". We use the `rayon` crate to process
/// points concurrently and `dashmap::DashMap` for a thread-safe histogram to handle
/// simultaneous writes from multiple threads.

#[pyclass]
pub struct Stats;

impl Stats {
    // Public constructor for use in main.rs test harness.
    pub fn new() -> Self {
        Stats
    }
}

#[pymethods]
impl Stats {
    #[new]
    fn __new__() -> Self {
        Stats::new()
    }

    /// Approximates the invariant measure of a system by efficient, single-call box counting.
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

        // This call is safe because it directly accesses memory managed by Python.
        let traj_view = trajectory.as_array();
        if traj_view.is_empty() {
            return Ok(PyDict::new_bound(py).into());
        }

        // --- 1. Create a standard HashMap for counting ---
        // Since we are not using Rayon here, a standard HashMap is more efficient.
        let mut histogram: HashMap<Vec<i64>, usize> = HashMap::new();

        // --- 2. Iterate Over Trajectory Sequentially within Rust ---
        // FIX: This is the core performance improvement. We iterate over the entire
        // array inside Rust, avoiding the high overhead of repeated calls from Python.
        for point_view in traj_view.axis_iter(Axis(0)) {
            // --- 3. Determine the Bin Coordinates for Each Point ---
            let bin_coords: Vec<i64> = point_view
                .iter()
                .map(|&coord| (coord / epsilon).floor() as i64)
                .collect();

            // --- 4. Increment the Count for the Corresponding Bin ---
            *histogram.entry(bin_coords).or_insert(0) += 1;
        }

        // --- 5. Convert the Rust HashMap to a Python Dictionary ---
        let result_dict = PyDict::new_bound(py);
        for (key_vec, value) in histogram.into_iter() {
            // Convert the Rust Vec into a Python tuple for the key, which is hashable.
            let key_tuple = PyTuple::new_bound(py, &key_vec);
            result_dict.set_item(key_tuple, value)?;
        }

        Ok(result_dict.into())
    }
}
