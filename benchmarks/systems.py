import numpy as np

# --- 1. Lorenz Attractor (Chaotic) ---
def lorenz_system(t: float, state: np.ndarray) -> np.ndarray:
    """Classic chaotic system."""
    sigma, rho, beta = 10.0, 28.0, 8.0 / 3.0
    x, y, z = state[:3]
    dxdt = sigma * (y - x)
    dydt = x * (rho - z) - y
    dzdt = x * y - beta * z
    # Handle stacked systems for problem size benchmarks
    if len(state) > 3:
        # Recursively call for the rest of the state vector
        rest_of_derivatives = lorenz_system(t, state[3:])
        return np.concatenate(([dxdt, dydt, dzdt], rest_of_derivatives))
    return np.array([dxdt, dydt, dzdt])

# --- 2. Rössler Attractor (Chaotic) ---
def rossler_system(t: float, state: np.ndarray) -> np.ndarray:
    """Another classic chaotic system."""
    a, b, c = 0.2, 0.2, 5.7
    x, y, z = state[:3]
    dxdt = -y - z
    dydt = x + a * y
    dzdt = b + z * (x - c)
    if len(state) > 3:
        rest_of_derivatives = rossler_system(t, state[3:])
        return np.concatenate(([dxdt, dydt, dzdt], rest_of_derivatives))
    return np.array([dxdt, dydt, dzdt])

# --- 3. Simple Harmonic Oscillator (Non-Chaotic, Periodic) ---
def harmonic_oscillator(t: float, state: np.ndarray) -> np.ndarray:
    """A simple, predictable, energy-conserving system."""
    y1, y2 = state[:2]
    dy1dt = y2
    dy2dt = -y1
    if len(state) > 2:
        rest_of_derivatives = harmonic_oscillator(t, state[2:])
        return np.concatenate(([dy1dt, dy2dt], rest_of_derivatives))
    return np.array([dy1dt, dy2dt])

# --- 4. Damped Pendulum (Non-Chaotic, Convergent) ---
def damped_pendulum(t: float, state: np.ndarray) -> np.ndarray:
    """A system that converges to a stable fixed point."""
    g, L, b = 9.81, 1.0, 0.5
    theta, omega = state[:2]
    dthetadt = omega
    domegadt = -(b/1.0) * omega - (g/L) * np.sin(theta)
    if len(state) > 2:
        rest_of_derivatives = damped_pendulum(t, state[2:])
        return np.concatenate(([dthetadt, domegadt], rest_of_derivatives))
    return np.array([dthetadt, domegadt])

# --- 5. Hénon Map (Discrete Chaotic Map) ---
# This is not an ODE, so it will be handled by a separate benchmark script.
def henon_map_step(state: np.ndarray) -> np.ndarray:
    """A classic discrete-time chaotic map."""
    a, b = 1.4, 0.3
    x, y = state
    x_next = 1 - a * x**2 + y
    y_next = b * x
    return np.array([x_next, y_next])

# Helper to run the discrete map for a number of steps
def simulate_henon_map(initial_state, steps):
    trajectory = np.zeros((steps, 2))
    trajectory[0] = initial_state
    for i in range(1, steps):
        trajectory[i] = henon_map_step(trajectory[i-1])
    return trajectory
