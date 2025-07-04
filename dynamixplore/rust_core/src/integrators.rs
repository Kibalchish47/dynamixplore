// This file contains the self-contained integrator functions, now ready for Python.

use nalgebra::DVector;
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

// --- 1. Traits for Different Solver Categories ---

/// A trait for EXPLICIT steppers. These calculate the next state using only known, current information.
trait ExplicitStepper {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<DVector<f64>>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

/// A trait for IMPLICIT steppers. These require solving an equation (often non-linear) at each step.
/// The implementation of an implicit step would be much more complex, likely requiring a root-finding algorithm.
trait ImplicitStepper {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<DVector<f64>>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

// --- 2. Structs and Implementations for Each Solver ---

// -- Explicit Solvers --

struct Rk45Explicit;
impl ExplicitStepper for Rk45Explicit {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<DVector<f64>>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        // Dormand-Prince 5(4) coefficients (hardcoded)
        // Note: Coefficients are defined inside the step where they are used.
        // C_i: time fractions
        // A_ij: stage coefficients
        // B_i: weights for the 5th-rder accurate result (the one we will actually use)
        const C2: f64 = 1.0 / 5.0; const C3: f64 = 3.0 / 10.0; const C4: f64 = 4.0 / 5.0; const C5: f64 = 8.0 / 9.0;
        
        const A21: f64 = 1.0 / 5.0; const A31: f64 = 3.0 / 40.0; const A32: f64 = 9.0 / 40.0; const A41: f64 = 44.0 / 45.0;
        const A42: f64 = -56.0 / 15.0; const A43: f64 = 32.0 / 9.0; const A51: f64 = 19372.0 / 6561.0;
        const A52: f64 = -25360.0 / 2187.0; const A53: f64 = 64448.0 / 6561.0; const A54: f64 = -212.0 / 729.0;
        const A61: f64 = 9017.0 / 3168.0; const A62: f64 = -355.0 / 33.0; const A63: f64 = 46732.0 / 5247.0;
        const A64: f64 = 49.0 / 176.0; const A65: f64 = -5103.0 / 18656.0; const A71: f64 = 35.0 / 384.0;
        const A72: f64 = 0.0; const A73: f64 = 500.0 / 1113.0; const A74: f64 = 125.0 / 192.0;
        const A75: f64 = -2187.0 / 6784.0; const A76: f64 = 11.0 / 84.0; const B1: f64 = 35.0 / 384.0;
        
        const B2: f64 = 0.0; const B3: f64 = 500.0 / 1113.0; const B4: f64 = 125.0 / 192.0;
        const B5: f64 = -2187.0 / 6784.0; const B6: f64 = 11.0 / 84.0; const B7: f64 = 0.0;

        // Calculate stages (k-values) by calling the provided dynamics
        let k1 = h * f(t, y)?;
        let k2 = h * f(t + C2 * h, &(y + A21 * &k1))?;
        let k3 = h * f(t + C3 * h, &(y + A31 * &k1 + A32 * &k2))?;
        let k4 = h * f(t + C4 * h, &(y + A41 * &k1 + A42 * &k2 + A43 * &k3))?;
        let k5 = h * f(t + C5 * h, &(y + A51 * &k1 + A52 * &k2 + A53 * &k3 + A54 * &k4))?;
        let k6 = h * f(t + h, &(y + A61 * &k1 + A62 * &k2 + A63 * &k3 + A64 * &k4 + A65 * &k5))?;
        let k7 = h * f(t + h, &(y + A71 * &k1 + A72 * &k2 + A73 * &k3 + A74 * &k4 + A75 * &k5 + A76 * &k6))?;

        let y_next = y + B1*k1 + B2*k2 + B3*k3 + B4*k4 + B5*k5 + B6*k6 + B7*k7;
        Ok(y_next)
    }
}

// Placeholder for the classic RK4 method.
struct Rk4Explicit;
impl ExplicitStepper for Rk4Explicit {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("The classic explicit RK4 method is not yet implemented."))
    }
}

struct EulerExplicit;
impl ExplicitStepper for EulerExplicit {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("The explicit Euler method is not yet implemented."))
    }
}

// -- Implicit Solvers (Placeholders) --

struct Rk45Implicit;
impl ImplicitStepper for Rk45Implicit {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("The implicit RK45 method is not yet implemented."))
    }
}

struct Rk4Implicit;
impl ImplicitStepper for Rk4Implicit {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("Implicit RK4 requires a non-linear solver, which is not yet implemented."))
    }
}

struct EulerImplicit;
impl ImplicitStepper for EulerImplicit {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("Implicit Euler requires a root-finding algorithm, which is not yet implemented."))
    }
}

// --- 3. The Generic Integration Loops ---
// Those functions contains all the shared boilerplate code. They are NOT pyfunctions.
// They are generic over any type `S` that implements our `ExplicitStepper` and 'ImplicitStepper' traits.

fn explicit_integration_loop<S: ExplicitStepper>(
    py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>,
    t_start: f64, t_end: f64, h: f64, stepper: &S,
) -> PyResult<PyObject> {
    let initial_y = DVector::from_column_slice(initial_state.as_slice()?);
    let mut current_t = t_start;
    let mut current_y = initial_y;

    let num_steps = ((t_end - t_start) / h).ceil() as usize;
    let mut trajectory: Vec<DVector<f64>> = Vec::with_capacity(num_steps + 1);
    trajectory.push(current_state.y.clone());

    // This closure wraps the call to the Python dynamics function. 
    // It's defined once and passed down into the stepper. 
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

    for _ in 0..num_steps { 
        // The magic happens here: we call the specific `step` method
        // of whatever `stepper` was passed in.
        let y_next = stepper.step(current_t, &current_y, h, &mut call_dynamics);

        current_y = y_next; 
        current_t += h; 
        trajectory.push(current_y.clone());  
    }

    // --- Output Conversion (Rust -> Python) ---
    let num_points = trajectory.len();
    let state_dim = trajectory[0].len();
    let flat_trajectory: Vec<f64> = trajectory.into_iter().flat_map(|v| v.into_iter().cloned()).collect();
    let result_array = PyArray::from_vec(py, flat_trajectory).reshape((num_points, state_dim))?;
    Ok(result_array.to_object(py))
}

// Placeholder for the implicit loop. A real implementation would be much more complex.
fn implicit_integration_loop<S: ImplicitStepper>(
    _py: Python, _dynamics: PyObject, _initial_state: PyReadonlyArray1<f64>,
    _t_start: f64, _t_end: f64, _h: f64, _stepper: &S,
) -> PyResult<PyObject> {
     Err(PyNotImplementedError::new_err("The implicit integration loop is not yet implemented."))
}

// --- 4. Simple, Public-Facing PyFunctions ---

#[pyfunction]
pub fn solve_rk4(
    py: Python, 
    dynamics: PyObject, 
    initial_state: PyReadonlyArray1<f64>, 
    t_start: f64, 
    t_end: f64, 
    h: f64,
) -> PyResult<PyObject>{ 
    // 1. Create the specific stepper construct.
    let stepper = Rk4; 

    // 2. Call the generic integration loop with it.
    integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}

#[pyfunction]
pub fn solve_rk45(
    py: Python, 
    dynamics: PyObject, 
    initial_state: PyReadonlyArray1<f64>, 
    t_start: f64, 
    t_end: f64, 
    h: f64,
) -> PyResult<PyObject>{ 
    let stepper = Rk45; 

    integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}
