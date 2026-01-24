#!/usr/bin/env python3
"""
Program Synthesis Benchmark ☧

Synthesizes small programs from input-output specifications.
Tests miniKanren's ability to search program space efficiently.

This is a realistic workload for AI reasoning systems that need
to generate code, SQL queries, or other structured outputs.
"""

import time
import argparse
from dataclasses import dataclass
from typing import List, Tuple, Set, Optional, Dict
from collections import defaultdict

# Domain size for each program component
DOMAIN_BITS_CHIRHO = 64

@dataclass
class SynthSpec_chirho:
    """Input-output specification for synthesis."""
    inputs_chirho: List[Tuple]
    outputs_chirho: List
    max_depth_chirho: int = 3

@dataclass
class SynthStats_chirho:
    """Benchmark statistics."""
    solutions_chirho: int = 0
    time_ms_chirho: float = 0
    branches_chirho: int = 0
    propagations_chirho: int = 0
    peak_states_chirho: int = 0

# Program AST representation
# Each node is (op, arg1, arg2) where args are indices into term store
# Operations: 0=const, 1=var, 2=add, 3=mul, 4=if, 5=eq, 6=lt

OP_CONST_CHIRHO = 0
OP_VAR_CHIRHO = 1
OP_ADD_CHIRHO = 2
OP_MUL_CHIRHO = 3
OP_IF_CHIRHO = 4
OP_EQ_CHIRHO = 5
OP_LT_CHIRHO = 6

OP_NAMES_CHIRHO = {
    0: "const", 1: "var", 2: "+", 3: "*", 4: "if", 5: "==", 6: "<"
}

class ProgramDomain_chirho:
    """
    Domain representation for program synthesis.
    Each variable represents a choice in the program structure.
    """

    def __init__(self, num_vars_chirho: int):
        self.num_vars_chirho = num_vars_chirho
        # Each variable has a 64-bit domain
        self.domains_chirho: List[int] = [
            (1 << DOMAIN_BITS_CHIRHO) - 1
            for _ in range(num_vars_chirho)
        ]
        self.stats_chirho = SynthStats_chirho()

    def constrain_chirho(self, var_chirho: int, mask_chirho: int) -> bool:
        """Constrain variable to mask, return False if empty."""
        old_chirho = self.domains_chirho[var_chirho]
        new_chirho = old_chirho & mask_chirho
        self.domains_chirho[var_chirho] = new_chirho
        self.stats_chirho.propagations_chirho += 1
        return new_chirho != 0

    def unify_chirho(self, v1_chirho: int, v2_chirho: int) -> bool:
        """Unify two variables (intersection of domains)."""
        d1_chirho = self.domains_chirho[v1_chirho]
        d2_chirho = self.domains_chirho[v2_chirho]
        result_chirho = d1_chirho & d2_chirho
        self.domains_chirho[v1_chirho] = result_chirho
        self.domains_chirho[v2_chirho] = result_chirho
        self.stats_chirho.propagations_chirho += 1
        return result_chirho != 0

    def is_singleton_chirho(self, var_chirho: int) -> bool:
        """Check if variable has exactly one value."""
        d_chirho = self.domains_chirho[var_chirho]
        return d_chirho != 0 and (d_chirho & (d_chirho - 1)) == 0

    def get_value_chirho(self, var_chirho: int) -> Optional[int]:
        """Get singleton value or None."""
        d_chirho = self.domains_chirho[var_chirho]
        if d_chirho != 0 and (d_chirho & (d_chirho - 1)) == 0:
            return (d_chirho & -d_chirho).bit_length() - 1
        return None

    def fork_chirho(self, var_chirho: int) -> Tuple['ProgramDomain_chirho', 'ProgramDomain_chirho']:
        """Split on a variable, return two branches."""
        d_chirho = self.domains_chirho[var_chirho]
        lowest_chirho = d_chirho & -d_chirho
        rest_chirho = d_chirho & (d_chirho - 1)

        # Branch 1: take lowest value
        branch1_chirho = ProgramDomain_chirho(self.num_vars_chirho)
        branch1_chirho.domains_chirho = self.domains_chirho.copy()
        branch1_chirho.domains_chirho[var_chirho] = lowest_chirho

        # Branch 2: remaining values
        branch2_chirho = ProgramDomain_chirho(self.num_vars_chirho)
        branch2_chirho.domains_chirho = self.domains_chirho.copy()
        branch2_chirho.domains_chirho[var_chirho] = rest_chirho

        self.stats_chirho.branches_chirho += 1
        return branch1_chirho, branch2_chirho

    def copy_chirho(self) -> 'ProgramDomain_chirho':
        """Deep copy."""
        result_chirho = ProgramDomain_chirho(self.num_vars_chirho)
        result_chirho.domains_chirho = self.domains_chirho.copy()
        return result_chirho

    def failed_chirho(self) -> bool:
        """Check if any domain is empty."""
        return any(d == 0 for d in self.domains_chirho)

    def popcount_chirho(self, var_chirho: int) -> int:
        """Count possible values for variable."""
        return bin(self.domains_chirho[var_chirho]).count('1')


def eval_program_chirho(
    program_chirho: Dict[int, Tuple[int, int, int]],
    root_chirho: int,
    env_chirho: Dict[int, int]
) -> Optional[int]:
    """Evaluate a program given variable bindings."""
    if root_chirho not in program_chirho:
        return None

    op_chirho, arg1_chirho, arg2_chirho = program_chirho[root_chirho]

    if op_chirho == OP_CONST_CHIRHO:
        return arg1_chirho
    elif op_chirho == OP_VAR_CHIRHO:
        return env_chirho.get(arg1_chirho)
    elif op_chirho == OP_ADD_CHIRHO:
        v1_chirho = eval_program_chirho(program_chirho, arg1_chirho, env_chirho)
        v2_chirho = eval_program_chirho(program_chirho, arg2_chirho, env_chirho)
        if v1_chirho is None or v2_chirho is None:
            return None
        return v1_chirho + v2_chirho
    elif op_chirho == OP_MUL_CHIRHO:
        v1_chirho = eval_program_chirho(program_chirho, arg1_chirho, env_chirho)
        v2_chirho = eval_program_chirho(program_chirho, arg2_chirho, env_chirho)
        if v1_chirho is None or v2_chirho is None:
            return None
        return v1_chirho * v2_chirho
    elif op_chirho == OP_IF_CHIRHO:
        cond_chirho = eval_program_chirho(program_chirho, arg1_chirho, env_chirho)
        if cond_chirho is None:
            return None
        if cond_chirho:
            return eval_program_chirho(program_chirho, arg2_chirho, env_chirho)
        else:
            # For simplicity, else branch is 0
            return 0
    elif op_chirho == OP_EQ_CHIRHO:
        v1_chirho = eval_program_chirho(program_chirho, arg1_chirho, env_chirho)
        v2_chirho = eval_program_chirho(program_chirho, arg2_chirho, env_chirho)
        if v1_chirho is None or v2_chirho is None:
            return None
        return 1 if v1_chirho == v2_chirho else 0
    elif op_chirho == OP_LT_CHIRHO:
        v1_chirho = eval_program_chirho(program_chirho, arg1_chirho, env_chirho)
        v2_chirho = eval_program_chirho(program_chirho, arg2_chirho, env_chirho)
        if v1_chirho is None or v2_chirho is None:
            return None
        return 1 if v1_chirho < v2_chirho else 0

    return None


def synthesize_chirho(
    spec_chirho: SynthSpec_chirho,
    max_nodes_chirho: int = 3
) -> Tuple[List[Dict], SynthStats_chirho]:
    """
    Synthesize programs matching the specification.

    Uses constraint propagation + backtracking search.
    """
    # Variables: for each node position, we have (op, arg1, arg2)
    # Total variables = max_nodes * 3
    num_vars_chirho = max_nodes_chirho * 3

    solutions_chirho = []
    stats_chirho = SynthStats_chirho()

    start_time_chirho = time.perf_counter()

    # Constrain domains to valid values
    initial_state_chirho = ProgramDomain_chirho(num_vars_chirho)
    # Op domain: 0-6 (7 operations)
    op_mask_chirho = 0b1111111  # 7 bits
    # Arg domains: 0-max_nodes (references to other nodes or constants 0-9)
    arg_mask_chirho = (1 << min(max_nodes_chirho + 10, 16)) - 1

    for node_chirho in range(max_nodes_chirho):
        initial_state_chirho.domains_chirho[node_chirho * 3] = op_mask_chirho
        initial_state_chirho.domains_chirho[node_chirho * 3 + 1] = arg_mask_chirho
        initial_state_chirho.domains_chirho[node_chirho * 3 + 2] = arg_mask_chirho

    # Work queue: (domain_state, depth)
    queue_chirho = [(initial_state_chirho, 0)]
    max_branches_chirho = 10000  # Limit exploration

    while queue_chirho and len(solutions_chirho) < 10 and stats_chirho.branches_chirho < max_branches_chirho:
        state_chirho, depth_chirho = queue_chirho.pop()
        stats_chirho.peak_states_chirho = max(stats_chirho.peak_states_chirho, len(queue_chirho))

        if state_chirho.failed_chirho():
            continue

        # Find first non-singleton variable
        branch_var_chirho = None
        for v_chirho in range(num_vars_chirho):
            if not state_chirho.is_singleton_chirho(v_chirho):
                # Prefer variables with smaller domains (fail-first)
                if branch_var_chirho is None or \
                   state_chirho.popcount_chirho(v_chirho) < state_chirho.popcount_chirho(branch_var_chirho):
                    branch_var_chirho = v_chirho

        if branch_var_chirho is None:
            # All variables are singletons - we have a candidate solution
            # Extract program and verify against spec
            program_chirho = {}
            for node_chirho in range(max_nodes_chirho):
                op_chirho = state_chirho.get_value_chirho(node_chirho * 3)
                arg1_chirho = state_chirho.get_value_chirho(node_chirho * 3 + 1)
                arg2_chirho = state_chirho.get_value_chirho(node_chirho * 3 + 2)
                if op_chirho is not None:
                    program_chirho[node_chirho] = (op_chirho, arg1_chirho or 0, arg2_chirho or 0)

            # Verify against spec
            valid_chirho = True
            for inp_chirho, out_chirho in zip(spec_chirho.inputs_chirho, spec_chirho.outputs_chirho):
                env_chirho = {i: v for i, v in enumerate(inp_chirho)}
                result_chirho = eval_program_chirho(program_chirho, 0, env_chirho)
                if result_chirho != out_chirho:
                    valid_chirho = False
                    break

            if valid_chirho:
                solutions_chirho.append(program_chirho)
                stats_chirho.solutions_chirho += 1
        else:
            # Branch on variable
            branch1_chirho, branch2_chirho = state_chirho.fork_chirho(branch_var_chirho)
            stats_chirho.branches_chirho += 1

            # Add branches with non-empty domains
            if not branch2_chirho.failed_chirho():
                queue_chirho.append((branch2_chirho, depth_chirho + 1))
            if not branch1_chirho.failed_chirho():
                queue_chirho.append((branch1_chirho, depth_chirho + 1))

    stats_chirho.time_ms_chirho = (time.perf_counter() - start_time_chirho) * 1000

    return solutions_chirho, stats_chirho


def format_program_chirho(program_chirho: Dict[int, Tuple[int, int, int]], node_chirho: int = 0) -> str:
    """Pretty-print a program."""
    if node_chirho not in program_chirho:
        return "?"

    op_chirho, arg1_chirho, arg2_chirho = program_chirho[node_chirho]

    if op_chirho == OP_CONST_CHIRHO:
        return str(arg1_chirho)
    elif op_chirho == OP_VAR_CHIRHO:
        return f"x{arg1_chirho}"
    elif op_chirho in (OP_ADD_CHIRHO, OP_MUL_CHIRHO, OP_EQ_CHIRHO, OP_LT_CHIRHO):
        left_chirho = format_program_chirho(program_chirho, arg1_chirho)
        right_chirho = format_program_chirho(program_chirho, arg2_chirho)
        return f"({left_chirho} {OP_NAMES_CHIRHO[op_chirho]} {right_chirho})"
    elif op_chirho == OP_IF_CHIRHO:
        cond_chirho = format_program_chirho(program_chirho, arg1_chirho)
        then_chirho = format_program_chirho(program_chirho, arg2_chirho)
        return f"(if {cond_chirho} then {then_chirho})"

    return "?"


# Benchmark specifications
SPECS_CHIRHO = {
    "double": SynthSpec_chirho(
        inputs_chirho=[(1,), (2,), (3,), (5,), (10,)],
        outputs_chirho=[2, 4, 6, 10, 20],
        max_depth_chirho=2
    ),
    "square": SynthSpec_chirho(
        inputs_chirho=[(1,), (2,), (3,), (4,), (5,)],
        outputs_chirho=[1, 4, 9, 16, 25],
        max_depth_chirho=2
    ),
    "add_xy": SynthSpec_chirho(
        inputs_chirho=[(1, 2), (3, 4), (0, 5), (10, 10)],
        outputs_chirho=[3, 7, 5, 20],
        max_depth_chirho=2
    ),
    "max_xy": SynthSpec_chirho(
        inputs_chirho=[(1, 2), (5, 3), (0, 0), (10, 10)],
        outputs_chirho=[2, 5, 0, 10],
        max_depth_chirho=3
    ),
}


def run_benchmark_chirho(name_chirho: str, size_chirho: str = "small"):
    """Run a synthesis benchmark."""
    spec_chirho = SPECS_CHIRHO[name_chirho]

    max_nodes_chirho = 3 if size_chirho == "small" else 5

    print(f"\n{'='*60}")
    print(f"Benchmark: {name_chirho} (size={size_chirho})")
    print(f"{'='*60}")
    print(f"Specification:")
    for inp_chirho, out_chirho in zip(spec_chirho.inputs_chirho, spec_chirho.outputs_chirho):
        print(f"  f{inp_chirho} = {out_chirho}")

    solutions_chirho, stats_chirho = synthesize_chirho(spec_chirho, max_nodes_chirho)

    print(f"\nResults:")
    print(f"  Solutions found: {stats_chirho.solutions_chirho}")
    print(f"  Time: {stats_chirho.time_ms_chirho:.2f}ms")
    print(f"  Branches explored: {stats_chirho.branches_chirho}")
    print(f"  Peak queue size: {stats_chirho.peak_states_chirho}")

    if solutions_chirho:
        print(f"\nFirst solution:")
        print(f"  {format_program_chirho(solutions_chirho[0])}")

    # Hardware projection
    print(f"\nHardware projection (100MHz FPGA):")
    cycles_chirho = stats_chirho.branches_chirho * 2 + stats_chirho.propagations_chirho
    print(f"  Estimated cycles: {cycles_chirho}")
    print(f"  Estimated time: {cycles_chirho * 10}ns = {cycles_chirho * 0.00001:.4f}ms")
    print(f"  Speedup vs Python: {stats_chirho.time_ms_chirho / (cycles_chirho * 0.00001):.1f}x")

    return stats_chirho


def main_chirho():
    parser_chirho = argparse.ArgumentParser(description="Program Synthesis Benchmark ☧")
    parser_chirho.add_argument("--size", choices=["small", "large"], default="small")
    parser_chirho.add_argument("--benchmark", choices=list(SPECS_CHIRHO.keys()) + ["all"], default="all")
    args_chirho = parser_chirho.parse_args()

    print("Program Synthesis Benchmark ☧")
    print("=" * 60)

    if args_chirho.benchmark == "all":
        benchmarks_chirho = list(SPECS_CHIRHO.keys())
    else:
        benchmarks_chirho = [args_chirho.benchmark]

    total_stats_chirho = SynthStats_chirho()

    for name_chirho in benchmarks_chirho:
        stats_chirho = run_benchmark_chirho(name_chirho, args_chirho.size)
        total_stats_chirho.solutions_chirho += stats_chirho.solutions_chirho
        total_stats_chirho.time_ms_chirho += stats_chirho.time_ms_chirho
        total_stats_chirho.branches_chirho += stats_chirho.branches_chirho

    print(f"\n{'='*60}")
    print(f"TOTAL: {total_stats_chirho.solutions_chirho} solutions, "
          f"{total_stats_chirho.time_ms_chirho:.2f}ms, "
          f"{total_stats_chirho.branches_chirho} branches")
    print("☧ Soli Deo Gloria ☧")


if __name__ == "__main__":
    main_chirho()
