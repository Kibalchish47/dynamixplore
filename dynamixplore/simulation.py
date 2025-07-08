from __future__ import annotations

import numpy as np
from typing import Callable, List, Tuple

# --- The Bridge and the Result ---
from . import rust_core

from .analysis import Analysis

class Simulation:
    """
    """

    # The __init__ method is the constructor for the class. It's called when a
    # user writes `sim = Simulation(...)`. Its job is to receive all the
    # parameters for the experiment, validate them, and store them.
    def __init__(self,
                dynamics_func: Callable[[float, np.ndarray], List[float]],
                initial_state: List[float] | np.ndarray,
                t_span: Tuple[float, float],
                dt: float,
                solver_options: dict = None):
        """
        """
        # --- Input Validation and Standardization ---
        # 1. Validate `dynamics_func`
        if not callable(dynamics_func):
            raise TypeError("The dynamics function must be callable.")
        self.dynamics_func = dynamics_func

        # 2. Validate and standardize `initial_state`
        initial_state_np = np.asarray(initial_state, dtype=np.float64)
        if initial_state_np.ndim != 1:
            raise ValueError("The initial state must be a 1D array.")
        self.initial_state = initial_state_np

        # 3. Validate `t_span`
        if not isinstance(t_span, tuple) or len(t_span) != 2:
            raise ValueError("The time span 't_span' must be a tuple of length 2.")
        t_start, t_end = t_span
        if t_end <= t_start:
            raise ValueError("The end time must be greater than the start time.")
        self.t_span = t_span

        # 4. Validate `dt`
        if not isinstance(dt, (int, float)) or dt <= 0:
            raise ValueError("The time step 'dt' must be a positive number.")
        self.dt = float(dt)

        # 5. Handle `solver_options`
        if solver_options is not None and not isinstance(solver_options, dict):
            raise TypeError("The solver options must be a dictionary.")
        # We store the provided dictionary or an empty one if None was given.
        # This makes it safe to access later.
        self.solver_options = solver_options or {}

    # The `run` method is where the action happens. It's designed to be simple
    # for the user to call: `sim.run()`.
    def run(self, solver: str = 'RK45') -> Analysis:
        """
        """
        pass
