# This file contains tests for the analysis functions (Lyapunov, entropy, etc.).

import pytest
import numpy as np
import dynamixplore as dx

# --- Fixtures defined in conftest.py will be automatically available ---

def test_lyapunov_spectrum_lorenz(lorenz_system_fixture):
    """
    Tests the Lyapunov spectrum calculation against the canonical value for the
    Lorenz system's largest exponent (~0.906).
    """
    # Step 1: Run a simulation to get a trajectory on the attractor.
    sim = dx.Simulation(
        dynamics_func=lorenz_system_fixture,
        initial_state=[1.0, 1.0, 1.0],
        t_span=(0.0, 100.0), # A reasonably long run to get onto the attractor
        dt=0.01
    )
    analysis_obj = sim.run(solver='RK45', mode='Adaptive')

    # Step 2: Call the analysis method on the resulting object.
    # A shorter run for testing purposes, but long enough for convergence.
    spectrum, history = analysis_obj.lyapunov_spectrum(
        dynamics=lorenz_system_fixture,
        t_transient=10.0,
        t_total=2000.0,
        t_reorth=1.0
    )

    # Assert that the largest exponent is close to the known value.
    largest_exponent = spectrum[0]
    assert largest_exponent == pytest.approx(0.906, abs=0.1)

    # Assert that the sum of exponents is negative (for a dissipative system).
    assert np.sum(spectrum) < 0

def test_permutation_entropy():
    """
    Tests permutation entropy on predictable and random signals.
    - A sine wave is highly predictable -> low entropy.
    - Random noise is highly unpredictable -> high entropy.
    """
    np.random.seed(42)
    t = np.linspace(0, 4 * np.pi, 1000)
    
    # Predictable signal
    sine_wave = np.sin(t).reshape(-1, 1) # Reshape to (n_points, n_dims)
    analysis_sine = dx.Analysis(trajectory=sine_wave, dt=0.1)
    pe_sine = analysis_sine.permutation_entropy(dim=0, m=3, tau=1)
    assert pe_sine < 0.1

    # Unpredictable signal
    random_noise = np.random.rand(1000).reshape(-1, 1)
    analysis_random = dx.Analysis(trajectory=random_noise, dt=0.1)
    pe_random = analysis_random.permutation_entropy(dim=0, m=3, tau=1)
    assert pe_random > 0.95

def test_invariant_measure():
    """
    Tests that the invariant measure correctly bins a simple trajectory.
    """
    # A simple 2D trajectory that visits specific bins.
    trajectory = np.array([
        [0.1, 1.1],  # Bin (0, 1)
        [0.2, 1.2],  # Bin (0, 1)
        [2.3, 0.4],  # Bin (2, 0)
        [0.8, 1.9],  # Bin (0, 1)
    ])
    
    analysis_obj = dx.Analysis(trajectory=trajectory, dt=1.0)
    hist, x_bins, y_bins = analysis_obj.invariant_measure(epsilon=1.0, dims=(0, 1))

    # The method now returns a 2D numpy array, not a dict.
    # We need to build the expected array.
    # Bins are (0,0), (0,1), (1,0), (1,1), (2,0), (2,1)
    # Coords are (0,1) -> 3 times, (2,0) -> 1 time
    expected_hist = np.array([
        [0., 3.], # x=0, y=0,1
        [0., 0.], # x=1, y=0,1
        [1., 0.]  # x=2, y=0,1
    ])
    
    assert np.array_equal(hist, expected_hist)

def test_analysis_error_handling():
    """
    Ensures that analysis functions raise appropriate errors for invalid input.
    """
    # Create a dummy analysis object for testing method calls
    analysis_obj = dx.Analysis(trajectory=np.random.rand(100, 2), dt=0.1)

    # Test permutation entropy with invalid parameters
    with pytest.raises(ValueError, match="Embedding dimension 'm' must be at least 2"):
        analysis_obj.permutation_entropy(m=1, tau=1)

    # Test that providing a 1D array to the Analysis constructor fails
    with pytest.raises(ValueError, match="Trajectory must be a 2D NumPy array"):
        dx.Analysis(trajectory=np.array([1.0, 2.0, 3.0]), dt=0.1)
