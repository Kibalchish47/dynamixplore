// In src/stats.rs
// This module is dedicated to computing statistical properties of trajectories,
// such as histograms that approximate the invariant measure of the system.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use numpy::PyReadonlyArray2;
use rayon::prelude::*; // Import Rayon's parallel processing traits.
use dashmap::DashMap; // A thread-safe, high-performance concurrent HashMap.

/// # Invariant Measure (Approximated by Histogram)
///
/// ## Mathematical and Scientific Motivation
///
/// For a dynamical system, the "invariant measure" describes the long-term statistical
/// behavior of its trajectories. It tells you which regions of the state space are visited
/// most frequently by a typical trajectory. If you let the system run for an infinitely
/// long time, the invariant measure gives you the probability of finding the system in any
/// given region.
///
/// For chaotic systems, trajectories quickly converge to a complex, often fractal, object
/// called a "strange attractor". The invariant measure is concentrated on this attractor.
/// Visualizing this measure is one of the most powerful ways to understand the geometric
/// structure of the system's dynamics.
///
/// This function provides a practical way to approximate the invariant measure by creating
/// a multi-dimensional histogram, also known as "box counting". We partition the state space
/// into a grid of hypercubes (boxes) of a given size (`epsilon`) and count how many points
/// from the trajectory fall into each box. The resulting counts are directly proportional
/// to the invariant measure.
///
/// ## Implementation: Parallel Box Counting
///
/// Calculating this histogram can be computationally intensive for large trajectories.
/// Fortunately, the problem is "embarrassingly parallel": the calculation for each point
/// is completely independent of all other points.
///
/// We leverage this property using the `rayon` crate for data parallelism.
/// 1. The input trajectory (a large array of points) is split among multiple threads.
/// 2. Each thread processes its subset of points concurrently.
/// 3. To handle simultaneous writes to the histogram from multiple threads, we cannot use a
///    standard `HashMap`. Instead, we use `dashmap::DashMap`, a high-performance,
///    concurrent hash map designed for exactly this purpose. It handles all the locking
///    and synchronization internally, providing a safe and efficient way to build the
///    histogram in parallel.

#[pyfunction]
#[pyo3(signature = (trajectory, epsilon))]
pub fn compute_invariant_measure(
    py: Python,
    trajectory: PyReadonlyArray2<f64>,
    epsilon: f64, // The size of each box in the grid.
) -> PyResult<Py<PyDict>> {
    // --- Input Validation ---
    if epsilon <= 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err("Box size 'epsilon' must be positive."));
    }

    let traj_view = trajectory.as_array();
    let n_points = traj_view.nrows();
    if n_points == 0 {
        return Ok(PyDict::new(py).into());
    }
    let state_dim = traj_view.ncols();

    // --- 1. Create a Concurrent HashMap for Thread-Safe Counting ---
    // The key is a Vec<i64> representing the integer coordinates of a grid box.
    // The value is the count of points that have fallen into that box.
    let histogram: DashMap<Vec<i64>, usize> = DashMap::new();

    // --- 2. Iterate Over Trajectory in Parallel ---
    // `par_iter()` from Rayon automatically splits the work across available CPU cores.
    traj_view.outer_iter().par_iter().for_each(|point_view| {
        // --- 3. Determine the Bin Coordinates for Each Point ---
        // For each dimension, calculate which bin the coordinate falls into.
        // `(coordinate / epsilon).floor() as i64` maps a continuous coordinate
        // to a discrete integer grid index.
        let bin_coords: Vec<i64> = point_view
            .iter()
            .map(|&coord| (coord / epsilon).floor() as i64)
            .collect();

        // --- 4. Increment the Count for the Corresponding Bin ---
        // `DashMap` handles the concurrent access. The `entry().or_insert(0).add(1)`
        // pattern is not directly available, so we use a mutable entry.
        // `histogram.entry(bin_coords).or_insert(0)` gets or creates the entry.
        // `.add(1)` would be ideal, but we use `.and_modify()` and `.or_insert()`
        // for a more explicit update.
        // A simpler way is to just increment the value directly.
        let mut entry = histogram.entry(bin_coords).or_insert(0);
        *entry += 1;
    });

    // --- 5. Convert the Rust DashMap to a Python Dictionary ---
    // The parallel computation is done. Now, on the main thread, we convert the result.
    let result_dict = PyDict::new(py);
    for item in histogram.into_iter() {
        let key = item.key(); // The Vec<i64> of bin coordinates
        let value = item.value(); // The count
        
        // The key must be a hashable Python type, so we convert the Vec<i64> to a Python tuple.
        result_dict.set_item(key, value)?;
    }

    Ok(result_dict.into())
}
