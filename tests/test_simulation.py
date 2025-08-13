# This file contains tests for the numerical integrators.

import pytest
import numpy as np
import dx_core
from scipy.integrate import solve_ivp

# --- Test against a known analytical solution: Simple Harmonic Oscillator ---
# The system is y'' = -y, which can be written as a first-order system:
# y1' = y2
# y2' = -y1
#
# With initial state [y1(0), y2(0)] = [1.0, 0.0], the exact solution is:
# y1(t) = cos(t)
# y2(t) = -sin(t)

def harmonic_oscillator(t, state):
    """Defines the simple harmonic oscillator system."""
    y1, y2 = state
    return np.array([y2, -y1])

def test_solve_rk4_explicit_on_sho():
    """
    Tests the fixed-step RK4 solver against the known solution for the
    simple harmonic oscillator.
    """
    initial_state = np.array([1.0, 0.0])
    t_start, t_end, h = 0.0, 2 * np.pi, 0.01

    trajectory = dx_core.solve_rk4_explicit(
        harmonic_oscillator, initial_state, t_start, t_end, h
    )

    # The final state should be very close to the initial state after one full period.
    final_state = trajectory[-1]
    expected_state = np.array([np.cos(t_end), -np.sin(t_end)])

    # pytest.approx handles floating-point comparisons gracefully.
    assert final_state == pytest.approx(expected_state, abs=1e-5)

def test_solve_euler_explicit_on_sho():
    """
    Tests the fixed-step Euler solver. We expect it to be much less accurate
    than RK4, so we use a larger tolerance.
    """
    initial_state = np.array([1.0, 0.0])
    t_start, t_end, h = 0.0, 2 * np.pi, 0.001 # Smaller step size needed for stability

    trajectory = dx_core.solve_euler_explicit(
        harmonic_oscillator, initial_state, t_start, t_end, h
    )

    final_state = trajectory[-1]
    expected_state = np.array([np.cos(t_end), -np.sin(t_end)])

    # Euler is a first-order method, so the error will be larger.
    assert final_state == pytest.approx(expected_state, abs=1e-2)

def test_solve_rk45_adaptive_on_sho():
    """
    Tests the adaptive RK45 solver against the SHO. It should be very accurate.
    """
    initial_state = np.array([1.0, 0.0])
    t_start, t_end, h_init = 0.0, 2 * np.pi, 0.1 # Can start with a large step

    trajectory, times = dx_core.solve_rk45_adaptive(
        harmonic_oscillator, initial_state, t_start, t_end, h_init
    )

    final_state = trajectory[-1]
    # The final time will not be exactly t_end, so we use the last time step from the solver.
    expected_state = np.array([np.cos(times[-1]), -np.sin(times[-1])])

    assert final_state == pytest.approx(expected_state, abs=1e-7)

def test_implicit_solvers_raise_not_implemented():
    """
    Ensures that the placeholder implicit solvers correctly raise a
    NotImplementedError when called.
    """
    initial_state = np.array([1.0, 0.0])
    with pytest.raises(NotImplementedError):
        dx_core.solve_rk4_implicit(harmonic_oscillator, initial_state, 0.0, 1.0, 0.1)

    with pytest.raises(NotImplementedError):
        dx_core.solve_euler_implicit(harmonic_oscillator, initial_state, 0.0, 1.0, 0.1)

