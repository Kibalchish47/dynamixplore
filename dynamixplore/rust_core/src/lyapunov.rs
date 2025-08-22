use crate::integrators::{Adaptive, Approach, Rk45}; // Import the Approach trait
use nalgebra::{DMatrix, DVector};
use numpy::{ndarray::Dim, PyArray, PyArrayMethods, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// # Lyapunov Spectrum Calculator
///
/// ## Purpose
/// This class computes the Lyapunov spectrum for a given dynamical system. It provides a
/// method-based interface that is consistent with the rest of the library's class-based design.
#[pyclass]
pub struct Lyapunov;

impl Lyapunov {
    pub fn new() -> Self {
        Lyapunov
    }
}

#[pymethods]
impl Lyapunov {
    #[new]
    fn __new__() -> Self {
        Lyapunov::new()
    }

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
        let state_dim = initial_state.len()?;

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

        let transient_traj_obj = transient_result.bind(py).get_item(0)?;
        // FIX: Explicitly specify the dimension as 2D for the PyArray.
        let transient_traj: &PyArray<f64, Dim<[usize; 2]>> = transient_traj_obj.extract()?;

        let traj_view = unsafe { transient_traj.as_array() };
        let last_row = traj_view.outer_iter().last().unwrap();

        let mut main_y = DVector::from_row_slice(last_row.as_slice().unwrap());

        let mut perturbation_w = DMatrix::<f64>::identity(state_dim, state_dim);
        let mut lyapunov_sums = DVector::<f64>::zeros(state_dim);
        let mut current_t = 0.0;
        let num_steps = (t_total / t_reorth).ceil() as usize;
        let mut spectrum_history: Vec<DVector<f64>> = Vec::with_capacity(num_steps);

        println!("For loop!");
        for s in 0..num_steps {
            println!("Step {s}");
            let mut initial_states: Vec<DVector<f64>> = Vec::with_capacity(state_dim + 1);
            initial_states.push(main_y.clone());
            for j in 0..state_dim {
                initial_states.push(&main_y + eps * perturbation_w.column(j));
            }

            let final_states: Vec<DVector<f64>> = initial_states
                // TODO: Fix this bug 
                // .par_iter() 
                .iter()
                .map(|y0| {
                    Python::with_gil(|py| {
                        let y0_py = y0.as_slice().to_pyarray_bound(py);
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

                        // Get the trajectory for the current parallel task
                        let traj_obj = result_tuple.bind(py).get_item(0).unwrap();
                        let traj: &PyArray<f64, Dim<[usize; 2]>> = traj_obj.extract().unwrap();

                        // Create a view from the correct `traj` variable
                        let traj_view = unsafe { traj.as_array() };
                        // Get the last state from that view
                        let last_state = traj_view.outer_iter().last().unwrap();
                        // Use the correct `last_state` variable to create the vector
                        DVector::from_row_slice(last_state.as_slice().unwrap())
                    })
                })
                .collect();

            main_y = final_states[0].clone();

            let mut evolved_w = DMatrix::<f64>::zeros(state_dim, state_dim);
            for j in 0..state_dim {
                let evolved_perturbation = (&final_states[j + 1] - &main_y) / eps;
                evolved_w.set_column(j, &evolved_perturbation);
            }

            let qr = evolved_w.qr();
            let q = qr.q();
            let r = qr.r();

            for j in 0..state_dim {
                lyapunov_sums[j] += r[(j, j)].abs().ln();
            }

            perturbation_w = q;
            current_t += t_reorth;
            if current_t > 0.0 {
                spectrum_history.push(&lyapunov_sums / current_t);
            }
        }

        let final_spectrum = lyapunov_sums / t_total;
        let final_spectrum_py = final_spectrum.as_slice().to_pyarray_bound(py);

        let history_flat: Vec<f64> = spectrum_history
            .into_iter()
            .flat_map(|v| v.iter().cloned().collect::<Vec<_>>())
            .collect();
        let history_array =
            PyArray::from_vec_bound(py, history_flat).reshape((num_steps, state_dim))?;

        let result_tuple = PyTuple::new_bound(
            py,
            &[final_spectrum_py.to_object(py), history_array.to_object(py)],
        );
        Ok(result_tuple.to_object(py))
    }
}
