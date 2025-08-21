// In src/lyapunov.rs
// This module is refactored to use a class-based API (`Lyapunov`) for consistency
// with the new architectural pattern.

use crate::integrators::{Adaptive, Rk45}; // Use the internal integrator structs
use nalgebra::{DMatrix, DVector};
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use rayon::prelude::*;

/// # Lyapunov Spectrum Calculator
///
/// ## Purpose
/// This class computes the Lyapunov spectrum for a given dynamical system. It provides a
/// method-based interface that is consistent with the rest of the library's class-based design.
///
/// The core scientific motivation and algorithm (Benettin's method with QR re-orthogonalization)
/// remain the same as the previous functional implementation. The primary change is the
/// encapsulation of this logic within a Python-facing class.

#[pyclass]
pub struct Lyapunov;

#[pymethods]
impl Lyapunov {
    #[new]
    fn new() -> Self {
        Lyapunov
    }

    /// Computes the full Lyapunov spectrum using Benettin's method.
    #[pyo3(signature = (
        dynamics, initial_state,
        t_transient, t_total, t_reorth,
        h_init, abstol, reltol,
        eps = 1e-8,
    ))]
    fn compute_spectrum(
        &self,
        py: Python,
        dynamics: PyObject,
        initial_state: PyReadonlyArray1<f64>,
        t_transient: f64,
        t_total: f64,
        t_reorth: f64,
        h_init: f64,
        abstol: f64,
        reltol: f64,
        eps: f64,
    ) -> PyResult<PyObject> {
        let initial_y = DVector::from_column_slice(initial_state.as_slice()?);
        let state_dim = initial_y.len();

        // --- 1. Run Transient Phase ---
        // The core logic now uses the internal `Adaptive` approach struct directly,
        // rather than calling a public-facing PyFunction.
        let transient_result = Adaptive {
            dynamics: dynamics.clone(),
            initial_state: initial_state.clone(),
            t_start: 0.0,
            t_end: t_transient,
            initial_h: h_init,
            abstol,
            reltol,
        }
        .integration_loop(py, Rk45)?;

        let transient_traj_obj = transient_result.as_ref(py).get_item(0)?;
        let transient_traj: &PyArray<f64, _> = transient_traj_obj.extract()?;
        let last_row = transient_traj.as_array().outer_iter().last().unwrap();
        let mut main_y = DVector::from_row_slice(last_row.as_slice().unwrap());

        // --- 2. Initialization for Main Loop ---
        let mut perturbation_w = DMatrix::<f64>::identity(state_dim, state_dim);
        let mut lyapunov_sums = DVector::<f64>::zeros(state_dim);
        let mut current_t = 0.0;
        let num_steps = (t_total / t_reorth).ceil() as usize;
        let mut spectrum_history: Vec<DVector<f64>> = Vec::with_capacity(num_steps);

        // --- 3. Main Loop ---
        for _ in 0..num_steps {
            let mut initial_states: Vec<DVector<f64>> = Vec::with_capacity(state_dim + 1);
            initial_states.push(main_y.clone());
            for j in 0..state_dim {
                initial_states.push(&main_y + eps * perturbation_w.column(j));
            }

            // --- 3a. Evolve in Parallel ---
            let final_states: Vec<DVector<f64>> = initial_states
                .par_iter()
                .map(|y0| {
                    Python::with_gil(|py| {
                        // This internal call is also updated to the new architecture.
                        let y0_py = y0.as_slice().to_pyarray(py);
                        let result_tuple = Adaptive {
                            dynamics: dynamics.clone(),
                            initial_state: y0_py.readonly(),
                            t_start: 0.0,
                            t_end: t_reorth,
                            initial_h: h_init,
                            abstol,
                            reltol,
                        }
                        .integration_loop(py, Rk45)
                        .unwrap();

                        let traj_obj = result_tuple.as_ref(py).get_item(0).unwrap();
                        let traj: &PyArray<f64, _> = traj_obj.extract().unwrap();
                        let last_state = traj.as_array().outer_iter().last().unwrap();
                        DVector::from_row_slice(last_state.as_slice().unwrap())
                    })
                })
                .collect();

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
        let history_flat: Vec<f64> = spectrum_history
            .into_iter()
            .flat_map(|v| v.into_iter().cloned())
            .collect();
        let history_array =
            PyArray::from_vec(py, history_flat).reshape((num_steps, state_dim))?;

        Ok(PyTuple::new(py, &[final_spectrum_py, history_array]).to_object(py))
    }
}
