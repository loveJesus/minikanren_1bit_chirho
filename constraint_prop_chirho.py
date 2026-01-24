#!/usr/bin/env python3
"""
Incremental Tensor with Constraint Propagation ☧

Combines:
1. Hash consing (terms → indices)
2. Sparse relation storage (triples)
3. Arc consistency propagation
4. Branching on remaining choices

This is closer to how a real constraint solver works.
"""
from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional, Set, Union, Callable
from collections import defaultdict
import numpy as np

# === Terms (same as before) ===

@dataclass(frozen=True)
class NilChirho:
    def __repr__(self_chirho): return "[]"

@dataclass(frozen=True)
class ConsChirho:
    head_chirho: int
    tail_chirho: Union['NilChirho', 'ConsChirho']
    
    def __repr__(self_chirho):
        elems_chirho = []
        curr_chirho = self_chirho
        while isinstance(curr_chirho, ConsChirho):
            elems_chirho.append(str(curr_chirho.head_chirho))
            curr_chirho = curr_chirho.tail_chirho
        return "[" + ",".join(elems_chirho) + "]"

TermChirho = Union[NilChirho, ConsChirho]

def list_to_term_chirho(lst_chirho: list) -> TermChirho:
    result_chirho = NilChirho()
    for x_chirho in reversed(lst_chirho):
        result_chirho = ConsChirho(x_chirho, result_chirho)
    return result_chirho

def term_to_list_chirho(t_chirho: TermChirho) -> list:
    result_chirho = []
    while isinstance(t_chirho, ConsChirho):
        result_chirho.append(t_chirho.head_chirho)
        t_chirho = t_chirho.tail_chirho
    return result_chirho


class TermStoreChirho:
    def __init__(self_chirho):
        self_chirho.term_to_idx_chirho: Dict[TermChirho, int] = {}
        self_chirho.idx_to_term_chirho: List[TermChirho] = []
    
    def intern_chirho(self_chirho, term_chirho: TermChirho) -> int:
        if term_chirho in self_chirho.term_to_idx_chirho:
            return self_chirho.term_to_idx_chirho[term_chirho]
        idx_chirho = len(self_chirho.idx_to_term_chirho)
        self_chirho.term_to_idx_chirho[term_chirho] = idx_chirho
        self_chirho.idx_to_term_chirho.append(term_chirho)
        return idx_chirho
    
    def lookup_chirho(self_chirho, idx_chirho: int) -> TermChirho:
        return self_chirho.idx_to_term_chirho[idx_chirho]
    
    def size_chirho(self_chirho) -> int:
        return len(self_chirho.idx_to_term_chirho)
    
    def intern_list_chirho(self_chirho, lst_chirho: list) -> int:
        return self_chirho.intern_chirho(list_to_term_chirho(lst_chirho))


# === Sparse Relation ===

class SparseRelationChirho:
    """
    A relation stored as sparse triples with indices for fast lookup.
    """
    def __init__(self_chirho, arity_chirho: int, name_chirho: str = ""):
        self_chirho.arity_chirho = arity_chirho
        self_chirho.name_chirho = name_chirho
        self_chirho.tuples_chirho: Set[Tuple[int, ...]] = set()
        # Index: for each position, map value → set of tuples
        self_chirho.indices_chirho: List[Dict[int, Set[Tuple[int, ...]]]] = [
            defaultdict(set) for _ in range(arity_chirho)
        ]
    
    def add_chirho(self_chirho, tup_chirho: Tuple[int, ...]):
        if tup_chirho in self_chirho.tuples_chirho:
            return
        self_chirho.tuples_chirho.add(tup_chirho)
        for i_chirho, val_chirho in enumerate(tup_chirho):
            self_chirho.indices_chirho[i_chirho][val_chirho].add(tup_chirho)
    
    def lookup_chirho(self_chirho, pos_chirho: int, val_chirho: int) -> Set[Tuple[int, ...]]:
        """Get all tuples where position pos has value val"""
        return self_chirho.indices_chirho[pos_chirho].get(val_chirho, set())
    
    def filter_by_domain_chirho(
        self_chirho, 
        domains_chirho: List[Optional[Set[int]]]
    ) -> Set[Tuple[int, ...]]:
        """Get tuples consistent with given domains (None = any)"""
        result_chirho = self_chirho.tuples_chirho
        for i_chirho, dom_chirho in enumerate(domains_chirho):
            if dom_chirho is not None:
                result_chirho = {
                    t_chirho for t_chirho in result_chirho 
                    if t_chirho[i_chirho] in dom_chirho
                }
        return result_chirho


# === Appendo Relation Builder ===

class AppendoBuilderChirho:
    """Lazily builds appendo relation on demand"""
    
    def __init__(self_chirho, store_chirho: TermStoreChirho, relation_chirho: SparseRelationChirho):
        self_chirho.store_chirho = store_chirho
        self_chirho.relation_chirho = relation_chirho
        self_chirho.computed_outputs_chirho: Set[int] = set()  # out values we've split
    
    def ensure_splits_chirho(self_chirho, out_idx_chirho: int):
        """Ensure we've computed all (l, s) that produce this output"""
        if out_idx_chirho in self_chirho.computed_outputs_chirho:
            return
        
        out_term_chirho = self_chirho.store_chirho.lookup_chirho(out_idx_chirho)
        out_list_chirho = term_to_list_chirho(out_term_chirho)
        
        # Generate all splits
        for i_chirho in range(len(out_list_chirho) + 1):
            l_list_chirho = out_list_chirho[:i_chirho]
            s_list_chirho = out_list_chirho[i_chirho:]
            
            l_idx_chirho = self_chirho.store_chirho.intern_list_chirho(l_list_chirho)
            s_idx_chirho = self_chirho.store_chirho.intern_list_chirho(s_list_chirho)
            
            self_chirho.relation_chirho.add_chirho((l_idx_chirho, s_idx_chirho, out_idx_chirho))
        
        self_chirho.computed_outputs_chirho.add(out_idx_chirho)
    
    def ensure_append_chirho(self_chirho, l_idx_chirho: int, s_idx_chirho: int):
        """Ensure we've computed append(l, s)"""
        l_list_chirho = term_to_list_chirho(self_chirho.store_chirho.lookup_chirho(l_idx_chirho))
        s_list_chirho = term_to_list_chirho(self_chirho.store_chirho.lookup_chirho(s_idx_chirho))
        out_list_chirho = l_list_chirho + s_list_chirho
        out_idx_chirho = self_chirho.store_chirho.intern_list_chirho(out_list_chirho)
        self_chirho.relation_chirho.add_chirho((l_idx_chirho, s_idx_chirho, out_idx_chirho))


# === Constraint Store ===

class ConstraintStoreChirho:
    """
    Manages variables and their domains during search.
    Supports arc consistency propagation.
    """
    def __init__(self_chirho, store_chirho: TermStoreChirho):
        self_chirho.store_chirho = store_chirho
        self_chirho.domains_chirho: Dict[str, Set[int]] = {}
        self_chirho.constraints_chirho: List[Tuple[str, ...]] = []  # (rel_name, var1, var2, ...)
        self_chirho.relations_chirho: Dict[str, SparseRelationChirho] = {}
        self_chirho.builders_chirho: Dict[str, AppendoBuilderChirho] = {}
    
    def add_var_chirho(self_chirho, name_chirho: str, domain_chirho: Optional[Set[int]] = None):
        """Add a variable with optional initial domain"""
        self_chirho.domains_chirho[name_chirho] = domain_chirho if domain_chirho else None
    
    def set_domain_chirho(self_chirho, name_chirho: str, domain_chirho: Set[int]) -> bool:
        """Set/constrain domain, returns False if empty"""
        current_chirho = self_chirho.domains_chirho.get(name_chirho)
        if current_chirho is None:
            self_chirho.domains_chirho[name_chirho] = domain_chirho.copy()
        else:
            new_dom_chirho = current_chirho & domain_chirho
            if not new_dom_chirho:
                return False
            self_chirho.domains_chirho[name_chirho] = new_dom_chirho
        return True
    
    def add_relation_chirho(self_chirho, name_chirho: str, relation_chirho: SparseRelationChirho, 
                            builder_chirho: Optional[AppendoBuilderChirho] = None):
        self_chirho.relations_chirho[name_chirho] = relation_chirho
        if builder_chirho:
            self_chirho.builders_chirho[name_chirho] = builder_chirho
    
    def add_constraint_chirho(self_chirho, rel_name_chirho: str, *vars_chirho: str):
        """Add a relational constraint"""
        self_chirho.constraints_chirho.append((rel_name_chirho,) + vars_chirho)
    
    def propagate_chirho(self_chirho) -> bool:
        """
        Arc consistency propagation.
        Returns False if any domain becomes empty.
        """
        changed_chirho = True
        while changed_chirho:
            changed_chirho = False
            
            for constraint_chirho in self_chirho.constraints_chirho:
                rel_name_chirho = constraint_chirho[0]
                var_names_chirho = constraint_chirho[1:]
                
                rel_chirho = self_chirho.relations_chirho[rel_name_chirho]
                builder_chirho = self_chirho.builders_chirho.get(rel_name_chirho)
                
                # Get current domains
                doms_chirho = [self_chirho.domains_chirho.get(v_chirho) for v_chirho in var_names_chirho]
                
                # If any domain is determined, ensure relation is expanded
                if builder_chirho:
                    for i_chirho, dom_chirho in enumerate(doms_chirho):
                        if dom_chirho is not None and len(dom_chirho) == 1:
                            val_chirho = next(iter(dom_chirho))
                            if rel_name_chirho == "appendo":
                                if i_chirho == 2:  # out is determined
                                    builder_chirho.ensure_splits_chirho(val_chirho)
                                elif i_chirho == 0:  # l is determined
                                    for s_chirho in (doms_chirho[1] or set()):
                                        builder_chirho.ensure_append_chirho(val_chirho, s_chirho)
                                elif i_chirho == 1:  # s is determined
                                    for l_chirho in (doms_chirho[0] or set()):
                                        builder_chirho.ensure_append_chirho(l_chirho, val_chirho)
                
                # Get consistent tuples
                valid_tuples_chirho = rel_chirho.filter_by_domain_chirho(doms_chirho)
                
                # Update each variable's domain
                for i_chirho, var_chirho in enumerate(var_names_chirho):
                    new_vals_chirho = {t_chirho[i_chirho] for t_chirho in valid_tuples_chirho}
                    old_dom_chirho = self_chirho.domains_chirho.get(var_chirho)
                    
                    if old_dom_chirho is None:
                        if new_vals_chirho:
                            self_chirho.domains_chirho[var_chirho] = new_vals_chirho
                            changed_chirho = True
                    else:
                        new_dom_chirho = old_dom_chirho & new_vals_chirho
                        if not new_dom_chirho:
                            return False
                        if new_dom_chirho != old_dom_chirho:
                            self_chirho.domains_chirho[var_chirho] = new_dom_chirho
                            changed_chirho = True
        
        return True
    
    def is_solved_chirho(self_chirho) -> bool:
        """Check if all variables are determined"""
        return all(
            d_chirho is not None and len(d_chirho) == 1 
            for d_chirho in self_chirho.domains_chirho.values()
        )
    
    def copy_chirho(self_chirho) -> 'ConstraintStoreChirho':
        new_chirho = ConstraintStoreChirho(self_chirho.store_chirho)
        new_chirho.domains_chirho = {
            k_chirho: v_chirho.copy() if v_chirho else None 
            for k_chirho, v_chirho in self_chirho.domains_chirho.items()
        }
        new_chirho.constraints_chirho = self_chirho.constraints_chirho.copy()
        new_chirho.relations_chirho = self_chirho.relations_chirho  # Shared
        new_chirho.builders_chirho = self_chirho.builders_chirho    # Shared
        return new_chirho
    
    def get_smallest_unassigned_chirho(self_chirho) -> Optional[str]:
        """Get variable with smallest domain > 1 (for branching)"""
        best_chirho = None
        best_size_chirho = float('inf')
        for var_chirho, dom_chirho in self_chirho.domains_chirho.items():
            if dom_chirho is not None and len(dom_chirho) > 1 and len(dom_chirho) < best_size_chirho:
                best_chirho = var_chirho
                best_size_chirho = len(dom_chirho)
        return best_chirho
    
    def display_chirho(self_chirho):
        for var_chirho, dom_chirho in self_chirho.domains_chirho.items():
            if dom_chirho is None:
                print(f"  {var_chirho}: (any)")
            elif len(dom_chirho) == 1:
                idx_chirho = next(iter(dom_chirho))
                term_chirho = self_chirho.store_chirho.lookup_chirho(idx_chirho)
                print(f"  {var_chirho} = {term_chirho}")
            else:
                terms_chirho = [str(self_chirho.store_chirho.lookup_chirho(i_chirho)) 
                               for i_chirho in sorted(dom_chirho)[:5]]
                more_chirho = f"...+{len(dom_chirho)-5}" if len(dom_chirho) > 5 else ""
                print(f"  {var_chirho} ∈ {{{', '.join(terms_chirho)}{more_chirho}}}")


def solve_chirho(store_chirho: ConstraintStoreChirho) -> List[ConstraintStoreChirho]:
    """
    Solve constraints via propagation + branching.
    Returns all solutions.
    """
    if not store_chirho.propagate_chirho():
        return []  # Failed
    
    if store_chirho.is_solved_chirho():
        return [store_chirho]  # Solution!
    
    # Branch on smallest unassigned variable
    var_chirho = store_chirho.get_smallest_unassigned_chirho()
    if var_chirho is None:
        return [store_chirho]  # All assigned
    
    solutions_chirho = []
    for val_chirho in store_chirho.domains_chirho[var_chirho]:
        branch_chirho = store_chirho.copy_chirho()
        branch_chirho.domains_chirho[var_chirho] = {val_chirho}
        solutions_chirho.extend(solve_chirho(branch_chirho))
    
    return solutions_chirho


def main():
    print("=== Constraint Propagation + Incremental Tensor ☧ ===\n")
    
    # Setup
    store_chirho = TermStoreChirho()
    appendo_rel_chirho = SparseRelationChirho(3, "appendo")
    appendo_builder_chirho = AppendoBuilderChirho(store_chirho, appendo_rel_chirho)
    
    # Pre-intern some lists we'll use
    nil_idx_chirho = store_chirho.intern_list_chirho([])
    l0_idx_chirho = store_chirho.intern_list_chirho([0])
    l1_idx_chirho = store_chirho.intern_list_chirho([1])
    l01_idx_chirho = store_chirho.intern_list_chirho([0, 1])
    l10_idx_chirho = store_chirho.intern_list_chirho([1, 0])
    l001_idx_chirho = store_chirho.intern_list_chirho([0, 0, 1])
    
    # === Query 1: Forward ===
    print("="*60)
    print("QUERY 1: appendo([0], [1], Out)")
    print("="*60)
    
    cs1_chirho = ConstraintStoreChirho(store_chirho)
    cs1_chirho.add_relation_chirho("appendo", appendo_rel_chirho, appendo_builder_chirho)
    cs1_chirho.add_var_chirho("l", {l0_idx_chirho})
    cs1_chirho.add_var_chirho("s", {l1_idx_chirho})
    cs1_chirho.add_var_chirho("out")
    cs1_chirho.add_constraint_chirho("appendo", "l", "s", "out")
    
    # Need to seed the relation for forward direction
    appendo_builder_chirho.ensure_append_chirho(l0_idx_chirho, l1_idx_chirho)
    
    solutions1_chirho = solve_chirho(cs1_chirho)
    print(f"\nSolutions ({len(solutions1_chirho)}):")
    for sol_chirho in solutions1_chirho:
        sol_chirho.display_chirho()
    
    # === Query 2: Backward ===
    print("\n" + "="*60)
    print("QUERY 2: appendo(L, S, [0,1]) - what splits into [0,1]?")
    print("="*60)
    
    cs2_chirho = ConstraintStoreChirho(store_chirho)
    cs2_chirho.add_relation_chirho("appendo", appendo_rel_chirho, appendo_builder_chirho)
    cs2_chirho.add_var_chirho("l")
    cs2_chirho.add_var_chirho("s")
    cs2_chirho.add_var_chirho("out", {l01_idx_chirho})
    cs2_chirho.add_constraint_chirho("appendo", "l", "s", "out")
    
    solutions2_chirho = solve_chirho(cs2_chirho)
    print(f"\nSolutions ({len(solutions2_chirho)}):")
    for sol_chirho in solutions2_chirho:
        sol_chirho.display_chirho()
        print()
    
    # === Query 3: Relational - double ===
    print("="*60)
    print("QUERY 3: appendo(X, X, Out) - X appended to itself")
    print("="*60)
    
    # First, let's make sure we have some lists to work with
    # and their doubles
    for lst_chirho in [[], [0], [1]]:
        idx_chirho = store_chirho.intern_list_chirho(lst_chirho)
        double_chirho = lst_chirho + lst_chirho
        double_idx_chirho = store_chirho.intern_list_chirho(double_chirho)
        appendo_builder_chirho.ensure_append_chirho(idx_chirho, idx_chirho)
    
    # Make the domains explicit
    possible_x_chirho = {nil_idx_chirho, l0_idx_chirho, l1_idx_chirho}
    
    cs3_chirho = ConstraintStoreChirho(store_chirho)
    cs3_chirho.add_relation_chirho("appendo", appendo_rel_chirho, appendo_builder_chirho)
    cs3_chirho.add_var_chirho("x", possible_x_chirho)
    cs3_chirho.add_var_chirho("out")
    # Constraint: appendo(x, x, out) - we need both l and s to be x
    # This requires a trick: we use the same variable name
    cs3_chirho.add_constraint_chirho("appendo", "x", "x", "out")
    
    solutions3_chirho = solve_chirho(cs3_chirho)
    print(f"\nSolutions ({len(solutions3_chirho)}):")
    for sol_chirho in solutions3_chirho:
        sol_chirho.display_chirho()
        print()
    
    # === Query 4: Composition ===
    print("="*60)
    print("QUERY 4: appendo([0], Y, Z), appendo(Z, [1], [0,0,1])")
    print("         'What Y and Z satisfy both constraints?'")
    print("="*60)
    
    # Ensure we have the final output split
    appendo_builder_chirho.ensure_splits_chirho(l001_idx_chirho)
    
    cs4_chirho = ConstraintStoreChirho(store_chirho)
    cs4_chirho.add_relation_chirho("appendo", appendo_rel_chirho, appendo_builder_chirho)
    cs4_chirho.add_var_chirho("y")
    cs4_chirho.add_var_chirho("z")
    cs4_chirho.add_var_chirho("const_0", {l0_idx_chirho})
    cs4_chirho.add_var_chirho("const_1", {l1_idx_chirho})
    cs4_chirho.add_var_chirho("const_001", {l001_idx_chirho})
    cs4_chirho.add_constraint_chirho("appendo", "const_0", "y", "z")
    cs4_chirho.add_constraint_chirho("appendo", "z", "const_1", "const_001")
    
    solutions4_chirho = solve_chirho(cs4_chirho)
    print(f"\nSolutions ({len(solutions4_chirho)}):")
    for sol_chirho in solutions4_chirho:
        y_idx_chirho = next(iter(sol_chirho.domains_chirho["y"]))
        z_idx_chirho = next(iter(sol_chirho.domains_chirho["z"]))
        y_term_chirho = store_chirho.lookup_chirho(y_idx_chirho)
        z_term_chirho = store_chirho.lookup_chirho(z_idx_chirho)
        print(f"  y = {y_term_chirho}, z = {z_term_chirho}")
    
    # === Stats ===
    print("\n" + "="*60)
    print("FINAL STATS")
    print("="*60)
    print(f"Terms in store: {store_chirho.size_chirho()}")
    print(f"Triples in appendo: {len(appendo_rel_chirho.tuples_chirho)}")
    
    print("\nAll terms:")
    for idx_chirho, term_chirho in enumerate(store_chirho.idx_to_term_chirho):
        print(f"  {idx_chirho}: {term_chirho}")
    
    print("\n" + "="*60)
    print("WHAT WE ACHIEVED")
    print("="*60)
    print("""
    ✓ Demand-driven term creation (hash consing)
    ✓ Sparse relation storage (only computed triples)
    ✓ Arc consistency propagation (domains shrink)
    ✓ Backtracking search when needed
    ✓ Backward queries work (splitting)
    ✓ Composition of constraints
    
    For 1-bit representation:
    - Domains are sets of indices → could be bitsets
    - Relation lookup → sparse matrix ops
    - Propagation → fixpoint on bit operations
    
    What's still missing:
    - Actual unification (variables in terms)
    - Occurs check
    - Efficient composition (tensor contraction)
    """)


if __name__ == "__main__":
    main()
