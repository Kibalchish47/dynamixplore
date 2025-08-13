User Guide
==========

This guide will walk you through the core workflow of `DynamiXplore`, from defining your first dynamical system to performing a comprehensive analysis of its behavior.

Defining a System
-----------------

The first step is to define your system's dynamics. This is done by creating a simple Python function that takes the current time `t` and state vector `state` as inputs and returns the vector of derivatives (the rate of change for each state variable).

Let's define the classic 3D Lorenz system:

.. code-block:: python

   import numpy as np

   def lorenz_system(t, state):
       # Unpack the state vector for clarity
       x, y, z = state

       # Lorenz system parameters
       sigma = 10.0
       rho = 28.0
       beta = 8.0 / 3.0

       # The system's differential equations
       dx_dt = sigma * (y - x)
       dy_dt = x * (rho - z) - y
       dz_dt = x * y - beta * z

       return np.array([dx_dt, dy_dt, dz_dt])

Running a Simulation
--------------------

With the dynamics defined, you can now run a numerical simulation. The `dx_core` module provides direct access to the high-performance solvers. For this example, we'll use the adaptive Runge-Kutta 4-5 solver.

.. code-block:: python

   import dx_core
   import numpy as np

   # Define the initial state [x, y, z]
   initial_state = np.array([1.0, 1.0, 1.0])

   # Run the simulation from t=0 to t=50
   trajectory, times = dx_core.solve_rk45_adaptive(
       lorenz_system,
       initial_state,
       t_start=0.0,
       t_end=50.0,
       h=0.01, # Initial step size
       abstol=1e-8,
       reltol=1e-8
   )

   print(f"Simulation finished! Trajectory shape: {trajectory.shape}")
   print(f"Number of time steps: {len(times)}")


The solver returns two NumPy arrays: `trajectory`, a 2D array where each row is a state vector, and `times`, a 1D array of the corresponding time points.

Analyzing the Results
---------------------

Once you have a trajectory, you can use `DynamiXplore`'s analysis functions to quantify its properties.

Computing the Lyapunov Spectrum
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The Lyapunov spectrum is a key indicator of chaos. A positive exponent signifies chaotic behavior.

.. code-block:: python

   # We need the initial_state from the previous step
   lyapunov_spectrum, history = dx_core.compute_lyapunov_spectrum(
       lorenz_system,
       initial_state,
       t_transient=10.0,
       t_total=1000.0,
       t_reorth=1.0,
       h_init=0.01,
       abstol=1e-8,
       reltol=1e-8
   )

   print(f"Lyapunov Spectrum: {lyapunov_spectrum}")

Measuring Complexity with Entropy
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Permutation Entropy provides a robust measure of the complexity of a time series. Let's analyze the x-component of our trajectory.

.. code-block:: python

   # Extract the x-component (the first column)
   x_series = trajectory[:, 0]

   pe = dx_core.compute_permutation_entropy(x_series, m=3, tau=1)

   print(f"Permutation Entropy of the x-series: {pe:.4f}")

This workflow forms the foundation for any analysis in `DynamiXplore`. From here, you can explore other analysis functions, visualize the results, and begin your own explorations.
