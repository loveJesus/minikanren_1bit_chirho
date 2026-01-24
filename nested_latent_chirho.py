#!/usr/bin/env python3
"""
Nested Latent Patterns for miniKanren ☧

Extends Gödel latent patterns to handle nested structure (cons cells).

Key insight: Instead of (position, value) encoding for flat lists,
use (path, constraint) encoding for trees.

Path = sequence of car/cdr traversals
Constraint = type (nil/cons/atom) + value for atoms

This connects:
- godel_latent_chirho.py (flat patterns)
- hashcons_chirho.py (nested terms)
- egraph_unify_chirho.py (equivalence classes for variables)
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Union, FrozenSet
from enum import Enum


# === Path Representation ===

class StepChirho(Enum):
    """Single step in a tree path"""
    CAR_CHIRHO = "car"  # Take the head of a cons
    CDR_CHIRHO = "cdr"  # Take the tail of a cons


@dataclass(frozen=True)
class PathChirho:
    """
    Path through a tree structure.

    Empty path () = root
    (car,) = head of root
    (cdr, car) = head of tail = second element
    (cdr, cdr) = tail of tail
    """
    steps_chirho: Tuple[StepChirho, ...] = ()

    def __repr__(self_chirho):
        if not self_chirho.steps_chirho:
            return "ε"  # epsilon for root
        return ".".join(s_chirho.value for s_chirho in self_chirho.steps_chirho)

    def car_chirho(self_chirho) -> 'PathChirho':
        """Extend path with car"""
        return PathChirho(self_chirho.steps_chirho + (StepChirho.CAR_CHIRHO,))

    def cdr_chirho(self_chirho) -> 'PathChirho':
        """Extend path with cdr"""
        return PathChirho(self_chirho.steps_chirho + (StepChirho.CDR_CHIRHO,))

    def depth_chirho(self_chirho) -> int:
        return len(self_chirho.steps_chirho)

    @classmethod
    def list_position_chirho(cls_chirho, index_chirho: int) -> 'PathChirho':
        """Path to the n-th element of a proper list"""
        steps_chirho = tuple([StepChirho.CDR_CHIRHO] * index_chirho + [StepChirho.CAR_CHIRHO])
        return PathChirho(steps_chirho)

    @classmethod
    def list_tail_chirho(cls_chirho, after_chirho: int) -> 'PathChirho':
        """Path to tail after n elements"""
        steps_chirho = tuple([StepChirho.CDR_CHIRHO] * after_chirho)
        return PathChirho(steps_chirho)


# === Type Constraints ===

class TypeChirho(Enum):
    """Type of a node in the tree"""
    NIL_CHIRHO = "nil"      # Empty list
    CONS_CHIRHO = "cons"    # Cons cell
    ATOM_CHIRHO = "atom"    # Atomic value (integer)
    ANY_CHIRHO = "any"      # Unknown (variable)


@dataclass(frozen=True)
class NodeConstraintChirho:
    """
    Constraint on a single node in the tree.

    type_chirho: What kind of node (nil/cons/atom/any)
    value_chirho: If atom, the specific value (None = any atom)
    var_id_chirho: If this is a logic variable, its ID
    """
    type_chirho: TypeChirho
    value_chirho: Optional[int] = None
    var_id_chirho: Optional[int] = None

    def __repr__(self_chirho):
        if self_chirho.type_chirho == TypeChirho.ANY_CHIRHO:
            if self_chirho.var_id_chirho is not None:
                return f"_{self_chirho.var_id_chirho}"
            return "?"
        elif self_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
            return "[]"
        elif self_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
            return "cons"
        elif self_chirho.type_chirho == TypeChirho.ATOM_CHIRHO:
            if self_chirho.value_chirho is not None:
                return str(self_chirho.value_chirho)
            return "atom"
        return "??"

    def is_ground_chirho(self_chirho) -> bool:
        """Is this a concrete (non-variable) constraint?"""
        return (self_chirho.type_chirho != TypeChirho.ANY_CHIRHO and
                self_chirho.var_id_chirho is None)

    @classmethod
    def nil_chirho(cls_chirho) -> 'NodeConstraintChirho':
        return NodeConstraintChirho(TypeChirho.NIL_CHIRHO)

    @classmethod
    def cons_chirho(cls_chirho) -> 'NodeConstraintChirho':
        return NodeConstraintChirho(TypeChirho.CONS_CHIRHO)

    @classmethod
    def atom_chirho(cls_chirho, value_chirho: int) -> 'NodeConstraintChirho':
        return NodeConstraintChirho(TypeChirho.ATOM_CHIRHO, value_chirho=value_chirho)

    @classmethod
    def var_chirho(cls_chirho, var_id_chirho: int) -> 'NodeConstraintChirho':
        return NodeConstraintChirho(TypeChirho.ANY_CHIRHO, var_id_chirho=var_id_chirho)

    @classmethod
    def any_chirho(cls_chirho) -> 'NodeConstraintChirho':
        return NodeConstraintChirho(TypeChirho.ANY_CHIRHO)


# === Nested Pattern ===

@dataclass
class NestedPatternChirho:
    """
    A pattern over nested structures.

    Represented as a map from paths to constraints.
    Paths not in the map are unconstrained (any value).

    Example: [?, 1, ?] as nested pattern
      ε → cons           (root is a cons)
      car → ?            (first element unknown)
      cdr → cons         (tail is cons)
      cdr.car → 1        (second element is 1)
      cdr.cdr → cons     (tail of tail is cons)
      cdr.cdr.car → ?    (third element unknown)
      cdr.cdr.cdr → nil  (end of list)
    """
    constraints_chirho: Dict[PathChirho, NodeConstraintChirho] = field(
        default_factory=dict
    )

    def __repr__(self_chirho):
        if not self_chirho.constraints_chirho:
            return "Pattern(?)"

        # Try to render as list if it's a proper list pattern
        try:
            return self_chirho._as_list_str_chirho()
        except Exception:
            pass

        # Fall back to path representation
        parts_chirho = []
        for path_chirho in sorted(self_chirho.constraints_chirho.keys(),
                                   key=lambda p_chirho: (p_chirho.depth_chirho(), str(p_chirho))):
            constraint_chirho = self_chirho.constraints_chirho[path_chirho]
            parts_chirho.append(f"{path_chirho}→{constraint_chirho}")
        return "Pattern{" + ", ".join(parts_chirho) + "}"

    def _as_list_str_chirho(self_chirho) -> str:
        """Try to render as [a, b, c, ...] notation"""
        elements_chirho = []
        path_chirho = PathChirho()

        while True:
            if path_chirho not in self_chirho.constraints_chirho:
                elements_chirho.append("...")
                break

            node_chirho = self_chirho.constraints_chirho[path_chirho]

            if node_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
                break
            elif node_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
                # Get the head
                car_path_chirho = path_chirho.car_chirho()
                if car_path_chirho in self_chirho.constraints_chirho:
                    head_chirho = self_chirho.constraints_chirho[car_path_chirho]
                    elements_chirho.append(str(head_chirho))
                else:
                    elements_chirho.append("?")

                # Move to tail
                path_chirho = path_chirho.cdr_chirho()
            else:
                # Not a proper list structure
                elements_chirho.append(str(node_chirho))
                break

        return "[" + ", ".join(elements_chirho) + "]"

    def set_constraint_chirho(self_chirho, path_chirho: PathChirho,
                              constraint_chirho: NodeConstraintChirho):
        """Add or update a constraint at a path"""
        self_chirho.constraints_chirho[path_chirho] = constraint_chirho

    def get_constraint_chirho(self_chirho, path_chirho: PathChirho) -> Optional[NodeConstraintChirho]:
        """Get constraint at path, or None if unconstrained"""
        return self_chirho.constraints_chirho.get(path_chirho)

    @classmethod
    def from_list_chirho(cls_chirho, elements_chirho: List[Union[int, None, Tuple[str, int]]],
                         ) -> 'NestedPatternChirho':
        """
        Create pattern from list notation.

        Elements can be:
        - int: concrete value
        - None: unknown (?)
        - ("var", id): named variable
        """
        pattern_chirho = NestedPatternChirho()
        path_chirho = PathChirho()

        for elem_chirho in elements_chirho:
            # Current position is a cons
            pattern_chirho.set_constraint_chirho(path_chirho, NodeConstraintChirho.cons_chirho())

            # Set the head constraint
            car_path_chirho = path_chirho.car_chirho()
            if elem_chirho is None:
                pattern_chirho.set_constraint_chirho(car_path_chirho, NodeConstraintChirho.any_chirho())
            elif isinstance(elem_chirho, tuple) and elem_chirho[0] == "var":
                pattern_chirho.set_constraint_chirho(car_path_chirho,
                                                     NodeConstraintChirho.var_chirho(elem_chirho[1]))
            else:
                pattern_chirho.set_constraint_chirho(car_path_chirho,
                                                     NodeConstraintChirho.atom_chirho(elem_chirho))

            # Move to tail
            path_chirho = path_chirho.cdr_chirho()

        # End with nil
        pattern_chirho.set_constraint_chirho(path_chirho, NodeConstraintChirho.nil_chirho())

        return pattern_chirho

    def get_variables_chirho(self_chirho) -> Dict[int, List[PathChirho]]:
        """Get all variable occurrences: var_id → [paths where it appears]"""
        variables_chirho: Dict[int, List[PathChirho]] = {}

        for path_chirho, constraint_chirho in self_chirho.constraints_chirho.items():
            if constraint_chirho.var_id_chirho is not None:
                var_id_chirho = constraint_chirho.var_id_chirho
                if var_id_chirho not in variables_chirho:
                    variables_chirho[var_id_chirho] = []
                variables_chirho[var_id_chirho].append(path_chirho)

        return variables_chirho


# === Unification of Nested Patterns ===

@dataclass
class UnifyResultChirho:
    """Result of unifying two nested patterns"""
    success_chirho: bool
    pattern_chirho: Optional[NestedPatternChirho] = None
    # Variable bindings discovered: var_id → constraint
    bindings_chirho: Dict[int, NodeConstraintChirho] = field(default_factory=dict)
    # Equality constraints: (var_id_1, var_id_2) must be equal
    equalities_chirho: Set[FrozenSet[int]] = field(default_factory=set)


def unify_constraints_chirho(c1_chirho: NodeConstraintChirho,
                              c2_chirho: NodeConstraintChirho) -> Optional[NodeConstraintChirho]:
    """
    Unify two node constraints.

    Returns the unified constraint, or None if incompatible.
    """
    # ANY matches anything
    if c1_chirho.type_chirho == TypeChirho.ANY_CHIRHO:
        return c2_chirho
    if c2_chirho.type_chirho == TypeChirho.ANY_CHIRHO:
        return c1_chirho

    # Types must match
    if c1_chirho.type_chirho != c2_chirho.type_chirho:
        return None

    # For atoms, values must match (if specified)
    if c1_chirho.type_chirho == TypeChirho.ATOM_CHIRHO:
        if c1_chirho.value_chirho is None:
            return c2_chirho
        if c2_chirho.value_chirho is None:
            return c1_chirho
        if c1_chirho.value_chirho != c2_chirho.value_chirho:
            return None

    return c1_chirho


def unify_nested_chirho(p1_chirho: NestedPatternChirho,
                        p2_chirho: NestedPatternChirho) -> UnifyResultChirho:
    """
    Unify two nested patterns.

    This is the KEY operation for nested structures!

    1. For each path, unify the constraints
    2. Collect variable bindings
    3. Generate equality constraints for variables
    """
    result_chirho = NestedPatternChirho()
    bindings_chirho: Dict[int, NodeConstraintChirho] = {}
    equalities_chirho: Set[FrozenSet[int]] = set()

    # All paths from both patterns
    all_paths_chirho = set(p1_chirho.constraints_chirho.keys()) | set(p2_chirho.constraints_chirho.keys())

    for path_chirho in all_paths_chirho:
        c1_chirho = p1_chirho.constraints_chirho.get(path_chirho, NodeConstraintChirho.any_chirho())
        c2_chirho = p2_chirho.constraints_chirho.get(path_chirho, NodeConstraintChirho.any_chirho())

        # Handle variables
        var1_chirho = c1_chirho.var_id_chirho
        var2_chirho = c2_chirho.var_id_chirho

        if var1_chirho is not None and var2_chirho is not None:
            # Both are variables - create equality constraint
            if var1_chirho != var2_chirho:
                equalities_chirho.add(frozenset([var1_chirho, var2_chirho]))
            # Keep one of them as placeholder
            result_chirho.set_constraint_chirho(path_chirho, c1_chirho)

        elif var1_chirho is not None:
            # c1 is variable, c2 is concrete
            if var1_chirho in bindings_chirho:
                # Check consistency with existing binding
                existing_chirho = bindings_chirho[var1_chirho]
                unified_chirho = unify_constraints_chirho(existing_chirho, c2_chirho)
                if unified_chirho is None:
                    return UnifyResultChirho(success_chirho=False)
                bindings_chirho[var1_chirho] = unified_chirho
            else:
                bindings_chirho[var1_chirho] = c2_chirho
            result_chirho.set_constraint_chirho(path_chirho, c2_chirho)

        elif var2_chirho is not None:
            # c2 is variable, c1 is concrete
            if var2_chirho in bindings_chirho:
                existing_chirho = bindings_chirho[var2_chirho]
                unified_chirho = unify_constraints_chirho(existing_chirho, c1_chirho)
                if unified_chirho is None:
                    return UnifyResultChirho(success_chirho=False)
                bindings_chirho[var2_chirho] = unified_chirho
            else:
                bindings_chirho[var2_chirho] = c1_chirho
            result_chirho.set_constraint_chirho(path_chirho, c1_chirho)

        else:
            # Both concrete - must unify
            unified_chirho = unify_constraints_chirho(c1_chirho, c2_chirho)
            if unified_chirho is None:
                return UnifyResultChirho(success_chirho=False)
            result_chirho.set_constraint_chirho(path_chirho, unified_chirho)

    return UnifyResultChirho(
        success_chirho=True,
        pattern_chirho=result_chirho,
        bindings_chirho=bindings_chirho,
        equalities_chirho=equalities_chirho
    )


# === Bit Matrix Encoding for Nested Patterns ===

class NestedPatternMatrixChirho:
    """
    Encode nested patterns as bit matrix.

    Columns are (path, constraint_type) pairs.
    This generalizes PatternMatrixChirho from flat to nested.

    For hardware: this could be a sparse CAM where
    - Key = path encoding
    - Value = constraint encoding
    """

    def __init__(self_chirho, max_depth_chirho: int = 5, max_value_chirho: int = 10):
        self_chirho.max_depth_chirho = max_depth_chirho
        self_chirho.max_value_chirho = max_value_chirho
        self_chirho.patterns_chirho: List[NestedPatternChirho] = []

        # Pre-compute all paths up to max_depth
        self_chirho.paths_chirho = self_chirho._enumerate_paths_chirho(max_depth_chirho)

        # Column layout: for each path, we have columns for:
        # - type=nil (1 bit)
        # - type=cons (1 bit)
        # - type=atom, value=0 (1 bit)
        # - type=atom, value=1 (1 bit)
        # - ... up to max_value
        # Total columns per path = 2 + max_value
        self_chirho.cols_per_path_chirho = 2 + max_value_chirho

    def _enumerate_paths_chirho(self_chirho, max_depth_chirho: int) -> List[PathChirho]:
        """Generate all paths up to given depth"""
        paths_chirho = [PathChirho()]  # Root

        frontier_chirho = [PathChirho()]
        for _ in range(max_depth_chirho):
            new_frontier_chirho = []
            for path_chirho in frontier_chirho:
                car_chirho = path_chirho.car_chirho()
                cdr_chirho = path_chirho.cdr_chirho()
                paths_chirho.append(car_chirho)
                paths_chirho.append(cdr_chirho)
                new_frontier_chirho.append(car_chirho)
                new_frontier_chirho.append(cdr_chirho)
            frontier_chirho = new_frontier_chirho

        return paths_chirho

    def add_pattern_chirho(self_chirho, pattern_chirho: NestedPatternChirho):
        self_chirho.patterns_chirho.append(pattern_chirho)

    def to_bit_matrix_chirho(self_chirho) -> List[List[int]]:
        """
        Convert patterns to bit matrix.

        Each row = one pattern
        Each column = (path, constraint_type, value) tuple
        """
        num_cols_chirho = len(self_chirho.paths_chirho) * self_chirho.cols_per_path_chirho
        matrix_chirho = []

        for pattern_chirho in self_chirho.patterns_chirho:
            row_chirho = [0] * num_cols_chirho

            for path_idx_chirho, path_chirho in enumerate(self_chirho.paths_chirho):
                base_col_chirho = path_idx_chirho * self_chirho.cols_per_path_chirho

                constraint_chirho = pattern_chirho.get_constraint_chirho(path_chirho)
                if constraint_chirho is None or constraint_chirho.type_chirho == TypeChirho.ANY_CHIRHO:
                    # Unconstrained - don't set any bits
                    continue

                if constraint_chirho.type_chirho == TypeChirho.NIL_CHIRHO:
                    row_chirho[base_col_chirho + 0] = 1
                elif constraint_chirho.type_chirho == TypeChirho.CONS_CHIRHO:
                    row_chirho[base_col_chirho + 1] = 1
                elif constraint_chirho.type_chirho == TypeChirho.ATOM_CHIRHO:
                    if constraint_chirho.value_chirho is not None:
                        val_chirho = constraint_chirho.value_chirho
                        if val_chirho < self_chirho.max_value_chirho:
                            row_chirho[base_col_chirho + 2 + val_chirho] = 1

            matrix_chirho.append(row_chirho)

        return matrix_chirho

    def column_labels_chirho(self_chirho) -> List[str]:
        """Get human-readable column labels"""
        labels_chirho = []
        for path_chirho in self_chirho.paths_chirho:
            path_str_chirho = str(path_chirho)
            labels_chirho.append(f"{path_str_chirho}:nil")
            labels_chirho.append(f"{path_str_chirho}:cons")
            for v_chirho in range(self_chirho.max_value_chirho):
                labels_chirho.append(f"{path_str_chirho}={v_chirho}")
        return labels_chirho


# === Occurs Check via Path Analysis ===

def is_prefix_chirho(p1_chirho: PathChirho, p2_chirho: PathChirho) -> bool:
    """Check if p1 is a proper prefix of p2 (p1 is ancestor of p2)"""
    s1_chirho = p1_chirho.steps_chirho
    s2_chirho = p2_chirho.steps_chirho

    if len(s1_chirho) >= len(s2_chirho):
        return False

    return s2_chirho[:len(s1_chirho)] == s1_chirho


def check_occurs_chirho(pattern_chirho: NestedPatternChirho) -> List[Tuple[int, PathChirho, PathChirho]]:
    """
    Detect occurs check violations in a pattern.

    An occurs check violation happens when a variable X appears at path P
    and also at a path that is a descendant of P (e.g., P.car or P.cdr).

    This would create an infinite structure: X = cons(X, ...)

    Returns list of (var_id, ancestor_path, descendant_path) violations.
    """
    violations_chirho: List[Tuple[int, PathChirho, PathChirho]] = []

    # Get all variable occurrences
    variables_chirho = pattern_chirho.get_variables_chirho()

    for var_id_chirho, paths_chirho in variables_chirho.items():
        # Check all pairs of paths for this variable
        for i_chirho, p1_chirho in enumerate(paths_chirho):
            for p2_chirho in paths_chirho[i_chirho + 1:]:
                if is_prefix_chirho(p1_chirho, p2_chirho):
                    violations_chirho.append((var_id_chirho, p1_chirho, p2_chirho))
                elif is_prefix_chirho(p2_chirho, p1_chirho):
                    violations_chirho.append((var_id_chirho, p2_chirho, p1_chirho))

    return violations_chirho


def unify_with_occurs_check_chirho(p1_chirho: NestedPatternChirho,
                                    p2_chirho: NestedPatternChirho) -> UnifyResultChirho:
    """
    Unify two patterns with occurs check.

    First unifies, then checks for cycles in the result.
    """
    result_chirho = unify_nested_chirho(p1_chirho, p2_chirho)

    if not result_chirho.success_chirho:
        return result_chirho

    # Check for occurs violations
    violations_chirho = check_occurs_chirho(result_chirho.pattern_chirho)

    if violations_chirho:
        return UnifyResultChirho(
            success_chirho=False,
            pattern_chirho=None,
            bindings_chirho={},
            equalities_chirho=set()
        )

    return result_chirho


# === Demo ===

def main():
    print("=== Nested Latent Patterns ☧ ===\n")

    # === Path examples ===
    print("=" * 60)
    print("PATHS IN A TREE")
    print("=" * 60)

    print("\nList [a, b, c] as paths:")
    print("  ε (root) → cons")
    print("  car → a (first element)")
    print("  cdr → cons")
    print("  cdr.car → b (second element)")
    print("  cdr.cdr → cons")
    print("  cdr.cdr.car → c (third element)")
    print("  cdr.cdr.cdr → nil (end)")

    print("\nUsing PathChirho:")
    root_chirho = PathChirho()
    print(f"  Root: {root_chirho}")
    print(f"  First elem: {PathChirho.list_position_chirho(0)}")
    print(f"  Second elem: {PathChirho.list_position_chirho(1)}")
    print(f"  Third elem: {PathChirho.list_position_chirho(2)}")
    print(f"  Tail after 2: {PathChirho.list_tail_chirho(2)}")

    # === Creating patterns ===
    print("\n" + "=" * 60)
    print("NESTED PATTERNS")
    print("=" * 60)

    # [?, 1, ?]
    p1_chirho = NestedPatternChirho.from_list_chirho([None, 1, None])
    print(f"\nPattern 1: [?, 1, ?]")
    print(f"  Rendered: {p1_chirho}")

    # [0, ?, 2]
    p2_chirho = NestedPatternChirho.from_list_chirho([0, None, 2])
    print(f"\nPattern 2: [0, ?, 2]")
    print(f"  Rendered: {p2_chirho}")

    # Pattern with explicit paths
    print("\nManual pattern construction:")
    for path_chirho, constraint_chirho in sorted(p1_chirho.constraints_chirho.items(),
                                                   key=lambda x_chirho: x_chirho[0].depth_chirho()):
        print(f"  {path_chirho} → {constraint_chirho}")

    # === Unification ===
    print("\n" + "=" * 60)
    print("UNIFICATION")
    print("=" * 60)

    result_chirho = unify_nested_chirho(p1_chirho, p2_chirho)
    print(f"\n{p1_chirho._as_list_str_chirho()} ∧ {p2_chirho._as_list_str_chirho()}")
    if result_chirho.success_chirho:
        print(f"  = {result_chirho.pattern_chirho._as_list_str_chirho()}")
    else:
        print("  = FAIL")

    # Conflicting patterns
    p3_chirho = NestedPatternChirho.from_list_chirho([1, 2, 3])
    p4_chirho = NestedPatternChirho.from_list_chirho([9, 2, 3])

    result2_chirho = unify_nested_chirho(p3_chirho, p4_chirho)
    print(f"\n{p3_chirho._as_list_str_chirho()} ∧ {p4_chirho._as_list_str_chirho()}")
    print(f"  = {'SUCCESS' if result2_chirho.success_chirho else 'FAIL (first elements differ)'}")

    # === Variables ===
    print("\n" + "=" * 60)
    print("VARIABLES (shared across positions)")
    print("=" * 60)

    # [X, 1, X] - same variable at positions 0 and 2
    p5_chirho = NestedPatternChirho.from_list_chirho([("var", 0), 1, ("var", 0)])
    print(f"\nPattern with shared variable: {p5_chirho._as_list_str_chirho()}")

    vars_chirho = p5_chirho.get_variables_chirho()
    print(f"  Variable occurrences:")
    for var_id_chirho, paths_chirho in vars_chirho.items():
        print(f"    _{var_id_chirho} appears at: {[str(p_chirho) for p_chirho in paths_chirho]}")

    # Unify with [2, 1, ?]
    p6_chirho = NestedPatternChirho.from_list_chirho([2, 1, None])
    result3_chirho = unify_nested_chirho(p5_chirho, p6_chirho)

    print(f"\nUnify [_0, 1, _0] with [2, 1, ?]:")
    if result3_chirho.success_chirho:
        print(f"  Result: {result3_chirho.pattern_chirho._as_list_str_chirho()}")
        print(f"  Bindings: {result3_chirho.bindings_chirho}")
        # Note: _0 gets bound to 2, and the third position should also become 2

    # Unify two patterns with variables
    p7_chirho = NestedPatternChirho.from_list_chirho([("var", 0), ("var", 1)])
    p8_chirho = NestedPatternChirho.from_list_chirho([("var", 2), ("var", 2)])

    result4_chirho = unify_nested_chirho(p7_chirho, p8_chirho)
    print(f"\nUnify [_0, _1] with [_2, _2]:")
    if result4_chirho.success_chirho:
        print(f"  Equalities: {result4_chirho.equalities_chirho}")
        print("  (This means _0=_2 and _1=_2, so _0=_1)")

    # === Bit Matrix ===
    print("\n" + "=" * 60)
    print("BIT MATRIX ENCODING")
    print("=" * 60)

    matrix_chirho = NestedPatternMatrixChirho(max_depth_chirho=3, max_value_chirho=4)

    # Add some patterns
    matrix_chirho.add_pattern_chirho(NestedPatternChirho.from_list_chirho([0, 1, 2]))
    matrix_chirho.add_pattern_chirho(NestedPatternChirho.from_list_chirho([1, 1, 0]))
    matrix_chirho.add_pattern_chirho(NestedPatternChirho.from_list_chirho([0, 2, 1]))

    bit_matrix_chirho = matrix_chirho.to_bit_matrix_chirho()
    labels_chirho = matrix_chirho.column_labels_chirho()

    print(f"\nMatrix shape: {len(bit_matrix_chirho)} patterns × {len(labels_chirho)} columns")
    print(f"(Columns encode path×constraint pairs)")

    print("\nNon-zero entries per pattern:")
    for i_chirho, row_chirho in enumerate(bit_matrix_chirho):
        nonzero_chirho = [(j_chirho, labels_chirho[j_chirho])
                         for j_chirho, bit_chirho in enumerate(row_chirho) if bit_chirho]
        # Just show a few
        preview_chirho = nonzero_chirho[:6]
        suffix_chirho = f" ... +{len(nonzero_chirho)-6} more" if len(nonzero_chirho) > 6 else ""
        print(f"  Pattern {i_chirho}: {[label_chirho for _, label_chirho in preview_chirho]}{suffix_chirho}")

    # === Occurs check ===
    print("\n" + "=" * 60)
    print("OCCURS CHECK (cycle detection)")
    print("=" * 60)

    # Create a pattern where X appears at root and inside root
    # This is like X = cons(X, nil) - an infinite structure!
    cyclic_pattern_chirho = NestedPatternChirho()
    cyclic_pattern_chirho.set_constraint_chirho(PathChirho(), NodeConstraintChirho.var_chirho(0))  # X at root
    cyclic_pattern_chirho.set_constraint_chirho(PathChirho().car_chirho(), NodeConstraintChirho.var_chirho(0))  # X at car

    print("\nCyclic pattern: X at ε, X at car")
    print("  (This means X = cons(X, ...) - infinite!)")

    violations_chirho = check_occurs_chirho(cyclic_pattern_chirho)
    print(f"\nOccurs check violations: {len(violations_chirho)}")
    for var_chirho, p1_chirho, p2_chirho in violations_chirho:
        print(f"  Variable _{var_chirho}: {p1_chirho} is prefix of {p2_chirho}")

    # Safe pattern: X at positions 0 and 2 (siblings, not ancestors)
    safe_pattern_chirho = NestedPatternChirho.from_list_chirho([("var", 0), 1, ("var", 0)])
    safe_violations_chirho = check_occurs_chirho(safe_pattern_chirho)
    print(f"\nSafe pattern [X, 1, X]:")
    print(f"  Violations: {len(safe_violations_chirho)} (none - positions are siblings)")

    # Unify with occurs check - direct cycle
    print("\nUnify with occurs check (direct cycle in result):")
    # Create a pattern that already has the cycle
    p_cyclic_direct_chirho = NestedPatternChirho()
    p_cyclic_direct_chirho.set_constraint_chirho(PathChirho(), NodeConstraintChirho.cons_chirho())
    p_cyclic_direct_chirho.set_constraint_chirho(PathChirho().car_chirho(), NodeConstraintChirho.var_chirho(0))
    p_cyclic_direct_chirho.set_constraint_chirho(PathChirho().cdr_chirho(), NodeConstraintChirho.var_chirho(0))  # Same var!

    # Constraint that car = whole structure (creates cycle)
    p_constraint_chirho = NestedPatternChirho()
    p_constraint_chirho.set_constraint_chirho(PathChirho(), NodeConstraintChirho.var_chirho(0))

    print("  Pattern 1: cons(X, X) where X at car and cdr")
    print("  Pattern 2: X at root (X = cons(X, X))")
    result_direct_chirho = unify_with_occurs_check_chirho(p_cyclic_direct_chirho, p_constraint_chirho)
    print(f"  Result: {'FAIL (occurs check)' if not result_direct_chirho.success_chirho else 'SUCCESS'}")

    print("\n  Note: Full occurs check requires tracking variable=structure bindings")
    print("  When X = cons(Y, Z), X's paths expand to include Y's and Z's paths")

    # === The insight ===
    print("\n" + "=" * 60)
    print("THE NESTED LATENT INSIGHT")
    print("=" * 60)
    print("""
    Extending flat patterns to nested:

    FLAT (godel_latent_chirho.py):
      - Position 0, 1, 2, ...
      - Column = (position, value)
      - Only works for fixed-length sequences

    NESTED (this file):
      - Path = car, cdr.car, cdr.cdr.car, ...
      - Column = (path, type/value)
      - Works for any tree structure!

    Key operations:
      - Unification: intersect constraints at each path
      - Variable binding: propagate constraints to all occurrences
      - Equality constraints: link variables that must be equal

    Connection to e-graphs:
      - Variables = e-class IDs
      - Variable binding = e-class merging
      - Shared variables = equality constraints

    Hardware mapping:
      - Paths can be encoded as bit strings (car=0, cdr=1)
      - Constraints as small integers
      - CAM (Content Addressable Memory) for path lookup
      - Parallel constraint checking across all paths

    The constraint lattice:
      ANY > NIL, CONS, ATOM(any) > ATOM(specific)

      Unification = meet in this lattice
      Failure = bottom (inconsistent constraints)

    What this ENABLES:
      ✓ Full miniKanren patterns (not just Datalog facts)
      ✓ Nested structure handling
      ✓ Shared variables across positions
      ✓ Natural bit-vector representation
      ✓ Parallelizable unification
      ✓ OCCURS CHECK AS PATH PREFIX TEST

    Occurs check insight:
      - Variable X at path P and at path P.car → cycle!
      - Test: is any var path a prefix of another?
      - This is a simple string prefix test on path encodings
      - Hardware: parallel prefix comparators

    What remains:
      - Infinite structures (lazy path generation)
      - Efficient path enumeration ordering
      - Integration with tabling system
    """)


if __name__ == "__main__":
    main()
