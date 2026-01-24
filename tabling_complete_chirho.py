#!/usr/bin/env python3
"""
Complete Tabling System with Goal Reordering ☧

The final bridge between streams and tensors.

Key insight: Goal REORDERING based on mode analysis
- When we have: (appendo A B X), (appendo X C [0,1,2])
- Reorder to: (appendo X C [0,1,2]), (appendo A B X)
- Because X gets grounded by the second, then first becomes backward

This is automatic mode-driven scheduling!
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Any
from collections import defaultdict
from enum import Enum, auto


# === Terms ===

@dataclass(frozen=True)
class VarChirho:
    id_chirho: int
    def __repr__(self_chirho): return f"_{self_chirho.id_chirho}"

@dataclass(frozen=True)
class NilChirho:
    def __repr__(self_chirho): return "[]"

@dataclass(frozen=True)
class ConsChirho:
    head_chirho: Any
    tail_chirho: Any

    def __repr__(self_chirho):
        elems_chirho = []
        curr_chirho = self_chirho
        while isinstance(curr_chirho, ConsChirho):
            elems_chirho.append(repr(curr_chirho.head_chirho))
            curr_chirho = curr_chirho.tail_chirho
        if isinstance(curr_chirho, NilChirho):
            return "[" + ", ".join(elems_chirho) + "]"
        return "[" + ", ".join(elems_chirho) + " | " + repr(curr_chirho) + "]"

def list_chirho(*elems_chirho):
    result_chirho = NilChirho()
    for e_chirho in reversed(elems_chirho):
        result_chirho = ConsChirho(e_chirho, result_chirho)
    return result_chirho

def term_to_pylist_chirho(t_chirho) -> Optional[list]:
    result_chirho = []
    while isinstance(t_chirho, ConsChirho):
        if isinstance(t_chirho.head_chirho, VarChirho):
            return None
        result_chirho.append(t_chirho.head_chirho)
        t_chirho = t_chirho.tail_chirho
    if isinstance(t_chirho, NilChirho):
        return result_chirho
    return None

def pylist_to_term_chirho(lst_chirho: list):
    result_chirho = NilChirho()
    for x_chirho in reversed(lst_chirho):
        result_chirho = ConsChirho(x_chirho, result_chirho)
    return result_chirho


# === State and Unification ===

@dataclass
class StateChirho:
    subst_chirho: Dict[int, Any]
    counter_chirho: int

    def copy_chirho(self_chirho):
        return StateChirho(self_chirho.subst_chirho.copy(), self_chirho.counter_chirho)

def empty_state_chirho():
    return StateChirho({}, 0)

def walk_chirho(t_chirho, subst_chirho):
    while isinstance(t_chirho, VarChirho) and t_chirho.id_chirho in subst_chirho:
        t_chirho = subst_chirho[t_chirho.id_chirho]
    return t_chirho

def walk_deep_chirho(t_chirho, subst_chirho):
    t_chirho = walk_chirho(t_chirho, subst_chirho)
    if isinstance(t_chirho, ConsChirho):
        return ConsChirho(
            walk_deep_chirho(t_chirho.head_chirho, subst_chirho),
            walk_deep_chirho(t_chirho.tail_chirho, subst_chirho)
        )
    return t_chirho

def occurs_chirho(var_id_chirho, t_chirho, subst_chirho):
    t_chirho = walk_chirho(t_chirho, subst_chirho)
    if isinstance(t_chirho, VarChirho):
        return t_chirho.id_chirho == var_id_chirho
    if isinstance(t_chirho, ConsChirho):
        return (occurs_chirho(var_id_chirho, t_chirho.head_chirho, subst_chirho) or
                occurs_chirho(var_id_chirho, t_chirho.tail_chirho, subst_chirho))
    return False

def unify_chirho(t1_chirho, t2_chirho, subst_chirho):
    t1_chirho = walk_chirho(t1_chirho, subst_chirho)
    t2_chirho = walk_chirho(t2_chirho, subst_chirho)
    if t1_chirho == t2_chirho:
        return subst_chirho
    if isinstance(t1_chirho, VarChirho):
        if occurs_chirho(t1_chirho.id_chirho, t2_chirho, subst_chirho):
            return None
        subst_chirho = subst_chirho.copy()
        subst_chirho[t1_chirho.id_chirho] = t2_chirho
        return subst_chirho
    if isinstance(t2_chirho, VarChirho):
        if occurs_chirho(t2_chirho.id_chirho, t1_chirho, subst_chirho):
            return None
        subst_chirho = subst_chirho.copy()
        subst_chirho[t2_chirho.id_chirho] = t1_chirho
        return subst_chirho
    if isinstance(t1_chirho, ConsChirho) and isinstance(t2_chirho, ConsChirho):
        subst_chirho = unify_chirho(t1_chirho.head_chirho, t2_chirho.head_chirho, subst_chirho)
        if subst_chirho is None:
            return None
        return unify_chirho(t1_chirho.tail_chirho, t2_chirho.tail_chirho, subst_chirho)
    return None

def is_ground_chirho(t_chirho):
    if isinstance(t_chirho, VarChirho):
        return False
    if isinstance(t_chirho, ConsChirho):
        return is_ground_chirho(t_chirho.head_chirho) and is_ground_chirho(t_chirho.tail_chirho)
    return True

def collect_vars_chirho(t_chirho) -> Set[int]:
    """Collect all variable IDs in a term"""
    if isinstance(t_chirho, VarChirho):
        return {t_chirho.id_chirho}
    if isinstance(t_chirho, ConsChirho):
        return collect_vars_chirho(t_chirho.head_chirho) | collect_vars_chirho(t_chirho.tail_chirho)
    return set()


# === Global Tensor Store ===

class TensorStoreChirho:
    """
    Stores all discovered relation tuples.
    This IS the sparse tensor representation.
    """
    def __init__(self_chirho):
        self_chirho.appendo_triples_chirho: Set[Tuple[tuple, tuple, tuple]] = set()

    def add_appendo_chirho(self_chirho, l_chirho, s_chirho, out_chirho):
        """Add a ground appendo triple"""
        l_list_chirho = term_to_pylist_chirho(l_chirho)
        s_list_chirho = term_to_pylist_chirho(s_chirho)
        out_list_chirho = term_to_pylist_chirho(out_chirho)

        if l_list_chirho is not None and s_list_chirho is not None and out_list_chirho is not None:
            self_chirho.appendo_triples_chirho.add((
                tuple(l_list_chirho),
                tuple(s_list_chirho),
                tuple(out_list_chirho)
            ))

TENSOR_STORE_CHIRHO = TensorStoreChirho()

def reset_tensor_chirho():
    global TENSOR_STORE_CHIRHO
    TENSOR_STORE_CHIRHO = TensorStoreChirho()


# === Core appendo with mode detection ===

def appendo_core_chirho(l_chirho, s_chirho, out_chirho, state_chirho: StateChirho) -> Iterator[StateChirho]:
    """
    Core appendo implementation with mode-aware execution.

    Modes:
    - BACKWARD: out is ground → enumerate splits (FINITE)
    - FORWARD: l,s ground → compute result (SINGLE)
    - SEMI: some ground → mixed strategy
    - GENERATE: none ground → bounded enumeration (LIMITED)
    """
    l_w_chirho = walk_deep_chirho(l_chirho, state_chirho.subst_chirho)
    s_w_chirho = walk_deep_chirho(s_chirho, state_chirho.subst_chirho)
    out_w_chirho = walk_deep_chirho(out_chirho, state_chirho.subst_chirho)

    l_ground_chirho = is_ground_chirho(l_w_chirho)
    s_ground_chirho = is_ground_chirho(s_w_chirho)
    out_ground_chirho = is_ground_chirho(out_w_chirho)

    # BACKWARD: out ground → finite split enumeration
    if out_ground_chirho:
        out_list_chirho = term_to_pylist_chirho(out_w_chirho)
        if out_list_chirho is None:
            return

        for i_chirho in range(len(out_list_chirho) + 1):
            l_split_chirho = pylist_to_term_chirho(out_list_chirho[:i_chirho])
            s_split_chirho = pylist_to_term_chirho(out_list_chirho[i_chirho:])

            subst1_chirho = unify_chirho(l_chirho, l_split_chirho, state_chirho.subst_chirho)
            if subst1_chirho is None:
                continue

            subst2_chirho = unify_chirho(s_chirho, s_split_chirho, subst1_chirho)
            if subst2_chirho is None:
                continue

            TENSOR_STORE_CHIRHO.add_appendo_chirho(l_split_chirho, s_split_chirho, out_w_chirho)
            yield StateChirho(subst2_chirho, state_chirho.counter_chirho)
        return

    # FORWARD: l and s ground → compute
    if l_ground_chirho and s_ground_chirho:
        l_list_chirho = term_to_pylist_chirho(l_w_chirho)
        s_list_chirho = term_to_pylist_chirho(s_w_chirho)

        if l_list_chirho is None or s_list_chirho is None:
            return

        result_chirho = pylist_to_term_chirho(l_list_chirho + s_list_chirho)
        subst_chirho = unify_chirho(out_chirho, result_chirho, state_chirho.subst_chirho)

        if subst_chirho is not None:
            TENSOR_STORE_CHIRHO.add_appendo_chirho(l_w_chirho, s_w_chirho, result_chirho)
            yield StateChirho(subst_chirho, state_chirho.counter_chirho)
        return

    # SEMI-BACKWARD: l ground, enumerate s via recursive structure
    if l_ground_chirho:
        # l = [] → s = out
        if isinstance(l_w_chirho, NilChirho):
            subst_chirho = unify_chirho(s_chirho, out_chirho, state_chirho.subst_chirho)
            if subst_chirho is not None:
                s_val_chirho = walk_deep_chirho(s_chirho, subst_chirho)
                out_val_chirho = walk_deep_chirho(out_chirho, subst_chirho)
                if is_ground_chirho(s_val_chirho) and is_ground_chirho(out_val_chirho):
                    TENSOR_STORE_CHIRHO.add_appendo_chirho(l_w_chirho, s_val_chirho, out_val_chirho)
                yield StateChirho(subst_chirho, state_chirho.counter_chirho)
            return

        # l = [h|t] → out = [h|r], appendo(t, s, r)
        if isinstance(l_w_chirho, ConsChirho):
            h_chirho = l_w_chirho.head_chirho
            t_chirho = l_w_chirho.tail_chirho

            r_var_chirho = VarChirho(state_chirho.counter_chirho)
            new_counter_chirho = state_chirho.counter_chirho + 1

            subst_chirho = unify_chirho(out_chirho, ConsChirho(h_chirho, r_var_chirho), state_chirho.subst_chirho)
            if subst_chirho is None:
                return

            new_state_chirho = StateChirho(subst_chirho, new_counter_chirho)
            for result_state_chirho in appendo_core_chirho(t_chirho, s_chirho, r_var_chirho, new_state_chirho):
                l_val_chirho = walk_deep_chirho(l_chirho, result_state_chirho.subst_chirho)
                s_val_chirho = walk_deep_chirho(s_chirho, result_state_chirho.subst_chirho)
                out_val_chirho = walk_deep_chirho(out_chirho, result_state_chirho.subst_chirho)
                if is_ground_chirho(l_val_chirho) and is_ground_chirho(s_val_chirho) and is_ground_chirho(out_val_chirho):
                    TENSOR_STORE_CHIRHO.add_appendo_chirho(l_val_chirho, s_val_chirho, out_val_chirho)
                yield result_state_chirho
        return

    # GENERATE: bounded enumeration
    yield from appendo_generate_chirho(l_chirho, s_chirho, out_chirho, state_chirho, max_depth_chirho=5)


def appendo_generate_chirho(l_chirho, s_chirho, out_chirho, state_chirho: StateChirho, max_depth_chirho: int) -> Iterator[StateChirho]:
    """Generate appendo results with bounded depth"""
    if max_depth_chirho <= 0:
        return

    # Base: l = [], s = out
    subst1_chirho = unify_chirho(l_chirho, NilChirho(), state_chirho.subst_chirho)
    if subst1_chirho is not None:
        subst2_chirho = unify_chirho(s_chirho, out_chirho, subst1_chirho)
        if subst2_chirho is not None:
            l_val_chirho = walk_deep_chirho(l_chirho, subst2_chirho)
            s_val_chirho = walk_deep_chirho(s_chirho, subst2_chirho)
            out_val_chirho = walk_deep_chirho(out_chirho, subst2_chirho)
            if is_ground_chirho(l_val_chirho) and is_ground_chirho(s_val_chirho) and is_ground_chirho(out_val_chirho):
                TENSOR_STORE_CHIRHO.add_appendo_chirho(l_val_chirho, s_val_chirho, out_val_chirho)
            yield StateChirho(subst2_chirho, state_chirho.counter_chirho)

    # Recursive: l = [h|t], out = [h|r], appendo(t,s,r)
    h_chirho = VarChirho(state_chirho.counter_chirho)
    t_chirho = VarChirho(state_chirho.counter_chirho + 1)
    r_chirho = VarChirho(state_chirho.counter_chirho + 2)
    new_counter_chirho = state_chirho.counter_chirho + 3

    subst1_chirho = unify_chirho(l_chirho, ConsChirho(h_chirho, t_chirho), state_chirho.subst_chirho)
    if subst1_chirho is None:
        return

    subst2_chirho = unify_chirho(out_chirho, ConsChirho(h_chirho, r_chirho), subst1_chirho)
    if subst2_chirho is None:
        return

    new_state_chirho = StateChirho(subst2_chirho, new_counter_chirho)
    for result_state_chirho in appendo_generate_chirho(t_chirho, s_chirho, r_chirho, new_state_chirho, max_depth_chirho - 1):
        l_val_chirho = walk_deep_chirho(l_chirho, result_state_chirho.subst_chirho)
        s_val_chirho = walk_deep_chirho(s_chirho, result_state_chirho.subst_chirho)
        out_val_chirho = walk_deep_chirho(out_chirho, result_state_chirho.subst_chirho)
        if is_ground_chirho(l_val_chirho) and is_ground_chirho(s_val_chirho) and is_ground_chirho(out_val_chirho):
            TENSOR_STORE_CHIRHO.add_appendo_chirho(l_val_chirho, s_val_chirho, out_val_chirho)
        yield result_state_chirho


def appendo_chirho(l_chirho, s_chirho, out_chirho):
    """Create appendo goal"""
    def goal_chirho(state_chirho):
        yield from appendo_core_chirho(l_chirho, s_chirho, out_chirho, state_chirho)
    return goal_chirho


# === Goal Combinators with Reordering ===

class GoalInfoChirho:
    """Information about a goal for scheduling"""
    def __init__(self_chirho, goal_chirho, args_chirho, name_chirho=""):
        self_chirho.goal_chirho = goal_chirho
        self_chirho.args_chirho = args_chirho  # Terms in the goal
        self_chirho.name_chirho = name_chirho

    def groundness_score_chirho(self_chirho, subst_chirho) -> int:
        """
        Score based on how ground the arguments are.
        Higher = more ground = should run first.
        """
        score_chirho = 0
        for arg_chirho in self_chirho.args_chirho:
            walked_chirho = walk_deep_chirho(arg_chirho, subst_chirho)
            if is_ground_chirho(walked_chirho):
                score_chirho += 10
            elif isinstance(walked_chirho, ConsChirho):
                # Partially ground
                score_chirho += 5
        return score_chirho


def smart_conj_chirho(goals_info_chirho: List[GoalInfoChirho]):
    """
    Conjunction with mode-driven reordering.
    Goals with more ground args run first.
    """
    def goal_chirho(state_chirho):
        if not goals_info_chirho:
            yield state_chirho
            return

        if len(goals_info_chirho) == 1:
            yield from goals_info_chirho[0].goal_chirho(state_chirho)
            return

        # Sort by groundness (descending)
        sorted_goals_chirho = sorted(
            goals_info_chirho,
            key=lambda g_chirho: -g_chirho.groundness_score_chirho(state_chirho.subst_chirho)
        )

        # Run first, recurse with rest
        first_chirho = sorted_goals_chirho[0]
        rest_chirho = sorted_goals_chirho[1:]

        for st1_chirho in first_chirho.goal_chirho(state_chirho):
            yield from smart_conj_chirho(rest_chirho)(st1_chirho)

    return goal_chirho


def conj_chirho(g1_chirho, g2_chirho):
    """Simple conjunction"""
    def goal_chirho(state_chirho):
        for st1_chirho in g1_chirho(state_chirho):
            yield from g2_chirho(st1_chirho)
    return goal_chirho


def disj_chirho(g1_chirho, g2_chirho):
    """Interleaving disjunction"""
    def goal_chirho(state_chirho):
        iters_chirho = [g1_chirho(state_chirho), g2_chirho(state_chirho)]
        while iters_chirho:
            next_iters_chirho = []
            for it_chirho in iters_chirho:
                try:
                    yield next(it_chirho)
                    next_iters_chirho.append(it_chirho)
                except StopIteration:
                    pass
            iters_chirho = next_iters_chirho
    return goal_chirho


def eq_chirho(t1_chirho, t2_chirho):
    """Unification goal"""
    def goal_chirho(state_chirho):
        subst_chirho = unify_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if subst_chirho is not None:
            yield StateChirho(subst_chirho, state_chirho.counter_chirho)
    return goal_chirho


def fresh_chirho(fn_chirho):
    """Fresh variable"""
    def goal_chirho(state_chirho):
        var_chirho = VarChirho(state_chirho.counter_chirho)
        new_st_chirho = StateChirho(state_chirho.subst_chirho, state_chirho.counter_chirho + 1)
        yield from fn_chirho(var_chirho)(new_st_chirho)
    return goal_chirho


# === Run ===

def run_chirho(n_chirho, goal_chirho, vars_chirho):
    from itertools import islice
    stream_chirho = goal_chirho(empty_state_chirho())
    states_chirho = list(islice(stream_chirho, n_chirho)) if n_chirho else list(stream_chirho)

    results_chirho = []
    for st_chirho in states_chirho:
        result_chirho = {}
        for v_chirho in vars_chirho:
            result_chirho[v_chirho.id_chirho] = walk_deep_chirho(v_chirho, st_chirho.subst_chirho)
        results_chirho.append(result_chirho)
    return results_chirho


# === Demo ===

def main():
    print("=== Complete Tabling with Goal Reordering ☧ ===\n")

    print("""
    The key insight: MODE-DRIVEN GOAL REORDERING

    Query: appendo(A, B, X), appendo(X, C, [0,1,2])

    Naive order (left to right):
    1. appendo(A,B,X) - X is free → GENERATE mode → infinite!
    2. Never reaches the filtering goal

    Smart order (by groundness):
    1. appendo(X,C,[0,1,2]) - out is ground → BACKWARD mode → finite!
       Yields X ∈ {[], [0], [0,1], [0,1,2]}
    2. appendo(A,B,X) - X now ground → BACKWARD mode → finite!
       For each X, yields all splits
    """)

    # === Test 1: Backward ===
    print("="*60)
    print("TEST 1: Backward - appendo(L, S, [0,1,2])")
    print("="*60)

    reset_tensor_chirho()

    l1_chirho = VarChirho(0)
    s1_chirho = VarChirho(1)
    goal1_chirho = appendo_chirho(l1_chirho, s1_chirho, list_chirho(0, 1, 2))
    results1_chirho = run_chirho(10, goal1_chirho, [l1_chirho, s1_chirho])

    print(f"\nResults ({len(results1_chirho)}):")
    for r_chirho in results1_chirho:
        print(f"  L = {r_chirho[0]}, S = {r_chirho[1]}")

    print(f"\nTensor triples: {len(TENSOR_STORE_CHIRHO.appendo_triples_chirho)}")

    # === Test 2: Composition with SMART reordering ===
    print("\n" + "="*60)
    print("TEST 2: Smart Composition - appendo(A,B,X), appendo(X,C,[0,1,2])")
    print("        Using mode-driven goal reordering!")
    print("="*60)

    reset_tensor_chirho()

    a_chirho = VarChirho(0)
    b_chirho = VarChirho(1)
    x_chirho = VarChirho(2)
    c_chirho = VarChirho(3)

    # Create goals with info for smart scheduling
    goal_a_chirho = GoalInfoChirho(
        appendo_chirho(a_chirho, b_chirho, x_chirho),
        [a_chirho, b_chirho, x_chirho],
        "appendo(A,B,X)"
    )
    goal_b_chirho = GoalInfoChirho(
        appendo_chirho(x_chirho, c_chirho, list_chirho(0, 1, 2)),
        [x_chirho, c_chirho, list_chirho(0, 1, 2)],
        "appendo(X,C,[0,1,2])"
    )

    goal2_chirho = smart_conj_chirho([goal_a_chirho, goal_b_chirho])
    results2_chirho = run_chirho(25, goal2_chirho, [a_chirho, b_chirho, x_chirho, c_chirho])

    print(f"\nResults ({len(results2_chirho)}):")
    for r_chirho in results2_chirho:
        print(f"  A={r_chirho[0]}, B={r_chirho[1]} → X={r_chirho[2]}, C={r_chirho[3]}")

    # Verify by checking A ++ B = X and X ++ C = [0,1,2]
    print("\nVerification:")
    verified_chirho = 0
    for r_chirho in results2_chirho:
        a_list_chirho = term_to_pylist_chirho(r_chirho[0])
        b_list_chirho = term_to_pylist_chirho(r_chirho[1])
        x_list_chirho = term_to_pylist_chirho(r_chirho[2])
        c_list_chirho = term_to_pylist_chirho(r_chirho[3])

        if a_list_chirho is not None and b_list_chirho is not None and x_list_chirho is not None and c_list_chirho is not None:
            check1_chirho = a_list_chirho + b_list_chirho == x_list_chirho
            check2_chirho = x_list_chirho + c_list_chirho == [0, 1, 2]
            if check1_chirho and check2_chirho:
                verified_chirho += 1
    print(f"  {verified_chirho}/{len(results2_chirho)} solutions verified correct")

    # === Test 3: Triple composition ===
    print("\n" + "="*60)
    print("TEST 3: Triple - appendo(A,B,X), appendo(X,C,Y), appendo(Y,D,[0,1,2,3])")
    print("="*60)

    reset_tensor_chirho()

    a3_chirho = VarChirho(0)
    b3_chirho = VarChirho(1)
    x3_chirho = VarChirho(2)
    c3_chirho = VarChirho(3)
    y3_chirho = VarChirho(4)
    d3_chirho = VarChirho(5)

    goals3_chirho = [
        GoalInfoChirho(
            appendo_chirho(a3_chirho, b3_chirho, x3_chirho),
            [a3_chirho, b3_chirho, x3_chirho],
            "appendo(A,B,X)"
        ),
        GoalInfoChirho(
            appendo_chirho(x3_chirho, c3_chirho, y3_chirho),
            [x3_chirho, c3_chirho, y3_chirho],
            "appendo(X,C,Y)"
        ),
        GoalInfoChirho(
            appendo_chirho(y3_chirho, d3_chirho, list_chirho(0, 1, 2, 3)),
            [y3_chirho, d3_chirho, list_chirho(0, 1, 2, 3)],
            "appendo(Y,D,[0,1,2,3])"
        ),
    ]

    goal3_chirho = smart_conj_chirho(goals3_chirho)
    results3_chirho = run_chirho(50, goal3_chirho, [a3_chirho, b3_chirho, x3_chirho, c3_chirho, y3_chirho, d3_chirho])

    print(f"\nResults ({len(results3_chirho)}):")
    for i_chirho, r_chirho in enumerate(results3_chirho[:10]):
        a_chirho = r_chirho[0]
        b_chirho = r_chirho[1]
        x_chirho = r_chirho[2]
        c_chirho = r_chirho[3]
        y_chirho = r_chirho[4]
        d_chirho = r_chirho[5]
        print(f"  {i_chirho+1}. A={a_chirho} ++ B={b_chirho} = X={x_chirho}")
        print(f"      X={x_chirho} ++ C={c_chirho} = Y={y_chirho}")
        print(f"      Y={y_chirho} ++ D={d_chirho} = [0,1,2,3]")
    if len(results3_chirho) > 10:
        print(f"  ... and {len(results3_chirho)-10} more")

    # === Tensor view ===
    print("\n" + "="*60)
    print("TENSOR VIEW: All discovered triples")
    print("="*60)

    print(f"\nTotal triples: {len(TENSOR_STORE_CHIRHO.appendo_triples_chirho)}")
    print("\nTriples (sorted by output length):")
    sorted_triples_chirho = sorted(
        TENSOR_STORE_CHIRHO.appendo_triples_chirho,
        key=lambda t_chirho: (len(t_chirho[2]), t_chirho)
    )
    for t_chirho in sorted_triples_chirho:
        l_chirho, s_chirho, o_chirho = t_chirho
        print(f"  {list(l_chirho)} ++ {list(s_chirho)} = {list(o_chirho)}")

    # === The bridge is complete ===
    print("\n" + "="*60)
    print("THE BRIDGE IS COMPLETE ☧")
    print("="*60)
    print(f"""
    What we achieved:

    1. TERMINATION:
       - {len(results2_chirho)} solutions for composition query
       - {len(results3_chirho)} solutions for triple composition
       - No infinite loops!

    2. TENSOR CONSTRUCTION:
       - {len(TENSOR_STORE_CHIRHO.appendo_triples_chirho)} ground triples discovered
       - Built incrementally via queries
       - This IS the sparse Boolean tensor!

    3. MODE ANALYSIS:
       - Ground output → backward (finite splits)
       - Ground inputs → forward (single result)
       - Smart reordering runs ground goals first

    4. THE EQUIVALENCE:

       ┌─────────────────────────────────────────────────────┐
       │                                                     │
       │   miniKanren stream    ←──── TABLING ────→  Tensor │
       │                                                     │
       │   (lazy, infinite)          (bridge)     (sparse)  │
       │                                                     │
       │   Interleaving search   Mode analysis    COO format │
       │   May not terminate     Reordering       Always     │
       │                                          terminates │
       │                                                     │
       └─────────────────────────────────────────────────────┘

    5. HARDWARE PATH:
       - Sparse tensor = coordinate list = GPU/FPGA friendly
       - Boolean operations = 1-bit = massive parallelism
       - Contraction = sparse matmul = existing libraries

    Soli Deo Gloria ☧
    """)


if __name__ == "__main__":
    main()
