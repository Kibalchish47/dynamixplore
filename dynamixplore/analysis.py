# This file provides the stateful Analysis class for post-simulation processing.

import numpy as np
import pandas as pd
from typing import Optional, Callable, List, Tuple

# Import the compiled Rust extension module.
# The `dx_core` name is defined in `src/lib.rs`.
from . import dx_core as rust_core

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

    def lyapunov_spectrum(
        self,

        # Don't be lazy here: which argument types, and which return type ?
        dynamics: Callable, 
        transient_time: float = 100.0, 
        run_time: float = 1000.0,
        reortho_time: float = 0.5
    ) -> np.ndarray:
        """
        Computes the full Lyapunov spectrum for the dynamical system.

        This method uses the canonical Benettin et al. algorithm with QR
        re-orthogonalization, implemented in the high-performance Rust core.

        Args:
            dynamics (Callable): The original Python function defining the system's dynamics.
            transient_time (float): The time to run the simulation to allow the
                                    trajectory to settle onto the attractor before measuring.
            run_time (float): The total time over which to average the exponents for convergence.
            reortho_time (float): The time interval between QR re-orthogonalizations.

        Returns:
            np.ndarray: A 1D NumPy array containing the full spectrum of Lyapunov exponents.
        """
        # Use the final state of the stored trajectory as the starting point,
        # assuming it's on the attractor.
        initial_state_on_attractor = self.trajectory[-1, :]
        
        # Placeholder for the actual Rust call. The name will match the function
        # exposed in our `lib.rs`.
        # Note: The Rust function will need its own internal integrator to run.
        lyap_exponents = rust_core.compute_lyapunov_spectrum(
            dynamics,
            initial_state_on_attractor,
            transient_time,
            run_time,
            reortho_time
        )
        return np.array(lyap_exponents)

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
        time_series = self.trajectory[:, dim]
        # Placeholder for the actual Rust call.
        entropy = rust_core.compute_permutation_entropy(time_series, m, tau)
        return entropy

    def to_dataframe(self, column_names: Optional[List[str]] = None) -> pd.DataFrame:
        """
        Converts the trajectory data into a pandas DataFrame.

        This is a convenience method for easier integration with other Python
        data science and plotting libraries.

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
        else:
            # Reconstruct time axis from dt if it exists
            time_axis = np.arange(0, self.n_points * self.dt, self.dt)
            df.insert(0, 'time', time_axis[:self.n_points])
            
        return df
