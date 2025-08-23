import pytest
import numpy as np
import dynamixplore as dx
from scipy.integrate import solve_ivp
from .systems import lorenz_system, rossler_system, harmonic_oscillator, damped_pendulum

# --- Benchmark Configuration ---
SYSTEMS = {
    "Lorenz": {"func": lorenz_system, "init": [1.0, 1.0, 1.0], "dim": 3},
    "Rössler": {"func": rossler_system, "init": [0.1, 0.1, 0.1], "dim": 3},
    "SHO": {"func": harmonic_oscillator, "init": [1.0, 0.0], "dim": 2},
}
SIMULATION_TIMES = [50.0, 200.0]
PROBLEM_SIZES = [1, 10] # Multiplier for the system's dimension
DT = 0.01

# --- Pytest Parametrization ---
# This creates a matrix of all possible test configurations
@pytest.mark.parametrize("system_name", SYSTEMS.keys())
@pytest.mark.parametrize("t_end", SIMULATION_TIMES)
@pytest.mark.parametrize("size_multiplier", PROBLEM_SIZES)
def test_simulation_performance(benchmark, system_name, t_end, size_multiplier):
    """
    Benchmarks DynamiXplore solvers against SciPy for a given configuration.
    The `benchmark` fixture is provided by pytest-benchmark.
    """
    config = SYSTEMS[system_name]
    func = config["func"]
    dim = config["dim"]
    initial_state = np.tile(config["init"], size_multiplier)
    
    # --- DynamiXplore Simulation Setup ---
    sim_dx = dx.Simulation(
        dynamics_func=func,
        initial_state=initial_state,
        t_span=(0.0, t_end),
        dt=DT
    )

    # --- SciPy Simulation Setup ---
    def run_scipy_adaptive():
        solve_ivp(func, (0.0, t_end), initial_state, method='RK45', dense_output=False, rtol=1e-6, atol=1e-6)

    def run_scipy_fixed():
        solve_ivp(func, (0.0, t_end), initial_state, method='RK45', dense_output=False, max_step=DT)
    
    # --- Group benchmarks for better reporting ---
    group_name = f"{system_name}-{t_end}s-{dim*size_multiplier}D"
    
    # Benchmark DynamiXplore
    benchmark.group = group_name
    benchmark.name = "DX_RK45_Adaptive"
    benchmark(sim_dx.run, solver='RK45', mode='Adaptive')
    
    benchmark.name = "DX_RK4_Fixed"
    benchmark(sim_dx.run, solver='RK4', mode='Explicit')

    # Benchmark SciPy
    benchmark.name = "SciPy_RK45_Adaptive"
    benchmark(run_scipy_adaptive)

    benchmark.name = "SciPy_RK45_FixedLike"
    benchmark(run_scipy_fixed)
