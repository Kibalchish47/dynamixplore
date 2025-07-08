# To make our code compatible with older versions of Python, we import annotations.
# This allows us to use type hints like 'Analysis' before the class is fully defined,
# which is crucial for the fluent interface in our `run` method.
from __future__ import annotations

# We import the necessary tools from Python's 'typing' module.
# This doesn't change how the code runs, but it provides "hints" to developers
# and tools like IDEs about what kind of data a function expects and returns.
# - Callable: Represents something that can be called, like a function.
# - List, Tuple: Represents Python's list and tuple data structures.
# - Union: Allows a variable to be one of several types (e.g., a list OR a NumPy array).
from typing import Callable, List, Tuple, Union

# NumPy is the cornerstone of scientific computing in Python. We import it
# and give it the standard alias 'np'. We will use it to handle all numerical
# arrays, as it's far more efficient than standard Python lists for math.
import numpy as np

# --- The Bridge and the Result ---
# This is where we connect our Python layer to the other parts of the project.

# We import the 'rust_core' module. This is the compiled Rust extension module
# that `maturin` creates. It now contains our expanded set of solver functions
# like `solve_rk45_adaptive`, `solve_rk4_explicit`, etc.
from . import rust_core

# We import the 'Analysis' class from our own 'analysis.py' file.
# The `run` method will wrap its results in this class to create a smooth,
# "fluent" user experience, allowing for method chaining like `sim.run().plot()`.
from .analysis import Analysis

# We define our main user-facing class. The name 'Simulation' clearly
# communicates its purpose.
class Simulation:
    """
    Configures and executes a numerical simulation of a dynamical system.

    This class acts as a high-level "Conductor" for the powerful Rust core engine.
    Its primary responsibilities are to:
    1. Provide a clean, intuitive, and user-friendly API.
    2. Rigorously validate all user inputs to prevent errors.
    3. Orchestrate the call to the correct high-performance Rust function.
    4. Return the results in a convenient, analysis-ready format.
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
        Initializes and validates the simulation parameters.

        This is the "Gatekeeper" of the simulation. It ensures that all inputs
        are valid and in a standard format before any computation begins.

        Args:
            dynamics_func (Callable): The function defining the system's dynamics,
                f(t, y). It must accept the current time `t` (a float) and the
                current state `y` (a 1D NumPy array) as input. It must return
                the derivatives (dy/dt) as a list of floats or a 1D NumPy array.
            initial_state (Union[List[float], np.ndarray]): The starting state
                vector of the system. Can be provided as a simple Python list or
                a NumPy array.
            t_span (Tuple[float, float]): A tuple `(t_start, t_end)` specifying
                the time interval for the simulation.
            dt (float): The time step for fixed-step solvers. For adaptive
                solvers, this is used as the *initial* time step.
            solver_options (dict, optional): A dictionary for advanced solver
                parameters, like 'abstol' and 'reltol' for adaptive solvers.
                Defaults to None.
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
        Runs the simulation using the configured parameters.

        This method acts as the "Orchestrator". It selects the correct backend
        solver, packages the arguments, and makes the call to the Rust core.
        It then wraps the raw numerical result into a user-friendly `Analysis`
        object.

        Args:
            solver (str): The name of the ODE solver to use.
                          Supported: 'rk45_adaptive', 'rk4_explicit', 'euler_explicit',
                          'rk4_implicit', 'euler_implicit'.
                          Defaults to 'rk45_adaptive'.

        Returns:
            Analysis: An Analysis object containing the resulting trajectory and
                      time points, ready for further processing and visualization.
        """
        # --- Backend Selection (Dispatcher) ---
        # Instead of a long `if/elif/else` chain, we use a dictionary to map the
        # user-friendly solver string to the actual Rust function. This is a
        # cleaner, more scalable design.
        solver_map = {
            'rk45_adaptive': rust_core.solve_rk45_adaptive,
            'rk4_explicit': rust_core.solve_rk4_explicit,
            'euler_explicit': rust_core.solve_euler_explicit,
            'rk4_implicit': rust_core.solve_rk4_implicit,
            'euler_implicit': rust_core.solve_euler_implicit
        }

        # We check if the user's requested solver is available.
        if solver not in solver_map:
            # If not, we raise an error with a helpful message listing all
            # available options, which we get directly from the dictionary keys.
            supported_solvers = list(solver_map.keys())
            raise ValueError(
                f"Solver '{solver}' is not supported."
                f"Please use one of: {supported_solvers}"
            )

        # We retrieve the correct Rust solver function from our map.
        solver_func = solver_map[solver]

        # --- Argument Preparation and The Bridge Call to Rust ---
        # We pack the arguments common to all solvers into a tuple.
        # This avoids repetition.
        common_args = (
            self.dynamics_func,
            self.initial_state,
            self.t_span[0],
            self.t_span[1],
            self.dt
        )

        # We check if the selected solver is the adaptive one, as it requires
        # special, additional arguments.
        if solver == 'rk45_adaptive':
            # We use the `.get()` method on the `solver_options` dictionary.
            # This is safer than direct access (`self.solver_options['abstol']`),
            # because it lets us provide a default value if the key is missing.
            # These defaults match the defaults in the Rust function signature.
            abstol = self.solver_options.get('abstol', 1e-6)
            reltol = self.solver_options.get('reltol', 1e-6)

            # We call the adaptive solver, passing the common arguments followed
            # by the specific tolerance keyword arguments.
            # Note: Since the Rust function returns (trajectory, times),
            # The order of `trajectory` and `times` here should match.
            trajectory, times = solver_func(*common_args, abstol=abstol, reltol=reltol)
        else:
            # For all other solvers (fixed-step explicit and the implicit placeholders),
            # we call the function using only the common arguments. The `*` operator
            # "unpacks" the `common_args` tuple into individual arguments for the function.
            trajectory, times = solver_func(*common_args)

        # --- Wrapping the Result ---
        # The Rust core has finished and returned the results. We now instantiate
        # our `Analysis` class. Because we use keyword arguments (`trajectory=...`),
        # the order doesn't matter here, making the code robust and readable.
        return Analysis(trajectory=trajectory, times=times)
