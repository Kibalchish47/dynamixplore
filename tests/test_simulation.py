# This file contains tests for the numerical integrators.

import pytest
import numpy as np
import dynamixplore as dx

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
    t_start, t_end, h = 0.0, 2 * np.pi, 0.01
    
    # Use the Simulation class to configure and run the simulation
    sim = dx.Simulation(
        dynamics_func=harmonic_oscillator,
        initial_state=[1.0, 0.0],
        t_span=(t_start, t_end),
        dt=h
    )
    analysis_obj = sim.run(solver='RK4', mode='Explicit')

    # The final state should be very close to the initial state after one full period.
    final_state = analysis_obj.trajectory[-1]
    expected_state = np.array([np.cos(t_end), -np.sin(t_end)])

    # pytest.approx handles floating-point comparisons gracefully.
    assert final_state == pytest.approx(expected_state, abs=1e-5)

def test_solve_euler_explicit_on_sho():
    """
    Tests the fixed-step Euler solver. We expect it to be much less accurate
    than RK4, so we use a larger tolerance.
    """
    t_start, t_end, h = 0.0, 2 * np.pi, 0.001 # Smaller step size needed for stability

    sim = dx.Simulation(
        dynamics_func=harmonic_oscillator,
        initial_state=[1.0, 0.0],
        t_span=(t_start, t_end),
        dt=h
    )
    analysis_obj = sim.run(solver='Euler', mode='Explicit')

    final_state = analysis_obj.trajectory[-1]
    expected_state = np.array([np.cos(t_end), -np.sin(t_end)])

    # Euler is a first-order method, so the error will be larger.
    assert final_state == pytest.approx(expected_state, abs=1e-2)

def test_solve_rk45_adaptive_on_sho():
    """
    Tests the adaptive RK45 solver against the SHO. It should be very accurate.
    """
    t_start, t_end, h_init = 0.0, 2 * np.pi, 0.1 # Can start with a large step

    sim = dx.Simulation(
        dynamics_func=harmonic_oscillator,
        initial_state=[1.0, 0.0],
        t_span=(t_start, t_end),
        dt=h_init
    )
    analysis_obj = sim.run(solver='RK45', mode='Adaptive')

    final_state = analysis_obj.trajectory[-1]
    # The final time will not be exactly t_end, so we use the last time step from the solver.
    final_time = analysis_obj.t[-1]
    expected_state = np.array([np.cos(final_time), -np.sin(final_time)])

    assert final_state == pytest.approx(expected_state, abs=1e-7)

def test_implicit_solvers():
    """
    Ensures that the placeholder implicit RK4 solver correctly raises a
    NotImplementedError, and that the implicit Euler solver runs without error.
    """
    sim = dx.Simulation(
        dynamics_func=harmonic_oscillator,
        initial_state=[1.0, 0.0],
        t_span=(0.0, 1.0),
        dt=0.1
    )
    
    # Implicit RK4 is not implemented and should raise an error from the Rust core.
    # PyO3 translates Rust's PyNotImplementedError to Python's NotImplementedError.
    with pytest.raises(NotImplementedError):
        sim.run(solver='RK4', mode='Implicit')

    # Implicit Euler IS implemented and should run without raising an error.
    try:
        sim.run(solver='Euler', mode='Implicit')
    except NotImplementedError:
        pytest.fail("Implicit Euler solver incorrectly raised NotImplementedError.")
