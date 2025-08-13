# This file contains tests for the analysis functions (Lyapunov, entropy, etc.).

import pytest
import numpy as np
import dx_core

# --- Fixtures defined in conftest.py will be automatically available ---

def test_lyapunov_spectrum_lorenz(lorenz_system_fixture):
    """
    Tests the Lyapunov spectrum calculation against the canonical value for the
    Lorenz system's largest exponent (~0.906).
    """
    initial_state = np.array([1.0, 1.0, 1.0])

    # A shorter run for testing purposes, but long enough for convergence.
    spectrum, history = dx_core.compute_lyapunov_spectrum(
        lorenz_system_fixture,
        initial_state,
        t_transient=10.0,
        t_total=2000.0,
        t_reorth=1.0,
        h_init=0.01,
        abstol=1e-8,
        reltol=1e-8
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
    sine_wave = np.sin(t)
    pe_sine = dx_core.compute_permutation_entropy(sine_wave, m=3, tau=1)
    assert pe_sine < 0.1

    # Unpredictable signal
    random_noise = np.random.rand(1000)
    pe_random = dx_core.compute_permutation_entropy(random_noise, m=3, tau=1)
    assert pe_random > 0.95

def test_approximate_entropy():
    """
    Tests approximate entropy on predictable and random signals.
    - A sine wave is highly regular -> low ApEn.
    - Random noise is highly irregular -> high ApEn.
    """
    np.random.seed(42)
    t = np.linspace(0, 4 * np.pi, 1000)
    
    # Predictable signal
    sine_wave = np.sin(t)
    r_sine = 0.2 * np.std(sine_wave)
    apen_sine = dx_core.compute_approximate_entropy(sine_wave, m=2, r=r_sine)
    assert apen_sine < 0.1

    # Unpredictable signal
    random_noise = np.random.rand(1000)
    r_random = 0.2 * np.std(random_noise)
    apen_random = dx_core.compute_approximate_entropy(random_noise, m=2, r=r_random)
    assert apen_random > 0.5 # ApEn is typically lower than PE

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
    epsilon = 1.0

    histogram = dx_core.compute_invariant_measure(trajectory, epsilon)

    # The keys in the returned dict are tuples of the bin coordinates.
    expected_histogram = {
        (0, 1): 3,
        (2, 0): 1,
    }
    assert histogram == expected_histogram

def test_analysis_error_handling():
    """
    Ensures that analysis functions raise appropriate errors for invalid input.
    """
    # Test permutation entropy with invalid parameters
    with pytest.raises(ValueError, match="Embedding dimension 'm' must be at least 2"):
        dx_core.compute_permutation_entropy(np.array([1.0, 2.0, 3.0]), m=1, tau=1)

    # Test that providing a 1D array to a function expecting 2D fails
    # Note: This will likely raise a TypeError from the numpy crate binding.
    with pytest.raises(TypeError):
        dx_core.compute_invariant_measure(np.array([1.0, 2.0, 3.0]), epsilon=1.0)