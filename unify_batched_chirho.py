#!/usr/bin/env python3
"""
Batched 1-Bit Unification - Multiple states as tensor rows ☧

This is where it becomes actual matrix ops:
- Each "world" (search branch) is a row
- Variables × Values = columns within each row
- conde = duplicate rows
- unify = AND across columns
- propagation = batched iteration
"""
import numpy as np

# Layout: each state is 3 vars × 3 values = 9 bits
# Flattened: [x_a, x_b, x_c, y_a, y_b, y_c, z_a, z_b, z_c]

VAR_X, VAR_Y, VAR_Z = 0, 1, 2
VAL_A, VAL_B, VAL_C = 0, 1, 2

def var_slice_chirho(v_chirho):
    """Get column indices for variable v"""
    return slice(v_chirho*3, v_chirho*3+3)

def val_mask_chirho(v_chirho, val_chirho):
    """Mask that's 1 only at variable v, value val"""
    m_chirho = np.zeros(9, dtype=np.uint8)
    m_chirho[v_chirho*3 + val_chirho] = 1
    return m_chirho

def fresh_state_chirho():
    """Single fresh state: all 1s"""
    return np.ones((1, 9), dtype=np.uint8)

def unify_value_chirho(states, var, val):
    """Constrain variable to specific value across all states"""
    # Create mask: 1s everywhere except var's other values
    mask = np.ones(9, dtype=np.uint8)
    for v in range(3):
        if v != val:
            mask[var*3 + v] = 0
    # AND with all states (broadcasting)
    return states & mask

def unify_vars_chirho(states, v1, v2):
    """Make two variables equal across all states"""
    # For each state: intersect possible values, set both vars to intersection
    result = states.copy()
    for i in range(states.shape[0]):
        # Get possible values for each var
        poss1 = states[i, var_slice_chirho(v1)]
        poss2 = states[i, var_slice_chirho(v2)]
        intersection = poss1 & poss2
        result[i, var_slice_chirho(v1)] = intersection
        result[i, var_slice_chirho(v2)] = intersection
    return result

def conde_chirho(states, *branches):
    """Disjunction: duplicate states, apply different constraints to each copy"""
    results = []
    for branch in branches:
        new_states = branch(states.copy())
        results.append(new_states)
    # Stack all branches
    combined = np.vstack(results)
    # Remove failed states (any row where a variable has all zeros)
    valid_mask = np.ones(combined.shape[0], dtype=bool)
    for v in range(3):
        var_has_value = combined[:, var_slice_chirho(v)].any(axis=1)
        valid_mask &= var_has_value
    return combined[valid_mask]

def display_states_chirho(states, label=""):
    """Show all states as bit matrices"""
    print(f"\n{'='*40}")
    print(f"{label} ({states.shape[0]} states)")
    print('='*40)
    for i, state in enumerate(states):
        print(f"\nState {i}:")
        print("       a b c")
        for v_chirho, name_chirho in enumerate(['x', 'y', 'z']):
            bits_chirho = state[var_slice_chirho(v_chirho)]
            print(f"    {name_chirho} [{bits_chirho[0]} {bits_chirho[1]} {bits_chirho[2]}]")

def main():
    print("=== Batched 1-Bit Unification ☧ ===")
    
    # Start with one fresh state
    s = fresh_state_chirho()
    display_states_chirho(s, "Initial (fresh)")
    
    # x = y (constrain)
    s = unify_vars_chirho(s, VAR_X, VAR_Y)
    display_states_chirho(s, "After x ≡ y")
    
    # conde: branch into x=a OR x=b OR x=c
    # This TRIPLES our state count
    s = conde_chirho(s,
        lambda st: unify_value_chirho(st, VAR_X, VAL_A),
        lambda st: unify_value_chirho(st, VAR_X, VAL_B),
        lambda st: unify_value_chirho(st, VAR_X, VAL_C),
    )
    display_states_chirho(s, "After conde(x=a, x=b, x=c)")
    
    # Now propagate x≡y by re-applying
    s = unify_vars_chirho(s, VAR_X, VAR_Y)
    display_states_chirho(s, "After propagating x≡y to all branches")
    
    # Set z = c in all states (batched AND)
    s = unify_value_chirho(s, VAR_Z, VAL_C)
    display_states_chirho(s, "After z=c in all branches")
    
    # Now let's show the FULL matrix as one object
    print("\n" + "="*50)
    print("THE FULL SEARCH SPACE AS ONE MATRIX:")
    print("="*50)
    print("\nRows = parallel universes (search branches)")
    print("Cols = (var, val) pairs flattened")
    print()
    print("        x           y           z")
    print("        a  b  c     a  b  c     a  b  c")
    print("      " + "-"*35)
    for i, state in enumerate(s):
        row = "  ".join(f"{state[j]}" for j in range(9))
        # Add visual grouping
        grouped = f"{state[0]}  {state[1]}  {state[2]}   | {state[3]}  {state[4]}  {state[5]}   | {state[6]}  {state[7]}  {state[8]}"
        print(f"  w{i} | {grouped}")
    
    print("\n" + "="*50)
    print("KEY REALIZATION:")
    print("="*50)
    print("""
    1. The entire miniKanren search tree is ONE matrix
    2. Each 'conde' = tensor expansion (add rows)
    3. Each 'unify' = bitwise AND (column masking)
    4. Failure = row becomes zero → filter out
    
    For 1-bit inference:
    - This IS the native format
    - No float conversion needed
    - XNOR + popcount for constraint checking
    - Memory: 1 bit per (world, var, val)
    
    Scaling:
    - 64 variables × 64 values = 4096 bits = 512 bytes per world
    - 1 million parallel worlds = 512 MB
    - All operations are embarrassingly parallel
    """)

    print("\n" + "="*50)
    print("WHAT MATRIX MULTIPLICATION WOULD LOOK LIKE:")
    print("="*50)
    print("""
    For substitution composition (not shown above):
    
    If substitution σ maps: x→a, y→x
    This is a permutation matrix!
    
         x  y  z        a  b  c
    σ = [0  0  0   |   1  0  0]   x → a
        [1  0  0   |   0  0  0]   y → x (which becomes a)
        [0  0  1   |   0  0  0]   z → z
    
    Composing substitutions = matrix multiplication
    (with Boolean semiring: OR for +, AND for ×)
    
    This is the tensor relational algebra connection!
    """)


if __name__ == "__main__":
    main()
