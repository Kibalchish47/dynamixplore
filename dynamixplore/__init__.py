# src/dynamixplore/__init__.py

"""
DynamiXplore: A high-performance toolkit for the simulation and analysis
of complex dynamical systems.
"""

from .simulation import Simulation
from .analysis import Analysis
from .visualize import plot_phase_portrait, plot_invariant_measure

# Optional: Define __version__ for easy access
__version__ = "0.4.0"
