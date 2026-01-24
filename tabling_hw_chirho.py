#!/usr/bin/env python3
"""
Hardware-Ready Tabling for miniKanren ☧

Proper casing: ClassChirho, function_chirho, CONSTANT_CHIRHO

This version is designed with hardware mapping in mind:
- Occurs check as matrix reachability
- Substitution as bit vectors
- Term store with integer IDs
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Any, FrozenSet
from collections import defaultdict
from enum import Enum, auto

try:
    import numpy as np
    HAS_NUMPY_CHIRHO = True
except ImportError:
    HAS_NUMPY_CHIRHO = False
    np = None


# === Constants ===

MAX_TERMS_CHIRHO = 256      # Maximum terms in store
MAX_VARS_CHIRHO = 64        # Maximum variables
MAX_WORLDS_CHIRHO = 1024    # Maximum parallel search states


# === Terms with proper casing ===

@dataclass(frozen=True)
class VarChirho:
    """Logic variable"""
    id_chirho: int
    def __repr__(self_chirho): return f"_{self_chirho.id_chirho}"


@dataclass(frozen=True)
class NilChirho:
    """Empty list"""
    def __repr__(self_chirho): return "[]"


@dataclass(frozen=True)
class ConsChirho:
    """List cons cell"""
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


# Type alias
TermChirho = Any  # Union[int, VarChirho, NilChirho, ConsChirho]


def list_chirho(*elems_chirho) -> TermChirho:
    """Build a list term"""
    result_chirho = NilChirho()
    for e_chirho in reversed(elems_chirho):
        result_chirho = ConsChirho(e_chirho, result_chirho)
    return result_chirho


def term_to_pylist_chirho(t_chirho: TermChirho) -> Optional[list]:
    """Convert ground list term to Python list"""
    result_chirho = []
    while isinstance(t_chirho, ConsChirho):
        if isinstance(t_chirho.head_chirho, VarChirho):
            return None
        result_chirho.append(t_chirho.head_chirho)
        t_chirho = t_chirho.tail_chirho
    if isinstance(t_chirho, NilChirho):
        return result_chirho
    return None


def pylist_to_term_chirho(lst_chirho: list) -> TermChirho:
    """Convert Python list to term"""
    result_chirho = NilChirho()
    for x_chirho in reversed(lst_chirho):
        result_chirho = ConsChirho(x_chirho, result_chirho)
    return result_chirho


# === Term Store (Hash Consing) ===

class TermStoreChirho:
    """
    Hash-consed term storage.

    Hardware mapping: Content-addressable memory (CAM)
    - term → ID: hash lookup
    - ID → term: direct addressing
    """

    def __init__(self_chirho):
        self_chirho.term_to_id_chirho: Dict[TermChirho, int] = {}
        self_chirho.id_to_term_chirho: List[TermChirho] = []
        # Pre-intern nil
        self_chirho.NIL_ID_CHIRHO = self_chirho.intern_chirho(NilChirho())

    def intern_chirho(self_chirho, term_chirho: TermChirho) -> int:
        """Get or create ID for term"""
        if term_chirho in self_chirho.term_to_id_chirho:
            return self_chirho.term_to_id_chirho[term_chirho]

        id_chirho = len(self_chirho.id_to_term_chirho)
        if id_chirho >= MAX_TERMS_CHIRHO:
            raise RuntimeError(f"Term store overflow: {MAX_TERMS_CHIRHO}")

        self_chirho.term_to_id_chirho[term_chirho] = id_chirho
        self_chirho.id_to_term_chirho.append(term_chirho)
        return id_chirho

    def lookup_chirho(self_chirho, id_chirho: int) -> TermChirho:
        """Get term by ID"""
        return self_chirho.id_to_term_chirho[id_chirho]

    def size_chirho(self_chirho) -> int:
        return len(self_chirho.id_to_term_chirho)


# Global store
TERM_STORE_CHIRHO = TermStoreChirho()


def reset_store_chirho():
    global TERM_STORE_CHIRHO
    TERM_STORE_CHIRHO = TermStoreChirho()


# === Containment Graph for Occurs Check ===

class ContainmentGraphChirho:
    """
    Tracks which terms contain which other terms.

    Hardware mapping: Boolean adjacency matrix
    - A[i,j] = 1 iff term i directly contains term j
    - Occurs check = reachability in A*
    """

    def __init__(self_chirho, store_chirho: TermStoreChirho):
        self_chirho.store_chirho = store_chirho
        # Adjacency: parent → set of children
        self_chirho.children_chirho: Dict[int, Set[int]] = defaultdict(set)

    def register_term_chirho(self_chirho, term_id_chirho: int):
        """Register containment edges for a term"""
        term_chirho = self_chirho.store_chirho.lookup_chirho(term_id_chirho)

        if isinstance(term_chirho, ConsChirho):
            head_chirho = term_chirho.head_chirho
            tail_chirho = term_chirho.tail_chirho

            # Get or create IDs for children
            if not isinstance(head_chirho, VarChirho):
                head_id_chirho = self_chirho.store_chirho.intern_chirho(head_chirho)
                self_chirho.children_chirho[term_id_chirho].add(head_id_chirho)
                self_chirho.register_term_chirho(head_id_chirho)

            if not isinstance(tail_chirho, VarChirho):
                tail_id_chirho = self_chirho.store_chirho.intern_chirho(tail_chirho)
                self_chirho.children_chirho[term_id_chirho].add(tail_id_chirho)
                self_chirho.register_term_chirho(tail_id_chirho)

    def to_adjacency_matrix_chirho(self_chirho) -> List[List[int]]:
        """
        Build Boolean adjacency matrix.

        Hardware: This IS the 1-bit representation!
        Returns list of lists (pure Python) or numpy array if available.
        """
        n_chirho = self_chirho.store_chirho.size_chirho()

        if HAS_NUMPY_CHIRHO:
            adj_chirho = np.zeros((n_chirho, n_chirho), dtype=np.uint8)
            for parent_chirho, children_chirho in self_chirho.children_chirho.items():
                for child_chirho in children_chirho:
                    adj_chirho[parent_chirho, child_chirho] = 1
            return adj_chirho

        # Pure Python fallback
        adj_chirho = [[0] * n_chirho for _ in range(n_chirho)]
        for parent_chirho, children_chirho in self_chirho.children_chirho.items():
            for child_chirho in children_chirho:
                adj_chirho[parent_chirho][child_chirho] = 1
        return adj_chirho

    def transitive_closure_chirho(self_chirho) -> List[List[int]]:
        """
        Compute A* = I + A + A² + A³ + ...

        Hardware: O(log n) Boolean matrix multiplies
        """
        adj_chirho = self_chirho.to_adjacency_matrix_chirho()

        if HAS_NUMPY_CHIRHO:
            n_chirho = adj_chirho.shape[0]
            closure_chirho = adj_chirho.copy()
        else:
            n_chirho = len(adj_chirho)
            closure_chirho = [row[:] for row in adj_chirho]

        # Warshall's algorithm (simpler for small n)
        for k_chirho in range(n_chirho):
            for i_chirho in range(n_chirho):
                for j_chirho in range(n_chirho):
                    if HAS_NUMPY_CHIRHO:
                        closure_chirho[i_chirho, j_chirho] |= (
                            closure_chirho[i_chirho, k_chirho] &
                            closure_chirho[k_chirho, j_chirho]
                        )
                    else:
                        closure_chirho[i_chirho][j_chirho] |= (
                            closure_chirho[i_chirho][k_chirho] &
                            closure_chirho[k_chirho][j_chirho]
                        )

        return closure_chirho

    def occurs_chirho(self_chirho, var_id_chirho: int, term_id_chirho: int) -> bool:
        """
        Check if variable occurs in term.

        Hardware: Single bit lookup in closure matrix!
        """
        closure_chirho = self_chirho.transitive_closure_chirho()

        # Check if var_id is reachable from term_id
        # Note: variables aren't in the term store directly,
        # we need to check if term contains any reference to var

        # For now, use recursive check (hardware would use matrix)
        return self_chirho._occurs_recursive_chirho(var_id_chirho, term_id_chirho, set())

    def _occurs_recursive_chirho(
        self_chirho,
        var_id_chirho: int,
        term_id_chirho: int,
        visited_chirho: Set[int]
    ) -> bool:
        """Recursive occurs check (software fallback)"""
        if term_id_chirho in visited_chirho:
            return False
        visited_chirho.add(term_id_chirho)

        term_chirho = self_chirho.store_chirho.lookup_chirho(term_id_chirho)

        if isinstance(term_chirho, VarChirho):
            return term_chirho.id_chirho == var_id_chirho

        if isinstance(term_chirho, ConsChirho):
            head_chirho = term_chirho.head_chirho
            tail_chirho = term_chirho.tail_chirho

            if isinstance(head_chirho, VarChirho) and head_chirho.id_chirho == var_id_chirho:
                return True
            if isinstance(tail_chirho, VarChirho) and tail_chirho.id_chirho == var_id_chirho:
                return True

            if not isinstance(head_chirho, VarChirho):
                head_id_chirho = self_chirho.store_chirho.intern_chirho(head_chirho)
                if self_chirho._occurs_recursive_chirho(var_id_chirho, head_id_chirho, visited_chirho):
                    return True

            if not isinstance(tail_chirho, VarChirho):
                tail_id_chirho = self_chirho.store_chirho.intern_chirho(tail_chirho)
                if self_chirho._occurs_recursive_chirho(var_id_chirho, tail_id_chirho, visited_chirho):
                    return True

        return False


# === State ===

@dataclass
class StateChirho:
    """
    Search state: substitution + variable counter.

    Hardware mapping: Bit vector per variable
    """
    subst_chirho: Dict[int, TermChirho]
    counter_chirho: int

    def copy_chirho(self_chirho) -> 'StateChirho':
        return StateChirho(self_chirho.subst_chirho.copy(), self_chirho.counter_chirho)


def empty_state_chirho() -> StateChirho:
    return StateChirho({}, 0)


# === Unification ===

def walk_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> TermChirho:
    """Follow variable bindings"""
    while isinstance(term_chirho, VarChirho) and term_chirho.id_chirho in subst_chirho:
        term_chirho = subst_chirho[term_chirho.id_chirho]
    return term_chirho


def walk_deep_chirho(term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> TermChirho:
    """Fully walk a term"""
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, ConsChirho):
        return ConsChirho(
            walk_deep_chirho(term_chirho.head_chirho, subst_chirho),
            walk_deep_chirho(term_chirho.tail_chirho, subst_chirho)
        )
    return term_chirho


def occurs_check_chirho(var_id_chirho: int, term_chirho: TermChirho, subst_chirho: Dict[int, TermChirho]) -> bool:
    """Check if var occurs in term (prevents infinite terms)"""
    term_chirho = walk_chirho(term_chirho, subst_chirho)
    if isinstance(term_chirho, VarChirho):
        return term_chirho.id_chirho == var_id_chirho
    if isinstance(term_chirho, ConsChirho):
        return (occurs_check_chirho(var_id_chirho, term_chirho.head_chirho, subst_chirho) or
                occurs_check_chirho(var_id_chirho, term_chirho.tail_chirho, subst_chirho))
    return False


def unify_chirho(
    t1_chirho: TermChirho,
    t2_chirho: TermChirho,
    subst_chirho: Dict[int, TermChirho]
) -> Optional[Dict[int, TermChirho]]:
    """Unify two terms"""
    t1_chirho = walk_chirho(t1_chirho, subst_chirho)
    t2_chirho = walk_chirho(t2_chirho, subst_chirho)

    if t1_chirho == t2_chirho:
        return subst_chirho

    if isinstance(t1_chirho, VarChirho):
        if occurs_check_chirho(t1_chirho.id_chirho, t2_chirho, subst_chirho):
            return None  # Occurs check failed
        subst_chirho = subst_chirho.copy()
        subst_chirho[t1_chirho.id_chirho] = t2_chirho
        return subst_chirho

    if isinstance(t2_chirho, VarChirho):
        if occurs_check_chirho(t2_chirho.id_chirho, t1_chirho, subst_chirho):
            return None
        subst_chirho = subst_chirho.copy()
        subst_chirho[t2_chirho.id_chirho] = t1_chirho
        return subst_chirho

    if isinstance(t1_chirho, ConsChirho) and isinstance(t2_chirho, ConsChirho):
        subst_chirho = unify_chirho(t1_chirho.head_chirho, t2_chirho.head_chirho, subst_chirho)
        if subst_chirho is None:
            return None
        return unify_chirho(t1_chirho.tail_chirho, t2_chirho.tail_chirho, subst_chirho)

    return None  # Constructor mismatch


def is_ground_chirho(term_chirho: TermChirho) -> bool:
    """Check if term has no variables"""
    if isinstance(term_chirho, VarChirho):
        return False
    if isinstance(term_chirho, ConsChirho):
        return is_ground_chirho(term_chirho.head_chirho) and is_ground_chirho(term_chirho.tail_chirho)
    return True


# === Tensor Store ===

class TensorStoreChirho:
    """
    Stores relation tuples as sparse tensor.

    Hardware mapping: COO sparse format
    - List of (i, j, k) tuples
    - Or: CSR for faster row access
    """

    def __init__(self_chirho):
        self_chirho.appendo_triples_chirho: Set[Tuple[tuple, tuple, tuple]] = set()

    def add_appendo_chirho(self_chirho, l_chirho: TermChirho, s_chirho: TermChirho, out_chirho: TermChirho):
        """Record ground appendo triple"""
        l_list_chirho = term_to_pylist_chirho(l_chirho)
        s_list_chirho = term_to_pylist_chirho(s_chirho)
        out_list_chirho = term_to_pylist_chirho(out_chirho)

        if l_list_chirho is not None and s_list_chirho is not None and out_list_chirho is not None:
            self_chirho.appendo_triples_chirho.add((
                tuple(l_list_chirho),
                tuple(s_list_chirho),
                tuple(out_list_chirho)
            ))

    def to_coo_chirho(self_chirho) -> List[Tuple[int, int, int]]:
        """
        Convert to COO format with integer indices.

        Hardware: Direct memory layout
        """
        # Build index maps
        all_lists_chirho: Set[tuple] = set()
        for l_chirho, s_chirho, o_chirho in self_chirho.appendo_triples_chirho:
            all_lists_chirho.add(l_chirho)
            all_lists_chirho.add(s_chirho)
            all_lists_chirho.add(o_chirho)

        list_to_idx_chirho = {lst_chirho: i_chirho for i_chirho, lst_chirho in enumerate(sorted(all_lists_chirho, key=len))}

        coo_chirho = []
        for l_chirho, s_chirho, o_chirho in self_chirho.appendo_triples_chirho:
            coo_chirho.append((
                list_to_idx_chirho[l_chirho],
                list_to_idx_chirho[s_chirho],
                list_to_idx_chirho[o_chirho]
            ))

        return coo_chirho


TENSOR_STORE_CHIRHO = TensorStoreChirho()


def reset_tensor_chirho():
    global TENSOR_STORE_CHIRHO
    TENSOR_STORE_CHIRHO = TensorStoreChirho()


# === Tabled appendo ===

def appendo_chirho(l_chirho: TermChirho, s_chirho: TermChirho, out_chirho: TermChirho):
    """
    Tabled appendo with mode analysis.

    Hardware mapping:
    - Backward mode → tensor slice
    - Forward mode → single lookup
    """
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        l_w_chirho = walk_deep_chirho(l_chirho, state_chirho.subst_chirho)
        s_w_chirho = walk_deep_chirho(s_chirho, state_chirho.subst_chirho)
        out_w_chirho = walk_deep_chirho(out_chirho, state_chirho.subst_chirho)

        l_ground_chirho = is_ground_chirho(l_w_chirho)
        s_ground_chirho = is_ground_chirho(s_w_chirho)
        out_ground_chirho = is_ground_chirho(out_w_chirho)

        # BACKWARD: out ground → enumerate splits
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

        # GENERATE: bounded
        yield from appendo_generate_chirho(l_chirho, s_chirho, out_chirho, state_chirho, 5)

    return goal_chirho


def appendo_generate_chirho(
    l_chirho: TermChirho,
    s_chirho: TermChirho,
    out_chirho: TermChirho,
    state_chirho: StateChirho,
    max_depth_chirho: int
) -> Iterator[StateChirho]:
    """Bounded generation"""
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

    # Recursive
    h_chirho = VarChirho(state_chirho.counter_chirho)
    t_chirho = VarChirho(state_chirho.counter_chirho + 1)
    r_chirho = VarChirho(state_chirho.counter_chirho + 2)

    subst1_chirho = unify_chirho(l_chirho, ConsChirho(h_chirho, t_chirho), state_chirho.subst_chirho)
    if subst1_chirho is None:
        return

    subst2_chirho = unify_chirho(out_chirho, ConsChirho(h_chirho, r_chirho), subst1_chirho)
    if subst2_chirho is None:
        return

    new_state_chirho = StateChirho(subst2_chirho, state_chirho.counter_chirho + 3)
    for result_chirho in appendo_generate_chirho(t_chirho, s_chirho, r_chirho, new_state_chirho, max_depth_chirho - 1):
        l_val_chirho = walk_deep_chirho(l_chirho, result_chirho.subst_chirho)
        s_val_chirho = walk_deep_chirho(s_chirho, result_chirho.subst_chirho)
        out_val_chirho = walk_deep_chirho(out_chirho, result_chirho.subst_chirho)
        if is_ground_chirho(l_val_chirho) and is_ground_chirho(s_val_chirho) and is_ground_chirho(out_val_chirho):
            TENSOR_STORE_CHIRHO.add_appendo_chirho(l_val_chirho, s_val_chirho, out_val_chirho)
        yield result_chirho


# === Goal Combinators ===

GoalChirho = Any  # Callable[[StateChirho], Iterator[StateChirho]]


def eq_chirho(t1_chirho: TermChirho, t2_chirho: TermChirho) -> GoalChirho:
    """Unification goal"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        subst_chirho = unify_chirho(t1_chirho, t2_chirho, state_chirho.subst_chirho)
        if subst_chirho is not None:
            yield StateChirho(subst_chirho, state_chirho.counter_chirho)
    return goal_chirho


def conj_chirho(g1_chirho: GoalChirho, g2_chirho: GoalChirho) -> GoalChirho:
    """Conjunction"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        for st1_chirho in g1_chirho(state_chirho):
            yield from g2_chirho(st1_chirho)
    return goal_chirho


def disj_chirho(g1_chirho: GoalChirho, g2_chirho: GoalChirho) -> GoalChirho:
    """Interleaving disjunction"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
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


def fresh_chirho(fn_chirho) -> GoalChirho:
    """Fresh variable"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        var_chirho = VarChirho(state_chirho.counter_chirho)
        new_st_chirho = StateChirho(state_chirho.subst_chirho, state_chirho.counter_chirho + 1)
        yield from fn_chirho(var_chirho)(new_st_chirho)
    return goal_chirho


# === Smart Conjunction with Reordering ===

@dataclass
class GoalInfoChirho:
    """Goal with metadata for scheduling"""
    goal_chirho: GoalChirho
    args_chirho: List[TermChirho]
    name_chirho: str = ""

    def groundness_score_chirho(self_chirho, subst_chirho: Dict[int, TermChirho]) -> int:
        """Higher = more ground = run first"""
        score_chirho = 0
        for arg_chirho in self_chirho.args_chirho:
            walked_chirho = walk_deep_chirho(arg_chirho, subst_chirho)
            if is_ground_chirho(walked_chirho):
                score_chirho += 10
            elif isinstance(walked_chirho, ConsChirho):
                score_chirho += 5
        return score_chirho


def smart_conj_chirho(goals_info_chirho: List[GoalInfoChirho]) -> GoalChirho:
    """Conjunction with mode-driven reordering"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        if not goals_info_chirho:
            yield state_chirho
            return

        if len(goals_info_chirho) == 1:
            yield from goals_info_chirho[0].goal_chirho(state_chirho)
            return

        # Sort by groundness descending
        sorted_goals_chirho = sorted(
            goals_info_chirho,
            key=lambda g_chirho: -g_chirho.groundness_score_chirho(state_chirho.subst_chirho)
        )

        first_chirho = sorted_goals_chirho[0]
        rest_chirho = sorted_goals_chirho[1:]

        for st1_chirho in first_chirho.goal_chirho(state_chirho):
            yield from smart_conj_chirho(rest_chirho)(st1_chirho)

    return goal_chirho


# === Run ===

def run_chirho(n_chirho: Optional[int], goal_chirho: GoalChirho, vars_chirho: List[VarChirho]) -> List[Dict[int, TermChirho]]:
    """Run goal, extract results"""
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
    print("=== Hardware-Ready Tabling ☧ ===\n")
    print("Proper casing: ClassChirho, function_chirho, CONSTANT_CHIRHO\n")

    reset_tensor_chirho()

    # Test composition
    print("="*60)
    print("Composition: appendo(A,B,X), appendo(X,C,[0,1,2])")
    print("="*60)

    a_chirho = VarChirho(0)
    b_chirho = VarChirho(1)
    x_chirho = VarChirho(2)
    c_chirho = VarChirho(3)

    goals_chirho = [
        GoalInfoChirho(appendo_chirho(a_chirho, b_chirho, x_chirho), [a_chirho, b_chirho, x_chirho]),
        GoalInfoChirho(appendo_chirho(x_chirho, c_chirho, list_chirho(0, 1, 2)), [x_chirho, c_chirho, list_chirho(0, 1, 2)]),
    ]

    goal_chirho = smart_conj_chirho(goals_chirho)
    results_chirho = run_chirho(15, goal_chirho, [a_chirho, b_chirho, x_chirho, c_chirho])

    print(f"\nResults ({len(results_chirho)}):")
    for r_chirho in results_chirho:
        print(f"  A={r_chirho[0]}, B={r_chirho[1]} → X={r_chirho[2]}, C={r_chirho[3]}")

    # Show tensor
    print("\n" + "="*60)
    print("Tensor (COO format)")
    print("="*60)

    print(f"\nTriples: {len(TENSOR_STORE_CHIRHO.appendo_triples_chirho)}")
    for t_chirho in sorted(TENSOR_STORE_CHIRHO.appendo_triples_chirho, key=lambda x: (len(x[2]), x)):
        l_chirho, s_chirho, o_chirho = t_chirho
        print(f"  {list(l_chirho)} ++ {list(s_chirho)} = {list(o_chirho)}")

    coo_chirho = TENSOR_STORE_CHIRHO.to_coo_chirho()
    print(f"\nCOO indices: {coo_chirho}")

    # Test occurs check infrastructure
    print("\n" + "="*60)
    print("Containment Graph (for occurs check)")
    print("="*60)

    reset_store_chirho()
    store_chirho = TERM_STORE_CHIRHO

    # Build some terms
    t1_chirho = store_chirho.intern_chirho(list_chirho(1, 2, 3))
    t2_chirho = store_chirho.intern_chirho(list_chirho(1))

    graph_chirho = ContainmentGraphChirho(store_chirho)
    graph_chirho.register_term_chirho(t1_chirho)

    adj_chirho = graph_chirho.to_adjacency_matrix_chirho()
    if HAS_NUMPY_CHIRHO:
        print(f"\nAdjacency matrix shape: {adj_chirho.shape}")
        print(f"Adjacency matrix:\n{adj_chirho}")
    else:
        print(f"\nAdjacency matrix ({len(adj_chirho)}x{len(adj_chirho)}):")
        for row_chirho in adj_chirho:
            print(f"  {row_chirho}")

    closure_chirho = graph_chirho.transitive_closure_chirho()
    if HAS_NUMPY_CHIRHO:
        print(f"\nTransitive closure:\n{closure_chirho}")
    else:
        print(f"\nTransitive closure:")
        for row_chirho in closure_chirho:
            print(f"  {row_chirho}")

    print("\n" + "="*60)
    print("Hardware mapping summary:")
    print("="*60)
    print("""
    TermStoreChirho      → Content-addressable memory (CAM)
    ContainmentGraphChirho → Boolean adjacency matrix
    transitive_closure   → O(log n) matrix multiply
    TensorStoreChirho    → Sparse COO tensor
    smart_conj_chirho    → Static scheduling
    """)

    print("\n*Soli Deo Gloria* ☧")


if __name__ == "__main__":
    main()
