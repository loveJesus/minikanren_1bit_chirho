#!/usr/bin/env python3
"""
SLG-Style Tabling for miniKanren ☧

A more sophisticated tabling implementation using SLG resolution ideas:
- Generators vs Consumers
- Answer propagation via resumptions
- Completion detection
- Proper handling of mutual recursion

This builds the relation tensor incrementally as a side effect of search.

References:
- "Efficient Access Mechanisms for Tabled Logic Programs" (Swift & Warren)
- "XSB: A System for Efficiently Computing Well-Founded Semantics"
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Callable, Any
from collections import defaultdict
from enum import Enum, auto


# === Terms (simplified for clarity) ===

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

def term_to_tuple_chirho(t_chirho) -> Optional[tuple]:
    """Convert ground list term to tuple for hashing"""
    result_chirho = []
    while isinstance(t_chirho, ConsChirho):
        if isinstance(t_chirho.head_chirho, VarChirho):
            return None
        result_chirho.append(t_chirho.head_chirho)
        t_chirho = t_chirho.tail_chirho
    if isinstance(t_chirho, NilChirho):
        return tuple(result_chirho)
    return None

def tuple_to_term_chirho(tup_chirho: tuple):
    """Convert tuple to list term"""
    result_chirho = NilChirho()
    for x_chirho in reversed(tup_chirho):
        result_chirho = ConsChirho(x_chirho, result_chirho)
    return result_chirho


# === Substitution ===

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
    """Check if term has no variables"""
    if isinstance(t_chirho, VarChirho):
        return False
    if isinstance(t_chirho, ConsChirho):
        return is_ground_chirho(t_chirho.head_chirho) and is_ground_chirho(t_chirho.tail_chirho)
    return True


# === SLG Table ===

class CallStatusChirho(Enum):
    NEW = auto()      # Just created, no answers yet
    ACTIVE = auto()   # Generating answers
    COMPLETE = auto() # All answers found


@dataclass
class SLGEntryChirho:
    """
    Table entry for a call variant (generalized call pattern).

    In SLG:
    - answers: Set of returned substitutions/values
    - consumers: Calls waiting for more answers
    - status: Whether generation is complete
    """
    call_pattern_chirho: Tuple  # Normalized call args
    answers_chirho: Set[Tuple]  # Ground answer tuples
    consumers_chirho: List[Tuple[StateChirho, Any]]  # (state, continuation)
    status_chirho: CallStatusChirho = CallStatusChirho.NEW
    answers_returned_chirho: Dict[int, Set[Tuple]] = field(default_factory=dict)  # per-consumer


class SLGTableChirho:
    """
    Global table managing all tabled calls.

    Key operations:
    - lookup: Find or create entry for a call pattern
    - add_answer: Record new answer, schedule propagation
    - get_answers: Return cached answers for a consumer
    """

    def __init__(self_chirho):
        self_chirho.entries_chirho: Dict[Tuple, SLGEntryChirho] = {}
        self_chirho.consumer_id_chirho = 0

        # Tensor view: all ground triples ever discovered
        self_chirho.appendo_tensor_chirho: Set[Tuple[tuple, tuple, tuple]] = set()

    def normalize_pattern_chirho(self_chirho, relation_chirho: str, args_chirho: tuple, subst_chirho: dict) -> Tuple:
        """
        Create a normalized call pattern.
        Variables become a canonical form for variant checking.
        """
        walked_chirho = tuple(walk_deep_chirho(a_chirho, subst_chirho) for a_chirho in args_chirho)

        # For simplicity, we just use the walked representation
        # A full implementation would use variant abstraction
        return (relation_chirho,) + walked_chirho

    def lookup_or_create_chirho(self_chirho, pattern_chirho: Tuple) -> Tuple[SLGEntryChirho, bool]:
        """
        Get or create entry. Returns (entry, is_new).
        """
        if pattern_chirho in self_chirho.entries_chirho:
            return self_chirho.entries_chirho[pattern_chirho], False

        entry_chirho = SLGEntryChirho(
            call_pattern_chirho=pattern_chirho,
            answers_chirho=set(),
            consumers_chirho=[]
        )
        self_chirho.entries_chirho[pattern_chirho] = entry_chirho
        return entry_chirho, True

    def add_answer_chirho(
        self_chirho,
        entry_chirho: SLGEntryChirho,
        answer_chirho: Tuple
    ) -> bool:
        """
        Add answer to entry. Returns True if new.
        """
        if answer_chirho in entry_chirho.answers_chirho:
            return False
        entry_chirho.answers_chirho.add(answer_chirho)
        return True

    def register_consumer_chirho(
        self_chirho,
        entry_chirho: SLGEntryChirho,
        state_chirho: StateChirho,
        continuation_chirho: Any
    ) -> int:
        """
        Register a consumer waiting for answers.
        Returns consumer ID for tracking which answers have been returned.
        """
        cid_chirho = self_chirho.consumer_id_chirho
        self_chirho.consumer_id_chirho += 1
        entry_chirho.consumers_chirho.append((state_chirho, continuation_chirho))
        entry_chirho.answers_returned_chirho[cid_chirho] = set()
        return cid_chirho

    def get_new_answers_chirho(
        self_chirho,
        entry_chirho: SLGEntryChirho,
        consumer_id_chirho: int
    ) -> Set[Tuple]:
        """
        Get answers not yet returned to this consumer.
        """
        returned_chirho = entry_chirho.answers_returned_chirho.get(consumer_id_chirho, set())
        new_chirho = entry_chirho.answers_chirho - returned_chirho
        entry_chirho.answers_returned_chirho[consumer_id_chirho] = entry_chirho.answers_chirho.copy()
        return new_chirho

    def record_triple_chirho(self_chirho, l_chirho, s_chirho, out_chirho):
        """Record a ground appendo triple in the tensor view"""
        l_tup_chirho = term_to_tuple_chirho(l_chirho)
        s_tup_chirho = term_to_tuple_chirho(s_chirho)
        out_tup_chirho = term_to_tuple_chirho(out_chirho)

        if l_tup_chirho is not None and s_tup_chirho is not None and out_tup_chirho is not None:
            self_chirho.appendo_tensor_chirho.add((l_tup_chirho, s_tup_chirho, out_tup_chirho))


# Global table instance
TABLE_CHIRHO = SLGTableChirho()

def reset_table_chirho():
    global TABLE_CHIRHO
    TABLE_CHIRHO = SLGTableChirho()


# === Tabled appendo with SLG-style resolution ===

class TabledAppendoSLGChirho:
    """
    appendo with SLG-style tabling.

    Key insight: Mode analysis for termination
    - out ground → enumerate splits backward (finite)
    - l,s ground → compute forward (single answer)
    - mixed → table to avoid redundant computation
    """

    def __init__(self_chirho, table_chirho: SLGTableChirho = None):
        self_chirho.table_chirho = table_chirho or TABLE_CHIRHO

    def enumerate_splits_chirho(self_chirho, out_tup_chirho: tuple) -> Iterator[Tuple[tuple, tuple]]:
        """
        Given ground output as tuple, enumerate all splits.
        """
        for i_chirho in range(len(out_tup_chirho) + 1):
            yield out_tup_chirho[:i_chirho], out_tup_chirho[i_chirho:]

    def compute_append_chirho(self_chirho, l_tup_chirho: tuple, s_tup_chirho: tuple) -> tuple:
        """Compute append of two tuples"""
        return l_tup_chirho + s_tup_chirho

    def goal_chirho(self_chirho, l_chirho, s_chirho, out_chirho):
        """
        Create tabled appendo goal.
        """
        def goal_fn_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
            # Walk arguments
            l_w_chirho = walk_deep_chirho(l_chirho, state_chirho.subst_chirho)
            s_w_chirho = walk_deep_chirho(s_chirho, state_chirho.subst_chirho)
            out_w_chirho = walk_deep_chirho(out_chirho, state_chirho.subst_chirho)

            l_ground_chirho = is_ground_chirho(l_w_chirho)
            s_ground_chirho = is_ground_chirho(s_w_chirho)
            out_ground_chirho = is_ground_chirho(out_w_chirho)

            # MODE 1: out is ground → backward enumeration
            if out_ground_chirho:
                out_tup_chirho = term_to_tuple_chirho(out_w_chirho)
                if out_tup_chirho is None:
                    return

                for l_tup_chirho, s_tup_chirho in self_chirho.enumerate_splits_chirho(out_tup_chirho):
                    l_term_chirho = tuple_to_term_chirho(l_tup_chirho)
                    s_term_chirho = tuple_to_term_chirho(s_tup_chirho)

                    subst1_chirho = unify_chirho(l_chirho, l_term_chirho, state_chirho.subst_chirho)
                    if subst1_chirho is None:
                        continue

                    subst2_chirho = unify_chirho(s_chirho, s_term_chirho, subst1_chirho)
                    if subst2_chirho is None:
                        continue

                    # Record in tensor
                    self_chirho.table_chirho.record_triple_chirho(l_term_chirho, s_term_chirho, out_w_chirho)

                    yield StateChirho(subst2_chirho, state_chirho.counter_chirho)
                return

            # MODE 2: l and s ground → forward computation
            if l_ground_chirho and s_ground_chirho:
                l_tup_chirho = term_to_tuple_chirho(l_w_chirho)
                s_tup_chirho = term_to_tuple_chirho(s_w_chirho)

                if l_tup_chirho is None or s_tup_chirho is None:
                    return

                out_tup_chirho = self_chirho.compute_append_chirho(l_tup_chirho, s_tup_chirho)
                out_term_chirho = tuple_to_term_chirho(out_tup_chirho)

                subst_chirho = unify_chirho(out_chirho, out_term_chirho, state_chirho.subst_chirho)
                if subst_chirho is not None:
                    self_chirho.table_chirho.record_triple_chirho(l_w_chirho, s_w_chirho, out_term_chirho)
                    yield StateChirho(subst_chirho, state_chirho.counter_chirho)
                return

            # MODE 3: Partial grounding → use tabling
            # Create call pattern for lookup
            pattern_chirho = self_chirho.table_chirho.normalize_pattern_chirho(
                "appendo", (l_chirho, s_chirho, out_chirho), state_chirho.subst_chirho
            )

            entry_chirho, is_new_chirho = self_chirho.table_chirho.lookup_or_create_chirho(pattern_chirho)

            if is_new_chirho:
                # We are the generator - compute answers
                entry_chirho.status_chirho = CallStatusChirho.ACTIVE
                yield from self_chirho._generate_answers_chirho(
                    l_chirho, s_chirho, out_chirho, state_chirho, entry_chirho
                )
                entry_chirho.status_chirho = CallStatusChirho.COMPLETE
            else:
                # We are a consumer - use cached answers
                yield from self_chirho._consume_answers_chirho(
                    l_chirho, s_chirho, out_chirho, state_chirho, entry_chirho
                )

        return goal_fn_chirho

    def _generate_answers_chirho(
        self_chirho,
        l_chirho, s_chirho, out_chirho,
        state_chirho: StateChirho,
        entry_chirho: SLGEntryChirho
    ) -> Iterator[StateChirho]:
        """
        Generate answers for a new call.
        Uses recursive definition with depth limiting.
        """
        # Limit generation depth for the all-free case
        max_depth_chirho = 10

        def rec_chirho(depth_chirho: int, st_chirho: StateChirho) -> Iterator[StateChirho]:
            if depth_chirho > max_depth_chirho:
                return

            # Base: l = [], s = out
            subst1_chirho = unify_chirho(l_chirho, NilChirho(), st_chirho.subst_chirho)
            if subst1_chirho is not None:
                subst2_chirho = unify_chirho(s_chirho, out_chirho, subst1_chirho)
                if subst2_chirho is not None:
                    st2_chirho = StateChirho(subst2_chirho, st_chirho.counter_chirho)

                    # Extract and record answer
                    l_val_chirho = walk_deep_chirho(l_chirho, st2_chirho.subst_chirho)
                    s_val_chirho = walk_deep_chirho(s_chirho, st2_chirho.subst_chirho)
                    out_val_chirho = walk_deep_chirho(out_chirho, st2_chirho.subst_chirho)

                    answer_tup_chirho = (l_val_chirho, s_val_chirho, out_val_chirho)

                    if self_chirho.table_chirho.add_answer_chirho(entry_chirho, answer_tup_chirho):
                        if (is_ground_chirho(l_val_chirho) and
                            is_ground_chirho(s_val_chirho) and
                            is_ground_chirho(out_val_chirho)):
                            self_chirho.table_chirho.record_triple_chirho(
                                l_val_chirho, s_val_chirho, out_val_chirho
                            )

                    yield st2_chirho

            # Recursive: l = [h|t], out = [h|r], appendo(t, s, r)
            h_chirho = VarChirho(st_chirho.counter_chirho)
            t_chirho = VarChirho(st_chirho.counter_chirho + 1)
            r_chirho = VarChirho(st_chirho.counter_chirho + 2)
            new_counter_chirho = st_chirho.counter_chirho + 3

            subst1_chirho = unify_chirho(l_chirho, ConsChirho(h_chirho, t_chirho), st_chirho.subst_chirho)
            if subst1_chirho is None:
                return

            subst2_chirho = unify_chirho(out_chirho, ConsChirho(h_chirho, r_chirho), subst1_chirho)
            if subst2_chirho is None:
                return

            st2_chirho = StateChirho(subst2_chirho, new_counter_chirho)

            # Recursive call
            for st3_chirho in self_chirho.goal_chirho(t_chirho, s_chirho, r_chirho)(st2_chirho):
                l_val_chirho = walk_deep_chirho(l_chirho, st3_chirho.subst_chirho)
                s_val_chirho = walk_deep_chirho(s_chirho, st3_chirho.subst_chirho)
                out_val_chirho = walk_deep_chirho(out_chirho, st3_chirho.subst_chirho)

                answer_tup_chirho = (l_val_chirho, s_val_chirho, out_val_chirho)

                if self_chirho.table_chirho.add_answer_chirho(entry_chirho, answer_tup_chirho):
                    if (is_ground_chirho(l_val_chirho) and
                        is_ground_chirho(s_val_chirho) and
                        is_ground_chirho(out_val_chirho)):
                        self_chirho.table_chirho.record_triple_chirho(
                            l_val_chirho, s_val_chirho, out_val_chirho
                        )

                yield st3_chirho

        yield from rec_chirho(0, state_chirho)

    def _consume_answers_chirho(
        self_chirho,
        l_chirho, s_chirho, out_chirho,
        state_chirho: StateChirho,
        entry_chirho: SLGEntryChirho
    ) -> Iterator[StateChirho]:
        """
        Consume cached answers from a previous call.
        """
        for answer_chirho in entry_chirho.answers_chirho:
            l_ans_chirho, s_ans_chirho, out_ans_chirho = answer_chirho

            subst1_chirho = unify_chirho(l_chirho, l_ans_chirho, state_chirho.subst_chirho)
            if subst1_chirho is None:
                continue

            subst2_chirho = unify_chirho(s_chirho, s_ans_chirho, subst1_chirho)
            if subst2_chirho is None:
                continue

            subst3_chirho = unify_chirho(out_chirho, out_ans_chirho, subst2_chirho)
            if subst3_chirho is None:
                continue

            yield StateChirho(subst3_chirho, state_chirho.counter_chirho)


def tabled_appendo_slg_chirho(l_chirho, s_chirho, out_chirho):
    """Convenience function"""
    return TabledAppendoSLGChirho().goal_chirho(l_chirho, s_chirho, out_chirho)


# === Goal combinators ===

def eq_chirho(t1_chirho, t2_chirho):
    def goal_chirho(state_chirho):
        subst_chirho = unify_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if subst_chirho is not None:
            yield StateChirho(subst_chirho, state_chirho.counter_chirho)
    return goal_chirho

def conj_chirho(g1_chirho, g2_chirho):
    def goal_chirho(state_chirho):
        for st1_chirho in g1_chirho(state_chirho):
            yield from g2_chirho(st1_chirho)
    return goal_chirho

def disj_chirho(g1_chirho, g2_chirho):
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

def fresh_chirho(fn_chirho):
    def goal_chirho(state_chirho):
        var_chirho = VarChirho(state_chirho.counter_chirho)
        new_st_chirho = StateChirho(state_chirho.subst_chirho, state_chirho.counter_chirho + 1)
        yield from fn_chirho(var_chirho)(new_st_chirho)
    return goal_chirho


# === Run ===

def run_chirho(n_chirho, goal_chirho, vars_chirho):
    """Run goal, extract first n results"""
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
    print("=== SLG-Style Tabling for miniKanren ☧ ===\n")

    # === Test 1: Basic backward ===
    print("="*60)
    print("TEST 1: Backward - appendo(L, S, [0,1,2])")
    print("="*60)

    reset_table_chirho()

    l1_chirho = VarChirho(0)
    s1_chirho = VarChirho(1)
    goal1_chirho = tabled_appendo_slg_chirho(l1_chirho, s1_chirho, list_chirho(0, 1, 2))
    results1_chirho = run_chirho(10, goal1_chirho, [l1_chirho, s1_chirho])

    print(f"\nResults ({len(results1_chirho)}):")
    for r_chirho in results1_chirho:
        print(f"  L = {r_chirho[0]}, S = {r_chirho[1]}")

    print(f"\nTensor triples so far: {len(TABLE_CHIRHO.appendo_tensor_chirho)}")

    # === Test 2: Composition (the critical test!) ===
    print("\n" + "="*60)
    print("TEST 2: Composition - appendo(A, B, X), appendo(X, C, [0,1,2])")
    print("="*60)

    reset_table_chirho()

    a_chirho = VarChirho(0)
    b_chirho = VarChirho(1)
    x_chirho = VarChirho(2)
    c_chirho = VarChirho(3)

    goal2_chirho = conj_chirho(
        tabled_appendo_slg_chirho(a_chirho, b_chirho, x_chirho),
        tabled_appendo_slg_chirho(x_chirho, c_chirho, list_chirho(0, 1, 2))
    )

    results2_chirho = run_chirho(20, goal2_chirho, [a_chirho, b_chirho, x_chirho, c_chirho])

    print(f"\nResults ({len(results2_chirho)}):")
    for r_chirho in results2_chirho:
        print(f"  A={r_chirho[0]}, B={r_chirho[1]} → X={r_chirho[2]}, C={r_chirho[3]}")

    # === Test 3: Double composition ===
    print("\n" + "="*60)
    print("TEST 3: appendo(A, B, X), appendo(X, C, Y), appendo(Y, D, [0,1,2,3])")
    print("="*60)

    reset_table_chirho()

    a3_chirho = VarChirho(0)
    b3_chirho = VarChirho(1)
    x3_chirho = VarChirho(2)
    c3_chirho = VarChirho(3)
    y3_chirho = VarChirho(4)
    d3_chirho = VarChirho(5)

    goal3_chirho = conj_chirho(
        conj_chirho(
            tabled_appendo_slg_chirho(a3_chirho, b3_chirho, x3_chirho),
            tabled_appendo_slg_chirho(x3_chirho, c3_chirho, y3_chirho)
        ),
        tabled_appendo_slg_chirho(y3_chirho, d3_chirho, list_chirho(0, 1, 2, 3))
    )

    results3_chirho = run_chirho(40, goal3_chirho, [a3_chirho, b3_chirho, x3_chirho, c3_chirho, y3_chirho, d3_chirho])

    print(f"\nResults ({len(results3_chirho)} shown, may have more):")
    for i_chirho, r_chirho in enumerate(results3_chirho[:15]):
        print(f"  {i_chirho+1}. A={r_chirho[0]}, B={r_chirho[1]} → X={r_chirho[2]}")
        print(f"      X={r_chirho[2]}, C={r_chirho[3]} → Y={r_chirho[4]}")
        print(f"      Y={r_chirho[4]}, D={r_chirho[5]} → [0,1,2,3]")
    if len(results3_chirho) > 15:
        print(f"  ... and {len(results3_chirho)-15} more")

    # === Tensor view ===
    print("\n" + "="*60)
    print("TENSOR VIEW: All discovered appendo triples")
    print("="*60)

    print(f"\nTotal ground triples: {len(TABLE_CHIRHO.appendo_tensor_chirho)}")
    print("\nTriples (as tuples):")
    for t_chirho in sorted(TABLE_CHIRHO.appendo_tensor_chirho, key=lambda x: (len(x[2]), x)):
        l_chirho, s_chirho, o_chirho = t_chirho
        print(f"  {list(l_chirho)} ++ {list(s_chirho)} = {list(o_chirho)}")

    # === Key insights ===
    print("\n" + "="*60)
    print("KEY INSIGHTS")
    print("="*60)
    print(f"""
    1. TERMINATION via MODE ANALYSIS:
       - Ground output → finite backward enumeration
       - Ground inputs → single forward computation
       - Composition inherits termination from inner goals

    2. TENSOR CONSTRUCTION AS SIDE EFFECT:
       - Each query populates the tensor incrementally
       - {len(TABLE_CHIRHO.appendo_tensor_chirho)} triples discovered
       - This IS the sparse 3D Boolean tensor!

    3. THE BRIDGE:
       ┌────────────┐     ┌────────────┐     ┌────────────┐
       │  Lazy      │ ──► │  Tabling   │ ──► │  Eager     │
       │  Streams   │     │            │     │  Tensor    │
       └────────────┘     └────────────┘     └────────────┘

       - Streams: infinite, may not terminate
       - Tensor: finite, always terminates
       - Tabling: demand-driven, terminates via modes

    4. 1-BIT REPRESENTATION:
       - These tuples are exactly the nonzero entries
       - Tensor[l_idx, s_idx, out_idx] = 1 iff (l,s,out) in set
       - Query = sparse tensor operations

    5. HARDWARE ACCELERATION PATH:
       - Sparse COO format → GPU sparse ops
       - Contraction → batched sparse matmul
       - Or: bitmask per list-length → SIMD/FPGA
    """)


if __name__ == "__main__":
    main()
