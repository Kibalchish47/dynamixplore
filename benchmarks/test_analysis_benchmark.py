import pytest
import numpy as np
import dynamixplore as dx
import nolds
from scipy.stats import binned_statistic_2d
from .systems import lorenz_system

# --- Fixture to generate a standard Analysis object for all tests ---
@pytest.fixture(scope="module")
def lorenz_analysis_object():
    """
    Pre-computes a Lorenz trajectory and returns a full Analysis object,
    so all analysis tests run on the same, consistent data structure.
    """
    print("\nGenerating Analysis object for analysis benchmarks...")
    sim = dx.Simulation(
        dynamics_func=lorenz_system,
        initial_state=[1.0, 1.0, 1.0],
        t_span=(0.0, 200.0),
        dt=0.01
    )
    trajectory = sim.run(solver='RK4', mode='Explicit')
    
    analysis_obj = dx.Analysis(trajectory=trajectory, dt=0.01)
    return analysis_obj

# --- Benchmark Functions ---

# FIX: Parametrize over the functions to benchmark to avoid FixtureAlreadyUsed error.
@pytest.mark.parametrize("method", ["DynamiXplore", "nolds"])
def test_lyapunov_benchmark(benchmark, lorenz_analysis_object, method):
    """Benchmarks Lyapunov spectrum calculation vs. nolds."""
    # FIX: Pass the correct 1D array to the nolds function.
    data = lorenz_analysis_object.trajectory
    data_1d = data[:, 0]
    
    benchmark.group = "Lyapunov Spectrum"
    benchmark.name = method

    if method == "DynamiXplore":
        def run_dx_lyapunov():
            lorenz_analysis_object.lyapunov_spectrum(
                dynamics=lorenz_system,
                t_transient=10.0,
                t_total=1000.0,
                t_reorth=1.0
            )
        benchmark(run_dx_lyapunov)
    
    elif method == "nolds":
        def run_nolds_lyapunov():
            # Nolds only calculates the largest exponent from a 1D time series
            nolds.lyap_r(data_1d, lag=10, min_tsep=200)
        benchmark(run_nolds_lyapunov)


def test_entropy_benchmark(benchmark, lorenz_analysis_object):
    """Benchmarks permutation entropy."""
    def run_dx_entropy():
        lorenz_analysis_object.permutation_entropy(dim=0, m=3, tau=1)
        
    benchmark.group = "Permutation Entropy"
    benchmark.name = "DynamiXplore"
    benchmark(run_dx_entropy)

# FIX: Parametrize over the functions to benchmark.
@pytest.mark.parametrize("method", ["DynamiXplore", "SciPy"])
def test_invariant_measure_benchmark(benchmark, lorenz_analysis_object, method):
    """Benchmarks invariant measure calculation vs. SciPy."""
    traj_xz = lorenz_analysis_object.trajectory[:, [0, 2]]
    epsilon = 0.5
    
    benchmark.group = "Invariant Measure"
    benchmark.name = method

    if method == "DynamiXplore":
        def run_dx_measure():
            lorenz_analysis_object.invariant_measure(epsilon=epsilon, dims=(0, 2))
        benchmark(run_dx_measure)

    elif method == "SciPy":
        def run_scipy_measure():
            x_min, x_max = traj_xz[:, 0].min(), traj_xz[:, 0].max()
            z_min, z_max = traj_xz[:, 1].min(), traj_xz[:, 1].max()
            bins_x = np.arange(x_min, x_max, epsilon)
            bins_z = np.arange(z_min, z_max, epsilon)
            binned_statistic_2d(traj_xz[:, 0], traj_xz[:, 1], None, 'count', bins=[bins_x, bins_z])
        benchmark(run_scipy_measure)
