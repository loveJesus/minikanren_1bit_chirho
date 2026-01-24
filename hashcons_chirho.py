#!/usr/bin/env python3
"""
Hash Consing + Incremental Tensor for miniKanren ☧

Instead of pre-enumerating all terms:
1. Hash cons: each unique term gets a unique ID
2. Tensor grows dynamically as we discover terms
3. Relations computed lazily on first encounter

This bridges infinite terms with finite (but growing) tensors.
"""
from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional, Set, Union
import numpy as np

# === Term Representation with Hash Consing ===

@dataclass(frozen=True)
class NilChirho:
    """Empty list"""
    def __repr__(self_chirho):
        return "[]"

@dataclass(frozen=True)
class ConsChirho:
    """List cons cell"""
    head_chirho: Union[int, 'VarChirho']
    tail_chirho: Union['NilChirho', 'ConsChirho', 'VarChirho']
    
    def __repr__(self_chirho):
        def to_list_chirho(t_chirho):
            if isinstance(t_chirho, NilChirho):
                return []
            elif isinstance(t_chirho, ConsChirho):
                return [t_chirho.head_chirho] + to_list_chirho(t_chirho.tail_chirho)
            else:
                return [f"...{t_chirho}"]
        return str(to_list_chirho(self_chirho))

@dataclass(frozen=True)
class VarChirho:
    """Logic variable (for unification)"""
    id_chirho: int
    
    def __repr__(self_chirho):
        return f"_{self_chirho.id_chirho}"

TermChirho = Union[NilChirho, ConsChirho, VarChirho, int]


class TermStoreChirho:
    """
    Hash consing store for terms.
    Every unique term gets a unique integer index.
    """
    def __init__(self_chirho):
        self_chirho.term_to_idx_chirho: Dict[TermChirho, int] = {}
        self_chirho.idx_to_term_chirho: List[TermChirho] = []
    
    def intern_chirho(self_chirho, term_chirho: TermChirho) -> int:
        """Get or create index for term"""
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
    
    def all_terms_chirho(self_chirho) -> List[Tuple[int, TermChirho]]:
        return list(enumerate(self_chirho.idx_to_term_chirho))


# === Incremental Relation Tensor ===

class IncrementalAppendoChirho:
    """
    appendo relation that grows as terms are discovered.
    
    Internally maintains a sparse set of (l, s, out) triples.
    Tensor representation computed on-demand.
    """
    def __init__(self_chirho, store_chirho: TermStoreChirho):
        self_chirho.store_chirho = store_chirho
        # Set of valid (l_idx, s_idx, out_idx) triples
        self_chirho.triples_chirho: Set[Tuple[int, int, int]] = set()
        # Track which terms we've fully expanded
        self_chirho.expanded_l_chirho: Set[int] = set()
        self_chirho.expanded_s_chirho: Set[int] = set()
    
    def _is_ground_list_chirho(self_chirho, term_chirho: TermChirho) -> bool:
        """Check if term is a ground (variable-free) list"""
        if isinstance(term_chirho, NilChirho):
            return True
        elif isinstance(term_chirho, ConsChirho):
            return (isinstance(term_chirho.head_chirho, int) and 
                    self_chirho._is_ground_list_chirho(term_chirho.tail_chirho))
        return False
    
    def _append_terms_chirho(self_chirho, l_chirho: TermChirho, s_chirho: TermChirho) -> Optional[TermChirho]:
        """Actually append two ground lists, return result or None if invalid"""
        if isinstance(l_chirho, NilChirho):
            return s_chirho
        elif isinstance(l_chirho, ConsChirho):
            rest_chirho = self_chirho._append_terms_chirho(l_chirho.tail_chirho, s_chirho)
            if rest_chirho is None:
                return None
            return ConsChirho(l_chirho.head_chirho, rest_chirho)
        return None
    
    def expand_for_l_chirho(self_chirho, l_idx_chirho: int):
        """Lazily compute all appendo triples with this l value"""
        if l_idx_chirho in self_chirho.expanded_l_chirho:
            return
        
        l_term_chirho = self_chirho.store_chirho.lookup_chirho(l_idx_chirho)
        if not self_chirho._is_ground_list_chirho(l_term_chirho):
            self_chirho.expanded_l_chirho.add(l_idx_chirho)
            return
        
        # Try appending with all known s values
        for s_idx_chirho, s_term_chirho in self_chirho.store_chirho.all_terms_chirho():
            if self_chirho._is_ground_list_chirho(s_term_chirho):
                out_term_chirho = self_chirho._append_terms_chirho(l_term_chirho, s_term_chirho)
                if out_term_chirho is not None:
                    out_idx_chirho = self_chirho.store_chirho.intern_chirho(out_term_chirho)
                    self_chirho.triples_chirho.add((l_idx_chirho, s_idx_chirho, out_idx_chirho))
        
        self_chirho.expanded_l_chirho.add(l_idx_chirho)
    
    def expand_for_s_chirho(self_chirho, s_idx_chirho: int):
        """Lazily compute all appendo triples with this s value"""
        if s_idx_chirho in self_chirho.expanded_s_chirho:
            return
        
        s_term_chirho = self_chirho.store_chirho.lookup_chirho(s_idx_chirho)
        if not self_chirho._is_ground_list_chirho(s_term_chirho):
            self_chirho.expanded_s_chirho.add(s_idx_chirho)
            return
        
        # Try appending with all known l values
        for l_idx_chirho, l_term_chirho in self_chirho.store_chirho.all_terms_chirho():
            if self_chirho._is_ground_list_chirho(l_term_chirho):
                out_term_chirho = self_chirho._append_terms_chirho(l_term_chirho, s_term_chirho)
                if out_term_chirho is not None:
                    out_idx_chirho = self_chirho.store_chirho.intern_chirho(out_term_chirho)
                    self_chirho.triples_chirho.add((l_idx_chirho, s_idx_chirho, out_idx_chirho))
        
        self_chirho.expanded_s_chirho.add(s_idx_chirho)
    
    def expand_for_out_chirho(self_chirho, out_idx_chirho: int):
        """
        Backward: given an output, find all (l, s) that produce it.
        This is the HARD direction - we need to enumerate all splits.
        """
        out_term_chirho = self_chirho.store_chirho.lookup_chirho(out_idx_chirho)
        if not self_chirho._is_ground_list_chirho(out_term_chirho):
            return
        
        # Convert to Python list for easier splitting
        def to_pylist_chirho(t_chirho):
            if isinstance(t_chirho, NilChirho):
                return []
            elif isinstance(t_chirho, ConsChirho):
                return [t_chirho.head_chirho] + to_pylist_chirho(t_chirho.tail_chirho)
            return None
        
        def from_pylist_chirho(lst_chirho):
            result_chirho = NilChirho()
            for x_chirho in reversed(lst_chirho):
                result_chirho = ConsChirho(x_chirho, result_chirho)
            return result_chirho
        
        pylist_chirho = to_pylist_chirho(out_term_chirho)
        if pylist_chirho is None:
            return
        
        # Enumerate all splits: out = l ++ s
        for i_chirho in range(len(pylist_chirho) + 1):
            l_py_chirho = pylist_chirho[:i_chirho]
            s_py_chirho = pylist_chirho[i_chirho:]
            
            l_term_chirho = from_pylist_chirho(l_py_chirho)
            s_term_chirho = from_pylist_chirho(s_py_chirho)
            
            l_idx_chirho = self_chirho.store_chirho.intern_chirho(l_term_chirho)
            s_idx_chirho = self_chirho.store_chirho.intern_chirho(s_term_chirho)
            
            self_chirho.triples_chirho.add((l_idx_chirho, s_idx_chirho, out_idx_chirho))
    
    def query_forward_chirho(self_chirho, l_idx_chirho: int, s_idx_chirho: int) -> List[int]:
        """Given l and s, find all valid out values"""
        self_chirho.expand_for_l_chirho(l_idx_chirho)
        self_chirho.expand_for_s_chirho(s_idx_chirho)
        
        results_chirho = []
        for (l_chirho, s_chirho, out_chirho) in self_chirho.triples_chirho:
            if l_chirho == l_idx_chirho and s_chirho == s_idx_chirho:
                results_chirho.append(out_chirho)
        return results_chirho
    
    def query_backward_chirho(self_chirho, out_idx_chirho: int) -> List[Tuple[int, int]]:
        """Given out, find all valid (l, s) pairs"""
        self_chirho.expand_for_out_chirho(out_idx_chirho)
        
        results_chirho = []
        for (l_chirho, s_chirho, out_chirho) in self_chirho.triples_chirho:
            if out_chirho == out_idx_chirho:
                results_chirho.append((l_chirho, s_chirho))
        return results_chirho
    
    def to_dense_tensor_chirho(self_chirho) -> np.ndarray:
        """Convert current triples to dense tensor (for debugging)"""
        n_chirho = self_chirho.store_chirho.size_chirho()
        tensor_chirho = np.zeros((n_chirho, n_chirho, n_chirho), dtype=np.uint8)
        for (l_chirho, s_chirho, out_chirho) in self_chirho.triples_chirho:
            tensor_chirho[l_chirho, s_chirho, out_chirho] = 1
        return tensor_chirho


# === Search State ===

class SearchStateChirho:
    """
    State of a relational search.
    Maps variable indices to sets of possible term indices.
    """
    def __init__(self_chirho, store_chirho: TermStoreChirho, num_vars_chirho: int):
        self_chirho.store_chirho = store_chirho
        # Each variable maps to set of possible term indices
        # None means "any term" (infinite domain)
        self_chirho.possible_chirho: List[Optional[Set[int]]] = [None] * num_vars_chirho
    
    def copy_chirho(self_chirho) -> 'SearchStateChirho':
        new_chirho = SearchStateChirho(self_chirho.store_chirho, len(self_chirho.possible_chirho))
        new_chirho.possible_chirho = [
            s_chirho.copy() if s_chirho is not None else None 
            for s_chirho in self_chirho.possible_chirho
        ]
        return new_chirho
    
    def constrain_chirho(self_chirho, var_idx_chirho: int, term_indices_chirho: Set[int]) -> bool:
        """
        Constrain variable to given term indices.
        Returns False if this causes failure (empty intersection).
        """
        current_chirho = self_chirho.possible_chirho[var_idx_chirho]
        if current_chirho is None:
            self_chirho.possible_chirho[var_idx_chirho] = term_indices_chirho.copy()
        else:
            intersection_chirho = current_chirho & term_indices_chirho
            if not intersection_chirho:
                return False
            self_chirho.possible_chirho[var_idx_chirho] = intersection_chirho
        return True
    
    def is_determined_chirho(self_chirho, var_idx_chirho: int) -> bool:
        """Check if variable has exactly one possible value"""
        poss_chirho = self_chirho.possible_chirho[var_idx_chirho]
        return poss_chirho is not None and len(poss_chirho) == 1
    
    def get_value_chirho(self_chirho, var_idx_chirho: int) -> Optional[int]:
        """Get determined value or None"""
        poss_chirho = self_chirho.possible_chirho[var_idx_chirho]
        if poss_chirho is not None and len(poss_chirho) == 1:
            return next(iter(poss_chirho))
        return None
    
    def display_chirho(self_chirho, var_names_chirho: List[str] = None):
        if var_names_chirho is None:
            var_names_chirho = [f"v{i_chirho}" for i_chirho in range(len(self_chirho.possible_chirho))]
        
        for i_chirho, name_chirho in enumerate(var_names_chirho):
            poss_chirho = self_chirho.possible_chirho[i_chirho]
            if poss_chirho is None:
                print(f"  {name_chirho}: (any)")
            elif len(poss_chirho) == 0:
                print(f"  {name_chirho}: FAILED (∅)")
            elif len(poss_chirho) == 1:
                idx_chirho = next(iter(poss_chirho))
                term_chirho = self_chirho.store_chirho.lookup_chirho(idx_chirho)
                print(f"  {name_chirho} = {term_chirho}")
            else:
                terms_chirho = [str(self_chirho.store_chirho.lookup_chirho(idx_chirho)) 
                               for idx_chirho in sorted(poss_chirho)[:5]]
                suffix_chirho = f"... +{len(poss_chirho)-5} more" if len(poss_chirho) > 5 else ""
                print(f"  {name_chirho} ∈ {{{', '.join(terms_chirho)}{suffix_chirho}}}")


def main():
    print("=== Hash Consing + Incremental Tensor ☧ ===\n")
    
    # Create store
    store_chirho = TermStoreChirho()
    
    # Intern some initial terms
    nil_chirho = store_chirho.intern_chirho(NilChirho())
    list_0_chirho = store_chirho.intern_chirho(ConsChirho(0, NilChirho()))
    list_1_chirho = store_chirho.intern_chirho(ConsChirho(1, NilChirho()))
    list_01_chirho = store_chirho.intern_chirho(ConsChirho(0, ConsChirho(1, NilChirho())))
    
    print("Initial terms in store:")
    for idx_chirho, term_chirho in store_chirho.all_terms_chirho():
        print(f"  {idx_chirho}: {term_chirho}")
    
    # Create appendo relation
    appendo_chirho = IncrementalAppendoChirho(store_chirho)
    
    # === Forward query ===
    print("\n" + "="*50)
    print("FORWARD: appendo([0], [1], out)")
    print("="*50)
    
    results_chirho = appendo_chirho.query_forward_chirho(list_0_chirho, list_1_chirho)
    print(f"\nResults ({len(results_chirho)}):")
    for out_idx_chirho in results_chirho:
        print(f"  out = {store_chirho.lookup_chirho(out_idx_chirho)}")
    
    print(f"\nStore now has {store_chirho.size_chirho()} terms")
    print(f"Relation has {len(appendo_chirho.triples_chirho)} triples")
    
    # === Backward query ===
    print("\n" + "="*50)
    print("BACKWARD: appendo(l, s, [0,1])")
    print("="*50)
    
    pairs_chirho = appendo_chirho.query_backward_chirho(list_01_chirho)
    print(f"\nResults ({len(pairs_chirho)}):")
    for (l_idx_chirho, s_idx_chirho) in pairs_chirho:
        l_term_chirho = store_chirho.lookup_chirho(l_idx_chirho)
        s_term_chirho = store_chirho.lookup_chirho(s_idx_chirho)
        print(f"  l = {l_term_chirho}, s = {s_term_chirho}")
    
    print(f"\nStore now has {store_chirho.size_chirho()} terms")
    
    # === Longer list ===
    print("\n" + "="*50)
    print("LONGER: appendo([0,1], [1,0], out)")
    print("="*50)
    
    list_10_chirho = store_chirho.intern_chirho(ConsChirho(1, ConsChirho(0, NilChirho())))
    results2_chirho = appendo_chirho.query_forward_chirho(list_01_chirho, list_10_chirho)
    
    print(f"\nResults ({len(results2_chirho)}):")
    for out_idx_chirho in results2_chirho:
        print(f"  out = {store_chirho.lookup_chirho(out_idx_chirho)}")
    
    print(f"\nStore now has {store_chirho.size_chirho()} terms (grew dynamically!)")
    print("\nAll terms:")
    for idx_chirho, term_chirho in store_chirho.all_terms_chirho():
        print(f"  {idx_chirho}: {term_chirho}")
    
    # === Show the growing tensor ===
    print("\n" + "="*50)
    print("CURRENT RELATION TENSOR")
    print("="*50)
    
    tensor_chirho = appendo_chirho.to_dense_tensor_chirho()
    print(f"\nShape: {tensor_chirho.shape}")
    print(f"Nonzeros: {tensor_chirho.sum()}")
    print(f"Density: {100*tensor_chirho.sum()/tensor_chirho.size:.2f}%")
    
    print("\nTriples:")
    for (l_chirho, s_chirho, o_chirho) in sorted(appendo_chirho.triples_chirho):
        l_t_chirho = store_chirho.lookup_chirho(l_chirho)
        s_t_chirho = store_chirho.lookup_chirho(s_chirho)
        o_t_chirho = store_chirho.lookup_chirho(o_chirho)
        print(f"  {l_t_chirho} ++ {s_t_chirho} = {o_t_chirho}")
    
    print("\n" + "="*50)
    print("KEY INSIGHT")
    print("="*50)
    print("""
    The tensor is now DEMAND-DRIVEN:
    
    1. Start with empty relation
    2. Query forces expansion
    3. New terms get interned on-the-fly
    4. Tensor dimensions grow as needed
    
    For infinite domains:
    - We never enumerate everything
    - Only materialize what the query touches
    - Hash consing ensures no duplicates
    
    The tradeoff:
    - Lost: "compile once, query many" benefit
    - Gained: Works with unbounded term depth
    
    For 1-bit: 
    - Sparse COO format: store (l, s, out) triples
    - Each triple = 3 integers = ~12 bytes
    - No dense tensor allocation needed
    """)


if __name__ == "__main__":
    main()
