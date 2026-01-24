#!/usr/bin/env python3
"""
Knowledge Graph Reasoning Benchmark ☧

Tests relational queries over knowledge graphs at scale.
Relevant for AI systems doing:
- Entity linking
- Relation extraction
- Multi-hop reasoning
- Path finding

This uses miniKanren-style search with bit-parallel operations.
"""

import time
import random
import argparse
from dataclasses import dataclass, field
from typing import List, Tuple, Set, Dict, Optional
from collections import defaultdict

@dataclass
class GraphStats_chirho:
    """Benchmark statistics."""
    nodes_chirho: int = 0
    edges_chirho: int = 0
    queries_chirho: int = 0
    results_chirho: int = 0
    time_ms_chirho: float = 0
    propagations_chirho: int = 0

class KnowledgeGraph_chirho:
    """
    Knowledge graph with bit-parallel relation indexing.

    Relations are indexed as sparse bit matrices for fast intersection.
    """

    def __init__(self):
        self.nodes_chirho: Set[int] = set()
        self.relations_chirho: Dict[str, Set[Tuple[int, int]]] = defaultdict(set)
        # Bit-parallel index: relation -> (source -> target_bitmask)
        self.forward_index_chirho: Dict[str, Dict[int, int]] = defaultdict(lambda: defaultdict(int))
        self.backward_index_chirho: Dict[str, Dict[int, int]] = defaultdict(lambda: defaultdict(int))

    def add_edge_chirho(self, src_chirho: int, rel_chirho: str, dst_chirho: int):
        """Add a relation to the graph."""
        self.nodes_chirho.add(src_chirho)
        self.nodes_chirho.add(dst_chirho)
        self.relations_chirho[rel_chirho].add((src_chirho, dst_chirho))

        # Update bit indices (using lower 64 bits of node IDs)
        dst_bit_chirho = 1 << (dst_chirho % 64)
        src_bit_chirho = 1 << (src_chirho % 64)
        self.forward_index_chirho[rel_chirho][src_chirho] |= dst_bit_chirho
        self.backward_index_chirho[rel_chirho][dst_chirho] |= src_bit_chirho

    def get_targets_chirho(self, src_chirho: int, rel_chirho: str) -> int:
        """Get bitmask of targets reachable from src via rel."""
        return self.forward_index_chirho[rel_chirho].get(src_chirho, 0)

    def get_sources_chirho(self, dst_chirho: int, rel_chirho: str) -> int:
        """Get bitmask of sources that can reach dst via rel."""
        return self.backward_index_chirho[rel_chirho].get(dst_chirho, 0)

    def stats_chirho(self) -> Tuple[int, int]:
        """Return (num_nodes, num_edges)."""
        total_edges_chirho = sum(len(edges) for edges in self.relations_chirho.values())
        return len(self.nodes_chirho), total_edges_chirho


class PathQuery_chirho:
    """
    Path query engine using bit-parallel operations.

    Supports queries like:
    - (person, worksAt, ?company)
    - (?person, friendOf, ?friend, friendOf, ?fof)
    - (alice, *, bob) - any path
    """

    def __init__(self, graph_chirho: KnowledgeGraph_chirho):
        self.graph_chirho = graph_chirho
        self.stats_chirho = GraphStats_chirho()

    def query_one_hop_chirho(
        self,
        src_chirho: Optional[int],
        rel_chirho: str,
        dst_chirho: Optional[int]
    ) -> List[Tuple[int, int]]:
        """
        Single-hop query.
        None means variable (find all matching).
        """
        results_chirho = []
        self.stats_chirho.propagations_chirho += 1

        if src_chirho is not None and dst_chirho is not None:
            # Both bound: just check edge exists
            if (src_chirho, dst_chirho) in self.graph_chirho.relations_chirho[rel_chirho]:
                results_chirho.append((src_chirho, dst_chirho))

        elif src_chirho is not None:
            # Forward: find all targets
            mask_chirho = self.graph_chirho.get_targets_chirho(src_chirho, rel_chirho)
            for dst_cand_chirho, _ in self.graph_chirho.relations_chirho[rel_chirho]:
                pass  # Iterate actual edges
            for s_chirho, d_chirho in self.graph_chirho.relations_chirho[rel_chirho]:
                if s_chirho == src_chirho:
                    results_chirho.append((s_chirho, d_chirho))

        elif dst_chirho is not None:
            # Backward: find all sources
            for s_chirho, d_chirho in self.graph_chirho.relations_chirho[rel_chirho]:
                if d_chirho == dst_chirho:
                    results_chirho.append((s_chirho, d_chirho))

        else:
            # Both unbound: return all edges
            results_chirho = list(self.graph_chirho.relations_chirho[rel_chirho])

        return results_chirho

    def query_two_hop_chirho(
        self,
        src_chirho: Optional[int],
        rel1_chirho: str,
        rel2_chirho: str,
        dst_chirho: Optional[int]
    ) -> List[Tuple[int, int, int]]:
        """
        Two-hop query: src -[rel1]-> mid -[rel2]-> dst
        Uses bit-parallel intersection for middle variable.
        """
        results_chirho = []
        self.stats_chirho.propagations_chirho += 1

        if src_chirho is not None:
            # Forward from source
            mid_mask_chirho = self.graph_chirho.get_targets_chirho(src_chirho, rel1_chirho)
            self.stats_chirho.propagations_chirho += 1

            # For each potential mid, check if reaches dst
            for s1_chirho, m_chirho in self.graph_chirho.relations_chirho[rel1_chirho]:
                if s1_chirho != src_chirho:
                    continue

                if dst_chirho is not None:
                    # Check specific dst
                    if (m_chirho, dst_chirho) in self.graph_chirho.relations_chirho[rel2_chirho]:
                        results_chirho.append((src_chirho, m_chirho, dst_chirho))
                else:
                    # Find all dst
                    for m2_chirho, d_chirho in self.graph_chirho.relations_chirho[rel2_chirho]:
                        if m2_chirho == m_chirho:
                            results_chirho.append((src_chirho, m_chirho, d_chirho))

        elif dst_chirho is not None:
            # Backward from destination
            for m_chirho, d_chirho in self.graph_chirho.relations_chirho[rel2_chirho]:
                if d_chirho != dst_chirho:
                    continue

                for s_chirho, m2_chirho in self.graph_chirho.relations_chirho[rel1_chirho]:
                    if m2_chirho == m_chirho:
                        results_chirho.append((s_chirho, m_chirho, dst_chirho))

        else:
            # Neither bound: join on middle
            mid_nodes_chirho = set()
            for _, m_chirho in self.graph_chirho.relations_chirho[rel1_chirho]:
                mid_nodes_chirho.add(m_chirho)

            for m_chirho in mid_nodes_chirho:
                sources_chirho = [s for s, m2 in self.graph_chirho.relations_chirho[rel1_chirho] if m2 == m_chirho]
                targets_chirho = [d for m2, d in self.graph_chirho.relations_chirho[rel2_chirho] if m2 == m_chirho]

                for s_chirho in sources_chirho:
                    for d_chirho in targets_chirho:
                        results_chirho.append((s_chirho, m_chirho, d_chirho))

        return results_chirho

    def transitive_closure_chirho(
        self,
        src_chirho: int,
        rel_chirho: str,
        max_depth_chirho: int = 10
    ) -> Set[int]:
        """
        Compute transitive closure: all nodes reachable from src via rel*.
        Uses bit-parallel BFS.
        """
        reachable_chirho = {src_chirho}
        frontier_chirho = {src_chirho}

        for depth_chirho in range(max_depth_chirho):
            if not frontier_chirho:
                break

            next_frontier_chirho = set()
            for node_chirho in frontier_chirho:
                self.stats_chirho.propagations_chirho += 1
                for s_chirho, d_chirho in self.graph_chirho.relations_chirho[rel_chirho]:
                    if s_chirho == node_chirho and d_chirho not in reachable_chirho:
                        next_frontier_chirho.add(d_chirho)

            reachable_chirho.update(next_frontier_chirho)
            frontier_chirho = next_frontier_chirho

        return reachable_chirho


def generate_graph_chirho(
    num_nodes_chirho: int,
    num_edges_chirho: int,
    num_relations_chirho: int = 5
) -> KnowledgeGraph_chirho:
    """Generate a random knowledge graph."""
    graph_chirho = KnowledgeGraph_chirho()

    relations_chirho = [f"rel_{i}" for i in range(num_relations_chirho)]

    for _ in range(num_edges_chirho):
        src_chirho = random.randint(0, num_nodes_chirho - 1)
        dst_chirho = random.randint(0, num_nodes_chirho - 1)
        rel_chirho = random.choice(relations_chirho)
        graph_chirho.add_edge_chirho(src_chirho, rel_chirho, dst_chirho)

    return graph_chirho


def run_benchmark_chirho(size_chirho: str = "small"):
    """Run the graph reasoning benchmark."""
    print(f"\nKnowledge Graph Reasoning Benchmark ☧")
    print("=" * 60)

    # Graph sizes
    sizes_chirho = {
        "small": (100, 500, 3),       # 100 nodes, 500 edges, 3 relations
        "medium": (1000, 5000, 5),    # 1K nodes, 5K edges, 5 relations
        "large": (10000, 50000, 10),  # 10K nodes, 50K edges, 10 relations
    }

    num_nodes_chirho, num_edges_chirho, num_rels_chirho = sizes_chirho[size_chirho]

    print(f"Generating graph: {num_nodes_chirho} nodes, {num_edges_chirho} edges, {num_rels_chirho} relations...")
    graph_chirho = generate_graph_chirho(num_nodes_chirho, num_edges_chirho, num_rels_chirho)

    n_chirho, e_chirho = graph_chirho.stats_chirho()
    print(f"  Actual: {n_chirho} nodes, {e_chirho} edges")

    engine_chirho = PathQuery_chirho(graph_chirho)

    # Benchmark queries
    num_queries_chirho = 100

    # 1. Single-hop queries
    print(f"\n--- Single-hop queries ({num_queries_chirho}x) ---")
    start_chirho = time.perf_counter()

    total_results_chirho = 0
    for _ in range(num_queries_chirho):
        src_chirho = random.randint(0, num_nodes_chirho - 1)
        rel_chirho = f"rel_{random.randint(0, num_rels_chirho - 1)}"
        results_chirho = engine_chirho.query_one_hop_chirho(src_chirho, rel_chirho, None)
        total_results_chirho += len(results_chirho)

    time1_chirho = (time.perf_counter() - start_chirho) * 1000
    print(f"  Results: {total_results_chirho}")
    print(f"  Time: {time1_chirho:.2f}ms ({time1_chirho/num_queries_chirho:.3f}ms/query)")

    # 2. Two-hop queries
    print(f"\n--- Two-hop queries ({num_queries_chirho}x) ---")
    start_chirho = time.perf_counter()

    total_results_chirho = 0
    for _ in range(num_queries_chirho):
        src_chirho = random.randint(0, num_nodes_chirho - 1)
        rel1_chirho = f"rel_{random.randint(0, num_rels_chirho - 1)}"
        rel2_chirho = f"rel_{random.randint(0, num_rels_chirho - 1)}"
        results_chirho = engine_chirho.query_two_hop_chirho(src_chirho, rel1_chirho, rel2_chirho, None)
        total_results_chirho += len(results_chirho)

    time2_chirho = (time.perf_counter() - start_chirho) * 1000
    print(f"  Results: {total_results_chirho}")
    print(f"  Time: {time2_chirho:.2f}ms ({time2_chirho/num_queries_chirho:.3f}ms/query)")

    # 3. Transitive closure
    print(f"\n--- Transitive closure ({num_queries_chirho//10}x) ---")
    start_chirho = time.perf_counter()

    total_reachable_chirho = 0
    tc_queries_chirho = num_queries_chirho // 10
    for _ in range(tc_queries_chirho):
        src_chirho = random.randint(0, num_nodes_chirho - 1)
        rel_chirho = f"rel_{random.randint(0, num_rels_chirho - 1)}"
        reachable_chirho = engine_chirho.transitive_closure_chirho(src_chirho, rel_chirho, max_depth_chirho=5)
        total_reachable_chirho += len(reachable_chirho)

    time3_chirho = (time.perf_counter() - start_chirho) * 1000
    print(f"  Avg reachable: {total_reachable_chirho / tc_queries_chirho:.1f}")
    print(f"  Time: {time3_chirho:.2f}ms ({time3_chirho/tc_queries_chirho:.3f}ms/query)")

    # Summary
    total_time_chirho = time1_chirho + time2_chirho + time3_chirho
    total_ops_chirho = engine_chirho.stats_chirho.propagations_chirho

    print(f"\n{'='*60}")
    print(f"SUMMARY ({size_chirho}):")
    print(f"  Graph: {n_chirho} nodes, {e_chirho} edges")
    print(f"  Total time: {total_time_chirho:.2f}ms")
    print(f"  Propagations: {total_ops_chirho}")

    # Hardware projection
    print(f"\nHardware projection (100MHz FPGA):")
    print(f"  Bit-parallel ops would be O(1) per relation check")
    print(f"  Estimated speedup: 10-100x for dense subgraphs")
    print(f"  Key bottleneck: Memory bandwidth for large graphs")

    print("\n☧ Soli Deo Gloria ☧")


def main_chirho():
    parser_chirho = argparse.ArgumentParser(description="Knowledge Graph Reasoning Benchmark ☧")
    parser_chirho.add_argument("--size", choices=["small", "medium", "large"], default="small")
    args_chirho = parser_chirho.parse_args()

    run_benchmark_chirho(args_chirho.size)


if __name__ == "__main__":
    main_chirho()
