#!/usr/bin/env python3
"""
appendo in 1-Bit Matrix Unification ☧

The classic miniKanren relation:
  (appendo l s out) means (append l s) = out

Challenge: appendo is recursive, lists are unbounded
Solution: Bounded domain - lists of length 0-2, elements from {0, 1}

This lets us see the STRUCTURE of the search as a matrix.
"""
import numpy as np
from itertools import product

# === Domain Definition ===
# Lists up to length 2 with elements from {0, 1}
# Possible lists: [], [0], [1], [0,0], [0,1], [1,0], [1,1]
LISTS = [
    (),
    (0,),
    (1,),
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
]
LIST_TO_IDX = {l: i for i, l in enumerate(LISTS)}
NUM_LISTS = len(LISTS)

def list_name_chirho(idx_chirho):
    """Pretty print a list value"""
    l_chirho = LISTS[idx_chirho]
    if len(l_chirho) == 0:
        return "[]"
    return "[" + ",".join(str(x_chirho) for x_chirho in l_chirho) + "]"

# === State Representation ===
# 3 variables: l, s, out
# Each variable can be any of NUM_LISTS values
# State shape: (num_worlds, 3, NUM_LISTS) - one-hot per variable

VAR_L, VAR_S, VAR_OUT = 0, 1, 2
VAR_NAMES = ['l', 's', 'out']

class AppendoStateChirho:
    def __init__(self, data=None):
        if data is None:
            # Fresh: all variables can be any list
            self.data = np.ones((1, 3, NUM_LISTS), dtype=np.uint8)
        else:
            self.data = data
    
    @property
    def num_worlds_chirho(self):
        return self.data.shape[0]

    def copy_chirho(self):
        return AppendoStateChirho(self.data.copy())

    def failed_worlds_chirho(self):
        """Return mask of which worlds have failed (any var has no possible values)"""
        return ~self.data.any(axis=2).all(axis=1)
    
    def prune_failed_chirho(self):
        """Remove failed worlds"""
        valid = ~self.failed_worlds_chirho()
        return AppendoStateChirho(self.data[valid])
    
    def unify_value_chirho(self, var, list_idx):
        """Constrain variable to specific list value"""
        result = self.data.copy()
        mask = np.zeros(NUM_LISTS, dtype=np.uint8)
        mask[list_idx] = 1
        result[:, var, :] &= mask
        return AppendoStateChirho(result).prune_failed_chirho()
    
    def unify_vars_chirho(self, v1, v2):
        """Make two variables equal"""
        result = self.data.copy()
        intersection = result[:, v1, :] & result[:, v2, :]
        result[:, v1, :] = intersection
        result[:, v2, :] = intersection
        return AppendoStateChirho(result).prune_failed_chirho()
    
    def conde_chirho(self, *branches):
        """Disjunction: apply each branch, combine results"""
        results = []
        for branch in branches:
            new_state = branch(self.copy())
            if new_state.num_worlds_chirho > 0:
                results.append(new_state.data)
        if not results:
            return AppendoStateChirho(np.zeros((0, 3, NUM_LISTS), dtype=np.uint8))
        return AppendoStateChirho(np.vstack(results))
    
    def constrain_appendo_chirho(self):
        """
        Apply the appendo constraint: for each world, 
        filter to only (l, s, out) triples where append(l, s) = out
        
        This is THE KEY OPERATION - it's a relational join!
        """
        result_worlds = []
        
        for w in range(self.num_worlds_chirho):
            # Get possible values for each var
            l_possible = np.where(self.data[w, VAR_L, :])[0]
            s_possible = np.where(self.data[w, VAR_S, :])[0]
            out_possible = np.where(self.data[w, VAR_OUT, :])[0]
            
            # Find all valid (l, s, out) combinations
            valid_combinations = []
            for l_idx in l_possible:
                for s_idx in s_possible:
                    # Compute actual append
                    l_val = LISTS[l_idx]
                    s_val = LISTS[s_idx]
                    result = l_val + s_val  # tuple concatenation
                    
                    # Check if result is in our domain and allowed
                    if result in LIST_TO_IDX:
                        out_idx = LIST_TO_IDX[result]
                        if out_idx in out_possible:
                            valid_combinations.append((l_idx, s_idx, out_idx))
            
            # Create a new world for each valid combination
            for l_idx, s_idx, out_idx in valid_combinations:
                new_world = np.zeros((1, 3, NUM_LISTS), dtype=np.uint8)
                new_world[0, VAR_L, l_idx] = 1
                new_world[0, VAR_S, s_idx] = 1
                new_world[0, VAR_OUT, out_idx] = 1
                result_worlds.append(new_world)
        
        if not result_worlds:
            return AppendoStateChirho(np.zeros((0, 3, NUM_LISTS), dtype=np.uint8))
        return AppendoStateChirho(np.vstack(result_worlds))
    
    def display_chirho(self, label=""):
        print(f"\n{'='*50}")
        print(f"{label} ({self.num_worlds_chirho} worlds)")
        print('='*50)
        
        if self.num_worlds_chirho == 0:
            print("  (no valid states)")
            return
        
        for w in range(min(self.num_worlds_chirho, 10)):  # Show max 10
            l_vals = [list_name_chirho(i) for i in range(NUM_LISTS) if self.data[w, VAR_L, i]]
            s_vals = [list_name_chirho(i) for i in range(NUM_LISTS) if self.data[w, VAR_S, i]]
            out_vals = [list_name_chirho(i) for i in range(NUM_LISTS) if self.data[w, VAR_OUT, i]]
            
            # If determined (single value), show cleanly
            l_str = l_vals[0] if len(l_vals) == 1 else "{" + ",".join(l_vals) + "}"
            s_str = s_vals[0] if len(s_vals) == 1 else "{" + ",".join(s_vals) + "}"
            out_str = out_vals[0] if len(out_vals) == 1 else "{" + ",".join(out_vals) + "}"
            
            print(f"  w{w}: l={l_str}, s={s_str}, out={out_str}")
        
        if self.num_worlds_chirho > 10:
            print(f"  ... and {self.num_worlds_chirho - 10} more")
    
    def matrix_view_chirho(self, label=""):
        """Show the raw bit tensor"""
        print(f"\n{'='*50}")
        print(f"MATRIX VIEW: {label}")
        print('='*50)
        print("\nShape:", self.data.shape, "(worlds × vars × list_values)")
        print("\nList encoding:", " ".join(f"{i}={list_name_chirho(i)}" for i in range(NUM_LISTS)))
        print()
        
        for w in range(min(self.num_worlds_chirho, 5)):
            print(f"World {w}:")
            print("        " + " ".join(f"{list_name_chirho(i):>5}" for i in range(NUM_LISTS)))
            for v_chirho, name_chirho in enumerate(VAR_NAMES):
                bits_chirho = " ".join(f"{self.data[w, v_chirho, i]:>5}" for i in range(NUM_LISTS))
                print(f"  {name_chirho:>4}: {bits_chirho}")
            print()


def main():
    print("=== appendo in 1-Bit Matrices ☧ ===")
    print("\nDomain: lists up to length 2, elements ∈ {0, 1}")
    print("Lists:", [list_name_chirho(i) for i in range(NUM_LISTS)])
    
    # === Query 1: Run appendo forward ===
    print("\n" + "#"*60)
    print("# QUERY 1: (appendo [0] [1] out) - what is out?")
    print("#"*60)
    
    s1 = AppendoStateChirho()
    s1 = s1.unify_value_chirho(VAR_L, LIST_TO_IDX[(0,)])
    s1 = s1.unify_value_chirho(VAR_S, LIST_TO_IDX[(1,)])
    s1.display_chirho("After constraining l=[0], s=[1]")
    
    s1 = s1.constrain_appendo_chirho()
    s1.display_chirho("After appendo constraint")
    # Should get: out = [0,1]
    
    # === Query 2: Run appendo backward ===
    print("\n" + "#"*60)
    print("# QUERY 2: (appendo l s [0,1]) - what splits into [0,1]?")
    print("#"*60)
    
    s2 = AppendoStateChirho()
    s2 = s2.unify_value_chirho(VAR_OUT, LIST_TO_IDX[(0, 1)])
    s2.display_chirho("After constraining out=[0,1]")
    
    s2 = s2.constrain_appendo_chirho()
    s2.display_chirho("After appendo constraint")
    # Should get multiple: l=[], s=[0,1]; l=[0], s=[1]; l=[0,1], s=[]
    
    # === Query 3: Fully relational ===
    print("\n" + "#"*60)
    print("# QUERY 3: (appendo l l out) - l appended to itself")
    print("#"*60)
    
    s3 = AppendoStateChirho()
    s3 = s3.unify_vars_chirho(VAR_L, VAR_S)  # l = s
    s3.display_chirho("After constraining l = s")
    
    s3 = s3.constrain_appendo_chirho()
    s3.display_chirho("After appendo constraint")
    # Should get: l=s=[], out=[]; l=s=[0], out=[0,0]; l=s=[1], out=[1,1]
    
    # === Show the matrix structure ===
    print("\n" + "#"*60)
    print("# THE APPENDO RELATION AS A BIT TENSOR")
    print("#"*60)
    
    # Generate ALL valid appendo triples
    all_triples = AppendoStateChirho()
    all_triples = all_triples.constrain_appendo_chirho()
    all_triples.display_chirho("Complete appendo relation")
    all_triples.matrix_view_chirho("Full appendo tensor")
    
    print("\n" + "="*60)
    print("KEY INSIGHT:")
    print("="*60)
    print("""
    The appendo RELATION is a 3D bit tensor:
    - Axis 0: l values
    - Axis 1: s values  
    - Axis 2: out values
    
    Entry [i,j,k] = 1 iff append(list_i, list_j) = list_k
    
    This is a SPARSE tensor (most entries 0).
    
    Queries are tensor operations:
    - Forward (l,s known): slice + argmax
    - Backward (out known): slice along out axis
    - Relational: tensor contraction with constraints
    
    The "search" is just finding nonzero entries!
    """)
    
    # Build and show the actual relation tensor
    print("\n" + "="*60)
    print("THE RELATION TENSOR (3D):")
    print("="*60)
    
    relation = np.zeros((NUM_LISTS, NUM_LISTS, NUM_LISTS), dtype=np.uint8)
    for l_idx, l_val in enumerate(LISTS):
        for s_idx, s_val in enumerate(LISTS):
            result = l_val + s_val
            if result in LIST_TO_IDX:
                out_idx = LIST_TO_IDX[result]
                relation[l_idx, s_idx, out_idx] = 1
    
    print(f"\nShape: {relation.shape} (l × s × out)")
    print(f"Nonzero entries: {relation.sum()} out of {relation.size}")
    print(f"Density: {100*relation.sum()/relation.size:.1f}%")
    
    print("\nSlice where l=[] (index 0):")
    print("       " + " ".join(f"{list_name_chirho(i):>5}" for i in range(NUM_LISTS)))
    for s_idx in range(NUM_LISTS):
        row = " ".join(f"{relation[0, s_idx, o]:>5}" for o in range(NUM_LISTS))
        print(f"s={list_name_chirho(s_idx):>5}: {row}")
    
    print("\n" + "="*60)
    print("NEXT STEP: TENSOR NETWORK")
    print("="*60)
    print("""
    For larger programs, multiple relations compose:
    
    (fresh (x y z)
      (appendo a b x)
      (appendo x c y)
      (reverseo y z))
    
    Each relation = tensor
    Composition = tensor contraction
    Variables = shared indices
    
    This is EXACTLY how tensor networks work!
    The "search" becomes: contract the network, find nonzeros.
    
    1-bit optimization:
    - Tensors are bit-packed
    - Contraction uses AND/OR (Boolean semiring)
    - Sparse tensor libraries apply directly
    """)


if __name__ == "__main__":
    main()
