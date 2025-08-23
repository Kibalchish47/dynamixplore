import json
from collections import defaultdict
from datetime import datetime

INPUT_JSON = "benchmark_output.json"
OUTPUT_TXT = "benchmark_results.txt"

def analyze_and_write_report():
    """
    Parses the JSON output from pytest-benchmark, calculates speedups,
    and writes a detailed analysis to a text file.
    """
    try:
        with open(INPUT_JSON, 'r') as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"Error: The input file '{INPUT_JSON}' was not found.")
        print("Please run the benchmarks first using 'py -m pytest benchmarks/ --benchmark-json=benchmark_output.json'")
        return

    benchmarks = data.get("benchmarks", [])
    grouped_results = defaultdict(dict)

    # --- 1. Parse and group all benchmark results ---
    for bench in benchmarks:
        group = bench.get("group")
        if not group:
            continue
        
        # Clean up potential encoding issues in group names
        group = group.replace("Rssler", "Rössler")
        
        name = bench.get("name")
        mean_time = bench.get("stats", {}).get("mean", 0)
        grouped_results[group][name] = mean_time

    # --- 2. Write the formatted report ---
    with open(OUTPUT_TXT, "w") as f:
        f.write("--- DynamiXplore Performance Benchmark Results ---\n")
        f.write(f"Generated on: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write("="*50 + "\n\n")

        # --- 3. Process and write results for each group ---
        for group, results in sorted(grouped_results.items()):
            f.write(f"Benchmark Group: {group}\n")
            f.write("-" * (len(group) + 18) + "\n")

            # Write raw timings
            for name, time in sorted(results.items()):
                f.write(f"{name:<28}: {time:.6f}s\n")

            # --- 4. Calculate and write speedups ---
            try:
                if "DX_RK45_Adaptive" in results and "SciPy_RK45_Adaptive" in results:
                    speedup = results["SciPy_RK45_Adaptive"] / results["DX_RK45_Adaptive"]
                    f.write(f"  -> Adaptive Speedup (SciPy/DX): {speedup:.2f}x\n")
                
                if "DX_RK4_Fixed" in results and "SciPy_RK45_FixedLike" in results:
                    speedup = results["SciPy_RK45_FixedLike"] / results["DX_RK4_Fixed"]
                    f.write(f"  -> Fixed-Step Speedup (SciPy/DX): {speedup:.2f}x\n")

                # FIX: Corrected the names to match the actual benchmark output.
                if "DynamiXplore" in results and "nolds" in results and group == "Lyapunov Spectrum":
                    speedup = results["nolds"] / results["DynamiXplore"]
                    f.write(f"  -> Speedup (nolds/DX): {speedup:.2f}x\n")

                if group == "Invariant Measure" and "DynamiXplore" in results and "SciPy" in results:
                    speedup = results["SciPy"] / results["DynamiXplore"]
                    note = "(Note: SciPy is faster due to single-call optimization vs. high-overhead Rust calls)" if speedup < 1 else ""
                    f.write(f"  -> Speedup (SciPy/DX): {speedup:.2f}x {note}\n")

            except ZeroDivisionError:
                f.write("  -> Speedup calculation failed (division by zero).\n")
            
            f.write("\n")

    print(f"✅ Benchmark report successfully generated at '{OUTPUT_TXT}'")

if __name__ == "__main__":
    analyze_and_write_report()
