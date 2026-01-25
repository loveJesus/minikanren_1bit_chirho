#!/usr/bin/env python3
"""
Generate LaTeX table from benchmark results ☧
"""

import csv
import sys
from pathlib import Path

def load_results_chirho(path_chirho):
    """Load results from CSV file."""
    results_chirho = {}
    with open(path_chirho, 'r') as f_chirho:
        reader_chirho = csv.DictReader(f_chirho)
        for row_chirho in reader_chirho:
            impl_chirho = row_chirho['implementation_chirho']
            bench_chirho = row_chirho['benchmark_chirho']
            time_chirho = row_chirho['time_ms_chirho']
            key_chirho = (impl_chirho, bench_chirho)
            results_chirho[key_chirho] = time_chirho
    return results_chirho

def generate_latex_chirho(results_chirho):
    """Generate LaTeX table."""
    implementations_chirho = ['ours_rust_chirho', 'ocanren_chirho', 'faster_mk_chirho']
    benchmarks_chirho = ['appendo', 'nqueens']

    impl_names_chirho = {
        'ours_rust_chirho': '\\textbf{Ours (1-bit)}',
        'ocanren_chirho': 'OCanren',
        'faster_mk_chirho': 'faster-miniKanren',
    }

    bench_names_chirho = {
        'appendo': 'appendo (backward)',
        'nqueens': 'N-Queens 8',
    }

    print(r"\begin{table}[h]")
    print(r"\centering")
    print(r"\begin{tabular}{l" + "r" * len(benchmarks_chirho) + r"r}")
    print(r"\toprule")

    # Header
    header_chirho = "Implementation"
    for bench_chirho in benchmarks_chirho:
        header_chirho += f" & {bench_names_chirho[bench_chirho]}"
    header_chirho += r" & Speedup \\"
    print(header_chirho)
    print(r"\midrule")

    # Get our time for speedup calculation
    our_times_chirho = {}
    for bench_chirho in benchmarks_chirho:
        key_chirho = ('ours_rust_chirho', bench_chirho)
        time_str_chirho = results_chirho.get(key_chirho, 'pending')
        try:
            our_times_chirho[bench_chirho] = float(time_str_chirho)
        except ValueError:
            our_times_chirho[bench_chirho] = None

    # Rows
    for impl_chirho in implementations_chirho:
        row_chirho = impl_names_chirho[impl_chirho]
        speedups_chirho = []

        for bench_chirho in benchmarks_chirho:
            key_chirho = (impl_chirho, bench_chirho)
            time_str_chirho = results_chirho.get(key_chirho, 'pending')

            if time_str_chirho == 'pending':
                row_chirho += r" & pending"
                speedups_chirho.append(None)
            else:
                try:
                    time_chirho = float(time_str_chirho)
                    row_chirho += f" & {time_chirho:.2f}ms"
                    if our_times_chirho[bench_chirho] and time_chirho > 0:
                        speedups_chirho.append(time_chirho / our_times_chirho[bench_chirho])
                    else:
                        speedups_chirho.append(None)
                except ValueError:
                    row_chirho += f" & {time_str_chirho}"
                    speedups_chirho.append(None)

        # Average speedup
        valid_speedups_chirho = [s_chirho for s_chirho in speedups_chirho if s_chirho is not None]
        if valid_speedups_chirho and impl_chirho != 'ours_rust_chirho':
            avg_chirho = sum(valid_speedups_chirho) / len(valid_speedups_chirho)
            row_chirho += f" & {avg_chirho:.1f}$\\times$"
        elif impl_chirho == 'ours_rust_chirho':
            row_chirho += r" & ---"
        else:
            row_chirho += r" & ---"

        row_chirho += r" \\"
        print(row_chirho)

    print(r"\bottomrule")
    print(r"\end{tabular}")
    print(r"\caption{Comparison with other miniKanren implementations.}")
    print(r"\label{tab:comparison}")
    print(r"\end{table}")

def main_chirho():
    script_dir_chirho = Path(__file__).parent
    results_path_chirho = script_dir_chirho.parent / "results_chirho.csv"

    if not results_path_chirho.exists():
        print(f"Error: {results_path_chirho} not found", file=sys.stderr)
        print("Run run_comparison_chirho.sh first", file=sys.stderr)
        sys.exit(1)

    results_chirho = load_results_chirho(results_path_chirho)
    generate_latex_chirho(results_chirho)

if __name__ == '__main__':
    main_chirho()
