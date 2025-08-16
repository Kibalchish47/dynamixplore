#![allow(unused)] // Remove me later!

use nalgebra::DVector;
use numpy::PyReadonlyArray1;
use pyo3::{pyfunction, PyObject, PyResult, Python};

// For this file, consider that you have three traits:
// {ExplicitStepper, AdaptiveStepper, ImplicitStepper}.
//
// These represent the same operation (stepping), and take in the same
// parameters. To make this more elegant, we can just combine these traits
// into one generic trait, Stepper<A>, for some struct A in {Explicit, AdaptiveStepper, ImplicitStepper}
// Hopefully, you can pick up about how one generic trait is equivalent to multiple non-generic traits...

// In order to represent this set of approaches, we'll need another trait:
pub trait Approach<'py>: /* (2) */ Sized {
    //
    // ...Except not all the return types for `step(...)` are the same! But no worries!
    //
    // To fix this, we need a map from A to some Type. In Rust, this is done using
    // a trait with an associated type.
    //
    // We can add an associated
    // type `Ret` on Approach, which acts as a 1-to-n map from Approach -> Type.
    type Ret;

    // (1) Come back to me later...
    //
    // Since each approach may take a different set of parameters,
    // let's use another associated type... Except, we have one available right now:
    // Self! We can simply make Self have all our parameters (except the Python GIL, and stepper).
    //
    // By using Self, we make this associated function a method, so we can also make use of the dot syntax:
    // e.g. Explicit { ... } .integration_loop(...).
    //
    // I would keep `stepper`, and `py` as separate parameters and not in self, for neatness.
    // 
    // It's also worth noting that I've intentionally chosen `self` (owned, not borrowed) to mimic the 
    // borrow-checking rules of the original functions -- we'll need to add Sized as an extra bound
    // for Self [See (2)]. 
    fn integration_loop<S>(self, py: Python<'py>, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>;
}

//
// Now we can actually write this trait out.
//
// Ignore the 'py lifetimes, focus on <A: Approach> generic.
pub trait Stepper<'py, A: Approach<'py>> {
    // Notice how the signature is the exact same, but the return type is defined
    // in A's Approach definition, which allows for different return types depending
    // on the approach.
    fn step<F>(&self, t: f64, y: &DVector<f64>, h: f64, f: F) -> PyResult<A::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>;
}

// Moving on to the `*_integration_loop` functions, we can see we want to have a one
// function for each approach. -- Wait a minute! -- Since the approaches are now going
// to be structs, we can add an associated function to Approach. [See (1)]
//
// Now, we can define each of the approaches.
pub struct Explicit<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
}

impl<'py> Approach<'py> for Explicit<'py> {
    type Ret = DVector<f64>;

    fn integration_loop<S>(self, py: Python<'py>, stepper: S) -> PyResult<PyObject> {
        todo!()
    }
}

pub struct Adaptive<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
    abstol: f64,
    reltol: f64,
}

impl<'py> Approach<'py> for Adaptive<'py> {
    type Ret = (DVector<f64>, DVector<f64>);

    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
        todo!()
    }
}

pub struct Implicit<'py> {
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<'py, f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
}

impl<'py> Approach<'py> for Implicit<'py> {
    type Ret = DVector<f64>;

    fn integration_loop<S>(self, py: Python, stepper: S) -> PyResult<PyObject>
    where
        S: Stepper<'py, Self>,
    {
        todo!()
    }
}

// Now, we can define the methods.
// You can think of this as trading extra structs for impls:
// E.g. { impl ExplicitStepper for Rk4Explicit,
//        impl ImplicitStepper Rk4Implicit, }
// goes to:
//      { impl Stepper<Explicit> for Rk4,
//        impl Stepper<Implicit> for Rk4, }
//
// I personally think the latter is better design:
// * It allows methods using different approaches to be grouped together,
// * Allows for better syntax.
// * Less imports.

pub struct Rk4;

impl<'py> Stepper<'py, Explicit<'py>> for Rk4 {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: F,
    ) -> PyResult<<Explicit as Approach>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        todo!()
    }
}

impl<'py> Stepper<'py, Implicit<'py>> for Rk4 {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: F,
    ) -> PyResult<<Explicit as Approach>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        todo!()
    }
}

pub struct Rk45;

impl<'py> Stepper<'py, Adaptive<'py>> for Rk45 {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: F,
    ) -> PyResult<<Adaptive as Approach>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        todo!()
    }
}

pub struct Euler;

impl<'py> Stepper<'py, Explicit<'py>> for Euler {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: F,
    ) -> PyResult<<Explicit as Approach>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        todo!()
    }
}

impl<'py> Stepper<'py, Implicit<'py>> for Euler {
    fn step<F>(
        &self,
        t: f64,
        y: &DVector<f64>,
        h: f64,
        f: F,
    ) -> PyResult<<Explicit as Approach>::Ret>
    where
        F: FnMut(f64, &DVector<f64>) -> PyResult<DVector<f64>>,
    {
        todo!()
    }
}

// For the functions you expose to Python:
#[pyfunction]
#[pyo3(signature = (dynamics, initial_state, t_start, t_end, h, abstol=1e-6, reltol=1e-6))]
pub fn solver_rk45_adaptive(
    py: Python,
    dynamics: PyObject,
    initial_state: PyReadonlyArray1<f64>,
    t_start: f64,
    t_end: f64,
    h: f64,
    abstol: f64,
    reltol: f64,
) -> PyResult<PyObject> {
    // You can use the new design:
    Adaptive {
        dynamics,
        initial_state,
        t_start,
        t_end,
        h,
        abstol,
        reltol,
    }
    .integration_loop(py, Rk45)
}

// And so on...

// Further credit, lol:
// * Think about how you'd expose this to Python in a similar way.
// * What if you wanted the reverse syntax (`Rk45.solve(Adaptive {}, py)`)? -- Your choice, lol.