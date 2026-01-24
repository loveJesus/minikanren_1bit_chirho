#!/usr/bin/env python3
"""
E-Graph Unification for miniKanren ☧

The hypothesis: Substitution chains can be replaced with e-graph equivalence classes.

Instead of:
    subst = {x → y, y → z, z → [1,2]}
    walk(x) = chase chain = [1,2]

We have:
    e-class 7 = {x, y, z, [1,2]}
    canonical(7) = [1,2]
    find(x) = find(y) = find(z) = 7
    walk(x) = canonical(find(x)) = [1,2]

Benefits:
- O(α(n)) ≈ O(1) amortized instead of O(chain length)
- Path compression built-in
- Well-studied for hardware (union-find)
- Connects miniKanren to egg/e-graphs

This file explores whether this actually works.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Any, Union
from collections import defaultdict


# === Terms ===

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
    """Cons cell - BUT stores e-class IDs, not terms directly"""
    head_chirho: int  # e-class ID
    tail_chirho: int  # e-class ID

    def __repr__(self_chirho):
        return f"Cons({self_chirho.head_chirho}, {self_chirho.tail_chirho})"


# For display purposes
@dataclass(frozen=True)
class IntChirho:
    """Integer constant"""
    val_chirho: int
    def __repr__(self_chirho): return str(self_chirho.val_chirho)


TermChirho = Union[VarChirho, NilChirho, ConsChirho, IntChirho]


# === E-Graph with Union-Find ===

class EGraphChirho:
    """
    E-graph that serves as the substitution mechanism.

    Key idea: Instead of {var → term}, we have equivalence classes.
    All equivalent terms (including variables) share an e-class.

    Operations:
    - add(term) → e-class ID
    - union(id1, id2) → merge classes (this IS unification!)
    - find(id) → canonical class ID
    - canonical(id) → canonical term for the class
    """

    def __init__(self_chirho):
        # Union-find parent pointers
        self_chirho.parent_chirho: Dict[int, int] = {}
        # Rank for union by rank
        self_chirho.rank_chirho: Dict[int, int] = {}

        # E-class contents: id → set of terms in this class
        self_chirho.classes_chirho: Dict[int, Set[TermChirho]] = {}

        # Term → e-class ID (for deduplication)
        self_chirho.term_to_id_chirho: Dict[TermChirho, int] = {}

        # Counter for fresh IDs
        self_chirho.next_id_chirho = 0

        # Variable counter
        self_chirho.var_counter_chirho = 0

    def fresh_var_chirho(self_chirho) -> Tuple[VarChirho, int]:
        """Create fresh variable and return (var, e-class ID)"""
        var_chirho = VarChirho(self_chirho.var_counter_chirho)
        self_chirho.var_counter_chirho += 1
        id_chirho = self_chirho.add_chirho(var_chirho)
        return var_chirho, id_chirho

    def add_chirho(self_chirho, term_chirho: TermChirho) -> int:
        """
        Add term to e-graph, return its e-class ID.
        If term already exists, return existing ID.
        """
        # Check if already present
        if term_chirho in self_chirho.term_to_id_chirho:
            return self_chirho.find_chirho(self_chirho.term_to_id_chirho[term_chirho])

        # Create new e-class
        id_chirho = self_chirho.next_id_chirho
        self_chirho.next_id_chirho += 1

        self_chirho.parent_chirho[id_chirho] = id_chirho  # Self-parent
        self_chirho.rank_chirho[id_chirho] = 0
        self_chirho.classes_chirho[id_chirho] = {term_chirho}
        self_chirho.term_to_id_chirho[term_chirho] = id_chirho

        return id_chirho

    def find_chirho(self_chirho, id_chirho: int) -> int:
        """
        Find canonical representative with path compression.
        This is the O(α(n)) operation!
        """
        if self_chirho.parent_chirho[id_chirho] != id_chirho:
            # Path compression: point directly to root
            self_chirho.parent_chirho[id_chirho] = self_chirho.find_chirho(self_chirho.parent_chirho[id_chirho])
        return self_chirho.parent_chirho[id_chirho]

    def union_chirho(self_chirho, id1_chirho: int, id2_chirho: int) -> Optional[int]:
        """
        Merge two e-classes. Returns merged ID or None if conflict.

        This IS unification! When we unify x and y:
        - If both are vars: merge classes
        - If one is ground: var's class gets the ground term
        - If both ground: check equality, merge if equal
        """
        root1_chirho = self_chirho.find_chirho(id1_chirho)
        root2_chirho = self_chirho.find_chirho(id2_chirho)

        if root1_chirho == root2_chirho:
            return root1_chirho  # Already unified

        # Union by rank
        if self_chirho.rank_chirho[root1_chirho] < self_chirho.rank_chirho[root2_chirho]:
            root1_chirho, root2_chirho = root2_chirho, root1_chirho

        self_chirho.parent_chirho[root2_chirho] = root1_chirho
        self_chirho.classes_chirho[root1_chirho] |= self_chirho.classes_chirho[root2_chirho]

        if self_chirho.rank_chirho[root1_chirho] == self_chirho.rank_chirho[root2_chirho]:
            self_chirho.rank_chirho[root1_chirho] += 1

        return root1_chirho

    def get_canonical_chirho(self_chirho, id_chirho: int) -> Optional[TermChirho]:
        """
        Get canonical (most ground) term for an e-class.
        Prefers: ground terms > partially ground > variables
        """
        root_chirho = self_chirho.find_chirho(id_chirho)
        terms_chirho = self_chirho.classes_chirho[root_chirho]

        # Priority: IntChirho/NilChirho > ConsChirho > VarChirho
        for term_chirho in terms_chirho:
            if isinstance(term_chirho, (IntChirho, NilChirho)):
                return term_chirho

        for term_chirho in terms_chirho:
            if isinstance(term_chirho, ConsChirho):
                return term_chirho

        # Return any variable
        for term_chirho in terms_chirho:
            if isinstance(term_chirho, VarChirho):
                return term_chirho

        return None

    def is_ground_class_chirho(self_chirho, id_chirho: int) -> bool:
        """Check if e-class contains a ground term"""
        root_chirho = self_chirho.find_chirho(id_chirho)
        terms_chirho = self_chirho.classes_chirho[root_chirho]

        for term_chirho in terms_chirho:
            if isinstance(term_chirho, (IntChirho, NilChirho)):
                return True
            if isinstance(term_chirho, ConsChirho):
                # Need to recursively check
                head_ground_chirho = self_chirho.is_ground_class_chirho(term_chirho.head_chirho)
                tail_ground_chirho = self_chirho.is_ground_class_chirho(term_chirho.tail_chirho)
                if head_ground_chirho and tail_ground_chirho:
                    return True

        return False

    def copy_chirho(self_chirho) -> 'EGraphChirho':
        """Deep copy for branching search"""
        new_chirho = EGraphChirho()
        new_chirho.parent_chirho = self_chirho.parent_chirho.copy()
        new_chirho.rank_chirho = self_chirho.rank_chirho.copy()
        new_chirho.classes_chirho = {k_chirho: v_chirho.copy() for k_chirho, v_chirho in self_chirho.classes_chirho.items()}
        new_chirho.term_to_id_chirho = self_chirho.term_to_id_chirho.copy()
        new_chirho.next_id_chirho = self_chirho.next_id_chirho
        new_chirho.var_counter_chirho = self_chirho.var_counter_chirho
        return new_chirho

    def display_chirho(self_chirho):
        """Show e-graph state"""
        print("E-Graph:")
        roots_chirho = set(self_chirho.find_chirho(id_chirho) for id_chirho in self_chirho.parent_chirho)
        for root_chirho in sorted(roots_chirho):
            terms_chirho = self_chirho.classes_chirho[root_chirho]
            canonical_chirho = self_chirho.get_canonical_chirho(root_chirho)
            print(f"  e{root_chirho}: {terms_chirho} → canonical: {canonical_chirho}")


# === E-Graph Unification ===

def unify_egraph_chirho(
    id1_chirho: int,
    id2_chirho: int,
    egraph_chirho: EGraphChirho
) -> Optional[EGraphChirho]:
    """
    Unify two e-classes in the e-graph.

    Returns new e-graph state or None on failure.

    This is where the magic happens:
    - Variables unify with anything (union classes)
    - Ground terms must match structurally
    - Cons cells: recursively unify head and tail
    """
    root1_chirho = egraph_chirho.find_chirho(id1_chirho)
    root2_chirho = egraph_chirho.find_chirho(id2_chirho)

    if root1_chirho == root2_chirho:
        return egraph_chirho  # Already equal

    # Get canonical terms for each class
    term1_chirho = egraph_chirho.get_canonical_chirho(root1_chirho)
    term2_chirho = egraph_chirho.get_canonical_chirho(root2_chirho)

    # Case 1: At least one is a variable - just union
    if isinstance(term1_chirho, VarChirho) or isinstance(term2_chirho, VarChirho):
        # TODO: Occurs check would go here
        egraph_chirho.union_chirho(root1_chirho, root2_chirho)
        return egraph_chirho

    # Case 2: Both are nil - trivially equal
    if isinstance(term1_chirho, NilChirho) and isinstance(term2_chirho, NilChirho):
        egraph_chirho.union_chirho(root1_chirho, root2_chirho)
        return egraph_chirho

    # Case 3: Both are integers - check equality
    if isinstance(term1_chirho, IntChirho) and isinstance(term2_chirho, IntChirho):
        if term1_chirho.val_chirho == term2_chirho.val_chirho:
            egraph_chirho.union_chirho(root1_chirho, root2_chirho)
            return egraph_chirho
        return None  # Conflict!

    # Case 4: Both are cons - recursively unify
    if isinstance(term1_chirho, ConsChirho) and isinstance(term2_chirho, ConsChirho):
        # Unify heads
        result_chirho = unify_egraph_chirho(term1_chirho.head_chirho, term2_chirho.head_chirho, egraph_chirho)
        if result_chirho is None:
            return None

        # Unify tails
        result_chirho = unify_egraph_chirho(term1_chirho.tail_chirho, term2_chirho.tail_chirho, result_chirho)
        if result_chirho is None:
            return None

        # Union the cons classes themselves
        result_chirho.union_chirho(root1_chirho, root2_chirho)
        return result_chirho

    # Case 5: Mismatched types
    return None


# === Building Lists in E-Graph ===

def build_list_chirho(elems_chirho: List[int], egraph_chirho: EGraphChirho) -> int:
    """
    Build a list in the e-graph from element values.
    Returns e-class ID of the list.
    """
    result_id_chirho = egraph_chirho.add_chirho(NilChirho())

    for elem_chirho in reversed(elems_chirho):
        elem_id_chirho = egraph_chirho.add_chirho(IntChirho(elem_chirho))
        cons_chirho = ConsChirho(elem_id_chirho, result_id_chirho)
        result_id_chirho = egraph_chirho.add_chirho(cons_chirho)

    return result_id_chirho


def extract_list_chirho(id_chirho: int, egraph_chirho: EGraphChirho) -> Optional[List[int]]:
    """Extract a ground list from e-graph, or None if not ground"""
    result_chirho = []
    current_chirho = id_chirho

    while True:
        term_chirho = egraph_chirho.get_canonical_chirho(current_chirho)

        if isinstance(term_chirho, NilChirho):
            return result_chirho

        if isinstance(term_chirho, ConsChirho):
            head_term_chirho = egraph_chirho.get_canonical_chirho(term_chirho.head_chirho)
            if isinstance(head_term_chirho, IntChirho):
                result_chirho.append(head_term_chirho.val_chirho)
                current_chirho = term_chirho.tail_chirho
            else:
                return None  # Head not ground

        elif isinstance(term_chirho, VarChirho):
            return None  # Not ground

        else:
            return None


# === Search State ===

@dataclass
class EStateChirho:
    """Search state using e-graph"""
    egraph_chirho: EGraphChirho

    def copy_chirho(self_chirho) -> 'EStateChirho':
        return EStateChirho(self_chirho.egraph_chirho.copy_chirho())


# === Goals ===

EGoalChirho = Any  # Callable[[EStateChirho], Iterator[EStateChirho]]


def eq_egraph_chirho(id1_chirho: int, id2_chirho: int) -> EGoalChirho:
    """Unification goal using e-graph"""
    def goal_chirho(state_chirho: EStateChirho) -> Iterator[EStateChirho]:
        new_egraph_chirho = unify_egraph_chirho(id1_chirho, id2_chirho, state_chirho.egraph_chirho.copy_chirho())
        if new_egraph_chirho is not None:
            yield EStateChirho(new_egraph_chirho)
    return goal_chirho


def conj_egraph_chirho(g1_chirho: EGoalChirho, g2_chirho: EGoalChirho) -> EGoalChirho:
    """Conjunction"""
    def goal_chirho(state_chirho: EStateChirho) -> Iterator[EStateChirho]:
        for st1_chirho in g1_chirho(state_chirho):
            yield from g2_chirho(st1_chirho)
    return goal_chirho


def disj_egraph_chirho(g1_chirho: EGoalChirho, g2_chirho: EGoalChirho) -> EGoalChirho:
    """Disjunction (interleaving)"""
    def goal_chirho(state_chirho: EStateChirho) -> Iterator[EStateChirho]:
        iters_chirho = [g1_chirho(state_chirho.copy_chirho()), g2_chirho(state_chirho.copy_chirho())]
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


# === Demo ===

def main():
    print("=== E-Graph Unification for miniKanren ☧ ===\n")

    print("""
    Hypothesis: Replace substitution chains with e-graph equivalence classes.

    Traditional:  subst = {x → y, y → [1,2]}
                  walk(x) = [1,2]  (2 lookups)

    E-graph:      e-class 5 = {x, y, [1,2]}
                  find(x) = find(y) = 5
                  canonical(5) = [1,2]  (O(α(n)) ≈ O(1))
    """)

    # === Test 1: Basic unification ===
    print("="*60)
    print("TEST 1: Basic unification")
    print("="*60)

    eg1_chirho = EGraphChirho()

    # Create x, y variables
    x_chirho, x_id_chirho = eg1_chirho.fresh_var_chirho()
    y_chirho, y_id_chirho = eg1_chirho.fresh_var_chirho()

    # Create list [1, 2]
    list_id_chirho = build_list_chirho([1, 2], eg1_chirho)

    print(f"\nBefore unification:")
    print(f"  x = {x_chirho} (e-class {x_id_chirho})")
    print(f"  y = {y_chirho} (e-class {y_id_chirho})")
    print(f"  [1,2] (e-class {list_id_chirho})")
    eg1_chirho.display_chirho()

    # Unify x = y
    print(f"\n--- Unify x = y ---")
    eg1_chirho = unify_egraph_chirho(x_id_chirho, y_id_chirho, eg1_chirho)
    eg1_chirho.display_chirho()

    # Unify y = [1,2]
    print(f"\n--- Unify y = [1,2] ---")
    eg1_chirho = unify_egraph_chirho(y_id_chirho, list_id_chirho, eg1_chirho)
    eg1_chirho.display_chirho()

    # Now x, y, and [1,2] should all be in same class
    print(f"\nAfter unification:")
    print(f"  find(x) = {eg1_chirho.find_chirho(x_id_chirho)}")
    print(f"  find(y) = {eg1_chirho.find_chirho(y_id_chirho)}")
    print(f"  find([1,2]) = {eg1_chirho.find_chirho(list_id_chirho)}")
    print(f"  canonical(x) = {eg1_chirho.get_canonical_chirho(x_id_chirho)}")

    extracted_chirho = extract_list_chirho(x_id_chirho, eg1_chirho)
    print(f"  extract_list(x) = {extracted_chirho}")

    # === Test 2: Conflicting unification ===
    print("\n" + "="*60)
    print("TEST 2: Conflicting unification")
    print("="*60)

    eg2_chirho = EGraphChirho()
    z_chirho, z_id_chirho = eg2_chirho.fresh_var_chirho()
    list1_id_chirho = build_list_chirho([1], eg2_chirho)
    list2_id_chirho = build_list_chirho([2], eg2_chirho)

    # Unify z = [1]
    eg2_chirho = unify_egraph_chirho(z_id_chirho, list1_id_chirho, eg2_chirho)
    print(f"\nAfter z = [1]:")
    print(f"  extract_list(z) = {extract_list_chirho(z_id_chirho, eg2_chirho)}")

    # Try to unify z = [2] - should fail!
    result_chirho = unify_egraph_chirho(z_id_chirho, list2_id_chirho, eg2_chirho.copy_chirho())
    print(f"\nTrying z = [2]: {'FAIL' if result_chirho is None else 'SUCCESS'}")

    # === Test 3: Structural unification ===
    print("\n" + "="*60)
    print("TEST 3: Structural unification (cons cells)")
    print("="*60)

    eg3_chirho = EGraphChirho()

    # Create [x | y] where x, y are fresh
    x3_chirho, x3_id_chirho = eg3_chirho.fresh_var_chirho()
    y3_chirho, y3_id_chirho = eg3_chirho.fresh_var_chirho()
    cons1_chirho = ConsChirho(x3_id_chirho, y3_id_chirho)
    cons1_id_chirho = eg3_chirho.add_chirho(cons1_chirho)

    # Create [1, 2]
    list3_id_chirho = build_list_chirho([1, 2], eg3_chirho)

    print(f"\nBefore: [x | y] vs [1, 2]")
    print(f"  x = {x3_chirho}, y = {y3_chirho}")

    # Unify [x | y] = [1, 2]
    eg3_chirho = unify_egraph_chirho(cons1_id_chirho, list3_id_chirho, eg3_chirho)

    if eg3_chirho:
        print(f"\nAfter unification:")
        print(f"  x → {eg3_chirho.get_canonical_chirho(x3_id_chirho)}")
        print(f"  y → {extract_list_chirho(y3_id_chirho, eg3_chirho)}")
    else:
        print("  FAILED")

    # === Performance comparison ===
    print("\n" + "="*60)
    print("PERFORMANCE: Chain vs E-Graph")
    print("="*60)

    # Build a long chain: x0 → x1 → x2 → ... → xn → [1]
    import time

    for n_chirho in [10, 100, 1000]:
        # E-graph approach
        eg_chirho = EGraphChirho()
        vars_chirho = []
        ids_chirho = []
        for i_chirho in range(n_chirho):
            v_chirho, id_chirho = eg_chirho.fresh_var_chirho()
            vars_chirho.append(v_chirho)
            ids_chirho.append(id_chirho)

        val_id_chirho = eg_chirho.add_chirho(IntChirho(42))

        # Chain them: x0=x1, x1=x2, ..., x(n-1)=42
        start_chirho = time.perf_counter()
        for i_chirho in range(n_chirho - 1):
            eg_chirho = unify_egraph_chirho(ids_chirho[i_chirho], ids_chirho[i_chirho + 1], eg_chirho)
        eg_chirho = unify_egraph_chirho(ids_chirho[-1], val_id_chirho, eg_chirho)

        # Now "walk" x0
        result_chirho = eg_chirho.get_canonical_chirho(ids_chirho[0])
        elapsed_chirho = time.perf_counter() - start_chirho

        print(f"\n  Chain length {n_chirho}:")
        print(f"    Build + unify: {elapsed_chirho*1000:.3f}ms")
        print(f"    find(x0) = e{eg_chirho.find_chirho(ids_chirho[0])}")
        print(f"    canonical(x0) = {result_chirho}")

    # === The insight ===
    print("\n" + "="*60)
    print("THE INSIGHT")
    print("="*60)
    print("""
    E-graphs give us:

    1. O(α(n)) ≈ O(1) unification lookups (vs O(chain length))
    2. Path compression automatically
    3. Natural representation of equivalence classes
    4. Well-studied for hardware (union-find in silicon)

    Connection to egg:

    - egg uses e-graphs for equality saturation
    - We use e-graphs for unification
    - Both: "these things are equivalent"

    What's different from traditional substitution:

    - Traditional: var → term (directed binding)
    - E-graph: var ≡ term (bidirectional equivalence)

    Open questions:

    1. Occurs check: Need to detect if var is reachable from term
       → Same as traditional, but e-graph structure may help

    2. Branching: Each search branch needs separate e-graph
       → Copy is O(n) but could use persistent data structures

    3. Extraction: Getting ground term from e-class
       → May need congruence closure for full reconstruction

    Is this a real improvement? MAYBE.

    - For long chains: definitely faster
    - For typical short chains: overhead may not be worth it
    - For hardware: union-find is very well understood

    The real value: CONCEPTUAL UNIFICATION

    miniKanren + egg + union-find = same underlying structure
    """)


if __name__ == "__main__":
    main()
