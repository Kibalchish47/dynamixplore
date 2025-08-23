// This module houses the numerical ODE solvers, refactored into a class-based, generic architecture.

use nalgebra::{DMatrix, DVector};
use numpy::{ndarray::Dim, PyArray, PyArrayMethods, PyReadonlyArray1, ToPyArray};
use pyo3::exceptions::{PyNotImplementedError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
//use pyo3::Bound;

// --- 1. Core Traits for Generic Solver Design ---

/// A generic trait representing the integration "approach" (e.g., explicit, adaptive).
/// It defines the high-level integration loop and the expected return type of a single step.
pub trait Approach<'py>: Sized {
    type Ret;
    fn integration_loop<S>(self, py: Python<'py>, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>;
}

/// A generic trait for a "stepper" (the specific integration algorithm, e.g., RK4).
pub trait Stepper<'py, A: Approach<'py>> {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<A::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

// --- 2. Approach Structs (Internal Logic) ---
// These structs hold the parameters for a given integration run.
pub struct Explicit<'py> {
    pub dynamics: PyObject,
    pub initial_state: PyReadonlyArray1<'py, f64>,
    pub t_start: f64,
    pub t_end: f64,
    pub h: f64,
}

pub struct Adaptive<'py> {
    pub dynamics: PyObject,
    pub initial_state: PyReadonlyArray1<'py, f64>,
    pub t_start: f64,
    pub t_end: f64,
    pub initial_h: f64,
    pub abstol: f64,
    pub reltol: f64,
}

pub struct Implicit<'py> {
    pub dynamics: PyObject,
    pub initial_state: PyReadonlyArray1<'py, f64>,
    pub t_start: f64,
    pub t_end: f64,
    pub h: f64,
}

// --- 3. Integration Loops (Internal Logic) ---

impl<'py> Approach<'py> for Explicit<'py> {
    type Ret = DVector<f64>;
    fn integration_loop<S>(self, py: Python<'py>, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
        let initial_y = DVector::from_column_slice(self.initial_state.as_slice()?);
        let mut current_t = self.t_start;
        let mut current_y = initial_y;

        let num_steps = ((self.t_end - self.t_start) / self.h).ceil() as usize;
        let mut trajectory: Vec<DVector<f64>> = Vec::with_capacity(num_steps + 1);
        trajectory.push(current_y.clone());

        let mut call_dynamics = |t_eval: f64, y_eval: &DVector<f64>| -> PyResult<DVector<f64>> {
            let y_py = y_eval.as_slice().to_pyarray_bound(py);
            let args = PyTuple::new_bound(py, &[t_eval.into_py(py), y_py.into_py(py)]);
            let result = self.dynamics.call_bound(py, args, None)?;
            let bound_result = result.bind(py);
            let py_array: &PyArray<f64, Dim<[usize; 1]>> = bound_result.extract()?;
            let readonly_array = py_array.readonly();
            Ok(DVector::from_column_slice(readonly_array.as_slice()?))
        };

        for _ in 0..num_steps {
            let y_next = stepper.step(current_t, &current_y, self.h, &mut call_dynamics)?;
            current_y = y_next;
            current_t += self.h;
            trajectory.push(current_y.clone());
        }

        let num_points = trajectory.len();
        let state_dim = if num_points > 0 {
            trajectory[0].len()
        } else {
            0
        };
        let flat_trajectory: Vec<f64> = trajectory
            .into_iter()
            .flat_map(|v| v.iter().cloned().collect::<Vec<f64>>())
            .collect();
        let result_array =
            PyArray::from_vec_bound(py, flat_trajectory).reshape((num_points, state_dim))?;
        Ok(result_array.to_object(py))
    }
}

impl<'py> Approach<'py> for Adaptive<'py> {
    type Ret = (DVector<f64>, DVector<f64>);
    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
        {
        let mut current_y = DVector::from_column_slice(self.initial_state.as_slice()?);
        let mut current_t = self.t_start;
        let mut current_h = self.initial_h;

        let mut times: Vec<f64> = Vec::new();
        let mut trajectory: Vec<DVector<f64>> = Vec::new();
        times.push(current_t);
        trajectory.push(current_y.clone());

        let mut call_dynamics = |t_eval: f64, y_eval: &DVector<f64>| -> PyResult<DVector<f64>> {
            let y_py = y_eval.as_slice().to_pyarray_bound(py);
            let args = PyTuple::new_bound(py, &[t_eval.into_py(py), y_py.into_py(py)]);
            let result = self.dynamics.call_bound(py, args, None)?;
            let bound_result = result.bind(py);
            let py_array: &PyArray<f64, Dim<[usize; 1]>> = bound_result.extract()?;
            let readonly_array = py_array.readonly();
            Ok(DVector::from_column_slice(readonly_array.as_slice()?))
        };

        const SAFETY: f64 = 0.9;
        const MIN_FACTOR: f64 = 0.2;
        const MAX_FACTOR: f64 = 10.0;

        while current_t < self.t_end {
            if current_t + current_h > self.t_end {
                current_h = self.t_end - current_t;
            }
            if current_h <= 0.0 {
                break;
            }

            let (y_next, error_vec) =
                stepper.step(current_t, &current_y, current_h, &mut call_dynamics)?;
            let error_norm = error_vec.norm();
            let y_norm = current_y.norm().max(y_next.norm());
            let tolerance = self.abstol + self.reltol * y_norm;
            let error = if tolerance > 0.0 {
                error_norm / tolerance
            } else {
                0.0
            };

            if error <= 1.0 {
                current_t += current_h;
                current_y = y_next;
                times.push(current_t);
                trajectory.push(current_y.clone());
            }

            let factor = if error > 0.0 {
                let factor = SAFETY * (1.0 / error).powf(0.2);
                factor.clamp(MIN_FACTOR, MAX_FACTOR)
            } else {
                MAX_FACTOR
            };
            current_h *= factor;
        }
        
        let num_points = trajectory.len();
        let state_dim = if num_points > 0 {
            trajectory[0].len()
        } else {
            0
        };
        let flat_trajectory: Vec<f64> = trajectory
        .into_iter()
        .flat_map(|v| v.iter().cloned().collect::<Vec<f64>>())
        .collect();
        let traj_array =
        PyArray::from_vec_bound(py, flat_trajectory).reshape((num_points, state_dim))?;
        let time_array = PyArray::from_vec_bound(py, times);
        let result_tuple =
        PyTuple::new_bound(py, &[traj_array.to_object(py), time_array.to_object(py)]);

        let a = Ok(result_tuple.to_object(py));
        a
    }
}

impl<'py> Approach<'py> for Implicit<'py> {
    type Ret = DVector<f64>;
    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
        let initial_y = DVector::from_column_slice(self.initial_state.as_slice()?);
        let mut current_t = self.t_start;
        let mut current_y = initial_y;

        let num_steps = ((self.t_end - self.t_start) / self.h).ceil() as usize;
        let mut trajectory: Vec<DVector<f64>> = Vec::with_capacity(num_steps + 1);
        trajectory.push(current_y.clone());

        let mut call_dynamics = |t_eval: f64, y_eval: &DVector<f64>| -> PyResult<DVector<f64>> {
            let y_py = y_eval.as_slice().to_pyarray_bound(py);
            let args = PyTuple::new_bound(py, &[t_eval.into_py(py), y_py.into_py(py)]);
            let result = self.dynamics.call_bound(py, args, None)?;
            let bound_result = result.bind(py);
            let py_array: &PyArray<f64, Dim<[usize; 1]>> = bound_result.extract()?;
            let readonly_array = py_array.readonly();
            Ok(DVector::from_column_slice(readonly_array.as_slice()?))
        };

        for _ in 0..num_steps {
            let y_next = stepper.step(current_t, &current_y, self.h, &mut call_dynamics)?;
            current_y = y_next;
            current_t += self.h;
            trajectory.push(current_y.clone());
        }

        let num_points = trajectory.len();
        let state_dim = if num_points > 0 {
            trajectory[0].len()
        } else {
            0
        };
        let flat_trajectory: Vec<f64> = trajectory
            .into_iter()
            .flat_map(|v| v.iter().cloned().collect::<Vec<f64>>())
            .collect();
        let result_array =
            PyArray::from_vec_bound(py, flat_trajectory).reshape((num_points, state_dim))?;
        Ok(result_array.to_object(py))
    }
}

// --- 4. Stepper Implementations (Internal Logic) ---
#[pyclass]
#[derive(Copy, Clone)]
pub struct Rk45;

impl<'py> Stepper<'py, Adaptive<'py>> for Rk45 {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: &mut F,
    ) -> PyResult<<Adaptive<'py> as Approach<'py>>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        // Dormand-Prince Coefficients
        const C2: f64 = 1.0 / 5.0;
        const C3: f64 = 3.0 / 10.0;
        const C4: f64 = 4.0 / 5.0;
        const C5: f64 = 8.0 / 9.0;
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
        const B1: f64 = 35.0 / 384.0;
        const B2: f64 = 0.0;
        const B3: f64 = 500.0 / 1113.0;
        const B4: f64 = 125.0 / 192.0;
        const B5: f64 = -2187.0 / 6784.0;
        const B6: f64 = 11.0 / 84.0;
        const B7: f64 = 0.0;
        const B_STAR_1: f64 = 5179.0 / 57600.0;
        const B_STAR_2: f64 = 0.0;
        const B_STAR_3: f64 = 7571.0 / 16695.0;
        const B_STAR_4: f64 = 393.0 / 640.0;
        const B_STAR_5: f64 = -92097.0 / 339200.0;
        const B_STAR_6: f64 = 187.0 / 2100.0;
        const B_STAR_7: f64 = 1.0 / 40.0;
        let k1 = h * f(t, y)?;
        let k2 = h * f(t + C2 * h, &(y + A21 * &k1))?;
        let k3 = h * f(t + C3 * h, &(y + A31 * &k1 + A32 * &k2))?;
        let k4 = h * f(t + C4 * h, &(y + A41 * &k1 + A42 * &k2 + A43 * &k3))?;
        let k5 = h * f(
            t + C5 * h,
            &(y + A51 * &k1 + A52 * &k2 + A53 * &k3 + A54 * &k4),
        )?;
        let k6 = h * f(
            t + h,
            &(y + A61 * &k1 + A62 * &k2 + A63 * &k3 + A64 * &k4 + A65 * &k5),
        )?;
        let k7 = h * f(
            t + h,
            &(y + A71 * &k1 + A72 * &k2 + A73 * &k3 + A74 * &k4 + A75 * &k5 + A76 * &k6),
        )?;
        let y_next_5 =
            y + B1 * &k1 + B2 * &k2 + B3 * &k3 + B4 * &k4 + B5 * &k5 + B6 * &k6 + B7 * &k7;
        let y_next_4 = y
            + B_STAR_1 * &k1
            + B_STAR_2 * &k2
            + B_STAR_3 * &k3
            + B_STAR_4 * &k4
            + B_STAR_5 * &k5
            + B_STAR_6 * &k6
            + B_STAR_7 * &k7;
        let error_vec = &y_next_5 - &y_next_4;
        Ok((y_next_5, error_vec))
    }
}

#[pyclass]
#[derive(Copy, Clone)]
pub struct Rk4;

impl<'py> Stepper<'py, Explicit<'py>> for Rk4 {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: &mut F,
    ) -> PyResult<<Explicit<'py> as Approach<'py>>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        let k1 = f(t, y)?;
        let k2 = f(t + 0.5 * h, &(y + 0.5 * h * &k1))?;
        let k3 = f(t + 0.5 * h, &(y + 0.5 * h * &k2))?;
        let k4 = f(t + h, &(y + h * &k3))?;
        Ok(y + (h / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4))
    }
}
impl<'py> Stepper<'py, Implicit<'py>> for Rk4 {
    fn step<F>(
        &self,
        _t: f64,
        _y: &DVector<f64>,
        _h: f64,
        _f: &mut F,
    ) -> PyResult<<Implicit<'py> as Approach<'py>>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        Err(PyNotImplementedError::new_err(
            "Implicit RK4 is not yet implemented.",
        ))
    }
}

#[pyclass]
#[derive(Copy, Clone)]
pub struct Euler;

impl<'py> Stepper<'py, Explicit<'py>> for Euler {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: &mut F,
    ) -> PyResult<<Explicit<'py> as Approach<'py>>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        let k1 = f(t, y)?;
        Ok(y + h * k1)
    }
}
impl<'py> Stepper<'py, Implicit<'py>> for Euler {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: &mut F,
    ) -> PyResult<<Implicit<'py> as Approach<'py>>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        let t_next = t + h;
        // FIX: Calculate initial_guess BEFORE defining the closure `g` to avoid borrow checker conflict.
        let initial_guess = y + h * f(t, y)?;
        // FIX: The call to newton_raphson_solve is now correct.
        newton_raphson_solve(y, initial_guess, t_next, h, f)
    }
}

fn approximate_jacobian<F>(t: f64, y: &DVector<f64>, f: &mut F, eps: f64) -> PyResult<DMatrix<f64>>
where
    F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
{
    let dim = y.len();
    let mut jacobian = DMatrix::<f64>::zeros(dim, dim);
    let mut y_plus = y.clone();
    let mut y_minus = y.clone();
    for j in 0..dim {
        let original_y_j = y[j];
        y_plus[j] += eps;
        y_minus[j] -= eps;
        let f_plus = f(t, &y_plus)?;
        let f_minus = f(t, &y_minus)?;
        let column = (f_plus - f_minus) / (2.0 * eps);
        jacobian.set_column(j, &column);
        y_plus[j] = original_y_j;
        y_minus[j] = original_y_j;
    }
    Ok(jacobian)
}

// FIX: Refactored the function signature and body to resolve the borrow-checker error.
// It no longer takes `g` as an argument. Instead, it takes the previous state `y`
// and constructs the closure `g` internally. This prevents `f` from being borrowed
// mutably by the closure and the function call simultaneously.
fn newton_raphson_solve<F>(
    y: &DVector<f64>,
    initial_guess: DVector<f64>,
    t_next: f64,
    h: f64,
    f: &mut F,
) -> PyResult<DVector<f64>>
where
    F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
{
    let mut x = initial_guess;
    let dim = x.len();
    let identity = DMatrix::<f64>::identity(dim, dim);
    let max_iter = 20;
    let tolerance = 1e-8;
    let jacobian_eps = 1e-6;

    for _ in 0..max_iter {
        // Define the function g(y_next) = y_next - y_prev - h * f(t_next, y_next)
        // whose root we want to find.
        let g_eval = {
            let f_eval = f(t_next, &x)?;
            &x - y - h * f_eval
        };

        if g_eval.norm() < tolerance {
            return Ok(x);
        }

        let jacobian_f = approximate_jacobian(t_next, &x, f, jacobian_eps)?;
        let jacobian_g = &identity - h * jacobian_f;

        if let Some(inv_jacobian_g) = jacobian_g.try_inverse() {
            x -= inv_jacobian_g * g_eval; // Corrected update step
        } else {
            return Err(PyValueError::new_err(
                "Failed to solve linear system in Newton's method (matrix is singular).",
            ));
        }
    }
    Err(PyValueError::new_err("Newton's method did not converge."))
}

// --- 5. PyO3 Class Definitions for Python API ---

#[pyclass(name = "Explicit")]
#[derive(Clone)]
pub struct ExplicitParams {
    #[pyo3(get, set)]
    dynamics: PyObject,
    #[pyo3(get, set)]
    initial_state: PyObject,
    #[pyo3(get, set)]
    t_start: f64,
    #[pyo3(get, set)]
    t_end: f64,
    #[pyo3(get, set)]
    h: f64,
}

#[pymethods]
impl ExplicitParams {
    #[new]
    fn new(dynamics: PyObject, initial_state: PyObject, t_start: f64, t_end: f64, h: f64) -> Self {
        Self {
            dynamics,
            initial_state,
            t_start,
            t_end,
            h,
        }
    }
}

#[pyclass(name = "Implicit")]
#[derive(Clone)]
pub struct ImplicitParams {
    #[pyo3(get, set)]
    dynamics: PyObject,
    #[pyo3(get, set)]
    initial_state: PyObject,
    #[pyo3(get, set)]
    t_start: f64,
    #[pyo3(get, set)]
    t_end: f64,
    #[pyo3(get, set)]
    h: f64,
}

#[pymethods]
impl ImplicitParams {
    #[new]
    fn new(dynamics: PyObject, initial_state: PyObject, t_start: f64, t_end: f64, h: f64) -> Self {
        Self {
            dynamics,
            initial_state,
            t_start,
            t_end,
            h,
        }
    }
}

#[pyclass(name = "Adaptive")]
#[derive(Clone)]
pub struct AdaptiveParams {
    #[pyo3(get, set)]
    dynamics: PyObject,
    #[pyo3(get, set)]
    initial_state: PyObject,
    #[pyo3(get, set)]
    t_start: f64,
    #[pyo3(get, set)]
    t_end: f64,
    #[pyo3(get, set)]
    h: f64,
    #[pyo3(get, set)]
    abstol: f64,
    #[pyo3(get, set)]
    reltol: f64,
}

#[pymethods]
impl AdaptiveParams {
    #[new]
    #[pyo3(signature = (dynamics, initial_state, t_start, t_end, h, abstol=1e-6, reltol=1e-3))]
    fn new(
        dynamics: PyObject,
        initial_state: PyObject,
        t_start: f64,
        t_end: f64,
        h: f64,
        abstol: f64,
        reltol: f64,
    ) -> Self {
        Self {
            dynamics,
            initial_state,
            t_start,
            t_end,
            h,
            abstol,
            reltol,
        }
    }
}

#[pymethods]
impl Rk45 {
    #[new]
    fn new() -> Self {
        Rk45
    }

    fn solve<'py>(&self, py: Python<'py>, mode: PyObject) -> PyResult<PyObject> {
        if let Ok(params) = mode.extract::<AdaptiveParams>(py) {
            let initial_state = params.initial_state.extract::<PyReadonlyArray1<f64>>(py)?;
            Adaptive {
                dynamics: params.dynamics,
                initial_state,
                t_start: params.t_start,
                t_end: params.t_end,
                initial_h: params.h,
                abstol: params.abstol,
                reltol: params.reltol,
            }
            .integration_loop(py, *self)
        } else {
            Err(PyTypeError::new_err(
                "RK45 solver requires an 'Adaptive' mode.",
            ))
        }
    }
}

#[pymethods]
impl Rk4 {
    #[new]
    fn new() -> Self {
        Rk4
    }

    fn solve<'py>(&self, py: Python<'py>, mode: PyObject) -> PyResult<PyObject> {
        if let Ok(params) = mode.extract::<ExplicitParams>(py) {
            let initial_state = params.initial_state.extract::<PyReadonlyArray1<f64>>(py)?;
            Explicit {
                dynamics: params.dynamics,
                initial_state,
                t_start: params.t_start,
                t_end: params.t_end,
                h: params.h,
            }
            .integration_loop(py, *self)
        } else if let Ok(params) = mode.extract::<ImplicitParams>(py) {
            let initial_state = params.initial_state.extract::<PyReadonlyArray1<f64>>(py)?;
            Implicit {
                dynamics: params.dynamics,
                initial_state,
                t_start: params.t_start,
                t_end: params.t_end,
                h: params.h,
            }
            .integration_loop(py, *self)
        } else {
            Err(PyTypeError::new_err(
                "RK4 solver requires an 'Explicit' or 'Implicit' mode.",
            ))
        }
    }
}

#[pymethods]
impl Euler {
    #[new]
    fn new() -> Self {
        Euler
    }

    fn solve<'py>(&self, py: Python<'py>, mode: PyObject) -> PyResult<PyObject> {
        if let Ok(params) = mode.extract::<ExplicitParams>(py) {
            let initial_state = params.initial_state.extract::<PyReadonlyArray1<f64>>(py)?;
            Explicit {
                dynamics: params.dynamics,
                initial_state,
                t_start: params.t_start,
                t_end: params.t_end,
                h: params.h,
            }
            .integration_loop(py, *self)
        } else if let Ok(params) = mode.extract::<ImplicitParams>(py) {
            let initial_state = params.initial_state.extract::<PyReadonlyArray1<f64>>(py)?;
            Implicit {
                dynamics: params.dynamics,
                initial_state,
                t_start: params.t_start,
                t_end: params.t_end,
                h: params.h,
            }
            .integration_loop(py, *self)
        } else {
            Err(PyTypeError::new_err(
                "Euler solver requires an 'Explicit' or 'Implicit' mode.",
            ))
        }
    }
}
