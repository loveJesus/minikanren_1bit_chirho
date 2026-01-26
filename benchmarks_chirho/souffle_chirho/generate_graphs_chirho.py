#!/usr/bin/env python3
"""
Generate random graphs for transitive closure benchmarking ☧

Creates edge files in TSV format for Soufflé.

"The lot is cast into the lap; but the whole disposing
 thereof is of the LORD." — Proverbs 16:33
"""

import random
import os

def generate_graph_chirho(n_nodes_chirho: int, n_edges_chirho: int, seed_chirho: int = 42) -> list:
    """Generate random directed graph with specified nodes and edges."""
    random.seed(seed_chirho)
    edges_chirho = set()

    while len(edges_chirho) < n_edges_chirho:
        src_chirho = random.randint(0, n_nodes_chirho - 1)
        dst_chirho = random.randint(0, n_nodes_chirho - 1)
        if src_chirho != dst_chirho:  # No self-loops
            edges_chirho.add((src_chirho, dst_chirho))

    return list(edges_chirho)

def write_edge_file_chirho(edges_chirho: list, filename_chirho: str):
    """Write edges in TSV format for Soufflé."""
    with open(filename_chirho, 'w') as f_chirho:
        for src_chirho, dst_chirho in edges_chirho:
            f_chirho.write(f"{src_chirho}\t{dst_chirho}\n")

def main_chirho():
    """Generate benchmark graphs of various sizes."""
    script_dir_chirho = os.path.dirname(os.path.abspath(__file__))

    configs_chirho = [
        (100, 1000, "1k"),      # 1K edges
        (500, 10000, "10k"),    # 10K edges
        (1000, 100000, "100k"), # 100K edges
    ]

    for n_nodes_chirho, n_edges_chirho, label_chirho in configs_chirho:
        edges_chirho = generate_graph_chirho(n_nodes_chirho, n_edges_chirho)
        filename_chirho = os.path.join(script_dir_chirho, f"edge_chirho_{label_chirho}.facts")
        write_edge_file_chirho(edges_chirho, filename_chirho)
        print(f"Generated {filename_chirho}: {n_nodes_chirho} nodes, {len(edges_chirho)} edges")

if __name__ == "__main__":
    main_chirho()
    print("\n☧ Soli Deo Gloria ☧")
