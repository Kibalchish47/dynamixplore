# This file provides high-level, easy-to-use plotting functions.

import numpy as np
import plotly.graph_objects as go
from typing import Optional, List, Tuple

def plot_phase_portrait(
    trajectory: np.ndarray,
    dims: Tuple[int, ...] = (0, 1, 2),
    title: str = "Phase Portrait"
) -> go.Figure:
    """
    Creates an interactive 2D or 3D phase portrait of the trajectory.

    Args:
        trajectory (np.ndarray): A 2D NumPy array of shape (n_points, n_dims).
        dims (Tuple[int, ...]): A tuple of 2 or 3 integers specifying the
                                dimensions (columns) to plot.
        title (str): The title for the plot.

    Returns:
        go.Figure: An interactive Plotly figure object.
    """
    if len(dims) not in [2, 3]:
        raise ValueError(f"Phase portrait can only be 2D or 3D, but got {len(dims)} dimensions.")

    if max(dims) >= trajectory.shape[1]:
        raise ValueError(f"Invalid dimension index in {dims}. Trajectory only has {trajectory.shape[1]} dimensions.")

    fig = go.Figure()

    if len(dims) == 2:
        fig.add_trace(go.Scatter(
            x=trajectory[:, dims[0]],
            y=trajectory[:, dims[1]],
            mode='lines',
            line=dict(width=1.5, color='royalblue')
        ))
        fig.update_layout(
            xaxis_title=f"Dimension {dims[0]}",
            yaxis_title=f"Dimension {dims[1]}"
        )
    else: # 3D case
        fig.add_trace(go.Scatter3d(
            x=trajectory[:, dims[0]],
            y=trajectory[:, dims[1]],
            z=trajectory[:, dims[2]],
            mode='lines',
            line=dict(width=1, color='crimson')
        ))
        fig.update_layout(scene=dict(
            xaxis_title=f"Dimension {dims[0]}",
            yaxis_title=f"Dimension {dims[1]}",
            zaxis_title=f"Dimension {dims[2]}"
        ))

    fig.update_layout(
        title=title,
        margin=dict(l=0, r=0, b=0, t=40) # Compact layout
    )
    return fig

def plot_invariant_measure(
    histogram: np.ndarray,
    x_bins: np.ndarray,
    y_bins: np.ndarray,
    title: str = "Invariant Measure Projection"
) -> go.Figure:
    """
    Visualizes a 2D projection of the invariant measure as a heatmap.

    Args:
        histogram (np.ndarray): A 2D NumPy array of counts from Analysis.invariant_measure.
        x_bins (np.ndarray): A 1D array of the bin edges for the x-axis.
        y_bins (np.ndarray): A 1D array of the bin edges for the y-axis.
        title (str): The title for the plot.

    Returns:
        go.Figure: An interactive Plotly figure object.
    """
    fig = go.Figure(data=go.Heatmap(
        z=histogram.T, # Transpose to match standard (x, y) orientation
        x=x_bins,
        y=y_bins,
        colorscale='Viridis',
        colorbar=dict(title='Density')
    ))

    fig.update_layout(
        title=title,
        xaxis_title="State Variable X",
        yaxis_title="State Variable Y"
    )
    return fig
