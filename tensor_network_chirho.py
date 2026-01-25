#!/usr/bin/env python3
"""
Tensor Network Composition of Relations ☧

The insight: miniKanren program = tensor network
- Each relation = sparse tensor
- Shared variables = contracted indices
- Query = contract network, find nonzeros

Example: (fresh (x) (appendo a b x) (appendo x c out))
This is: "append a and b, then append c to the result"
"""
import numpy as np

# Same domain as before
LISTS = [(), (0,), (1,), (0,0), (0,1), (1,0), (1,1)]
LIST_TO_IDX = {l: i for i, l in enumerate(LISTS)}
NUM_LISTS = len(LISTS)

def list_name_chirho(idx_chirho):
    l_chirho = LISTS[idx_chirho]
    return "[]" if len(l_chirho) == 0 else "[" + ",".join(str(x_chirho) for x_chirho in l_chirho) + "]"

def build_appendo_tensor_chirho():
    """Build the appendo relation as a 3D tensor"""
    tensor = np.zeros((NUM_LISTS, NUM_LISTS, NUM_LISTS), dtype=np.uint8)
    for l_idx, l_val in enumerate(LISTS):
        for s_idx, s_val in enumerate(LISTS):
            result = l_val + s_val
            if result in LIST_TO_IDX:
                out_idx = LIST_TO_IDX[result]
                tensor[l_idx, s_idx, out_idx] = 1
    return tensor

def build_reverseo_tensor_chirho():
    """Build the reverseo relation as a 2D tensor"""
    tensor = np.zeros((NUM_LISTS, NUM_LISTS), dtype=np.uint8)
    for i, l in enumerate(LISTS):
        rev = tuple(reversed(l))
        if rev in LIST_TO_IDX:
            tensor[i, LIST_TO_IDX[rev]] = 1
    return tensor

def contract_boolean_chirho(A, B, axis_A, axis_B):
    """
    Boolean tensor contraction (using OR for sum, AND for product)
    
    This is einsum but with Boolean semiring.
    """
    # Move contracted axes to last/first positions
    A = np.moveaxis(A, axis_A, -1)
    B = np.moveaxis(B, axis_B, 0)
    
    # Result shape
    result_shape = A.shape[:-1] + B.shape[1:]
    result = np.zeros(result_shape, dtype=np.uint8)
    
    # Contract (loop over shared dimension, OR the ANDed slices)
    for k in range(A.shape[-1]):
        # A[..., k] AND B[k, ...] 
        outer = np.outer(A[..., k].flatten(), B[k, ...].flatten())
        outer = outer.reshape(A.shape[:-1] + B.shape[1:])
        result |= outer  # OR accumulate
    
    return result


def main():
    print("=== Tensor Network Composition ☧ ===\n")
    
    appendo = build_appendo_tensor_chirho()
    reverseo = build_reverseo_tensor_chirho()
    
    print("Built tensors:")
    print(f"  appendo:  {appendo.shape} (l × s × out), {appendo.sum()} nonzeros")
    print(f"  reverseo: {reverseo.shape} (in × out), {reverseo.sum()} nonzeros")
    
    # === Composition 1: append then append ===
    print("\n" + "="*60)
    print("QUERY: (fresh (x) (appendo a b x) (appendo x c out))")
    print("       'Append a to b, then append c to that'")
    print("="*60)
    
    # appendo(a,b,x) has shape (a, b, x)
    # appendo(x,c,out) has shape (x, c, out)
    # Contract on x → result shape (a, b, c, out)
    
    # Tensor 1: appendo(a, b, x) → shape (7, 7, 7) indexed as [a, b, x]
    T1 = appendo  # [a, b, x]
    
    # Tensor 2: appendo(x, c, out) → shape (7, 7, 7) indexed as [x, c, out]
    T2 = appendo  # [x, c, out]
    
    # Contract on x (axis 2 of T1, axis 0 of T2)
    # Result: [a, b, c, out]
    composed = contract_boolean_chirho(T1, T2, axis_A=2, axis_B=0)
    
    print(f"\nComposed tensor shape: {composed.shape} (a × b × c × out)")
    print(f"Nonzeros: {composed.sum()} out of {composed.size}")
    print(f"Density: {100*composed.sum()/composed.size:.2f}%")
    
    # Query: a=[0], b=[1], c=? → out=?
    print("\n--- Query: a=[0], b=[1], c=?, out=? ---")
    a_idx = LIST_TO_IDX[(0,)]
    b_idx = LIST_TO_IDX[(1,)]
    
    slice_ab = composed[a_idx, b_idx, :, :]  # shape (c, out)
    print("\nSlice [a=[0], b=[1], :, :]:")
    print("         " + " ".join(f"{list_name_chirho(i):>5}" for i in range(NUM_LISTS)))
    for c_idx in range(NUM_LISTS):
        row = " ".join(f"{slice_ab[c_idx, o]:>5}" for o in range(NUM_LISTS))
        if slice_ab[c_idx].any():
            print(f"c={list_name_chirho(c_idx):>5}: {row}")
    
    print("\nResults (nonzeros):")
    for c_idx in range(NUM_LISTS):
        for out_idx in range(NUM_LISTS):
            if slice_ab[c_idx, out_idx]:
                print(f"  c={list_name_chirho(c_idx)}, out={list_name_chirho(out_idx)}")
    
    # === Composition 2: append then reverse ===
    print("\n" + "="*60)
    print("QUERY: (fresh (x) (appendo a b x) (reverseo x out))")
    print("       'Append a to b, then reverse'")
    print("="*60)
    
    # appendo(a, b, x) → [a, b, x]
    # reverseo(x, out) → [x, out]
    # Contract on x → [a, b, out]
    
    composed2 = contract_boolean_chirho(appendo, reverseo, axis_A=2, axis_B=0)
    
    print(f"\nComposed tensor shape: {composed2.shape} (a × b × out)")
    print(f"Nonzeros: {composed2.sum()}")
    
    print("\n--- Query: a=[0], b=[1] → out=? ---")
    slice_result = composed2[a_idx, b_idx, :]
    for out_idx in range(NUM_LISTS):
        if slice_result[out_idx]:
            print(f"  out={list_name_chirho(out_idx)}")
    # [0] ++ [1] = [0,1], reversed = [1,0]
    
    # === Backward query ===
    print("\n" + "="*60)
    print("BACKWARD: (fresh (a b) (appendo a b x) (reverseo x [1,0]))")
    print("          'What a,b append to something that reverses to [1,0]?'")
    print("="*60)
    
    out_idx = LIST_TO_IDX[(1, 0)]
    slice_back = composed2[:, :, out_idx]  # shape (a, b)
    
    print("\nSlice [:, :, out=[1,0]]:")
    print("         " + " ".join(f"{list_name_chirho(i):>5}" for i in range(NUM_LISTS)))
    for a_i in range(NUM_LISTS):
        row = " ".join(f"{slice_back[a_i, b_i]:>5}" for b_i in range(NUM_LISTS))
        if slice_back[a_i].any():
            print(f"a={list_name_chirho(a_i):>5}: {row}")
    
    print("\nResults:")
    for a_i in range(NUM_LISTS):
        for b_i in range(NUM_LISTS):
            if slice_back[a_i, b_i]:
                # Verify: a ++ b = x, reverse(x) = [1,0]
                x = LISTS[a_i] + LISTS[b_i]
                print(f"  a={list_name_chirho(a_i)}, b={list_name_chirho(b_i)} → x={list(x)} → rev={list(reversed(x))}")
    
    # === The Big Picture ===
    print("\n" + "="*60)
    print("THE TENSOR NETWORK VIEW")
    print("="*60)
    print(r"""
    miniKanren program:
    
    (fresh (x y)
      (appendo a b x)
      (reverseo x y)
      (== y out))
    
    Tensor network diagram:
    
        a ──┬                    
            │                    
        b ──┼── [appendo] ── x ── [reverseo] ── y ── out
            │                    
            └                    
    
    Each box = tensor
    Each line = index
    Shared lines = contract
    
    Contraction order matters for efficiency!
    (Just like in quantum tensor networks)
    """)
    
    print("\n" + "="*60)
    print("1-BIT IMPLEMENTATION PATH")
    print("="*60)
    print("""
    Current: numpy uint8 (wasteful)
    
    Optimal 1-bit implementation:
    
    1. Bit-pack tensors: 
       - 64 values per uint64
       - appendo becomes ~6KB instead of 343 bytes
       - (Actually smaller due to sparsity)
    
    2. Boolean contraction:
       - AND = bitwise &
       - OR  = bitwise |
       - Use popcount for cardinality
    
    3. Sparse representation:
       - Store only nonzero indices
       - Coordinate format: (i, j, k) tuples
       - Or CSR/CSC for matrices
    
    4. Hardware:
       - XNOR + popcount on CPU/GPU
       - Or actual binary neural network hardware
       - FPGAs excel at this
    
    The entire miniKanren search becomes:
    - Build relation tensors (once, offline)
    - Contract network (query-time)
    - Read out nonzeros (results)
    
    No backtracking, no streams, no interleaving.
    Just tensor ops.
    """)


if __name__ == "__main__":
    main()
