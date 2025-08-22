from __future__ import annotations
from typing import Callable, List, Tuple, Union
import numpy as np
from . import dx_rust as rust_core
from .analysis import Analysis

class Simulation:
    """
    Configures and executes a numerical simulation of a dynamical system.
    Provides a high-level interface to the high-performance Rust core.
    """
    def __init__(self,
                 dynamics_func: Callable[[float, np.ndarray], Union[List[float], np.ndarray]],
                 initial_state: Union[List[float], np.ndarray],
                 t_span: Tuple[float, float],
                 dt: float):
        """
        Initializes and validates the simulation parameters.

        Args:
            dynamics_func (Callable): The function f(t, y) defining the system's dynamics.
            initial_state (Union[List[float], np.ndarray]): The starting state vector.
            t_span (Tuple[float, float]): A tuple (t_start, t_end) for the simulation.
            dt (float): The time step for fixed-step solvers or the initial step for adaptive ones.
        """
        if not callable(dynamics_func):
            raise TypeError("The dynamics function must be callable.")
        self.dynamics_func = dynamics_func

        initial_state_np = np.asarray(initial_state, dtype=np.float64)
        if initial_state_np.ndim != 1:
            raise ValueError("The initial state must be a 1D array.")
        self.initial_state = initial_state_np

        if not isinstance(t_span, tuple) or len(t_span) != 2:
            raise ValueError("The time span 't_span' must be a tuple of (t_start, t_end).")
        t_start, t_end = t_span
        if t_end <= t_start:
            raise ValueError("The end time must be greater than the start time.")
        self.t_span = t_span

        if not isinstance(dt, (int, float)) or dt <= 0:
            raise ValueError("The time step 'dt' must be a positive number.")
        self.dt = float(dt)

    def run(self, solver: str = 'RK45', mode: str = 'Adaptive', **kwargs) -> Analysis:
        """
        Runs the simulation using the configured parameters and a specified solver.

        Args:
            solver (str): The integration algorithm to use. Supported: 'RK45', 'RK4', 'Euler'.
            mode (str): The integration mode. Supported: 'Adaptive', 'Explicit', 'Implicit'.
            **kwargs: Additional keyword arguments for the solver mode (e.g., abstol, reltol).

        Returns:
            Analysis: An Analysis object containing the resulting trajectory.
        """
        # --- 1. Instantiate the correct Rust solver object ---
        solver_map = {
            'RK45': rust_core.Rk45,
            'RK4': rust_core.Rk4,
            'Euler': rust_core.Euler
        }
        solver_class = solver_map.get(solver)
        if solver_class is None:
            raise ValueError(f"Solver '{solver}' not supported. Use one of {list(solver_map.keys())}")

        rust_solver = solver_class()

        # --- 2. Create the correct parameter "mode" object ---
        t_start, t_end = self.t_span
        params = {
            "dynamics": self.dynamics_func,
            "initial_state": self.initial_state,
            "t_start": t_start,
            "t_end": t_end,
            "h": self.dt
        }

        if mode == 'Adaptive':
            if solver != 'RK45':
                raise ValueError("Adaptive mode is only compatible with the RK45 solver.")
            params.update({
                "abstol": kwargs.get('abstol', 1e-6),
                "reltol": kwargs.get('reltol', 1e-3)
            })
            mode_obj = rust_core.Adaptive(**params)
        elif mode == 'Explicit':
            mode_obj = rust_core.Explicit(**params)
        elif mode == 'Implicit':
            mode_obj = rust_core.Implicit(**params)
        else:
            raise ValueError(f"Mode '{mode}' not supported. Use 'Adaptive', 'Explicit', or 'Implicit'.")

        # --- 3. Call the solve method and wrap the results ---
        result = rust_solver.solve(mode_obj)

        if mode == 'Adaptive':
            trajectory, times = result
            return Analysis(trajectory=trajectory, t=times)
        else: # Explicit or Implicit
            trajectory = result
            return Analysis(trajectory=trajectory, dt=self.dt)
