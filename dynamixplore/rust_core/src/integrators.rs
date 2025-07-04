// This file will contain various integrator functions.

// The State struct is a general-purpose container for the system's state.
// It's public so it can be used by main.rs or any other part of your library.
pub struct State {
    pub t: f64,
    pub y: f64,
}

// This function will implement the Dormand-Prince 5(4) method, often called RK45.
// It takes the current state, the derivative function `f`, and the step size `h`.
pub fn solve_rk45<F>(state: &State, f: F, h: f64) -> State
where
    F: Fn(f64, f64) -> f64,
{
    // Dormand-Prince 5(4) coefficients (hardcoded)
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

    // Calculate the 7 stages (k values)
    let k1 = h * f(state.t, state.y);
    let k2 = h * f(state.t + C2 * h, state.y + A21 * k1);
    let k3 = h * f(state.t + C3 * h, state.y + A31 * k1 + A32 * k2);
    let k4 = h * f(state.t + C4 * h, state.y + A41 * k1 + A42 * k2 + A43 * k3);
    let k5 = h * f(
        state.t + C5 * h,
        state.y + A51 * k1 + A52 * k2 + A53 * k3 + A54 * k4,
    );
    let k6 = h * f(
        state.t + C6 * h,
        state.y + A61 * k1 + A62 * k2 + A63 * k3 + A64 * k4 + A65 * k5,
    );
    let k7 = h * f(
        state.t + C7 * h,
        state.y + A71 * k1 + A72 * k2 + A73 * k3 + A74 * k4 + A75 * k5 + A76 * k6,
    );

    // Calculate the 5th-order accurate result
    let y_next = state.y + B1 * k1 + B3 * k3 + B4 * k4 + B5 * k5 + B6 * k6 + B7 * k7;

    // Return the new state
    State {
        t: state.t + h,
        y: y_next,
    }
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
