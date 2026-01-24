#!/usr/bin/env python3
"""
Variable Propagation with Union-Find + Bit Matrices ☧

The problem: When variable X binds to value V, all occurrences of X
must see the new value. Traditional substitution chains have O(n) walk.

Solution: Union-Find + Bit Matrix integration

1. Union-Find for equivalence classes (O(α(n)) lookup)
2. Bit matrices encode which variables appear in which terms
3. Binding propagation = sparse matrix update + union

This solves the "Variable propagation" open problem from AGENTS.md.

Key insight:
  - Variables form equivalence classes (union-find)
  - Terms are indices into term store (hash consing)
  - Binding X=V means: union(class(X), class(V))
  - Walk = find representative + get canonical term

Hardware mapping:
  - Union-find → CAM (content addressable memory) for parallel find
  - Bit matrix → sparse storage of var→term relationships
  - Propagation → parallel broadcast on equivalence class change

References:
- "Persistent Union-Find" (Conchon & Filliâtre)
- "Near-optimal Union-Find" (Tarjan)
"""

from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, Set, Iterator, Any, FrozenSet
from collections import defaultdict


# === Union-Find with Path Compression and Union by Rank ===

class UnionFindChirho:
    """
    Classic union-find with path compression.

    O(α(n)) amortized per operation where α is inverse Ackermann.
    For practical purposes, O(1) per operation.
    """

    def __init__(self_chirho):
        self_chirho.parent_chirho: Dict[int, int] = {}
        self_chirho.rank_chirho: Dict[int, int] = {}

    def make_set_chirho(self_chirho, x_chirho: int):
        """Create singleton set containing x"""
        if x_chirho not in self_chirho.parent_chirho:
            self_chirho.parent_chirho[x_chirho] = x_chirho
            self_chirho.rank_chirho[x_chirho] = 0

    def find_chirho(self_chirho, x_chirho: int) -> int:
        """Find representative with path compression"""
        if x_chirho not in self_chirho.parent_chirho:
            self_chirho.make_set_chirho(x_chirho)
            return x_chirho

        if self_chirho.parent_chirho[x_chirho] != x_chirho:
            # Path compression: point directly to root
            self_chirho.parent_chirho[x_chirho] = self_chirho.find_chirho(self_chirho.parent_chirho[x_chirho])

        return self_chirho.parent_chirho[x_chirho]

    def union_chirho(self_chirho, x_chirho: int, y_chirho: int) -> int:
        """Union by rank, returns new root"""
        root_x_chirho = self_chirho.find_chirho(x_chirho)
        root_y_chirho = self_chirho.find_chirho(y_chirho)

        if root_x_chirho == root_y_chirho:
            return root_x_chirho

        # Union by rank
        if self_chirho.rank_chirho[root_x_chirho] < self_chirho.rank_chirho[root_y_chirho]:
            root_x_chirho, root_y_chirho = root_y_chirho, root_x_chirho

        self_chirho.parent_chirho[root_y_chirho] = root_x_chirho

        if self_chirho.rank_chirho[root_x_chirho] == self_chirho.rank_chirho[root_y_chirho]:
            self_chirho.rank_chirho[root_x_chirho] += 1

        return root_x_chirho

    def same_class_chirho(self_chirho, x_chirho: int, y_chirho: int) -> bool:
        """Check if x and y are in same equivalence class"""
        return self_chirho.find_chirho(x_chirho) == self_chirho.find_chirho(y_chirho)

    def copy_chirho(self_chirho) -> 'UnionFindChirho':
        """Deep copy for branching search"""
        new_chirho = UnionFindChirho()
        new_chirho.parent_chirho = self_chirho.parent_chirho.copy()
        new_chirho.rank_chirho = self_chirho.rank_chirho.copy()
        return new_chirho

    def all_classes_chirho(self_chirho) -> Dict[int, Set[int]]:
        """Get all equivalence classes"""
        classes_chirho: Dict[int, Set[int]] = defaultdict(set)
        for x_chirho in self_chirho.parent_chirho:
            root_chirho = self_chirho.find_chirho(x_chirho)
            classes_chirho[root_chirho].add(x_chirho)
        return dict(classes_chirho)


# === Terms with E-Class IDs ===

@dataclass(frozen=True)
class VarChirho:
    """Variable - identified by its e-class ID"""
    id_chirho: int
    def __repr__(self_chirho): return f"_{self_chirho.id_chirho}"

@dataclass(frozen=True)
class NilChirho:
    def __repr__(self_chirho): return "[]"

@dataclass(frozen=True)
class IntChirho:
    val_chirho: int
    def __repr__(self_chirho): return str(self_chirho.val_chirho)

@dataclass(frozen=True)
class ConsChirho:
    """Cons stores e-class IDs, not raw terms"""
    head_class_chirho: int  # E-class ID of head
    tail_class_chirho: int  # E-class ID of tail

    def __repr__(self_chirho):
        return f"Cons({self_chirho.head_class_chirho}, {self_chirho.tail_class_chirho})"


TermChirho = VarChirho | NilChirho | IntChirho | ConsChirho


# === Term Store with E-Classes ===

class TermStoreChirho:
    """
    Hash-consed term store integrated with union-find.

    Each term gets an e-class ID. Variables are their own e-classes.
    Binding X=V means union(class(X), class(V)).

    This is the core data structure for variable propagation.
    """

    def __init__(self_chirho):
        # Union-find for equivalence classes
        self_chirho.uf_chirho = UnionFindChirho()

        # E-class ID → canonical term for that class
        self_chirho.canonical_chirho: Dict[int, TermChirho] = {}

        # Term → e-class ID (for hash consing)
        self_chirho.term_to_class_chirho: Dict[TermChirho, int] = {}

        # Counter for fresh e-class IDs
        self_chirho.next_class_chirho = 0

        # Bit matrix: which variables (by class ID) appear in which terms (by class ID)
        # var_class → set of term_classes that contain it
        self_chirho.var_occurrences_chirho: Dict[int, Set[int]] = defaultdict(set)

    def fresh_var_chirho(self_chirho) -> Tuple[VarChirho, int]:
        """Create fresh variable with new e-class"""
        class_id_chirho = self_chirho.next_class_chirho
        self_chirho.next_class_chirho += 1

        var_chirho = VarChirho(class_id_chirho)
        self_chirho.uf_chirho.make_set_chirho(class_id_chirho)
        self_chirho.canonical_chirho[class_id_chirho] = var_chirho
        self_chirho.term_to_class_chirho[var_chirho] = class_id_chirho

        return var_chirho, class_id_chirho

    def add_int_chirho(self_chirho, val_chirho: int) -> int:
        """Add integer constant, return e-class ID"""
        term_chirho = IntChirho(val_chirho)

        if term_chirho in self_chirho.term_to_class_chirho:
            return self_chirho.uf_chirho.find_chirho(self_chirho.term_to_class_chirho[term_chirho])

        class_id_chirho = self_chirho.next_class_chirho
        self_chirho.next_class_chirho += 1

        self_chirho.uf_chirho.make_set_chirho(class_id_chirho)
        self_chirho.canonical_chirho[class_id_chirho] = term_chirho
        self_chirho.term_to_class_chirho[term_chirho] = class_id_chirho

        return class_id_chirho

    def add_nil_chirho(self_chirho) -> int:
        """Add nil constant, return e-class ID"""
        term_chirho = NilChirho()

        if term_chirho in self_chirho.term_to_class_chirho:
            return self_chirho.uf_chirho.find_chirho(self_chirho.term_to_class_chirho[term_chirho])

        class_id_chirho = self_chirho.next_class_chirho
        self_chirho.next_class_chirho += 1

        self_chirho.uf_chirho.make_set_chirho(class_id_chirho)
        self_chirho.canonical_chirho[class_id_chirho] = term_chirho
        self_chirho.term_to_class_chirho[term_chirho] = class_id_chirho

        return class_id_chirho

    def add_cons_chirho(self_chirho, head_class_chirho: int, tail_class_chirho: int) -> int:
        """Add cons cell, return e-class ID"""
        # Normalize to canonical class IDs
        head_root_chirho = self_chirho.uf_chirho.find_chirho(head_class_chirho)
        tail_root_chirho = self_chirho.uf_chirho.find_chirho(tail_class_chirho)

        term_chirho = ConsChirho(head_root_chirho, tail_root_chirho)

        if term_chirho in self_chirho.term_to_class_chirho:
            return self_chirho.uf_chirho.find_chirho(self_chirho.term_to_class_chirho[term_chirho])

        class_id_chirho = self_chirho.next_class_chirho
        self_chirho.next_class_chirho += 1

        self_chirho.uf_chirho.make_set_chirho(class_id_chirho)
        self_chirho.canonical_chirho[class_id_chirho] = term_chirho
        self_chirho.term_to_class_chirho[term_chirho] = class_id_chirho

        # Track variable occurrences for propagation
        self_chirho._track_vars_chirho(class_id_chirho, term_chirho)

        return class_id_chirho

    def _track_vars_chirho(self_chirho, term_class_chirho: int, term_chirho: TermChirho):
        """Track which variables appear in this term"""
        if isinstance(term_chirho, VarChirho):
            var_class_chirho = self_chirho.uf_chirho.find_chirho(term_chirho.id_chirho)
            self_chirho.var_occurrences_chirho[var_class_chirho].add(term_class_chirho)
        elif isinstance(term_chirho, ConsChirho):
            # Track head and tail
            head_term_chirho = self_chirho.get_canonical_chirho(term_chirho.head_class_chirho)
            tail_term_chirho = self_chirho.get_canonical_chirho(term_chirho.tail_class_chirho)

            if head_term_chirho:
                self_chirho._track_vars_chirho(term_class_chirho, head_term_chirho)
            if tail_term_chirho:
                self_chirho._track_vars_chirho(term_class_chirho, tail_term_chirho)

    def build_list_chirho(self_chirho, elems_chirho: List[int]) -> int:
        """Build a list from integer elements"""
        result_chirho = self_chirho.add_nil_chirho()
        for elem_chirho in reversed(elems_chirho):
            elem_class_chirho = self_chirho.add_int_chirho(elem_chirho)
            result_chirho = self_chirho.add_cons_chirho(elem_class_chirho, result_chirho)
        return result_chirho

    def get_canonical_chirho(self_chirho, class_id_chirho: int) -> Optional[TermChirho]:
        """Get canonical term for an e-class"""
        root_chirho = self_chirho.uf_chirho.find_chirho(class_id_chirho)

        if root_chirho not in self_chirho.canonical_chirho:
            return None

        return self_chirho.canonical_chirho[root_chirho]

    def unify_classes_chirho(self_chirho, class1_chirho: int, class2_chirho: int) -> bool:
        """
        Unify two e-classes. Returns True on success, False on conflict.

        This is where variable propagation happens:
        - If both are vars: just union
        - If one is ground: var's class gets ground term
        - If both are ground cons: recursively unify children
        """
        root1_chirho = self_chirho.uf_chirho.find_chirho(class1_chirho)
        root2_chirho = self_chirho.uf_chirho.find_chirho(class2_chirho)

        if root1_chirho == root2_chirho:
            return True  # Already unified

        term1_chirho = self_chirho.canonical_chirho.get(root1_chirho)
        term2_chirho = self_chirho.canonical_chirho.get(root2_chirho)

        # Case 1: At least one is a variable
        if isinstance(term1_chirho, VarChirho) or isinstance(term2_chirho, VarChirho):
            # Occurs check: is var in the other term?
            if isinstance(term1_chirho, VarChirho):
                if self_chirho._occurs_in_chirho(root1_chirho, root2_chirho):
                    return False  # Occurs check failure
            if isinstance(term2_chirho, VarChirho):
                if self_chirho._occurs_in_chirho(root2_chirho, root1_chirho):
                    return False

            # Union the classes
            new_root_chirho = self_chirho.uf_chirho.union_chirho(root1_chirho, root2_chirho)

            # Pick non-variable as canonical if possible
            if isinstance(term1_chirho, VarChirho) and not isinstance(term2_chirho, VarChirho):
                self_chirho.canonical_chirho[new_root_chirho] = term2_chirho
            elif isinstance(term2_chirho, VarChirho) and not isinstance(term1_chirho, VarChirho):
                self_chirho.canonical_chirho[new_root_chirho] = term1_chirho

            # Merge var occurrence tracking
            self_chirho._merge_occurrences_chirho(root1_chirho, root2_chirho, new_root_chirho)

            return True

        # Case 2: Both nil
        if isinstance(term1_chirho, NilChirho) and isinstance(term2_chirho, NilChirho):
            self_chirho.uf_chirho.union_chirho(root1_chirho, root2_chirho)
            return True

        # Case 3: Both ints
        if isinstance(term1_chirho, IntChirho) and isinstance(term2_chirho, IntChirho):
            if term1_chirho.val_chirho == term2_chirho.val_chirho:
                self_chirho.uf_chirho.union_chirho(root1_chirho, root2_chirho)
                return True
            return False  # Conflict!

        # Case 4: Both cons
        if isinstance(term1_chirho, ConsChirho) and isinstance(term2_chirho, ConsChirho):
            # Recursively unify head and tail
            if not self_chirho.unify_classes_chirho(term1_chirho.head_class_chirho, term2_chirho.head_class_chirho):
                return False
            if not self_chirho.unify_classes_chirho(term1_chirho.tail_class_chirho, term2_chirho.tail_class_chirho):
                return False

            self_chirho.uf_chirho.union_chirho(root1_chirho, root2_chirho)
            return True

        # Case 5: Type mismatch
        return False

    def _occurs_in_chirho(self_chirho, var_class_chirho: int, term_class_chirho: int, top_level_chirho: bool = True) -> bool:
        """
        Check if variable class appears in term (for occurs check).

        top_level_chirho: True for initial call, False for recursive subterm checks.
        At top level, var == term is OK (x = x). In subterms, finding var means cyclic.
        """
        var_root_chirho = self_chirho.uf_chirho.find_chirho(var_class_chirho)
        term_root_chirho = self_chirho.uf_chirho.find_chirho(term_class_chirho)

        # If we find the variable in a subterm, that's a cycle
        if var_root_chirho == term_root_chirho:
            # At top level: x = x is fine (not cyclic)
            # In subterm: x = f(...x...) is cyclic
            return not top_level_chirho

        term_chirho = self_chirho.canonical_chirho.get(term_root_chirho)

        if term_chirho is None:
            return False
        if isinstance(term_chirho, VarChirho):
            # Check if this var is the one we're looking for
            return self_chirho.uf_chirho.find_chirho(term_chirho.id_chirho) == var_root_chirho
        if isinstance(term_chirho, (NilChirho, IntChirho)):
            return False
        if isinstance(term_chirho, ConsChirho):
            # Recursive check in subterms (not top level anymore)
            return (self_chirho._occurs_in_chirho(var_class_chirho, term_chirho.head_class_chirho, False) or
                    self_chirho._occurs_in_chirho(var_class_chirho, term_chirho.tail_class_chirho, False))

        return False

    def _merge_occurrences_chirho(self_chirho, old1_chirho: int, old2_chirho: int, new_root_chirho: int):
        """Merge variable occurrence tracking after union"""
        # Combine occurrence sets
        occ1_chirho = self_chirho.var_occurrences_chirho.pop(old1_chirho, set())
        occ2_chirho = self_chirho.var_occurrences_chirho.pop(old2_chirho, set())
        if occ1_chirho or occ2_chirho:
            self_chirho.var_occurrences_chirho[new_root_chirho] = occ1_chirho | occ2_chirho

    def extract_list_chirho(self_chirho, class_id_chirho: int) -> Optional[List[int]]:
        """Extract ground list from e-class"""
        result_chirho = []
        current_chirho = class_id_chirho

        while True:
            term_chirho = self_chirho.get_canonical_chirho(current_chirho)

            if isinstance(term_chirho, NilChirho):
                return result_chirho
            if isinstance(term_chirho, ConsChirho):
                head_term_chirho = self_chirho.get_canonical_chirho(term_chirho.head_class_chirho)
                if isinstance(head_term_chirho, IntChirho):
                    result_chirho.append(head_term_chirho.val_chirho)
                    current_chirho = term_chirho.tail_class_chirho
                else:
                    return None  # Not ground
            else:
                return None  # Not a list

    def copy_chirho(self_chirho) -> 'TermStoreChirho':
        """Deep copy for branching search"""
        new_chirho = TermStoreChirho()
        new_chirho.uf_chirho = self_chirho.uf_chirho.copy_chirho()
        new_chirho.canonical_chirho = self_chirho.canonical_chirho.copy()
        new_chirho.term_to_class_chirho = self_chirho.term_to_class_chirho.copy()
        new_chirho.next_class_chirho = self_chirho.next_class_chirho
        new_chirho.var_occurrences_chirho = defaultdict(set, {
            k_chirho: v_chirho.copy() for k_chirho, v_chirho in self_chirho.var_occurrences_chirho.items()
        })
        return new_chirho

    def display_chirho(self_chirho):
        """Display e-graph state"""
        classes_chirho = self_chirho.uf_chirho.all_classes_chirho()
        print(f"Term Store: {len(classes_chirho)} equivalence classes")
        for root_chirho, members_chirho in sorted(classes_chirho.items()):
            canonical_chirho = self_chirho.canonical_chirho.get(root_chirho, "?")
            if len(members_chirho) > 1:
                print(f"  e{root_chirho}: {sorted(members_chirho)} → {canonical_chirho}")
            else:
                print(f"  e{root_chirho}: {canonical_chirho}")


# === Bit Matrix View ===

class BitMatrixViewChirho:
    """
    Bit matrix showing variable→term relationships.

    Rows = variable e-classes
    Columns = term e-classes
    Entry (i,j) = 1 iff variable i appears in term j

    This enables parallel propagation: when var X binds to V,
    all terms containing X need updating.
    """

    def __init__(self_chirho, store_chirho: TermStoreChirho):
        self_chirho.store_chirho = store_chirho

    def get_matrix_chirho(self_chirho) -> Dict[int, Set[int]]:
        """Get current bit matrix as sparse dict"""
        return dict(self_chirho.store_chirho.var_occurrences_chirho)

    def terms_containing_var_chirho(self_chirho, var_class_chirho: int) -> Set[int]:
        """Get all terms containing this variable (O(1))"""
        root_chirho = self_chirho.store_chirho.uf_chirho.find_chirho(var_class_chirho)
        return self_chirho.store_chirho.var_occurrences_chirho.get(root_chirho, set())

    def display_chirho(self_chirho):
        """Display bit matrix"""
        matrix_chirho = self_chirho.get_matrix_chirho()
        if not matrix_chirho:
            print("  (empty)")
            return

        for var_class_chirho, term_classes_chirho in sorted(matrix_chirho.items()):
            canonical_chirho = self_chirho.store_chirho.canonical_chirho.get(var_class_chirho, "?")
            if isinstance(canonical_chirho, VarChirho):
                print(f"  var_{var_class_chirho} appears in: {sorted(term_classes_chirho)}")


# === Demo ===

def main():
    print("=== Variable Propagation with Union-Find + Bit Matrices ☧ ===\n")

    # === Test 1: Basic propagation ===
    print("="*60)
    print("TEST 1: Basic variable propagation")
    print("="*60)

    store1_chirho = TermStoreChirho()

    # Create variables x, y, z
    x_chirho, x_class_chirho = store1_chirho.fresh_var_chirho()
    y_chirho, y_class_chirho = store1_chirho.fresh_var_chirho()
    z_chirho, z_class_chirho = store1_chirho.fresh_var_chirho()

    # Create list [1, 2]
    list_class_chirho = store1_chirho.build_list_chirho([1, 2])

    print(f"\nInitial state:")
    print(f"  x = {x_chirho} (class {x_class_chirho})")
    print(f"  y = {y_chirho} (class {y_class_chirho})")
    print(f"  z = {z_chirho} (class {z_class_chirho})")
    print(f"  [1,2] (class {list_class_chirho})")
    store1_chirho.display_chirho()

    # Unify x = y
    print(f"\n--- Unify x = y ---")
    ok_chirho = store1_chirho.unify_classes_chirho(x_class_chirho, y_class_chirho)
    print(f"  Result: {'OK' if ok_chirho else 'FAIL'}")
    print(f"  find(x) = e{store1_chirho.uf_chirho.find_chirho(x_class_chirho)}")
    print(f"  find(y) = e{store1_chirho.uf_chirho.find_chirho(y_class_chirho)}")

    # Unify y = z
    print(f"\n--- Unify y = z ---")
    ok_chirho = store1_chirho.unify_classes_chirho(y_class_chirho, z_class_chirho)
    print(f"  Result: {'OK' if ok_chirho else 'FAIL'}")
    print(f"  find(x) = e{store1_chirho.uf_chirho.find_chirho(x_class_chirho)}")
    print(f"  find(z) = e{store1_chirho.uf_chirho.find_chirho(z_class_chirho)}")
    print(f"  x, y, z same class: {store1_chirho.uf_chirho.same_class_chirho(x_class_chirho, z_class_chirho)}")

    # Unify z = [1, 2]
    print(f"\n--- Unify z = [1, 2] ---")
    ok_chirho = store1_chirho.unify_classes_chirho(z_class_chirho, list_class_chirho)
    print(f"  Result: {'OK' if ok_chirho else 'FAIL'}")

    print(f"\nFinal state:")
    print(f"  canonical(x) = {store1_chirho.get_canonical_chirho(x_class_chirho)}")
    print(f"  canonical(y) = {store1_chirho.get_canonical_chirho(y_class_chirho)}")
    print(f"  canonical(z) = {store1_chirho.get_canonical_chirho(z_class_chirho)}")
    print(f"  extract_list(x) = {store1_chirho.extract_list_chirho(x_class_chirho)}")

    # === Test 2: Conflict detection ===
    print("\n" + "="*60)
    print("TEST 2: Conflict detection")
    print("="*60)

    store2_chirho = TermStoreChirho()
    a_chirho, a_class_chirho = store2_chirho.fresh_var_chirho()
    list1_chirho = store2_chirho.build_list_chirho([1])
    list2_chirho = store2_chirho.build_list_chirho([2])

    # Bind a = [1]
    ok1_chirho = store2_chirho.unify_classes_chirho(a_class_chirho, list1_chirho)
    print(f"\na = [1]: {'OK' if ok1_chirho else 'FAIL'}")

    # Try a = [2] (should fail!)
    store2_copy_chirho = store2_chirho.copy_chirho()
    ok2_chirho = store2_copy_chirho.unify_classes_chirho(a_class_chirho, list2_chirho)
    print(f"a = [2]: {'OK' if ok2_chirho else 'FAIL (expected)'}")

    # === Test 3: Structural unification ===
    print("\n" + "="*60)
    print("TEST 3: Structural unification [x|y] = [1,2]")
    print("="*60)

    store3_chirho = TermStoreChirho()

    # Create [x | y] (cons with fresh vars)
    x3_chirho, x3_class_chirho = store3_chirho.fresh_var_chirho()
    y3_chirho, y3_class_chirho = store3_chirho.fresh_var_chirho()
    cons_class_chirho = store3_chirho.add_cons_chirho(x3_class_chirho, y3_class_chirho)

    # Create [1, 2]
    list3_chirho = store3_chirho.build_list_chirho([1, 2])

    print(f"\nBefore:")
    print(f"  x = {x3_chirho}, y = {y3_chirho}")
    print(f"  [x|y] class = {cons_class_chirho}")
    print(f"  [1,2] class = {list3_chirho}")

    # Unify
    ok3_chirho = store3_chirho.unify_classes_chirho(cons_class_chirho, list3_chirho)
    print(f"\nUnify [x|y] = [1,2]: {'OK' if ok3_chirho else 'FAIL'}")

    print(f"\nAfter:")
    print(f"  canonical(x) = {store3_chirho.get_canonical_chirho(x3_class_chirho)}")
    print(f"  canonical(y) = {store3_chirho.get_canonical_chirho(y3_class_chirho)}")
    print(f"  extract_list(y) = {store3_chirho.extract_list_chirho(y3_class_chirho)}")

    # === Test 4: Occurs check ===
    print("\n" + "="*60)
    print("TEST 4: Occurs check (x = [x|y] should fail)")
    print("="*60)

    store4_chirho = TermStoreChirho()
    x4_chirho, x4_class_chirho = store4_chirho.fresh_var_chirho()
    y4_chirho, y4_class_chirho = store4_chirho.fresh_var_chirho()

    # Create [x | y]
    cons4_chirho = store4_chirho.add_cons_chirho(x4_class_chirho, y4_class_chirho)

    # Try x = [x|y] (should fail - x appears in RHS)
    ok4_chirho = store4_chirho.unify_classes_chirho(x4_class_chirho, cons4_chirho)
    print(f"\nx = [x|y]: {'OK' if ok4_chirho else 'FAIL (expected - occurs check)'}")

    # === Test 5: Bit matrix view ===
    print("\n" + "="*60)
    print("TEST 5: Bit matrix for variable occurrences")
    print("="*60)

    store5_chirho = TermStoreChirho()

    # Create structure with multiple variable occurrences
    v1_chirho, v1_class_chirho = store5_chirho.fresh_var_chirho()
    v2_chirho, v2_class_chirho = store5_chirho.fresh_var_chirho()
    v3_chirho, v3_class_chirho = store5_chirho.fresh_var_chirho()

    # [v1, v2]
    list_v1v2_chirho = store5_chirho.add_cons_chirho(
        v1_class_chirho,
        store5_chirho.add_cons_chirho(v2_class_chirho, store5_chirho.add_nil_chirho())
    )

    # [v2, v3]
    list_v2v3_chirho = store5_chirho.add_cons_chirho(
        v2_class_chirho,
        store5_chirho.add_cons_chirho(v3_class_chirho, store5_chirho.add_nil_chirho())
    )

    print(f"\nStructures:")
    print(f"  [v1, v2] = class {list_v1v2_chirho}")
    print(f"  [v2, v3] = class {list_v2v3_chirho}")

    bm_chirho = BitMatrixViewChirho(store5_chirho)
    print(f"\nBit matrix (var → terms containing it):")
    bm_chirho.display_chirho()

    # Bind v2 = 42
    v2_bound_chirho = store5_chirho.add_int_chirho(42)
    store5_chirho.unify_classes_chirho(v2_class_chirho, v2_bound_chirho)

    print(f"\nAfter v2 = 42:")
    print(f"  Terms affected by binding v2: {bm_chirho.terms_containing_var_chirho(v2_class_chirho)}")

    # === Performance comparison ===
    print("\n" + "="*60)
    print("PERFORMANCE: Chain walk vs Union-Find")
    print("="*60)

    import time

    for n_chirho in [10, 100, 1000]:
        store_perf_chirho = TermStoreChirho()
        vars_chirho = []
        classes_chirho = []

        for _ in range(n_chirho):
            v_chirho, c_chirho = store_perf_chirho.fresh_var_chirho()
            vars_chirho.append(v_chirho)
            classes_chirho.append(c_chirho)

        val_class_chirho = store_perf_chirho.add_int_chirho(42)

        # Chain them
        start_chirho = time.perf_counter()
        for i_chirho in range(n_chirho - 1):
            store_perf_chirho.unify_classes_chirho(classes_chirho[i_chirho], classes_chirho[i_chirho + 1])
        store_perf_chirho.unify_classes_chirho(classes_chirho[-1], val_class_chirho)

        # "Walk" from first var
        result_chirho = store_perf_chirho.get_canonical_chirho(classes_chirho[0])
        elapsed_chirho = time.perf_counter() - start_chirho

        print(f"\n  Chain length {n_chirho}:")
        print(f"    Build + unify: {elapsed_chirho*1000:.3f}ms")
        print(f"    canonical(v0) = {result_chirho}")

    # === Key Insights ===
    print("\n" + "="*60)
    print("KEY INSIGHTS: Variable Propagation")
    print("="*60)
    print("""
    1. UNION-FIND FOR EQUIVALENCE CLASSES:
       - O(α(n)) ≈ O(1) amortized per operation
       - Path compression built-in
       - Binding = union of e-classes

    2. BIT MATRIX FOR OCCURRENCE TRACKING:
       - var_class → set of term_classes
       - Sparse representation (most terms have few vars)
       - Enables parallel propagation

    3. THE MAPPING:
       ┌──────────────────────────────────────────────┐
       │  Traditional:  subst = {x→y, y→z, z→42}     │
       │                walk(x) = 42  (3 lookups)    │
       │                                              │
       │  Union-Find:   find(x) = find(y) = find(z)  │
       │                canonical = 42                │
       │                (O(α(n)) ≈ 1 lookup)          │
       └──────────────────────────────────────────────┘

    4. HARDWARE MAPPING:
       - Union-Find → CAM (content addressable memory)
         - find(x) = parallel lookup of x's root
         - union(x,y) = write y's entry to point at x
       - Bit matrix → sparse memory
         - Rows indexed by var class
         - Columns indexed by term class
       - Propagation → broadcast on class change

    5. CONNECTION TO E-GRAPHS:
       - This IS an e-graph (equality graph)
       - Unification = equality saturation step
       - Canonical form = extraction from e-class
       - egg library uses same ideas for term rewriting
    """)


if __name__ == "__main__":
    main()
