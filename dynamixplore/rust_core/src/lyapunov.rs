// In src/lyapunov.rs
// This module is dedicated to calculating the Lyapunov spectrum of a dynamical system,
// which is a primary indicator of chaos.

use crate::integrators; // Use the integrators from the parent crate.
use nalgebra::{DMatrix, DVector};
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use rayon::prelude::*; // For parallel computation

/// # Lyapunov Spectrum Calculation (Benettin's Method)
///
/// ## Mathematical and Scientific Motivation
///
/// The Lyapunov spectrum is a set of exponents (λ₁, λ₂, ..., λ_D) that measure the
/// average exponential rates of divergence or convergence of nearby trajectories in the
/// D-dimensional state space of a dynamical system. It is the most definitive quantitative
/// measure of deterministic chaos.
///
/// - λ > 0 (Positive Exponent): A hallmark of chaos. Indicates that trajectories
///   diverge exponentially, leading to sensitive dependence on initial conditions (the "butterfly effect").
/// - λ = 0 (Zero Exponent): Corresponds to motion along the trajectory itself. For a
///   continuous chaotic system, there will always be at least one zero exponent.
/// - λ < 0 (Negative Exponent): Indicates convergence of trajectories onto the attractor.
///   These exponents are associated with the rate of dissipation in the system.
///
/// This function implements the classic algorithm by Benettin et al. for calculating the
/// full Lyapunov spectrum. It works by evolving a set of D orthogonal perturbation vectors
/// along a main trajectory and repeatedly measuring their stretching and rotation.
///
/// ## The Algorithm
///
/// 1.  Transient Phase: The main trajectory is first integrated for a `t_transient`
///     period to ensure it has settled onto the system's attractor.
///
/// 2.  Initialization: An orthonormal matrix `W` is initialized (typically the identity
///     matrix). Each column of `W` represents a small perturbation vector from the main trajectory.
///
/// 3.  Main Loop (Evolve & Re-orthogonalize):
///     a. Evolve: The main trajectory and D perturbed trajectories (`y + ε * w_i`) are
///        simultaneously integrated forward for a short time interval `t_reorth`. This is
///        done in parallel using Rayon for performance.
///     b. Measure Stretch: The evolved perturbation vectors are calculated from the
///        difference between the final states of the main and perturbed trajectories.
///     c. QR Decomposition: This is the core of the method. The matrix of evolved
///        perturbation vectors is decomposed into `Q` (an orthogonal matrix representing the
///        new orientations) and `R` (an upper-triangular matrix representing the stretching
///        factors in each of those new directions).
///     d. Accumulate: The natural logarithms of the diagonal elements of the `R` matrix
///        give the instantaneous rates of expansion. These are summed over time.
///     e. Reset: The perturbation matrix `W` is reset to `Q`, providing a new, stable
///        orthonormal basis for the next step.
///
/// 4.  Average: After the total simulation time, the accumulated sums are divided by the
///     total time to get the final average Lyapunov exponents.

#[pyfunction]
#[pyo3(signature = (
    dynamics, initial_state,
    t_transient, t_total, t_reorth,
    h_init, abstol, reltol,
    eps = 1e-8,
))]
pub fn compute_lyapunov_spectrum(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_transient: f64,
    t_total: f64,
    t_reorth: f64,
    h_init: f64,
    abstol: f64,
    reltol: f64,
    eps: f64, // Small perturbation size
) -> PyResult<PyObject> {
    let initial_y = DVector::from_column_slice(initial_state.as_slice()?);
    let state_dim = initial_y.len();

    // --- 1. Run Transient Phase ---
    // We integrate the system to let it settle onto the attractor.
    // We only need the final state, so we can discard the trajectory itself.
    let transient_result = integrators::solve_rk45_adaptive(
        py, dynamics.clone(), initial_state, 0.0, t_transient, h_init, abstol, reltol
    )?;
    // .as_ref(py): gets a safe reference to that Python object
    // .get_item(0): attempts to get the first item from the tuple (the trajectory array)
    let transient_traj_obj = transient_result.as_ref(py).get_item(0)?;

    let transient_traj: &PyArray<f64, _> = transient_traj_obj.extract()?;
    
    // .as_array() = convert PyArray into an ArrayView for ndarray to perform fast operations on
    let last_row = transient_traj.as_array().outer_iter().last().unwrap();
    let mut main_y = DVector::from_row_slice(last_row.as_slice().unwrap());

    // --- 2. Initialization for Main Loop ---
    // DMatrix: A Dynamically sized Matrix
    let mut perturbation_w = DMatrix::<f64>::identity(state_dim, state_dim);
    // DVector: A Dynamically sized column Vector
    let mut lyapunov_sums = DVector::<f64>::zeros(state_dim);
    let mut current_t = 0.0;
    
    let num_steps = (t_total / t_reorth).ceil() as usize;
    // ::with_capacity = performance optimization for creating a Rust `Vec`
    let mut spectrum_history: Vec<DVector<f64>> = Vec::with_capacity(num_steps);

    // --- 3. Main Loop ---
    for i in 0..num_steps {
        // Create a list of D+1 initial states for parallel integration
        let mut initial_states: Vec<DVector<f64>> = Vec::with_capacity(state_dim + 1);
        initial_states.push(main_y.clone());
        for j in 0..state_dim {
            initial_states.push(&main_y + eps * perturbation_w.column(j));
        }

        // --- 3a. Evolve in Parallel ---
        // Use Rayon's parallel iterator to process the list of initial states concurrently
        let final_states: Vec<DVector<f64>> = initial_states
            .par_iter()
            // .map() applies the following closure to each item in parallel
            .map(|y0| {
                // For each starting state y0, we must acquire the GIL to call Python
                Python::with_gil(|py| {
                    // Convert the Rust DVector `y0` into a NumPy array `y0_py`
                    let y0_py = y0.as_slice().to_pyarray(py);
                    // Call our adaptive integrator function from Rust
                    let result_tuple = integrators::solve_rk45_adaptive(
                        py, dynamics.clone(), y0_py.readonly(), 0.0, t_reorth, h_init, abstol, reltol
                    ).unwrap(); // .unwrap() assumes the integration was successful
                    
                    // Get the trajectory (item 0) from the (trajectory, times) tuple
                    let traj_obj = result_tuple.as_ref(py).get_item(0).unwrap();
                    // Extract it into a Rust-readable NumPy array reference
                    let traj: &PyArray<f64, _> = traj_obj.extract().unwrap();
                    // Get the last row of the trajectory array
                    let last_state = traj.as_array().outer_iter().last().unwrap();
                    // Convert that last row back into a DVector, which is the return value of the closure
                    DVector::from_row_slice(last_state.as_slice().unwrap())
                })
            })
            // .collect() gathers the results from all parallel tasks into a single Vec<DVector<f64>>
            .collect();
        
        // Update the main trajectory's position
        // note: .clone() is done because we need to have the same data in two places at once (ownership)
        main_y = final_states[0].clone(); 

        // --- 3b. Calculate Evolved Perturbation Matrix ---
        let mut evolved_w = DMatrix::<f64>::zeros(state_dim, state_dim);
        for j in 0..state_dim {
            let evolved_perturbation = (&final_states[j + 1] - &main_y) / eps;
            evolved_w.set_column(j, &evolved_perturbation);
        }

        // --- 3c. QR Decomposition ---
        // It is a standard linear algebra technique that splits a matrix into two special matrices:
        // - An orthogonal matrix Q = new orientations of your stretched vectors
        // - An upper-triangular matrix R = stretching factors (the growth or shrinkage)
        //   that happened along each of the new Q directions
        let qr = evolved_w.qr();
        let q = qr.q();
        let r = qr.r();

        // --- 3d. Accumulate Logarithms ---
        for j in 0..state_dim {
            lyapunov_sums[j] += r[(j, j)].abs().ln();
        }
        
        // --- 3e. Reset Orthonormal Basis ---
        perturbation_w = q;
        
        current_t += t_reorth;
        if current_t > 0.0 {
            spectrum_history.push(&lyapunov_sums / current_t);
        }
    }

    // --- 4. Average to get the final spectrum ---
    let final_spectrum = lyapunov_sums / t_total;

    // --- 5. Convert to Python objects and return ---
    let final_spectrum_py = final_spectrum.as_slice().to_pyarray(py);
    
    let history_flat: Vec<f64> = spectrum_history.into_iter().flat_map(|v| v.into_iter().cloned()).collect();
    let history_array = PyArray::from_vec(py, history_flat).reshape((num_steps, state_dim))?;

    Ok(PyTuple::new(py, &[final_spectrum_py, history_array]).to_object(py))
}
