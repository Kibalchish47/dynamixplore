// In src/main.rs
// This is the entry point of our application.

// We declare that we are using the `integrators` module, which corresponds to `integrators.rs`.
mod integrators;

// We use the `use` keyword to bring our functions and structs into the current scope.
use integrators::{solve_rk45, State};

fn main() {
    // Define the differential equation: dy/dt = y
    // The analytical solution is y(t) = e^t
    let f = |_t: f64, y: f64| y;

    // Initial state: t=0, y=1 (which is e^0)
    let mut state = State { t: 0.0, y: 1.0 };

    // Define the step size.
    let h = 0.1;

    println!("Starting integration of dy/dt = y with h = {}", h);
    println!("Initial State: t = {:.2}, y = {:.4}", state.t, state.y);

    // Let's integrate for 10 steps to see it evolve.
    for i in 0..10 {
        state = solve_rk45(&state, f, h);
        // We print the state and the analytical solution for comparison.
        println!(
            "Step {:2}: t = {:.2}, y_numerical = {:.4}, y_analytical = {:.4}",
            i + 1,
            state.t,
            state.y,
            state.t.exp()
        );
    }
}
