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
/// This function implements the classic algorithm by Benettin et al. for calculating the
/// full Lyapunov spectrum. It works by evolving a set of D orthogonal perturbation vectors
/// along a main trajectory and repeatedly measuring their stretching and rotation.
///
/// ## The Algorithm
///
/// 1.  Transient Phase
///
/// 2.  Initialization
///
/// 3.  Main Loop (Evolve & Re-orthogonalize)
///
/// 4.  Average

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
    let transient_traj_obj = transient_result.as_ref(py).get_item(0)?;
    let transient_traj: &PyArray<f64, _> = transient_traj_obj.extract()?;
    
    // Convert PyArray into an ArrayView for ndarray to perform fast operations on
    let last_row = transient_traj.as_array().outer_iter().last().unwrap();
    let mut main_y = DVector::from_row_slice(last_row.as_slice().unwrap());

    // --- 2. Initialization for Main Loop ---
    let mut perturbation_w = DMatrix::<f64>::identity(state_dim, state_dim);
    let mut lyapunov_sums = DVector::<f64>::zeros(state_dim);
    let mut current_t = 0.0;
    
    let num_steps = (t_total / t_reorth).ceil() as usize;
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
        // Use Rayon's `par_iter` to integrate all trajectories concurrently.
        let final_states: Vec<DVector<f64>> = initial_states
            .par_iter()
            .map(|y0| {
                
                Python::with_gil(|py| {
                    let y0_py = y0.as_slice().to_pyarray(py);
                    let result_tuple = integrators::solve_rk45_adaptive(
                        py, dynamics.clone(), y0_py.readonly(), 0.0, t_reorth, h_init, abstol, reltol
                    ).unwrap();
                    
                    let traj_obj = result_tuple.as_ref(py).get_item(0).unwrap();
                    let traj: &PyArray<f64, _> = traj_obj.extract().unwrap();
                    let last_state = traj.as_array().outer_iter().last().unwrap();
                    DVector::from_row_slice(last_state.as_slice().unwrap())
                })
            })
            .collect();
        
        // Update the main trajectory's position
        main_y = final_states[0].clone();

        // --- 3b. Calculate Evolved Perturbation Matrix ---
        let mut evolved_w = DMatrix::<f64>::zeros(state_dim, state_dim);
        for j in 0..state_dim {
            let evolved_perturbation = (&final_states[j + 1] - &main_y) / eps;
            evolved_w.set_column(j, &evolved_perturbation);
        }

        // --- 3c. QR Decomposition ---
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
