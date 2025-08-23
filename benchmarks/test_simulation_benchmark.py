import pytest
import numpy as np
import dynamixplore as dx
from scipy.integrate import solve_ivp
from .systems import lorenz_system, rossler_system, harmonic_oscillator

# --- Benchmark Configuration ---
SYSTEMS = {
    "Lorenz": {"func": lorenz_system, "init": [1.0, 1.0, 1.0], "dim": 3},
    "Rössler": {"func": rossler_system, "init": [0.1, 0.1, 0.1], "dim": 3},
    "SHO": {"func": harmonic_oscillator, "init": [1.0, 0.0], "dim": 2},
}
SIMULATION_TIMES = [50.0, 200.0]
PROBLEM_SIZES = [1, 10]
SOLVER_TYPES = [
    "DX_RK45_Adaptive",
    "SciPy_RK45_Adaptive",
    "DX_RK4_Fixed",
    "SciPy_RK45_FixedLike"
]
DT = 0.01

# --- Pytest Parametrization ---
# This structure correctly creates a separate test for each combination,
# ensuring the benchmark fixture is only used once per test.
@pytest.mark.parametrize("system_name", SYSTEMS.keys())
@pytest.mark.parametrize("t_end", SIMULATION_TIMES)
@pytest.mark.parametrize("size_multiplier", PROBLEM_SIZES)
@pytest.mark.parametrize("solver_type", SOLVER_TYPES)
def test_simulation_performance(benchmark, system_name, t_end, size_multiplier, solver_type):
    """
    Benchmarks a single solver configuration. `pytest` will run this function
    for every combination of the parameters defined above.
    """
    config = SYSTEMS[system_name]
    func = config["func"]
    dim = config["dim"]
    initial_state = np.tile(config["init"], size_multiplier)
    
    # --- Set up benchmark group and name ---
    benchmark.group = f"{system_name}-{t_end}s-{dim*size_multiplier}D"
    benchmark.name = solver_type

    # --- Select and run the correct function based on the solver_type parameter ---
    if solver_type == "DX_RK45_Adaptive":
        sim_dx = dx.Simulation(dynamics_func=func, initial_state=initial_state, t_span=(0.0, t_end), dt=DT)
        benchmark(sim_dx.run, solver='RK45', mode='Adaptive')
        
    elif solver_type == "DX_RK4_Fixed":
        sim_dx = dx.Simulation(dynamics_func=func, initial_state=initial_state, t_span=(0.0, t_end), dt=DT)
        benchmark(sim_dx.run, solver='RK4', mode='Explicit')

    elif solver_type == "SciPy_RK45_Adaptive":
        def run_scipy_adaptive():
            solve_ivp(func, (0.0, t_end), initial_state, method='RK45', dense_output=False, rtol=1e-6, atol=1e-6)
        benchmark(run_scipy_adaptive)

    elif solver_type == "SciPy_RK45_FixedLike":
        def run_scipy_fixed():
            solve_ivp(func, (0.0, t_end), initial_state, method='RK45', dense_output=False, max_step=DT)
        benchmark(run_scipy_fixed)
