import numpy as np
import dynamixplore as dx

def lorenz_system(t: float, state: np.ndarray) -> np.ndarray:
    sigma = 10.0
    rho = 28.0
    beta = 8.0 / 3.0
    x, y, z = state
    dx_dt = sigma * (y - x)
    dy_dt = x * (rho - z) - y
    dz_dt = x * y - beta * z
    return np.array([dx_dt, dy_dt, dz_dt])

def main():
    sim = dx.Simulation(
        dynamics_func=lorenz_system,
        initial_state=[1.0, 1.0, 1.0],
        t_span=(0.0, 50.0),
        dt=0.01
    )

    print("🚀 Running adaptive RK45 simulation...")
    # FIX: `sim.run()` now returns the raw data.
    trajectory, times = sim.run(solver='RK45', mode='Adaptive', abstol=1e-8, reltol=1e-8)

    # FIX: Create the Analysis object manually with the simulation results.
    analysis_obj = dx.Analysis(trajectory=trajectory, t=times)
    print(f"✅ Simulation complete. Generated {analysis_obj.n_points} points.")

    print("\n🧠 Computing Lyapunov spectrum...")
    spectrum, history = analysis_obj.lyapunov_spectrum(
        dynamics=lorenz_system,
        t_transient=10.0,
        t_total=1000.0,
        t_reorth=1.0
    )
    print(f"✅ Lyapunov spectrum computed: {np.round(spectrum, 4)}")
    ks_entropy_est = np.sum([le for le in spectrum if le > 0])
    print(f"📈 Estimated KS-Entropy from spectrum: {ks_entropy_est:.4f}")

    print("\n📊 Computing entropy and statistical measures...")
    perm_entropy = analysis_obj.permutation_entropy(dim=0, m=3, tau=1)
    print(f"✅ Permutation Entropy (m=3, tau=1): {perm_entropy:.4f}")

    hist, x_bins, y_bins = analysis_obj.invariant_measure(epsilon=0.5, dims=(0, 2))
    print(f"✅ Invariant measure computed. Found {np.count_nonzero(hist)} populated bins.")

if __name__ == "__main__":
    main()
