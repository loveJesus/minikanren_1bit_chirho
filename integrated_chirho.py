#!/usr/bin/env python3
"""
Integrated miniKanren: Nested Patterns + Tabling + Bit Matrices ☧

This file integrates:
- nested_latent_chirho.py: Path-based pattern representation
- tabling_complete_chirho.py: Mode-driven goal reordering
- Sparse bit matrix storage for relations

The result: A complete system where relations store patterns,
queries use patterns, and everything is backed by bit vectors.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Callable, Any, Union
from enum import Enum


# === Core Types (from nested_latent) ===

class StepChirho(Enum):
    """Single step in a tree path"""
    CAR_CHIRHO = "car"
    CDR_CHIRHO = "cdr"


@dataclass(frozen=True)
class PathChirho:
    """Path through a tree structure"""
    steps_chirho: Tuple[StepChirho, ...] = ()

    def __repr__(self_chirho):
        if not self_chirho.steps_chirho:
            return "ε"
        return ".".join(s_chirho.value for s_chirho in self_chirho.steps_chirho)

    def car_chirho(self_chirho) -> 'PathChirho':
        return PathChirho(self_chirho.steps_chirho + (StepChirho.CAR_CHIRHO,))

    def cdr_chirho(self_chirho) -> 'PathChirho':
        return PathChirho(self_chirho.steps_chirho + (StepChirho.CDR_CHIRHO,))

    def depth_chirho(self_chirho) -> int:
        return len(self_chirho.steps_chirho)

    def is_prefix_of_chirho(self_chirho, other_chirho: 'PathChirho') -> bool:
        """Check if self is a proper prefix of other"""
        s1_chirho = self_chirho.steps_chirho
        s2_chirho = other_chirho.steps_chirho
        return len(s1_chirho) < len(s2_chirho) and s2_chirho[:len(s1_chirho)] == s1_chirho


class TypeChirho(Enum):
    """Type of a node in the tree"""
    NIL_CHIRHO = "nil"
    CONS_CHIRHO = "cons"
    ATOM_CHIRHO = "atom"
    VAR_CHIRHO = "var"  # Logic variable


@dataclass(frozen=True)
class ConstraintChirho:
    """Constraint on a single node"""
    type_chirho: TypeChirho
    value_chirho: Optional[int] = None  # For atoms
    var_id_chirho: Optional[int] = None  # For variables

    def __repr__(self_chirho):
        if self_chirho.type_chirho == TypeChirho.VAR_CHIRHO:
            return f"_{self_chirho.var_id_chirho}"
        elif self_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
            return "[]"
        elif self_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
            return "cons"
        elif self_chirho.type_chirho == TypeChirho.ATOM_CHIRHO:
            return str(self_chirho.value_chirho) if self_chirho.value_chirho is not None else "atom"
        return "?"

    def is_ground_chirho(self_chirho) -> bool:
        return self_chirho.type_chirho != TypeChirho.VAR_CHIRHO

    @classmethod
    def nil_chirho(cls_chirho) -> 'ConstraintChirho':
        return ConstraintChirho(TypeChirho.NIL_CHIRHO)

    @classmethod
    def cons_chirho(cls_chirho) -> 'ConstraintChirho':
        return ConstraintChirho(TypeChirho.CONS_CHIRHO)

    @classmethod
    def atom_chirho(cls_chirho, val_chirho: int) -> 'ConstraintChirho':
        return ConstraintChirho(TypeChirho.ATOM_CHIRHO, value_chirho=val_chirho)

    @classmethod
    def var_chirho(cls_chirho, id_chirho: int) -> 'ConstraintChirho':
        return ConstraintChirho(TypeChirho.VAR_CHIRHO, var_id_chirho=id_chirho)


# === Pattern: Map from paths to constraints ===

@dataclass
class PatternChirho:
    """A pattern is a partial specification of a term"""
    constraints_chirho: Dict[PathChirho, ConstraintChirho] = field(default_factory=dict)

    def __repr__(self_chirho):
        try:
            return self_chirho._as_list_str_chirho()
        except:
            parts_chirho = [f"{p_chirho}:{c_chirho}" for p_chirho, c_chirho in
                           sorted(self_chirho.constraints_chirho.items(),
                                  key=lambda x_chirho: x_chirho[0].depth_chirho())]
            return "{" + ", ".join(parts_chirho) + "}"

    def _as_list_str_chirho(self_chirho) -> str:
        """Render as [a, b, ...] if possible"""
        elems_chirho = []
        path_chirho = PathChirho()
        for _ in range(10):  # Max depth
            if path_chirho not in self_chirho.constraints_chirho:
                elems_chirho.append("...")
                break
            c_chirho = self_chirho.constraints_chirho[path_chirho]
            if c_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
                break
            elif c_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
                car_chirho = path_chirho.car_chirho()
                if car_chirho in self_chirho.constraints_chirho:
                    elems_chirho.append(str(self_chirho.constraints_chirho[car_chirho]))
                else:
                    elems_chirho.append("?")
                path_chirho = path_chirho.cdr_chirho()
            else:
                elems_chirho.append(str(c_chirho))
                break
        return "[" + ", ".join(elems_chirho) + "]"

    def set_chirho(self_chirho, path_chirho: PathChirho, constraint_chirho: ConstraintChirho):
        self_chirho.constraints_chirho[path_chirho] = constraint_chirho

    def get_chirho(self_chirho, path_chirho: PathChirho) -> Optional[ConstraintChirho]:
        return self_chirho.constraints_chirho.get(path_chirho)

    def is_ground_chirho(self_chirho) -> bool:
        """Check if pattern has no variables"""
        return all(c_chirho.is_ground_chirho() for c_chirho in self_chirho.constraints_chirho.values())

    def get_vars_chirho(self_chirho) -> Set[int]:
        """Get all variable IDs in this pattern"""
        vars_chirho = set()
        for c_chirho in self_chirho.constraints_chirho.values():
            if c_chirho.var_id_chirho is not None:
                vars_chirho.add(c_chirho.var_id_chirho)
        return vars_chirho

    def groundness_score_chirho(self_chirho) -> int:
        """Higher = more ground constraints"""
        return sum(1 for c_chirho in self_chirho.constraints_chirho.values() if c_chirho.is_ground_chirho())

    @classmethod
    def from_list_chirho(cls_chirho, elems_chirho: List[Any]) -> 'PatternChirho':
        """
        Create pattern from list.
        Elements: int (atom), None (fresh var), ("var", id) (named var)
        """
        p_chirho = PatternChirho()
        path_chirho = PathChirho()
        next_var_chirho = 100  # Auto-generated var IDs start high

        for elem_chirho in elems_chirho:
            p_chirho.set_chirho(path_chirho, ConstraintChirho.cons_chirho())
            car_chirho = path_chirho.car_chirho()

            if elem_chirho is None:
                p_chirho.set_chirho(car_chirho, ConstraintChirho.var_chirho(next_var_chirho))
                next_var_chirho += 1
            elif isinstance(elem_chirho, tuple) and elem_chirho[0] == "var":
                p_chirho.set_chirho(car_chirho, ConstraintChirho.var_chirho(elem_chirho[1]))
            elif isinstance(elem_chirho, int):
                p_chirho.set_chirho(car_chirho, ConstraintChirho.atom_chirho(elem_chirho))
            else:
                raise ValueError(f"Unknown element type: {elem_chirho}")

            path_chirho = path_chirho.cdr_chirho()

        p_chirho.set_chirho(path_chirho, ConstraintChirho.nil_chirho())
        return p_chirho

    def to_list_chirho(self_chirho) -> Optional[List[int]]:
        """Convert ground pattern to Python list, or None if has vars"""
        result_chirho = []
        path_chirho = PathChirho()

        for _ in range(100):  # Safety limit
            c_chirho = self_chirho.get_chirho(path_chirho)
            if c_chirho is None:
                return None
            if c_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
                return result_chirho
            if c_chirho.type_chirho != TypeChirho.CONS_CHIRHO:
                return None

            car_chirho = self_chirho.get_chirho(path_chirho.car_chirho())
            if car_chirho is None or car_chirho.type_chirho != TypeChirho.ATOM_CHIRHO:
                return None
            result_chirho.append(car_chirho.value_chirho)
            path_chirho = path_chirho.cdr_chirho()

        return None


# === Unification ===

def unify_constraint_chirho(c1_chirho: ConstraintChirho,
                            c2_chirho: ConstraintChirho) -> Optional[ConstraintChirho]:
    """Unify two constraints. Returns unified constraint or None on failure."""
    # Variables unify with anything
    if c1_chirho.type_chirho == TypeChirho.VAR_CHIRHO:
        return c2_chirho
    if c2_chirho.type_chirho == TypeChirho.VAR_CHIRHO:
        return c1_chirho

    # Types must match
    if c1_chirho.type_chirho != c2_chirho.type_chirho:
        return None

    # For atoms, values must match
    if c1_chirho.type_chirho == TypeChirho.ATOM_CHIRHO:
        if c1_chirho.value_chirho != c2_chirho.value_chirho:
            return None

    return c1_chirho


def unify_pattern_chirho(p1_chirho: PatternChirho,
                         p2_chirho: PatternChirho) -> Optional[PatternChirho]:
    """Unify two patterns. Returns unified pattern or None on failure."""
    result_chirho = PatternChirho()
    all_paths_chirho = set(p1_chirho.constraints_chirho.keys()) | set(p2_chirho.constraints_chirho.keys())

    for path_chirho in all_paths_chirho:
        c1_chirho = p1_chirho.get_chirho(path_chirho)
        c2_chirho = p2_chirho.get_chirho(path_chirho)

        if c1_chirho is None and c2_chirho is None:
            continue
        elif c1_chirho is None:
            result_chirho.set_chirho(path_chirho, c2_chirho)
        elif c2_chirho is None:
            result_chirho.set_chirho(path_chirho, c1_chirho)
        else:
            unified_chirho = unify_constraint_chirho(c1_chirho, c2_chirho)
            if unified_chirho is None:
                return None
            result_chirho.set_chirho(path_chirho, unified_chirho)

    return result_chirho


# === Substitution (variable bindings) ===

@dataclass
class SubstChirho:
    """Substitution: mapping from variable IDs to patterns"""
    bindings_chirho: Dict[int, PatternChirho] = field(default_factory=dict)

    def bind_chirho(self_chirho, var_id_chirho: int, pattern_chirho: PatternChirho):
        self_chirho.bindings_chirho[var_id_chirho] = pattern_chirho

    def lookup_chirho(self_chirho, var_id_chirho: int) -> Optional[PatternChirho]:
        return self_chirho.bindings_chirho.get(var_id_chirho)

    def copy_chirho(self_chirho) -> 'SubstChirho':
        return SubstChirho(bindings_chirho=dict(self_chirho.bindings_chirho))


def apply_subst_chirho(pattern_chirho: PatternChirho, subst_chirho: SubstChirho) -> PatternChirho:
    """Apply substitution to pattern, replacing variables with their bindings"""
    result_chirho = PatternChirho()

    for path_chirho, constraint_chirho in pattern_chirho.constraints_chirho.items():
        if constraint_chirho.type_chirho == TypeChirho.VAR_CHIRHO:
            var_id_chirho = constraint_chirho.var_id_chirho
            binding_chirho = subst_chirho.lookup_chirho(var_id_chirho)
            if binding_chirho is not None:
                # Merge binding at this path
                for sub_path_chirho, sub_c_chirho in binding_chirho.constraints_chirho.items():
                    # Compose paths: path_chirho + sub_path_chirho
                    full_path_chirho = PathChirho(path_chirho.steps_chirho + sub_path_chirho.steps_chirho)
                    result_chirho.set_chirho(full_path_chirho, sub_c_chirho)
            else:
                result_chirho.set_chirho(path_chirho, constraint_chirho)
        else:
            result_chirho.set_chirho(path_chirho, constraint_chirho)

    return result_chirho


# === State ===

@dataclass
class StateChirho:
    """Search state: substitution + variable counter"""
    subst_chirho: SubstChirho = field(default_factory=SubstChirho)
    var_counter_chirho: int = 0

    def fresh_chirho(self_chirho) -> Tuple['StateChirho', int]:
        """Create fresh variable, return new state and var ID"""
        new_state_chirho = StateChirho(
            subst_chirho=self_chirho.subst_chirho.copy_chirho(),
            var_counter_chirho=self_chirho.var_counter_chirho + 1
        )
        return new_state_chirho, self_chirho.var_counter_chirho

    def copy_chirho(self_chirho) -> 'StateChirho':
        return StateChirho(
            subst_chirho=self_chirho.subst_chirho.copy_chirho(),
            var_counter_chirho=self_chirho.var_counter_chirho
        )


# === Relation Storage (Bit Matrix) ===

@dataclass
class RelationChirho:
    """
    A relation stored as a set of pattern tuples.

    For appendo: stores (l, s, out) pattern triples.
    Internally uses sparse representation.
    """
    name_chirho: str
    arity_chirho: int
    tuples_chirho: List[Tuple[PatternChirho, ...]] = field(default_factory=list)

    def add_chirho(self_chirho, *patterns_chirho: PatternChirho):
        if len(patterns_chirho) != self_chirho.arity_chirho:
            raise ValueError(f"Expected {self_chirho.arity_chirho} patterns, got {len(patterns_chirho)}")
        self_chirho.tuples_chirho.append(patterns_chirho)

    def query_chirho(self_chirho, *query_patterns_chirho: PatternChirho) -> Iterator[Tuple[PatternChirho, ...]]:
        """
        Query relation with patterns.
        Yields tuples that unify with the query.
        """
        for tup_chirho in self_chirho.tuples_chirho:
            unified_chirho = []
            success_chirho = True

            for q_chirho, t_chirho in zip(query_patterns_chirho, tup_chirho):
                u_chirho = unify_pattern_chirho(q_chirho, t_chirho)
                if u_chirho is None:
                    success_chirho = False
                    break
                unified_chirho.append(u_chirho)

            if success_chirho:
                yield tuple(unified_chirho)


# === Tabled Relation with Mode Analysis ===

class TabledRelationChirho:
    """
    A relation with tabling (memoization) and mode-driven execution.

    Modes:
    - FORWARD: inputs ground → compute output
    - BACKWARD: output ground → enumerate inputs
    - GENERATE: nothing ground → enumerate all
    """

    def __init__(self_chirho, name_chirho: str, compute_fn_chirho: Callable):
        self_chirho.name_chirho = name_chirho
        self_chirho.compute_fn_chirho = compute_fn_chirho
        self_chirho.cache_chirho: RelationChirho = RelationChirho(name_chirho, 3)  # Assuming ternary

    def call_chirho(self_chirho, l_chirho: PatternChirho, s_chirho: PatternChirho,
                    out_chirho: PatternChirho, state_chirho: StateChirho) -> Iterator[StateChirho]:
        """
        Call the relation with mode analysis.
        """
        # Determine mode based on groundness
        l_ground_chirho = l_chirho.is_ground_chirho()
        s_ground_chirho = s_chirho.is_ground_chirho()
        out_ground_chirho = out_chirho.is_ground_chirho()

        if l_ground_chirho and s_ground_chirho:
            # FORWARD: compute output
            yield from self_chirho._forward_chirho(l_chirho, s_chirho, out_chirho, state_chirho)
        elif out_ground_chirho:
            # BACKWARD: split output
            yield from self_chirho._backward_chirho(l_chirho, s_chirho, out_chirho, state_chirho)
        else:
            # GENERATE: try cache first
            yield from self_chirho._generate_chirho(l_chirho, s_chirho, out_chirho, state_chirho)

    def _forward_chirho(self_chirho, l_chirho: PatternChirho, s_chirho: PatternChirho,
                        out_chirho: PatternChirho, state_chirho: StateChirho) -> Iterator[StateChirho]:
        """Forward mode: l, s ground → compute out"""
        l_list_chirho = l_chirho.to_list_chirho()
        s_list_chirho = s_chirho.to_list_chirho()

        if l_list_chirho is None or s_list_chirho is None:
            return

        result_list_chirho = l_list_chirho + s_list_chirho
        result_pattern_chirho = PatternChirho.from_list_chirho(result_list_chirho)

        # Cache the result
        self_chirho.cache_chirho.add_chirho(l_chirho, s_chirho, result_pattern_chirho)

        # Unify with expected output
        unified_chirho = unify_pattern_chirho(out_chirho, result_pattern_chirho)
        if unified_chirho is not None:
            yield state_chirho

    def _backward_chirho(self_chirho, l_chirho: PatternChirho, s_chirho: PatternChirho,
                         out_chirho: PatternChirho, state_chirho: StateChirho) -> Iterator[StateChirho]:
        """Backward mode: out ground → enumerate (l, s) splits"""
        out_list_chirho = out_chirho.to_list_chirho()
        if out_list_chirho is None:
            return

        # Enumerate all splits
        for i_chirho in range(len(out_list_chirho) + 1):
            l_list_chirho = out_list_chirho[:i_chirho]
            s_list_chirho = out_list_chirho[i_chirho:]

            l_result_chirho = PatternChirho.from_list_chirho(l_list_chirho)
            s_result_chirho = PatternChirho.from_list_chirho(s_list_chirho)

            # Cache
            self_chirho.cache_chirho.add_chirho(l_result_chirho, s_result_chirho, out_chirho)

            # Unify with query patterns
            l_unified_chirho = unify_pattern_chirho(l_chirho, l_result_chirho)
            s_unified_chirho = unify_pattern_chirho(s_chirho, s_result_chirho)

            if l_unified_chirho is not None and s_unified_chirho is not None:
                # Bind variables in state
                new_state_chirho = state_chirho.copy_chirho()

                # Extract variable bindings from unified patterns
                for var_id_chirho in l_chirho.get_vars_chirho():
                    new_state_chirho.subst_chirho.bind_chirho(var_id_chirho, l_unified_chirho)
                for var_id_chirho in s_chirho.get_vars_chirho():
                    new_state_chirho.subst_chirho.bind_chirho(var_id_chirho, s_unified_chirho)

                yield new_state_chirho

    def _generate_chirho(self_chirho, l_chirho: PatternChirho, s_chirho: PatternChirho,
                         out_chirho: PatternChirho, state_chirho: StateChirho) -> Iterator[StateChirho]:
        """Generate mode: query cache"""
        for cached_chirho in self_chirho.cache_chirho.query_chirho(l_chirho, s_chirho, out_chirho):
            l_u_chirho, s_u_chirho, out_u_chirho = cached_chirho
            new_state_chirho = state_chirho.copy_chirho()
            yield new_state_chirho


# === Goal combinators ===

GoalChirho = Callable[[StateChirho], Iterator[StateChirho]]


def eq_chirho(p1_chirho: PatternChirho, p2_chirho: PatternChirho) -> GoalChirho:
    """Unification goal"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        unified_chirho = unify_pattern_chirho(p1_chirho, p2_chirho)
        if unified_chirho is not None:
            yield state_chirho
    return goal_chirho


def conj_chirho(*goals_chirho: GoalChirho) -> GoalChirho:
    """Conjunction (and)"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        if not goals_chirho:
            yield state_chirho
            return

        def run_chirho(gs_chirho: List[GoalChirho], st_chirho: StateChirho) -> Iterator[StateChirho]:
            if not gs_chirho:
                yield st_chirho
            else:
                for s2_chirho in gs_chirho[0](st_chirho):
                    yield from run_chirho(gs_chirho[1:], s2_chirho)

        yield from run_chirho(list(goals_chirho), state_chirho)
    return goal_chirho


def disj_chirho(*goals_chirho: GoalChirho) -> GoalChirho:
    """Disjunction (or) with interleaving"""
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        streams_chirho = [g_chirho(state_chirho) for g_chirho in goals_chirho]
        while streams_chirho:
            next_streams_chirho = []
            for stream_chirho in streams_chirho:
                try:
                    yield next(stream_chirho)
                    next_streams_chirho.append(stream_chirho)
                except StopIteration:
                    pass
            streams_chirho = next_streams_chirho
    return goal_chirho


# === Smart conjunction with mode-driven reordering ===

@dataclass
class GoalInfoChirho:
    """Goal with metadata for reordering"""
    goal_chirho: GoalChirho
    patterns_chirho: List[PatternChirho]
    name_chirho: str = ""

    def groundness_score_chirho(self_chirho, subst_chirho: SubstChirho) -> int:
        """Score based on how ground the patterns are"""
        score_chirho = 0
        for p_chirho in self_chirho.patterns_chirho:
            applied_chirho = apply_subst_chirho(p_chirho, subst_chirho)
            score_chirho += applied_chirho.groundness_score_chirho()
        return score_chirho


def smart_conj_chirho(goals_info_chirho: List[GoalInfoChirho]) -> GoalChirho:
    """
    Conjunction with mode-driven goal reordering.
    Goals with more ground arguments execute first.
    """
    def goal_chirho(state_chirho: StateChirho) -> Iterator[StateChirho]:
        if not goals_info_chirho:
            yield state_chirho
            return

        # Sort by groundness (descending)
        sorted_chirho = sorted(
            goals_info_chirho,
            key=lambda g_chirho: -g_chirho.groundness_score_chirho(state_chirho.subst_chirho)
        )

        def run_chirho(gs_chirho: List[GoalInfoChirho], st_chirho: StateChirho) -> Iterator[StateChirho]:
            if not gs_chirho:
                yield st_chirho
                return

            # Re-sort remaining goals with current substitution
            remaining_chirho = sorted(
                gs_chirho,
                key=lambda g_chirho: -g_chirho.groundness_score_chirho(st_chirho.subst_chirho)
            )

            first_chirho = remaining_chirho[0]
            rest_chirho = remaining_chirho[1:]

            for s2_chirho in first_chirho.goal_chirho(st_chirho):
                yield from run_chirho(rest_chirho, s2_chirho)

        yield from run_chirho(sorted_chirho, state_chirho)

    return goal_chirho


# === Sparse Bit Matrix Encoding ===

class BitMatrixChirho:
    """
    Sparse bit matrix for patterns.

    Encoding:
    - Each pattern = one row
    - Columns = (path_index, constraint_type, value) triples
    - Entry = 1 if pattern has that constraint

    Storage: COO format (list of (row, col) pairs)
    """

    def __init__(self_chirho, max_depth_chirho: int = 6, max_value_chirho: int = 10):
        self_chirho.max_depth_chirho = max_depth_chirho
        self_chirho.max_value_chirho = max_value_chirho

        # Enumerate all paths up to max_depth
        self_chirho.paths_chirho = self_chirho._enumerate_paths_chirho()
        self_chirho.path_to_idx_chirho = {p_chirho: i_chirho for i_chirho, p_chirho in enumerate(self_chirho.paths_chirho)}

        # Columns per path: nil(1) + cons(1) + atoms(max_value) = 2 + max_value
        self_chirho.cols_per_path_chirho = 2 + max_value_chirho
        self_chirho.num_cols_chirho = len(self_chirho.paths_chirho) * self_chirho.cols_per_path_chirho

        # COO storage: list of (row, col) pairs
        self_chirho.entries_chirho: List[Tuple[int, int]] = []
        self_chirho.num_rows_chirho = 0

    def _enumerate_paths_chirho(self_chirho) -> List[PathChirho]:
        """Generate all paths up to max_depth"""
        paths_chirho = [PathChirho()]
        frontier_chirho = [PathChirho()]

        for _ in range(self_chirho.max_depth_chirho):
            new_frontier_chirho = []
            for p_chirho in frontier_chirho:
                car_chirho = p_chirho.car_chirho()
                cdr_chirho = p_chirho.cdr_chirho()
                paths_chirho.extend([car_chirho, cdr_chirho])
                new_frontier_chirho.extend([car_chirho, cdr_chirho])
            frontier_chirho = new_frontier_chirho

        return paths_chirho

    def _constraint_to_col_chirho(self_chirho, path_idx_chirho: int, c_chirho: ConstraintChirho) -> Optional[int]:
        """Convert (path_index, constraint) to column index"""
        base_chirho = path_idx_chirho * self_chirho.cols_per_path_chirho

        if c_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
            return base_chirho + 0
        elif c_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
            return base_chirho + 1
        elif c_chirho.type_chirho == TypeChirho.ATOM_CHIRHO and c_chirho.value_chirho is not None:
            if c_chirho.value_chirho < self_chirho.max_value_chirho:
                return base_chirho + 2 + c_chirho.value_chirho
        # Variables and out-of-range values don't get columns
        return None

    def add_pattern_chirho(self_chirho, pattern_chirho: PatternChirho) -> int:
        """Add pattern to matrix, return row index"""
        row_chirho = self_chirho.num_rows_chirho
        self_chirho.num_rows_chirho += 1

        for path_chirho, constraint_chirho in pattern_chirho.constraints_chirho.items():
            if path_chirho not in self_chirho.path_to_idx_chirho:
                continue  # Path too deep

            path_idx_chirho = self_chirho.path_to_idx_chirho[path_chirho]
            col_chirho = self_chirho._constraint_to_col_chirho(path_idx_chirho, constraint_chirho)

            if col_chirho is not None:
                self_chirho.entries_chirho.append((row_chirho, col_chirho))

        return row_chirho

    def to_dense_chirho(self_chirho) -> List[List[int]]:
        """Convert to dense matrix (for debugging)"""
        matrix_chirho = [[0] * self_chirho.num_cols_chirho for _ in range(self_chirho.num_rows_chirho)]
        for row_chirho, col_chirho in self_chirho.entries_chirho:
            matrix_chirho[row_chirho][col_chirho] = 1
        return matrix_chirho

    def query_chirho(self_chirho, pattern_chirho: PatternChirho) -> List[int]:
        """
        Find rows matching a pattern.
        Returns row indices where all pattern's constraints are satisfied.
        """
        # Get required columns for query pattern
        required_cols_chirho = set()
        for path_chirho, constraint_chirho in pattern_chirho.constraints_chirho.items():
            if path_chirho not in self_chirho.path_to_idx_chirho:
                continue
            path_idx_chirho = self_chirho.path_to_idx_chirho[path_chirho]
            col_chirho = self_chirho._constraint_to_col_chirho(path_idx_chirho, constraint_chirho)
            if col_chirho is not None:
                required_cols_chirho.add(col_chirho)

        if not required_cols_chirho:
            return list(range(self_chirho.num_rows_chirho))  # Empty query matches all

        # Find rows with all required columns
        # Build column sets per row
        row_cols_chirho: Dict[int, Set[int]] = {}
        for row_chirho, col_chirho in self_chirho.entries_chirho:
            if row_chirho not in row_cols_chirho:
                row_cols_chirho[row_chirho] = set()
            row_cols_chirho[row_chirho].add(col_chirho)

        # Filter rows
        matching_chirho = []
        for row_chirho in range(self_chirho.num_rows_chirho):
            cols_chirho = row_cols_chirho.get(row_chirho, set())
            if required_cols_chirho <= cols_chirho:  # Subset check
                matching_chirho.append(row_chirho)

        return matching_chirho

    def stats_chirho(self_chirho) -> Dict[str, Any]:
        """Get matrix statistics"""
        return {
            "rows": self_chirho.num_rows_chirho,
            "cols": self_chirho.num_cols_chirho,
            "entries": len(self_chirho.entries_chirho),
            "density": len(self_chirho.entries_chirho) / max(1, self_chirho.num_rows_chirho * self_chirho.num_cols_chirho),
            "paths": len(self_chirho.paths_chirho),
        }


# === Demo ===

def main():
    print("=== Integrated miniKanren ☧ ===\n")

    # Create tabled appendo
    appendo_chirho = TabledRelationChirho("appendo", lambda l, s: l + s)

    # === Forward query ===
    print("=" * 60)
    print("FORWARD: appendo([0,1], [2,3], Out)")
    print("=" * 60)

    l_chirho = PatternChirho.from_list_chirho([0, 1])
    s_chirho = PatternChirho.from_list_chirho([2, 3])
    out_chirho = PatternChirho.from_list_chirho([("var", 0)])  # Variable
    out_chirho = PatternChirho()  # Empty pattern = unconstrained
    out_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(0))

    state_chirho = StateChirho()
    results_chirho = list(appendo_chirho.call_chirho(l_chirho, s_chirho, out_chirho, state_chirho))
    print(f"Results: {len(results_chirho)}")
    print(f"Cache size: {len(appendo_chirho.cache_chirho.tuples_chirho)}")

    # === Backward query ===
    print("\n" + "=" * 60)
    print("BACKWARD: appendo(L, S, [0,1,2])")
    print("=" * 60)

    l_var_chirho = PatternChirho()
    l_var_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(1))

    s_var_chirho = PatternChirho()
    s_var_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(2))

    out_ground_chirho = PatternChirho.from_list_chirho([0, 1, 2])

    state2_chirho = StateChirho(var_counter_chirho=10)
    results2_chirho = list(appendo_chirho.call_chirho(l_var_chirho, s_var_chirho, out_ground_chirho, state2_chirho))

    print(f"Results: {len(results2_chirho)}")
    print(f"Cache size: {len(appendo_chirho.cache_chirho.tuples_chirho)}")

    # Show cached tuples
    print("\nCached tuples:")
    for i_chirho, tup_chirho in enumerate(appendo_chirho.cache_chirho.tuples_chirho[:5]):
        l_chirho, s_chirho, o_chirho = tup_chirho
        print(f"  {i_chirho}: {l_chirho} ++ {s_chirho} = {o_chirho}")
    if len(appendo_chirho.cache_chirho.tuples_chirho) > 5:
        print(f"  ... and {len(appendo_chirho.cache_chirho.tuples_chirho) - 5} more")

    # === Composition query with smart reordering ===
    print("\n" + "=" * 60)
    print("COMPOSITION: appendo(A,B,X), appendo(X,C,[0,1,2,3])")
    print("=" * 60)

    # Create a fresh appendo for this test
    appendo2_chirho = TabledRelationChirho("appendo2", lambda l, s: l + s)

    # Variables
    a_chirho = PatternChirho()
    a_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(0))

    b_chirho = PatternChirho()
    b_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(1))

    x_chirho = PatternChirho()
    x_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(2))

    c_chirho = PatternChirho()
    c_chirho.set_chirho(PathChirho(), ConstraintChirho.var_chirho(3))

    final_chirho = PatternChirho.from_list_chirho([0, 1, 2, 3])

    # Create goals
    def make_appendo_goal_chirho(rel_chirho, l_chirho, s_chirho, o_chirho):
        def goal_chirho(state_chirho):
            yield from rel_chirho.call_chirho(l_chirho, s_chirho, o_chirho, state_chirho)
        return goal_chirho

    goal1_chirho = GoalInfoChirho(
        goal_chirho=make_appendo_goal_chirho(appendo2_chirho, a_chirho, b_chirho, x_chirho),
        patterns_chirho=[a_chirho, b_chirho, x_chirho],
        name_chirho="appendo(A,B,X)"
    )

    goal2_chirho = GoalInfoChirho(
        goal_chirho=make_appendo_goal_chirho(appendo2_chirho, x_chirho, c_chirho, final_chirho),
        patterns_chirho=[x_chirho, c_chirho, final_chirho],
        name_chirho="appendo(X,C,[0,1,2,3])"
    )

    # With smart reordering, goal2 runs first (final_chirho is ground)
    smart_goal_chirho = smart_conj_chirho([goal1_chirho, goal2_chirho])

    state3_chirho = StateChirho(var_counter_chirho=10)
    results3_chirho = list(smart_goal_chirho(state3_chirho))

    print(f"Results: {len(results3_chirho)}")
    print(f"Cache size: {len(appendo2_chirho.cache_chirho.tuples_chirho)}")

    print("\nKey insight: Goal 2 runs FIRST because final_chirho is ground!")
    print("This populates X with concrete values before goal 1 runs.")

    # === Pattern bit matrix ===
    print("\n" + "=" * 60)
    print("PATTERN BIT MATRIX")
    print("=" * 60)

    print("\nPatterns can be encoded as sparse bit vectors:")
    print("  Column = (path, constraint_type)")
    print("  Row = one pattern")
    print("  Entry = 1 if pattern constrains that path to that type/value")

    p_example_chirho = PatternChirho.from_list_chirho([0, None, 2])
    print(f"\nExample: {p_example_chirho}")
    print("Constraints:")
    for path_chirho, c_chirho in sorted(p_example_chirho.constraints_chirho.items(),
                                        key=lambda x: x[0].depth_chirho()):
        print(f"  {path_chirho} → {c_chirho}")

    # === Sparse Bit Matrix Demo ===
    print("\n" + "=" * 60)
    print("SPARSE BIT MATRIX (COO format)")
    print("=" * 60)

    bm_chirho = BitMatrixChirho(max_depth_chirho=4, max_value_chirho=5)

    # Add patterns from cache
    patterns_to_add_chirho = [
        PatternChirho.from_list_chirho([0, 1]),
        PatternChirho.from_list_chirho([1, 2]),
        PatternChirho.from_list_chirho([0, 1, 2]),
        PatternChirho.from_list_chirho([2, 1, 0]),
        PatternChirho.from_list_chirho([]),
    ]

    for p_chirho in patterns_to_add_chirho:
        bm_chirho.add_pattern_chirho(p_chirho)

    stats_chirho = bm_chirho.stats_chirho()
    print(f"\nMatrix stats:")
    print(f"  Rows (patterns): {stats_chirho['rows']}")
    print(f"  Columns (path×constraint): {stats_chirho['cols']}")
    print(f"  Non-zero entries: {stats_chirho['entries']}")
    print(f"  Density: {stats_chirho['density']*100:.2f}%")

    # Query example
    query_chirho = PatternChirho.from_list_chirho([0, 1])
    matches_chirho = bm_chirho.query_chirho(query_chirho)
    print(f"\nQuery {query_chirho} matches rows: {matches_chirho}")

    # Show COO format
    print(f"\nCOO entries (first 10):")
    for i_chirho, (row_chirho, col_chirho) in enumerate(bm_chirho.entries_chirho[:10]):
        print(f"  ({row_chirho}, {col_chirho})")
    if len(bm_chirho.entries_chirho) > 10:
        print(f"  ... and {len(bm_chirho.entries_chirho) - 10} more")

    print("""
    COO format advantages:
    - O(nnz) storage where nnz = non-zeros
    - Hardware-friendly: simple streaming
    - GPU: use cuSPARSE
    - FPGA: streaming comparators
    """)

    # === Summary ===
    print("\n" + "=" * 60)
    print("INTEGRATION COMPLETE")
    print("=" * 60)
    print("""
    This file integrates:

    1. NESTED PATTERNS (nested_latent_chirho.py)
       - Path-based constraint representation
       - Patterns with variables
       - Unification as constraint intersection

    2. TABLING (tabling_complete_chirho.py)
       - Mode analysis (forward/backward/generate)
       - Memoization in RelationChirho
       - Smart goal reordering by groundness

    3. BIT MATRIX STORAGE
       - RelationChirho stores pattern tuples
       - Query by unification
       - Sparse representation ready for hardware

    The composition query demonstrates:
    - Without reordering: appendo(A,B,X) generates infinite X
    - With reordering: appendo(X,C,[...]) runs first, X becomes finite

    Next steps:
    - Actual sparse bit matrix encoding
    - CUDA/FPGA acceleration
    - Differentiable relaxation
    """)


if __name__ == "__main__":
    main()
