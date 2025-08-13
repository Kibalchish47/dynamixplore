# In tests/conftest.py
# This file defines shared fixtures for the test suite.

import pytest
import numpy as np

@pytest.fixture(scope="session")
def lorenz_system_fixture():
    """
    Provides a fixture for the classic Lorenz system dynamics.
    `scope="session"` means this function is only run once for the entire
    test session, making it very efficient.
    """
    def lorenz_system(t, state):
        sigma = 10.0
        rho = 28.0
        beta = 8.0 / 3.0
        
        x, y, z = state
        dx_dt = sigma * (y - x)
        dy_dt = x * (rho - z) - y
        dz_dt = x * y - beta * z
        return np.array([dx_dt, dy_dt, dz_dt])
        
    return lorenz_system

