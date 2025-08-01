// This file contains the self-contained integrator functions, now ready for Python.

use nalgebra::{DVector, Normed};
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::exceptions::PyNotImplementedError;

// --- 1. Traits for Different Solver Categories ---

/// A trait for FIXED-STEP EXPLICIT steppers.
trait ExplicitStepper {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

/// A trait for ADAPTIVE steppers. The step method returns the higher-order result
/// and an error vector (the difference between the high and low order results).
trait AdaptiveStepper {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<(DVector<f64>, DVector<f64>)>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

/// A trait for IMPLICIT steppers.
trait ImplicitStepper {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<DVector<f64>>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

// --- 2. Structs and Implementations for Each Solver ---

struct Rk45Adaptive;
impl AdaptiveStepper for Rk45Adaptive {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<(DVector<f64>, DVector<f64>)>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        // Dormand-Prince Coefficients
        const C2: f64 = 1.0/5.0; const C3: f64 = 3.0/10.0; const C4: f64 = 4.0/5.0; const C5: f64 = 8.0/9.0;
        const A21: f64 = 1.0/5.0; const A31: f64 = 3.0/40.0; const A32: f64 = 9.0/40.0; const A41: f64 = 44.0/45.0;
        const A42: f64 = -56.0/15.0; const A43: f64 = 32.0/9.0; const A51: f64 = 19372.0/6561.0;
        const A52: f64 = -25360.0/2187.0; const A53: f64 = 64448.0/6561.0; const A54: f64 = -212.0/729.0;
        const A61: f64 = 9017.0/3168.0; const A62: f64 = -355.0/33.0; const A63: f64 = 46732.0/5247.0;
        const A64: f64 = 49.0/176.0; const A65: f64 = -5103.0/18656.0; const A71: f64 = 35.0/384.0;
        const A72: f64 = 0.0; const A73: f64 = 500.0/1113.0; const A74: f64 = 125.0/192.0;
        const A75: f64 = -2187.0/6784.0; const A76: f64 = 11.0/84.0;
        // b_i for 5th order result (the final result)
        const B1: f64 = 35.0/384.0; const B2: f64 = 0.0; const B3: f64 = 500.0/1113.0;
        const B4: f64 = 125.0/192.0; const B5: f64 = -2187.0/6784.0; const B6: f64 = 11.0/84.0; const B7: f64 = 0.0;
        // b_star_i for 4th order result (for error estimation)
        const B_STAR_1: f64 = 5179.0/57600.0; const B_STAR_2: f64 = 0.0; const B_STAR_3: f64 = 7571.0/16695.0;
        const B_STAR_4: f64 = 393.0/640.0; const B_STAR_5: f64 = -92097.0/339200.0; const B_STAR_6: f64 = 187.0/2100.0;
        const B_STAR_7: f64 = 1.0/40.0;

        let k1 = h * f(t, y)?;
        let k2 = h * f(t + C2 * h, &(y + A21 * &k1))?;
        let k3 = h * f(t + C3 * h, &(y + A31 * &k1 + A32 * &k2))?;
        let k4 = h * f(t + C4 * h, &(y + A41 * &k1 + A42 * &k2 + A43 * &k3))?;
        let k5 = h * f(t + C5 * h, &(y + A51 * &k1 + A52 * &k2 + A53 * &k3 + A54 * &k4))?;
        let k6 = h * f(t + h, &(y + A61 * &k1 + A62 * &k2 + A63 * &k3 + A64 * &k4 + A65 * &k5))?;
        let k7 = h * f(t + h, &(y + A71 * &k1 + A72 * &k2 + A73 * &k3 + A74 * &k4 + A75 * &k5 + A76 * &k6))?;

        let y_next_5 = y + B1*&k1 + B2*&k2 + B3*&k3 + B4*&k4 + B5*&k5 + B6*&k6 + B7*&k7;
        let y_next_4 = y + B_STAR_1*&k1 + B_STAR_2*&k2 + B_STAR_3*&k3 + B_STAR_4*&k4 + B_STAR_5*&k5 + B_STAR_6*&k6 + B_STAR_7*&k7;
        
        let error_vec = &y_next_5 - &y_next_4;
        Ok((y_next_5, error_vec))
    }
}

// --- Other solver placeholders (unchanged) ---
struct Rk4Explicit;
impl ExplicitStepper for Rk4Explicit { fn step<F>(&self, _: f64, _: &DVector<f64>, _: f64, _: &mut F) -> PyResult<DVector<f64>> 
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> { 
        Err(PyNotImplementedError::new_err("...")) } 
}

struct EulerExplicit;
impl ExplicitStepper for EulerExplicit { fn step<F>(&self, _: f64, _: &DVector<f64>, _: f64, _: &mut F) -> PyResult<DVector<f64>> 
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> { 
        Err(PyNotImplementedError::new_err("...")) } 
}

struct Rk4Implicit;
impl ImplicitStepper for Rk4Implicit { fn step<F>(&self, _: f64, _: &DVector<f64>, _: f64, _: &mut F) -> PyResult<DVector<f64>> 
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> { 
        Err(PyNotImplementedError::new_err("...")) } 
}

struct EulerImplicit;
impl ImplicitStepper for EulerImplicit { fn step<F>(&self, _: f64, _: &DVector<f64>, _: f64, _: &mut F) -> PyResult<DVector<f64>> 
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> { 
        Err(PyNotImplementedError::new_err("...")) } 
}

// --- 3. Generic Integration Loops ---

fn adaptive_integration_loop<S: AdaptiveStepper>(
    py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>,
    t_start: f64, t_end: f64, initial_h: f64, abstol: f64, reltol: f64, stepper: &S,
) -> PyResult<PyObject> {
    let mut current_y = DVector::from_column_slice(initial_state.as_slice()?);
    let mut current_t = t_start;
    let mut current_h = initial_h;

    let mut times: Vec<f64> = Vec::new();
    let mut trajectory: Vec<DVector<f64>> = Vec::new();
    times.push(current_t);
    trajectory.push(current_y.clone());

    let mut call_dynamics = |t_eval: f64, y_eval: &DVector<f64>| -> PyResult<DVector<f64>> {
        let y_py = y_eval.as_slice().to_pyarray(py);
        let args = PyTuple::new(py, &[t_eval.into_py(py), y_py.into_py(py)]);
        let result = dynamics.call(py, args, None)?;
        let py_array: &PyArray<f64, _> = result.extract(py)?;
        Ok(DVector::from_column_slice(py_array.readonly().as_slice()?))
    };
    
    // Safety factors for step size control
    const SAFETY: f64 = 0.9;
    const MIN_FACTOR: f64 = 0.2;
    const MAX_FACTOR: f64 = 10.0;

    while current_t < t_end {
        if current_t + current_h > t_end {
            current_h = t_end - current_t;
        }

        let (y_next, error_vec) = stepper.step(current_t, &current_y, current_h, &mut call_dynamics)?;
        
        // Calculate error scalar
        let error_norm = error_vec.norm();
        let y_norm = current_y.norm().max(y_next.norm());
        let tolerance = abstol + reltol * y_norm;
        let error = error_norm / tolerance;

        if error <= 1.0 { // Step is accepted
            current_t += current_h;
            current_y = y_next;
            times.push(current_t);
            trajectory.push(current_y.clone());
        }
        
        // Calculate optimal step size for the next step, whether this one was accepted or not
        let mut factor = SAFETY * (1.0 / error).powf(0.2);
        factor = factor.max(MIN_FACTOR).min(MAX_FACTOR);
        current_h *= factor;
    }

    // --- Output Conversion (Rust -> Python) ---
    let num_points = trajectory.len();
    let state_dim = if num_points > 0 { trajectory[0].len() } else { 0 };
    let flat_trajectory: Vec<f64> = trajectory.into_iter().flat_map(|v| v.into_iter().cloned()).collect();
    let traj_array = PyArray::from_vec(py, flat_trajectory).reshape((num_points, state_dim))?;
    let time_array = PyArray::from_vec(py, times);

    Ok(PyTuple::new(py, &[traj_array, time_array]).to_object(py))
}

// The old fixed-step loop remains for the other explicit solvers
fn explicit_integration_loop<S: ExplicitStepper>(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64, stepper: &S) -> PyResult<PyObject> { /* ... unchanged ... */ Ok(PyTuple::new(py, &[]).to_object(py)) }
fn implicit_integration_loop<S: ImplicitStepper>(_py: Python, _dynamics: PyObject, _initial_state: PyReadonlyArray1<f64>, _t_start: f64, _t_end: f64, _h: f64, _stepper: &S) -> PyResult<PyObject> { Err(PyNotImplementedError::new_err("...")) }


// --- 4. Simple, Public-Facing PyFunctions ---

#[pyfunction]
#[pyo3(signature = (dynamics, initial_state, t_start, t_end, h, abstol=1e-6, reltol=1e-6))]
pub fn solve_rk45_adaptive(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64, abstol: f64, reltol: f64) -> PyResult<PyObject> {
    let stepper = Rk45Adaptive;
    adaptive_integration_loop(py, dynamics, initial_state, t_start, t_end, h, abstol, reltol, &stepper)
}

// Other pyfunctions remain unchanged
#[pyfunction]
pub fn solve_rk4_explicit(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64) -> PyResult<PyObject> {
    let stepper = Rk4Explicit;
    explicit_integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}

#[pyfunction]
pub fn solve_euler_explicit(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64) -> PyResult<PyObject> {
    let stepper = EulerExplicit;
    explicit_integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}

#[pyfunction]
pub fn solve_rk4_implicit(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64) -> PyResult<PyObject> {
    let stepper = Rk4Implicit;
    implicit_integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}

#[pyfunction]
pub fn solve_euler_implicit(py: Python, dynamics: PyObject, initial_state: PyReadonlyArray1<f64>, t_start: f64, t_end: f64, h: f64) -> PyResult<PyObject> {
    let stepper = EulerImplicit;
    implicit_integration_loop(py, dynamics, initial_state, t_start, t_end, h, &stepper)
}
