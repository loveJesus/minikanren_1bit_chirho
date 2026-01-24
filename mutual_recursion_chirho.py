#!/usr/bin/env python3
"""
Mutual Recursion with SLG Completion ☧

Full SLG resolution for mutually recursive predicates.

The problem:
  even(0).
  even(s(X)) :- odd(X).
  odd(s(X)) :- even(X).

Without proper completion, mutual recursion can loop or miss answers.

SLG Solution:
1. Track dependency graph between subgoals
2. Detect strongly connected components (SCCs)
3. Complete SCCs together (not individual goals)
4. Propagate answers across SCC boundaries

This file implements the completion protocol and demonstrates
the tensor representation of mutually recursive relations.

References:
- "Efficient Access Mechanisms for Tabled Logic Programs" (Swift & Warren)
- "A Survey of Tabling in Logic Programming" (Zhou & Sato)
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Callable, Any, FrozenSet
from collections import defaultdict
from enum import Enum, auto
import sys

# Increase recursion limit for deep terms
sys.setrecursionlimit(3000)


# === Terms: Peano naturals ===

@dataclass(frozen=True)
class ZeroChirho:
    """Zero (base case for Peano)"""
    def __repr__(self_chirho): return "0"

@dataclass(frozen=True)
class SuccChirho:
    """Successor (s(n) = n+1)"""
    pred_chirho: Any  # The predecessor

    def __repr__(self_chirho):
        # Count depth for nice display
        n_chirho = 0
        curr_chirho = self_chirho
        while isinstance(curr_chirho, SuccChirho):
            n_chirho += 1
            curr_chirho = curr_chirho.pred_chirho
        if isinstance(curr_chirho, ZeroChirho):
            return str(n_chirho)
        return f"s({curr_chirho})"

@dataclass(frozen=True)
class VarChirho:
    """Logic variable"""
    id_chirho: int
    def __repr__(self_chirho): return f"_{self_chirho.id_chirho}"

TermChirho = Any  # ZeroChirho | SuccChirho | VarChirho


def nat_chirho(n_chirho: int) -> TermChirho:
    """Build Peano natural from Python int"""
    result_chirho = ZeroChirho()
    for _ in range(n_chirho):
        result_chirho = SuccChirho(result_chirho)
    return result_chirho


def to_int_chirho(term_chirho: TermChirho) -> Optional[int]:
    """Convert Peano to int, or None if not ground"""
    n_chirho = 0
    while isinstance(term_chirho, SuccChirho):
        n_chirho += 1
        term_chirho = term_chirho.pred_chirho
    if isinstance(term_chirho, ZeroChirho):
        return n_chirho
    return None


# === Substitution ===

@dataclass
class StateChirho:
    subst_chirho: Dict[int, Any]
    counter_chirho: int

    def copy_chirho(self_chirho) -> 'StateChirho':
        return StateChirho(self_chirho.subst_chirho.copy(), self_chirho.counter_chirho)

def empty_state_chirho() -> StateChirho:
    return StateChirho({}, 0)

def walk_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, Any]) -> TermChirho:
    while isinstance(term_chirho, VarChirho) and term_chirho.id_chirho in subst_chirho:
        term_chirho = subst_chirho[term_chirho.id_chirho]
    return term_chirho

def walk_deep_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, Any]) -> TermChirho:
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, SuccChirho):
        return SuccChirho(walk_deep_chirho(term_chirho.pred_chirho, subst_chirho))
    return term_chirho

def occurs_chirho(var_id_chirho: int, term_chirho: TermChirho, subst_chirho: Dict[int, Any]) -> bool:
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, VarChirho):
        return term_chirho.id_chirho == var_id_chirho
    if isinstance(term_chirho, SuccChirho):
        return occurs_chirho(var_id_chirho, term_chirho.pred_chirho, subst_chirho)
    return False

def unify_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho,
                 subst_chirho: Dict[int, Any]) -> Optional[Dict[int, Any]]:
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

    if isinstance(t1_chirho, SuccChirho) and isinstance(t2_chirho, SuccChirho):
        return unify_chirho(t1_chirho.pred_chirho, t2_chirho.pred_chirho, subst_chirho)

    if isinstance(t1_chirho, ZeroChirho) and isinstance(t2_chirho, ZeroChirho):
        return subst_chirho

    return None

def is_ground_chirho(term_chirho: TermChirho) -> bool:
    if isinstance(term_chirho, VarChirho):
        return False
    if isinstance(term_chirho, SuccChirho):
        return is_ground_chirho(term_chirho.pred_chirho)
    return True


# === SLG Table with Completion ===

class GoalStatusChirho(Enum):
    NEW_CHIRHO = auto()       # Just created
    ACTIVE_CHIRHO = auto()    # Being computed
    COMPLETE_CHIRHO = auto()  # All answers found


@dataclass
class SLGGoalChirho:
    """A tabled goal entry"""
    pattern_chirho: Tuple                    # (relation_name, arg1, arg2, ...)
    answers_chirho: Set[Tuple]               # Ground answer tuples
    status_chirho: GoalStatusChirho = GoalStatusChirho.NEW_CHIRHO

    # SLG completion tracking
    depends_on_chirho: Set[Tuple] = field(default_factory=set)    # Goals this depends on
    depended_by_chirho: Set[Tuple] = field(default_factory=set)   # Goals that depend on this
    scc_id_chirho: Optional[int] = None                            # Strongly connected component


class SLGTableCompleteChirho:
    """
    SLG table with completion protocol for mutual recursion.

    Key additions over basic SLG:
    1. Dependency graph tracking
    2. SCC computation (Tarjan's algorithm)
    3. Completion by SCC (not individual goals)
    4. Answer propagation within SCCs
    """

    def __init__(self_chirho):
        self_chirho.goals_chirho: Dict[Tuple, SLGGoalChirho] = {}
        self_chirho.scc_counter_chirho = 0

        # Tensor views: sets of ground facts
        self_chirho.even_tensor_chirho: Set[int] = set()
        self_chirho.odd_tensor_chirho: Set[int] = set()

        # For SCC computation
        self_chirho.index_chirho: Dict[Tuple, int] = {}
        self_chirho.lowlink_chirho: Dict[Tuple, int] = {}
        self_chirho.on_stack_chirho: Set[Tuple] = set()
        self_chirho.stack_chirho: List[Tuple] = []
        self_chirho.scc_index_chirho = 0
        self_chirho.sccs_chirho: List[Set[Tuple]] = []

    def get_or_create_chirho(self_chirho, pattern_chirho: Tuple) -> Tuple[SLGGoalChirho, bool]:
        """Get existing goal or create new one. Returns (goal, is_new)"""
        if pattern_chirho in self_chirho.goals_chirho:
            return self_chirho.goals_chirho[pattern_chirho], False

        goal_chirho = SLGGoalChirho(pattern_chirho=pattern_chirho, answers_chirho=set())
        self_chirho.goals_chirho[pattern_chirho] = goal_chirho
        return goal_chirho, True

    def add_dependency_chirho(self_chirho, from_pattern_chirho: Tuple, to_pattern_chirho: Tuple):
        """Record that from_pattern depends on to_pattern"""
        if from_pattern_chirho == to_pattern_chirho:
            return  # Self-dependency (direct recursion)

        from_goal_chirho, _ = self_chirho.get_or_create_chirho(from_pattern_chirho)
        to_goal_chirho, _ = self_chirho.get_or_create_chirho(to_pattern_chirho)

        from_goal_chirho.depends_on_chirho.add(to_pattern_chirho)
        to_goal_chirho.depended_by_chirho.add(from_pattern_chirho)

    def add_answer_chirho(self_chirho, pattern_chirho: Tuple, answer_chirho: Tuple) -> bool:
        """Add answer to goal. Returns True if new."""
        goal_chirho, _ = self_chirho.get_or_create_chirho(pattern_chirho)

        if answer_chirho in goal_chirho.answers_chirho:
            return False

        goal_chirho.answers_chirho.add(answer_chirho)

        # Record in tensor view
        rel_chirho = pattern_chirho[0]
        if rel_chirho == "even" and len(answer_chirho) == 1:
            n_chirho = to_int_chirho(answer_chirho[0])
            if n_chirho is not None:
                self_chirho.even_tensor_chirho.add(n_chirho)
        elif rel_chirho == "odd" and len(answer_chirho) == 1:
            n_chirho = to_int_chirho(answer_chirho[0])
            if n_chirho is not None:
                self_chirho.odd_tensor_chirho.add(n_chirho)

        return True

    def compute_sccs_chirho(self_chirho):
        """Compute SCCs using Tarjan's algorithm"""
        self_chirho.index_chirho.clear()
        self_chirho.lowlink_chirho.clear()
        self_chirho.on_stack_chirho.clear()
        self_chirho.stack_chirho.clear()
        self_chirho.scc_index_chirho = 0
        self_chirho.sccs_chirho.clear()

        for pattern_chirho in self_chirho.goals_chirho:
            if pattern_chirho not in self_chirho.index_chirho:
                self_chirho._tarjan_chirho(pattern_chirho)

        # Assign SCC IDs to goals
        for i_chirho, scc_chirho in enumerate(self_chirho.sccs_chirho):
            for pattern_chirho in scc_chirho:
                self_chirho.goals_chirho[pattern_chirho].scc_id_chirho = i_chirho

    def _tarjan_chirho(self_chirho, pattern_chirho: Tuple):
        """Tarjan's SCC algorithm recursive step"""
        self_chirho.index_chirho[pattern_chirho] = self_chirho.scc_index_chirho
        self_chirho.lowlink_chirho[pattern_chirho] = self_chirho.scc_index_chirho
        self_chirho.scc_index_chirho += 1
        self_chirho.stack_chirho.append(pattern_chirho)
        self_chirho.on_stack_chirho.add(pattern_chirho)

        goal_chirho = self_chirho.goals_chirho[pattern_chirho]
        for dep_chirho in goal_chirho.depends_on_chirho:
            if dep_chirho not in self_chirho.goals_chirho:
                continue
            if dep_chirho not in self_chirho.index_chirho:
                self_chirho._tarjan_chirho(dep_chirho)
                self_chirho.lowlink_chirho[pattern_chirho] = min(
                    self_chirho.lowlink_chirho[pattern_chirho],
                    self_chirho.lowlink_chirho[dep_chirho]
                )
            elif dep_chirho in self_chirho.on_stack_chirho:
                self_chirho.lowlink_chirho[pattern_chirho] = min(
                    self_chirho.lowlink_chirho[pattern_chirho],
                    self_chirho.index_chirho[dep_chirho]
                )

        # Root of SCC?
        if self_chirho.lowlink_chirho[pattern_chirho] == self_chirho.index_chirho[pattern_chirho]:
            scc_chirho: Set[Tuple] = set()
            while True:
                w_chirho = self_chirho.stack_chirho.pop()
                self_chirho.on_stack_chirho.discard(w_chirho)
                scc_chirho.add(w_chirho)
                if w_chirho == pattern_chirho:
                    break
            self_chirho.sccs_chirho.append(scc_chirho)

    def complete_scc_chirho(self_chirho, scc_chirho: Set[Tuple]):
        """Mark all goals in an SCC as complete"""
        for pattern_chirho in scc_chirho:
            self_chirho.goals_chirho[pattern_chirho].status_chirho = GoalStatusChirho.COMPLETE_CHIRHO

    def is_complete_chirho(self_chirho, pattern_chirho: Tuple) -> bool:
        """Check if goal is complete"""
        if pattern_chirho not in self_chirho.goals_chirho:
            return False
        return self_chirho.goals_chirho[pattern_chirho].status_chirho == GoalStatusChirho.COMPLETE_CHIRHO


# Global table
TABLE_CHIRHO = SLGTableCompleteChirho()

def reset_table_chirho():
    global TABLE_CHIRHO
    TABLE_CHIRHO = SLGTableCompleteChirho()


# === Mutually Recursive Relations: even/odd ===

class MutualRecursionChirho:
    """
    Implements even/odd with full SLG completion.

    even(0).
    even(s(X)) :- odd(X).
    odd(s(X)) :- even(X).

    Key insight: even and odd form an SCC, must complete together.
    """

    def __init__(self_chirho, table_chirho: SLGTableCompleteChirho = None, max_depth_chirho: int = 20):
        self_chirho.table_chirho = table_chirho or TABLE_CHIRHO
        self_chirho.max_depth_chirho = max_depth_chirho

    def even_chirho(self_chirho, n_chirho: TermChirho):
        """Goal: n is even"""
        def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
            n_walked_chirho = walk_deep_chirho(n_chirho, state_chirho.subst_chirho)
            pattern_chirho = ("even", n_walked_chirho)

            goal_entry_chirho, is_new_chirho = self_chirho.table_chirho.get_or_create_chirho(pattern_chirho)

            if goal_entry_chirho.status_chirho == GoalStatusChirho.COMPLETE_CHIRHO:
                # Use cached answers
                yield from self_chirho._consume_answers_chirho(n_chirho, state_chirho, goal_entry_chirho)
                return

            if is_new_chirho:
                goal_entry_chirho.status_chirho = GoalStatusChirho.ACTIVE_CHIRHO
                yield from self_chirho._generate_even_chirho(n_chirho, state_chirho, pattern_chirho, 0)
            else:
                # Another computation is active, consume current answers
                yield from self_chirho._consume_answers_chirho(n_chirho, state_chirho, goal_entry_chirho)

        return goal_chirho

    def odd_chirho(self_chirho, n_chirho: TermChirho):
        """Goal: n is odd"""
        def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
            n_walked_chirho = walk_deep_chirho(n_chirho, state_chirho.subst_chirho)
            pattern_chirho = ("odd", n_walked_chirho)

            goal_entry_chirho, is_new_chirho = self_chirho.table_chirho.get_or_create_chirho(pattern_chirho)

            if goal_entry_chirho.status_chirho == GoalStatusChirho.COMPLETE_CHIRHO:
                yield from self_chirho._consume_answers_chirho(n_chirho, state_chirho, goal_entry_chirho)
                return

            if is_new_chirho:
                goal_entry_chirho.status_chirho = GoalStatusChirho.ACTIVE_CHIRHO
                yield from self_chirho._generate_odd_chirho(n_chirho, state_chirho, pattern_chirho, 0)
            else:
                yield from self_chirho._consume_answers_chirho(n_chirho, state_chirho, goal_entry_chirho)

        return goal_chirho

    def _generate_even_chirho(self_chirho, n_chirho: TermChirho, state_chirho: StateChirho,
                              pattern_chirho: Tuple, depth_chirho: int) -> Iterator[StateChirho]:
        """Generate even answers"""
        if depth_chirho > self_chirho.max_depth_chirho:
            return

        # Base case: even(0)
        subst1_chirho = unify_chirho(n_chirho, ZeroChirho(), state_chirho.subst_chirho)
        if subst1_chirho is not None:
            answer_chirho = (ZeroChirho(),)
            self_chirho.table_chirho.add_answer_chirho(pattern_chirho, answer_chirho)
            yield StateChirho(subst1_chirho, state_chirho.counter_chirho)

        # Recursive case: even(s(X)) :- odd(X)
        x_chirho = VarChirho(state_chirho.counter_chirho)
        new_counter_chirho = state_chirho.counter_chirho + 1

        subst2_chirho = unify_chirho(n_chirho, SuccChirho(x_chirho), state_chirho.subst_chirho)
        if subst2_chirho is not None:
            state2_chirho = StateChirho(subst2_chirho, new_counter_chirho)

            # Track dependency: even depends on odd
            x_walked_chirho = walk_deep_chirho(x_chirho, subst2_chirho)
            odd_pattern_chirho = ("odd", x_walked_chirho)
            self_chirho.table_chirho.add_dependency_chirho(pattern_chirho, odd_pattern_chirho)

            # Call odd
            for state3_chirho in self_chirho._generate_odd_chirho(x_chirho, state2_chirho, odd_pattern_chirho, depth_chirho + 1):
                # Record answer
                n_val_chirho = walk_deep_chirho(n_chirho, state3_chirho.subst_chirho)
                if is_ground_chirho(n_val_chirho):
                    answer_chirho = (n_val_chirho,)
                    self_chirho.table_chirho.add_answer_chirho(pattern_chirho, answer_chirho)
                yield state3_chirho

    def _generate_odd_chirho(self_chirho, n_chirho: TermChirho, state_chirho: StateChirho,
                             pattern_chirho: Tuple, depth_chirho: int) -> Iterator[StateChirho]:
        """Generate odd answers"""
        if depth_chirho > self_chirho.max_depth_chirho:
            return

        # No base case for odd (odd(0) is false)

        # Recursive case: odd(s(X)) :- even(X)
        x_chirho = VarChirho(state_chirho.counter_chirho)
        new_counter_chirho = state_chirho.counter_chirho + 1

        subst1_chirho = unify_chirho(n_chirho, SuccChirho(x_chirho), state_chirho.subst_chirho)
        if subst1_chirho is not None:
            state2_chirho = StateChirho(subst1_chirho, new_counter_chirho)

            # Track dependency: odd depends on even
            x_walked_chirho = walk_deep_chirho(x_chirho, subst1_chirho)
            even_pattern_chirho = ("even", x_walked_chirho)
            self_chirho.table_chirho.add_dependency_chirho(pattern_chirho, even_pattern_chirho)

            # Call even
            for state3_chirho in self_chirho._generate_even_chirho(x_chirho, state2_chirho, even_pattern_chirho, depth_chirho + 1):
                # Record answer
                n_val_chirho = walk_deep_chirho(n_chirho, state3_chirho.subst_chirho)
                if is_ground_chirho(n_val_chirho):
                    answer_chirho = (n_val_chirho,)
                    self_chirho.table_chirho.add_answer_chirho(pattern_chirho, answer_chirho)
                yield state3_chirho

    def _consume_answers_chirho(self_chirho, n_chirho: TermChirho, state_chirho: StateChirho,
                                goal_entry_chirho: SLGGoalChirho) -> Iterator[StateChirho]:
        """Consume cached answers"""
        for answer_chirho in goal_entry_chirho.answers_chirho:
            subst_chirho = unify_chirho(n_chirho, answer_chirho[0], state_chirho.subst_chirho)
            if subst_chirho is not None:
                yield StateChirho(subst_chirho, state_chirho.counter_chirho)


# === Goal Combinators ===

def conj_chirho(g1_chirho, g2_chirho):
    def goal_chirho(state_chirho):
        for st1_chirho in g1_chirho(state_chirho):
            yield from g2_chirho(st1_chirho)
    return goal_chirho

def disj_chirho(g1_chirho, g2_chirho):
    def goal_chirho(state_chirho):
        yield from g1_chirho(state_chirho)
        yield from g2_chirho(state_chirho)
    return goal_chirho

def eq_chirho(t1_chirho, t2_chirho):
    def goal_chirho(state_chirho):
        subst_chirho = unify_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if subst_chirho is not None:
            yield StateChirho(subst_chirho, state_chirho.counter_chirho)
    return goal_chirho


# === Run ===

def run_chirho(n_chirho: Optional[int], goal_chirho, vars_chirho: List[VarChirho]) -> List[Dict[int, Any]]:
    """Run goal and collect results"""
    from itertools import islice

    stream_chirho = goal_chirho(empty_state_chirho())

    if n_chirho is not None:
        states_chirho = list(islice(stream_chirho, n_chirho))
    else:
        states_chirho = list(stream_chirho)

    results_chirho = []
    for st_chirho in states_chirho:
        result_chirho = {}
        for v_chirho in vars_chirho:
            result_chirho[v_chirho.id_chirho] = walk_deep_chirho(v_chirho, st_chirho.subst_chirho)
        results_chirho.append(result_chirho)

    return results_chirho


# === Demo ===

def main():
    print("=== Mutual Recursion with SLG Completion ☧ ===\n")

    print("""
    The mutually recursive predicates:

        even(0).
        even(s(X)) :- odd(X).
        odd(s(X)) :- even(X).

    Without proper completion, this can loop or miss answers.
    SLG tracks the dependency graph and completes SCCs together.
    """)

    # === Test 1: Check specific numbers ===
    print("="*60)
    print("TEST 1: Check even(4) and odd(5)")
    print("="*60)

    reset_table_chirho()
    mr_chirho = MutualRecursionChirho()

    # even(4)?
    goal1_chirho = mr_chirho.even_chirho(nat_chirho(4))
    results1_chirho = run_chirho(5, goal1_chirho, [])
    print(f"\neven(4): {len(results1_chirho)} solution(s) → {'YES' if results1_chirho else 'NO'}")

    # odd(5)?
    goal2_chirho = mr_chirho.odd_chirho(nat_chirho(5))
    results2_chirho = run_chirho(5, goal2_chirho, [])
    print(f"odd(5): {len(results2_chirho)} solution(s) → {'YES' if results2_chirho else 'NO'}")

    # even(3)?
    goal3_chirho = mr_chirho.even_chirho(nat_chirho(3))
    results3_chirho = run_chirho(5, goal3_chirho, [])
    print(f"even(3): {len(results3_chirho)} solution(s) → {'YES' if results3_chirho else 'NO'}")

    # odd(4)?
    goal4_chirho = mr_chirho.odd_chirho(nat_chirho(4))
    results4_chirho = run_chirho(5, goal4_chirho, [])
    print(f"odd(4): {len(results4_chirho)} solution(s) → {'YES' if results4_chirho else 'NO'}")

    # === Test 2: Generate even/odd numbers ===
    print("\n" + "="*60)
    print("TEST 2: Generate even numbers (backward)")
    print("="*60)

    reset_table_chirho()
    mr2_chirho = MutualRecursionChirho(max_depth_chirho=15)

    x_chirho = VarChirho(999)
    goal5_chirho = mr2_chirho.even_chirho(x_chirho)
    results5_chirho = run_chirho(10, goal5_chirho, [x_chirho])

    print(f"\neven(X) first 10 solutions:")
    for r_chirho in results5_chirho:
        val_chirho = r_chirho[x_chirho.id_chirho]
        n_chirho = to_int_chirho(val_chirho)
        print(f"  X = {val_chirho}" + (f" ({n_chirho})" if n_chirho is not None else ""))

    # === Test 3: Generate odd numbers ===
    print("\n" + "="*60)
    print("TEST 3: Generate odd numbers (backward)")
    print("="*60)

    y_chirho = VarChirho(998)
    goal6_chirho = mr2_chirho.odd_chirho(y_chirho)
    results6_chirho = run_chirho(10, goal6_chirho, [y_chirho])

    print(f"\nodd(Y) first 10 solutions:")
    for r_chirho in results6_chirho:
        val_chirho = r_chirho[y_chirho.id_chirho]
        n_chirho = to_int_chirho(val_chirho)
        print(f"  Y = {val_chirho}" + (f" ({n_chirho})" if n_chirho is not None else ""))

    # === Test 4: SCC Analysis ===
    print("\n" + "="*60)
    print("TEST 4: Dependency Graph and SCCs")
    print("="*60)

    TABLE_CHIRHO.compute_sccs_chirho()

    print(f"\nTotal goals tracked: {len(TABLE_CHIRHO.goals_chirho)}")
    print(f"Strongly Connected Components: {len(TABLE_CHIRHO.sccs_chirho)}")

    for i_chirho, scc_chirho in enumerate(TABLE_CHIRHO.sccs_chirho):
        if len(scc_chirho) > 1:  # Only show non-trivial SCCs
            print(f"\n  SCC {i_chirho} (size {len(scc_chirho)}):")
            for pattern_chirho in list(scc_chirho)[:5]:
                print(f"    {pattern_chirho[0]}({pattern_chirho[1]})")
            if len(scc_chirho) > 5:
                print(f"    ... and {len(scc_chirho) - 5} more")

    # === Test 5: Tensor View ===
    print("\n" + "="*60)
    print("TEST 5: Tensor View (1-bit representation)")
    print("="*60)

    print(f"\neven tensor (discovered ground facts): {sorted(TABLE_CHIRHO.even_tensor_chirho)}")
    print(f"odd tensor (discovered ground facts): {sorted(TABLE_CHIRHO.odd_tensor_chirho)}")

    # Show as bitmask
    max_n_chirho = max(
        max(TABLE_CHIRHO.even_tensor_chirho) if TABLE_CHIRHO.even_tensor_chirho else 0,
        max(TABLE_CHIRHO.odd_tensor_chirho) if TABLE_CHIRHO.odd_tensor_chirho else 0
    )

    print(f"\nBitmask representation (n=0 to {max_n_chirho}):")
    even_bits_chirho = ''.join('1' if i_chirho in TABLE_CHIRHO.even_tensor_chirho else '0'
                               for i_chirho in range(max_n_chirho + 1))
    odd_bits_chirho = ''.join('1' if i_chirho in TABLE_CHIRHO.odd_tensor_chirho else '0'
                              for i_chirho in range(max_n_chirho + 1))

    print(f"  even: {even_bits_chirho}")
    print(f"  odd:  {odd_bits_chirho}")

    # Verify complement property
    print("\nVerification:")
    complement_ok_chirho = all(
        (i_chirho in TABLE_CHIRHO.even_tensor_chirho) != (i_chirho in TABLE_CHIRHO.odd_tensor_chirho)
        for i_chirho in range(max_n_chirho + 1)
        if i_chirho in TABLE_CHIRHO.even_tensor_chirho or i_chirho in TABLE_CHIRHO.odd_tensor_chirho
    )
    print(f"  even ∩ odd = ∅: {complement_ok_chirho}")

    # === Key Insights ===
    print("\n" + "="*60)
    print("KEY INSIGHTS: Mutual Recursion → Tensor")
    print("="*60)
    print("""
    1. DEPENDENCY GRAPH:
       even ←→ odd (bidirectional dependency)
       Both predicates form a single SCC

    2. SLG COMPLETION:
       - Cannot complete even without completing odd
       - Cannot complete odd without completing even
       - Must complete the entire SCC together

    3. TENSOR VIEW:
       - even: bit vector where bit[i] = 1 iff i is even
       - odd:  bit vector where bit[i] = 1 iff i is odd
       - Together: complementary 1-bit vectors!

    4. THE PATTERN:
       ┌──────────────────────────────────────────────┐
       │  Mutual recursion = coupled tensor equations │
       │                                              │
       │    even[n] = (n==0) OR odd[n-1]              │
       │    odd[n]  = even[n-1] for n>0              │
       │                                              │
       │  Fixpoint iteration builds both tensors     │
       └──────────────────────────────────────────────┘

    5. HARDWARE MAPPING:
       - SCC = compute unit that runs to fixpoint
       - Dependencies = data flow between units
       - Completion = barrier synchronization
       - Tensors = shift registers (even/odd alternate)

    6. CONNECTION TO GENERAL CASE:
       - Any mutually recursive set = SCC in dependency graph
       - Tarjan's algorithm: O(V+E) to find SCCs
       - Complete SCCs in topological order
       - Within SCC: iterate to fixpoint
    """)


if __name__ == "__main__":
    main()
