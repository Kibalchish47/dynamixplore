// This file contains the self-contained integrator functions, now ready for Python.

use nalgebra::DVector;
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[derive(Clone)]
// The State struct is a general-purpose container for the system's state
// i.e. a convenient internal representation.
// It's public so it can be used by main.rs or any other part of your library.
pub struct State {
    pub t: f64,
    pub y: f64,
}

// This function will implement the Dormand-Prince 5(4) method, often called RK45.
// It takes the current state, the derivative function `f`, and the step size `h`.
pub fn solve_rk45<F>(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
) -> PyResult<PyObject> {
    // --- Input Conversion (Python -> Rust) ---
    // Get a view of the initial state numpy array to avoid copying.
    let initial_state_view = initial_state.as_slice()?;
    // Convert it to a nalgebra DVector.
    let initial_y = DVector::from_column_slice(initial_state_view);

    // --- Internal integration loop ---
    let mut current_state = State {
        t: t_start,
        y: initial_y,
    };

    let num_steps = ((t_end - t_start) / h).ceil() as usize;
    let mut trajectory: Vec<DVector<f64>> = Vec::with_capacity(num_steps + 1);
    trajectory.push(current_state.y.clone());

    // --- Dormand-Prince 5(4) coefficients (hardcoded) ---
    // c_i: time fractions
    const C2: f64 = 1.0 / 5.0;
    const C3: f64 = 3.0 / 10.0;
    const C4: f64 = 4.0 / 5.0;
    const C5: f64 = 8.0 / 9.0;
    const C6: f64 = 1.0;
    const C7: f64 = 1.0;

    // a_ij: stage coefficients
    const A21: f64 = 1.0 / 5.0;

    const A31: f64 = 3.0 / 40.0;
    const A32: f64 = 9.0 / 40.0;

    const A41: f64 = 44.0 / 45.0;
    const A42: f64 = -56.0 / 15.0;
    const A43: f64 = 32.0 / 9.0;

    const A51: f64 = 19372.0 / 6561.0;
    const A52: f64 = -25360.0 / 2187.0;
    const A53: f64 = 64448.0 / 6561.0;
    const A54: f64 = -212.0 / 729.0;

    const A61: f64 = 9017.0 / 3168.0;
    const A62: f64 = -355.0 / 33.0;
    const A63: f64 = 46732.0 / 5247.0;
    const A64: f64 = 49.0 / 176.0;
    const A65: f64 = -5103.0 / 18656.0;

    const A71: f64 = 35.0 / 384.0;
    const A72: f64 = 0.0;
    const A73: f64 = 500.0 / 1113.0;
    const A74: f64 = 125.0 / 192.0;
    const A75: f64 = -2187.0 / 6784.0;
    const A76: f64 = 11.0 / 84.0;

    // b_i: weights for the 5th-rder accurate result (the one we will actually use)
    const B1: f64 = 35.0 / 384.0;
    const B2: f64 = 0.0;
    const B3: f64 = 500.0 / 1113.0;
    const B4: f64 = 125.0 / 192.0;
    const B5: f64 = -2187.0 / 6784.0;
    const B6: f64 = 11.0 / 84.0;
    const B7: f64 = 0.0;

    for _ in 0..num_steps {
        let y = &current_state.y;
        let t = current_state.t;

        // --- The Python Callback ---
        // This is the core of the interaction. We define a helper closure
        // to reduce boilerplate code for calling the Python dynamics function.
        let call_dynamics = |t_eval: f64, y_eval: &DVector<f64>| -> PyResult<DVector<f64>> {
            // Convert the Rust vector into a Python Numpy array for the callback
            let y_py = y_eval.as_slice().to_pyarray(py);

            // Prepare arguments for the Python function (t, y)
            let args = PyTuple::new(py, &[t_eval.into_py(py), y_py.into_py(py)]);

            // Call the Python function with the prepared arguments
            let result = dynamics.call(py, args, None)?;

            // Extract the result back into a Rust-readable NumPy Array View
            let py_array: &PyArray<f64, _> = result.extract(py)?;
            let readonly_array = py_array.readonly();

            // Convert the result back to a nlagebra DVector
            Ok(DVector::from_column_slice(
                readonly_array.as_slice().unwrap(),
            ))
        };

        // Calculate stages by calling back into Python for each derivative evaluation
        let k1 = h * call_dynamics(t, y)?;
        let k2 = h * call_dynamics(t + C2 * h, &(y + A21 * &k1))?;
        let k3 = h * call_dynamics(t + C3 * h, &(y + A31 * &k1 + A32 * &k2))?;
        let k4 = h * call_dynamics(t + C4 * h, &(y + A41 * &k1 + A42 * &k2 + A43 * &k3))?;
        let k5 = h * call_dynamics(
            t + C5 * h,
            &(y + A51 * &k1 + A52 * &k2 + A53 * &k3 + A54 * &k4),
        )?;
        let k6 = h * call_dynamics(
            t + h,
            &(y + A61 * &k1 + A62 * &k2 + A63 * &k3 + A64 * &k4 + A65 * &k5),
        )?;
        let k7 = h * call_dynamics(
            t + h,
            &(y + A71 * &k1 + A72 * &k2 + A73 * &k3 + A74 * &k4 + A75 * &k5 + A76 * &k6),
        )?;

        let y_next = y + B1 * k1 + B2 * k2 + B3 * k3 + B4 * k4 + B5 * k5 + B6 * k6 + B7 * k7;

        current_state = State {
            t: t + h,
            y: y_next,
        };
        trajectory.push(current_state.y.clone());
    }

    // --- Output Conversion (Rust -> Python) ---
    // Get the dimensions of the final trajectory.
    let num_points = trajectory.len();
    let state_dim = trajectory[0].len();

    // Flatten the Vec<DVector<f64>> into a single Vec<f64>.
    let flat_trajectory: Vec<f64> = trajectory
        .into_iter()
        .flat_map(|v| v.into_iter().cloned())
        .collect();

    // Create a 2D NumPy array from the flattened data and return it.
    let result_array = PyArray::from_vec(py, flat_trajectory).reshape((num_points, state_dim))?;

    Ok(result_array.to_object(py))
}

// A placeholder for a standard RK4 integrator.
pub fn solve_rk4<F>(_state: &State, _f: F, _h: f64) -> State
where
    F: Fn(f64, f64) -> f64,
{
    // The `unimplemented!` macro is a useful way to mark functions that
    // are not yet written. It will cause the program to panic if called.
    unimplemented!("The classic RK4 method is not yet implemented.");
}
