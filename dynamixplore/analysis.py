# This file provides the stateful Analysis class for post-simulation processing.

import numpy as np
import pandas as pd
from typing import Optional, Callable, List, Tuple

# Import the compiled Rust extension module.
# The `dx_rust` name is defined in `Cargo.toml` and `src/lib.rs`.
from . import dx_rust as rust_core

class Analysis:
    """
    A class to perform analysis on the trajectory of a dynamical system.

    This object is stateful, holding the trajectory data to allow for multiple
    different analyses to be performed on the same simulation results.
    """
    def __init__(self, trajectory: np.ndarray, t: Optional[np.ndarray] = None, dt: Optional[float] = None):
        """
        Initializes the Analysis object.

        Args:
            trajectory (np.ndarray): A 2D NumPy array of shape (n_points, n_dims)
                                    representing the system's state over time.
            t (Optional[np.ndarray]): A 1D NumPy array of time points, required for
                                      trajectories with non-uniform time steps (e.g., from
                                      an adaptive solver).
            dt (Optional[float]): The time step between points, required for
                                  trajectories with a fixed time step.

        Raises:
            ValueError: If the trajectory is not a 2D NumPy array or if time
                        information (either `t` or `dt`) is missing.
        """
        if not isinstance(trajectory, np.ndarray) or trajectory.ndim != 2:
            raise ValueError("Trajectory must be a 2D NumPy array.")

        if t is None and dt is None:
            raise ValueError("Time information is required. Please provide either 't' (for adaptive steps) or 'dt' (for fixed steps).")

        self.trajectory = trajectory
        self.t = t
        self.dt = dt
        self.n_points, self.n_dims = trajectory.shape

        # Instantiate the Rust analysis tool objects once.
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
        """
        Computes the full Lyapunov spectrum for the dynamical system.

        This method uses the canonical Benettin et al. algorithm with QR
        re-orthogonalization, implemented in the high-performance Rust core.

        Args:
            dynamics (Callable): The original Python function defining the system's dynamics.
            t_transient (float): Time to run to let the trajectory settle onto the attractor.
            t_total (float): Total time over which to average the exponents for convergence.
            t_reorth (float): Time interval between QR re-orthogonalizations.
            h_init (float): Initial step size for the internal adaptive solver.
            abstol (float): Absolute tolerance for the internal adaptive solver.
            reltol (float): Relative tolerance for the internal adaptive solver.

        Returns:
            Tuple[np.ndarray, np.ndarray]: A tuple containing:
                - The final converged Lyapunov spectrum (1D array).
                - The history of the spectrum's convergence over time (2D array).
        """
        initial_state_on_attractor = np.ascontiguousarray(self.trajectory[-1, :])

        spectrum, history = self._lyapunov_solver.compute_spectrum(
            dynamics,
            initial_state_on_attractor,
            t_transient,
            t_total,
            t_reorth,
            h_init,
            abstol,
            reltol
        )
        return spectrum, history

    def permutation_entropy(self, dim: int = 0, m: int = 3, tau: int = 1) -> float:
        """
        Computes the permutation entropy for a single dimension of the trajectory.

        Args:
            dim (int): The index of the dimension (column) of the trajectory to analyze.
            m (int): The embedding dimension (typically 3 to 7).
            tau (int): The time delay for phase space reconstruction.

        Returns:
            float: The normalized permutation entropy, a value between 0 and 1.
        """
        time_series = np.ascontiguousarray(self.trajectory[:, dim])
        return self._entropy_solver.compute_permutation(time_series, m, tau)

    def invariant_measure(
        self,
        epsilon: float,
        dims: Tuple[int, int] = (0, 1)
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Approximates the invariant measure on a 2D projection of the attractor.

        Args:
            epsilon (float): The side length of the hypercubes (bins) for box-counting.
            dims (Tuple[int, int]): A tuple of two integers specifying the dimensions
                                    to project the trajectory onto.

        Returns:
            Tuple[np.ndarray, np.ndarray, np.ndarray]: A tuple containing:
                - A 2D NumPy array representing the histogram of visit frequencies.
                - A 1D NumPy array of the bin edges for the x-axis.
                - A 1D NumPy array of the bin edges for the y-axis.
        """
        if len(dims) != 2:
            raise ValueError("Invariant measure projection currently only supports 2 dimensions.")

        projected_traj = np.ascontiguousarray(self.trajectory[:, list(dims)])

        # The Rust function returns a dictionary of {(x_bin, y_bin): count}
        histogram_dict = self._stats_solver.compute_invariant_measure(projected_traj, epsilon)

        if not histogram_dict:
            return np.array([[]]), np.array([]), np.array([])

        # Convert the dictionary from Rust into a plottable 2D NumPy array
        coords = np.array(list(histogram_dict.keys()))
        counts = np.array(list(histogram_dict.values()))

        min_coords = coords.min(axis=0)
        max_coords = coords.max(axis=0)

        grid_shape = (max_coords - min_coords) + 1
        histogram = np.zeros(grid_shape, dtype=np.uint64)

        # Map dictionary coordinates to grid indices
        grid_indices = coords - min_coords
        histogram[grid_indices[:, 0], grid_indices[:, 1]] = counts

        # Create bin edges for plotting
        x_bins = np.arange(min_coords[0], max_coords[0] + 2) * epsilon
        y_bins = np.arange(min_coords[1], max_coords[1] + 2) * epsilon

        return histogram, x_bins, y_bins


    def to_dataframe(self, column_names: Optional[List[str]] = None) -> pd.DataFrame:
        """
        Converts the trajectory data into a pandas DataFrame.

        Args:
            column_names (Optional[List[str]]): A list of names for the columns.
                                                If None, defaults like ['x0', 'x1', ...]
                                                will be generated.

        Returns:
            pd.DataFrame: A pandas DataFrame containing the trajectory data.
        """
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
