// This file contains the refactored, self-contained integrator functions,
// ready for Python binding with PyO3. The design is based on a generic
// `Stepper` and `Approach` trait system for better modularity and extensibility.

use nalgebra::{DMatrix, DVector, Normed};
use numpy::{PyArray, PyReadonlyArray1, ToPyArray};
use pyo3::exceptions::{PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

// --- 1. Core Traits for Generic Solver Design ---

/// A generic trait representing the integration "approach" (e.g., explicit, adaptive).
/// It defines the high-level integration loop and the expected return type of a single step.
/// The `'py` lifetime ensures that any Python objects held within implementors of this
/// trait (like `PyObject` or `PyReadonlyArray1`) do not outlive the Python GIL session.
pub trait Approach<'py>: Sized {
    /// Associated type representing the return value of a single `step` operation.
    /// - For `Explicit` and `Implicit`, this is the next state vector (`DVector<f64>`).
    /// - For `Adaptive`, this is a tuple containing the next state and the error vector
    ///   (`(DVector<f64>, DVector<f64>)`).
    type Ret;

    /// The main integration loop. This method takes ownership of `self` to consume the
    /// integration parameters it holds. It orchestrates the process of repeatedly calling
    /// the stepper's `step` method from `t_start` to `t_end`.
    fn integration_loop<S>(self, py: Python<'py>, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>;
}

/// A generic trait for a "stepper" (the specific integration algorithm, e.g., RK4).
/// It is parameterized by the `Approach` (`A`) it is being used with. This allows a single
/// stepper struct (like `Rk4`) to be implemented for multiple approaches (like `Explicit` and `Implicit`).
pub trait Stepper<'py, A: Approach<'py>> {
    /// Performs a single integration step.
    /// The return type `A::Ret` is determined by the `Approach` being used, making this
    /// method flexible enough to support fixed-step, adaptive, and other methods.
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<A::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

// --- 2. Structs Defining Different Integration Approaches ---

/// Holds parameters for a fixed-step, explicit integration approach.
pub struct Explicit<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
}

/// Holds parameters for an adaptive-step integration approach.
pub struct Adaptive<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    initial_h: f64,
    abstol: f64,
    reltol: f64,
}

/// Holds parameters for a fixed-step, implicit integration approach.
pub struct Implicit<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
}

// --- 3. Implementations of the `Approach` Trait (Integration Loops) ---

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
            let y_py = y_eval.as_slice().to_pyarray(py);
            let args = PyTuple::new(py, &[t_eval.into_py(py), y_py.into_py(py)]);
            let result = self.dynamics.call(py, args, None)?;
            let py_array: &PyArray<f64, _> = result.extract(py)?;
            Ok(DVector::from_column_slice(py_array.readonly().as_slice()?))
        };

        for _ in 0..num_steps {
            let y_next = stepper.step(current_t, &current_y, self.h, &mut call_dynamics)?;
            current_y = y_next;
            current_t += self.h;
            trajectory.push(current_y.clone());
        }

        // Convert trajectory to Python object
        let num_points = trajectory.len();
        let state_dim = if num_points > 0 { trajectory[0].len() } else { 0 };
        let flat_trajectory: Vec<f64> = trajectory.into_iter().flat_map(|v| v.into_iter().cloned()).collect();
        let result_array = PyArray::from_vec(py, flat_trajectory).reshape((num_points, state_dim))?;
        Ok(result_array.to_object(py))
    }
}

impl<'py> Approach<'py> for Adaptive<'py> {
    type Ret = (DVector<f64>, DVector<f64>);

    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
        todo!();
    }
}

impl<'py> Approach<'py> for Implicit<'py> {
    type Ret = DVector<f64>;

    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
       todo!();
    }
}

// --- 4. Stepper Structs and Implementations ---

pub struct Rk45;
impl<'py> Stepper<'py, Adaptive<'py>> for Rk45 {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<<Adaptive<'py> as Approach<'py>>::Ret>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        todo!();
    }
}

pub struct Rk4;
impl<'py> Stepper<'py, Explicit<'py>> for Rk4 {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<<Explicit<'py> as Approach<'py>>::Ret>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        todo!();
    }
}

impl<'py> Stepper<'py, Implicit<'py>> for Rk4 {
    fn step<F>(&self, _t: f64, _y: &DVector<f64>, _h: f64, _f: &mut F) -> PyResult<<Implicit<'py> as Approach<'py>>::Ret>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        Err(PyNotImplementedError::new_err("Implicit RK4 requires a non-linear solver for the coupled stage equations, which is not yet implemented."))
    }
}

pub struct Euler;
impl<'py> Stepper<'py, Explicit<'py>> for Euler {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<<Explicit<'py> as Approach<'py>>::Ret>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        let k1 = f(t, y)?;
        let y_next = y + h * k1;
        Ok(y_next)
    }
}

impl<'py> Stepper<'py, Implicit<'py>> for Euler {
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: &mut F) -> PyResult<<Implicit<'py> as Approach<'py>>::Ret>
    where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
        todo!();
    }
}

// --- 5. Helper Functions for Implicit Solvers ---

/// Approximates the Jacobian matrix of the dynamics function `f` at a point (t, y)
/// using central finite differences. J_ij = ∂f_i / ∂y_j
fn approximate_jacobian<F>(t: f64, y: &DVector<f64>, f: &mut F, eps: f64) -> PyResult<DMatrix<f64>>
where F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>> {
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

        // Reset for the next iteration
        y_plus[j] = original_y_j;
        y_minus[j] = original_y_j;
    }
    Ok(jacobian)
}

/// Solves the non-linear system G(x) = 0 for x using the Newton-Raphson method.
fn newton_raphson_solve<G, F>(
    g: G,
    initial_guess: DVector<f64>,
    t_next: f64,
    h: f64, // Step size of the implicit method
    f: &mut F,
) -> PyResult<DVector<f64>>
where
    G: Fn(&DVector<f64>) -> PyResult<DVector<f64>>,
    F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
{
    let mut x = initial_guess;
    let dim = x.len();
    let identity = DMatrix::<f64>::identity(dim, dim);

    // Solver parameters
    let max_iter = 20;
    let tolerance = 1e-8;
    let jacobian_eps = 1e-6; // Epsilon for finite difference Jacobian of f

    for _ in 0..max_iter {
        let g_eval = g(&x)?;
        if g_eval.norm() < tolerance {
            return Ok(x); // Converged
        }

        // The Jacobian of G(x) = x - y_current - h*f(t_next, x) is J_G = I - h * J_f(t_next, x)
        let jacobian_f = approximate_jacobian(t_next, &x, f, jacobian_eps)?;
        let jacobian_g = &identity - h * jacobian_f;

        // Solve the linear system J_G * delta_x = -G(x) for the update step delta_x
        if let Some(inv_jacobian_g) = jacobian_g.try_inverse() {
            let delta_x = inv_jacobian_g * -g_eval;
            x += delta_x;
        } else {
            return Err(PyValueError::new_err("Failed to solve linear system in Newton's method (matrix is singular)."));
        }
    }

    Err(PyValueError::new_err("Newton's method did not converge."))
}


// --- 6. Public-Facing PyFunctions ---

#[pyfunction]
#[pyo3(signature = (dynamics, initial_state, t_start, t_end, h, abstol=1e-6, reltol=1e-3))]
pub fn solve_rk45_adaptive(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
    abstol: f64,
    reltol: f64,
) -> PyResult<PyObject> {
    Adaptive {
        dynamics,
        initial_state,
        t_start,
        t_end,
        initial_h: h,
        abstol,
        reltol,
    }
    .integration_loop(py, Rk45)
}

#[pyfunction]
pub fn solve_rk4_explicit(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
) -> PyResult<PyObject> {
    Explicit {
        dynamics,
        initial_state,
        t_start,
        t_end,
        h,
    }
    .integration_loop(py, Rk4)
}

#[pyfunction]
pub fn solve_euler_explicit(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
) -> PyResult<PyObject> {
    Explicit {
        dynamics,
        initial_state,
        t_start,
        t_end,
        h,
    }
    .integration_loop(py, Euler)
}

#[pyfunction]
pub fn solve_rk4_implicit(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
) -> PyResult<PyObject> {
    Implicit {
        dynamics,
        initial_state,
        t_start,
        t_end,
        h,
    }
    .integration_loop(py, Rk4)
}

#[pyfunction]
pub fn solve_euler_implicit(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
) -> PyResult<PyObject> {
    Implicit {
        dynamics,
        initial_state,
        t_start,
        t_end,
        h,
    }
    .integration_loop(py, Euler)
}