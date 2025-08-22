from __future__ import annotations
from typing import Callable, List, Tuple, Union
import numpy as np

# This relative import is now safe because the circular dependency
# on the Analysis class has been removed.
from dynamixplore import _core as rust_core

class Simulation:
    """
    Configures and executes a numerical simulation of a dynamical system.
    """
    def __init__(self,
                 dynamics_func: Callable[[float, np.ndarray], Union[List[float], np.ndarray]],
                 initial_state: Union[List[float], np.ndarray],
                 t_span: Tuple[float, float],
                 dt: float):
        """
        Initializes and validates the simulation parameters.
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

    def run(self, solver: str = 'RK45', mode: str = 'Adaptive', **kwargs) -> Union[np.ndarray, Tuple[np.ndarray, np.ndarray]]:
        """
        Runs the simulation and returns the raw trajectory data.

        Returns:
            - For adaptive solvers: A tuple of (trajectory, times).
            - For fixed-step solvers: The trajectory array.
        """
        solver_map = {
            'RK45': rust_core.Rk45,
            'RK4': rust_core.Rk4,
            'Euler': rust_core.Euler
        }
        solver_class = solver_map.get(solver)
        if solver_class is None:
            raise ValueError(f"Solver '{solver}' not supported. Use one of {list(solver_map.keys())}")

        rust_solver = solver_class()

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
            raise ValueError(f"Mode '{mode}' not supported.")

        # FIX: Return the raw result directly instead of an Analysis object.
        result = rust_solver.solve(mode_obj)
        return result
