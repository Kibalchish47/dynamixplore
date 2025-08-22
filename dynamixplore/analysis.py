import numpy as np
import pandas as pd
from typing import Optional, Callable, List, Tuple

# This relative import is safe because there are no more circular dependencies.
from . import _core as rust_core

class Analysis:
    """
    A class to perform analysis on the trajectory of a dynamical system.
    """
    def __init__(self, trajectory: np.ndarray, t: Optional[np.ndarray] = None, dt: Optional[float] = None):
        if not isinstance(trajectory, np.ndarray) or trajectory.ndim != 2:
            raise ValueError("Trajectory must be a 2D NumPy array.")

        if t is None and dt is None:
            raise ValueError("Time information is required ('t' or 'dt').")

        self.trajectory = trajectory
        self.t = t
        self.dt = dt
        self.n_points, self.n_dims = trajectory.shape

        self._lyapunov_solver = rust_core.Lyapunov()
        self._entropy_solver = rust_core.Entropy()
        self._stats_solver = rust_core.Stats()

    def lyapunov_spectrum(
        self,
        dynamics: Callable,
        t_transient: float = 100.0,
        t_total: float = 1000.0,
        t_reorth: float = 0.5,
        h_init: float = 0.01,
        abstol: float = 1e-6,
        reltol: float = 1e-3
    ) -> Tuple[np.ndarray, np.ndarray]:
        initial_state = np.ascontiguousarray(self.trajectory[-1, :])
        return self._lyapunov_solver.compute_spectrum(
            dynamics, initial_state, t_transient, t_total,
            t_reorth, h_init, abstol, reltol
        )

    def permutation_entropy(self, dim: int = 0, m: int = 3, tau: int = 1) -> float:
        time_series = np.ascontiguousarray(self.trajectory[:, dim])
        return self._entropy_solver.compute_permutation(time_series, m, tau)

    def invariant_measure(
        self,
        epsilon: float,
        dims: Tuple[int, int] = (0, 1)
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        if len(dims) != 2:
            raise ValueError("Invariant measure projection only supports 2D.")

        projected_traj = np.ascontiguousarray(self.trajectory[:, list(dims)])
        hist_dict = self._stats_solver.compute_invariant_measure(projected_traj, epsilon)

        if not hist_dict:
            return np.array([[]]), np.array([]), np.array([])

        coords = np.array(list(hist_dict.keys()))
        counts = np.array(list(hist_dict.values()))

        min_coords = coords.min(axis=0)
        max_coords = coords.max(axis=0)

        grid_shape = (max_coords - min_coords) + 1
        hist = np.zeros(grid_shape, dtype=np.uint64)

        grid_indices = coords - min_coords
        hist[grid_indices[:, 0], grid_indices[:, 1]] = counts

        x_bins = np.arange(min_coords[0], max_coords[0] + 2) * epsilon
        y_bins = np.arange(min_coords[1], max_coords[1] + 2) * epsilon

        return hist, x_bins, y_bins

    def to_dataframe(self, column_names: Optional[List[str]] = None) -> pd.DataFrame:
        if column_names is None:
            column_names = [f'x{i}' for i in range(self.n_dims)]

        if len(column_names) != self.n_dims:
            raise ValueError(f"Expected {self.n_dims} column names, but got {len(column_names)}.")

        df = pd.DataFrame(self.trajectory, columns=column_names)
        if self.t is not None:
            df.insert(0, 'time', self.t)
        elif self.dt is not None:
            time_axis = np.arange(0, self.n_points * self.dt, self.dt)
            df.insert(0, 'time', time_axis[:self.n_points])

        return df
