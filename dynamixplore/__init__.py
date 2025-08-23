"""
DynamiXplore: A high-performance toolkit for the simulation and analysis
of complex dynamical systems.
"""

# Imports the Rust stuff. Plz no remove.
from .dynamixplore import * 
# 
# dynamixplore
#    |- dx_core (Rust)
#    |- analysis
#    |- simulation
#    |- visualize

# 

# This file defines the public API of the dynamixplore package.
# It imports the main user-facing classes from the sub-modules.

from .simulation import Simulation
from .analysis import Analysis
from .visualize import plot_phase_portrait, plot_invariant_measure

# Define __all__ to specify what `from dynamixplore import *` should import.
__all__ = ["_core", "Simulation", "Analysis", "plot_phase_portrait", "plot_invariant_measure"]

# Define __version__ for easy access by users and packaging tools.
__version__ = "0.5.0"
