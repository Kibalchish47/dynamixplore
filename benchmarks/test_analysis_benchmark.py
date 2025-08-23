import pytest
import numpy as np
import dynamixplore as dx
import nolds
from scipy.stats import binned_statistic_2d
from .systems import lorenz_system

# --- Fixture to generate a standard trajectory for all analysis tests ---
@pytest.fixture(scope="module")
def lorenz_trajectory():
    """Pre-computes a Lorenz trajectory so all analysis tests run on the same data."""
    print("\nGenerating trajectory for analysis benchmarks...")
    sim = dx.Simulation(
        dynamics_func=lorenz_system,
        initial_state=[1.0, 1.0, 1.0],
        t_span=(0.0, 200.0),
        dt=0.01
    )
    # Use a fixed-step solver for consistent data points
    analysis_obj = sim.run(solver='RK4', mode='Explicit')
    return analysis_obj

# --- Benchmark Functions ---

def test_lyapunov_benchmark(benchmark, lorenz_trajectory):
    """Benchmarks Lyapunov spectrum calculation vs. nolds."""
    data = lorenz_trajectory.trajectory
    
    # Benchmark DynamiXplore
    def run_dx_lyapunov():
        lorenz_trajectory.lyapunov_spectrum(
            dynamics=lorenz_system,
            t_transient=10.0,
            t_total=1000.0,
            t_reorth=1.0
        )
    
    # Benchmark nolds (for largest exponent only)
    def run_nolds_lyapunov():
        nolds.lyap_r(data, lag=10, min_tsep=200)

    benchmark.group = "Lyapunov Spectrum"
    benchmark.name = "DynamiXplore (Full Spectrum)"
    benchmark(run_dx_lyapunov)
    
    benchmark.name = "nolds (Largest Exponent)"
    benchmark(run_nolds_lyapunov)

def test_entropy_benchmark(benchmark, lorenz_trajectory):
    """Benchmarks permutation entropy."""
    x_series = lorenz_trajectory.trajectory[:, 0]
    
    # Benchmark DynamiXplore
    def run_dx_entropy():
        lorenz_trajectory.permutation_entropy(dim=0, m=3, tau=1)
        
    benchmark.group = "Permutation Entropy"
    benchmark.name = "DynamiXplore"
    benchmark(run_dx_entropy)
    
    # Note: nolds does not have a direct permutation entropy function.
    # A pure Python comparison would be unfairly slow.

def test_invariant_measure_benchmark(benchmark, lorenz_trajectory):
    """Benchmarks invariant measure calculation vs. SciPy."""
    traj_xz = lorenz_trajectory.trajectory[:, [0, 2]]
    epsilon = 0.5
    
    # Benchmark DynamiXplore
    def run_dx_measure():
        lorenz_trajectory.invariant_measure(epsilon=epsilon, dims=(0, 2))

    # Benchmark SciPy
    def run_scipy_measure():
        x_min, x_max = traj_xz[:, 0].min(), traj_xz[:, 0].max()
        z_min, z_max = traj_xz[:, 1].min(), traj_xz[:, 1].max()
        bins_x = np.arange(x_min, x_max, epsilon)
        bins_z = np.arange(z_min, z_max, epsilon)
        binned_statistic_2d(traj_xz[:, 0], traj_xz[:, 1], None, 'count', bins=[bins_x, bins_z])

    benchmark.group = "Invariant Measure"
    benchmark.name = "DynamiXplore"
    benchmark(run_dx_measure)
    
    benchmark.name = "SciPy"
    benchmark(run_scipy_measure)
