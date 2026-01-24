#!/usr/bin/env python3
"""
1-Bit Matrix Unification - miniKanren-style logic as bit ops ☧

3 variables (x, y, z), 3 possible values (a, b, c)
Each variable = 3-bit mask of possible values
Unification = AND operation
Variable equality = propagation through constraint matrix
"""

# Constants
VAR_X, VAR_Y, VAR_Z = 0, 1, 2
VAL_A = 0b001
VAL_B = 0b010
VAL_C = 0b100
VAL_ANY = 0b111
VAL_NONE = 0b000

class StateChirho:
    def __init__(self, possible=None, equals=None):
        # possible[i] = bitmask of values variable i can take
        self.possible = possible if possible else [VAL_ANY, VAL_ANY, VAL_ANY]
        # equals[i] = bitmask of variables that must equal variable i
        self.equals = equals if equals else [0b001, 0b010, 0b100]
    
    def copy(self):
        return StateChirho(self.possible.copy(), self.equals.copy())
    
    def failed_chirho(self):
        return any(p == VAL_NONE for p in self.possible)
    
    def unify_value_chirho(self, var, val):
        """Unify variable with specific value"""
        s = self.copy()
        s.possible[var] &= val
        return s.propagate_chirho()
    
    def unify_vars_chirho(self, v1, v2):
        """Unify two variables (they must be equal)"""
        s = self.copy()
        merged = s.equals[v1] | s.equals[v2]
        for i in range(3):
            if (merged >> i) & 1:
                s.equals[i] = merged
        return s.propagate_chirho()
    
    def propagate_chirho(self):
        """Propagate constraints through equality relationships"""
        while True:
            old = self.possible.copy()
            for i in range(3):
                # Intersection of all possible values in equality class
                intersection = VAL_ANY
                for j in range(3):
                    if (self.equals[i] >> j) & 1:
                        intersection &= self.possible[j]
                # Update all in class
                for j in range(3):
                    if (self.equals[i] >> j) & 1:
                        self.possible[j] = intersection
            if self.possible == old:
                break
        return self
    
    def conde_chirho(self, *branches):
        """Disjunction - OR of multiple goal functions"""
        results = []
        for branch in branches:
            s = branch(self.copy())
            if not s.failed_chirho():
                results.append(s)
        return results
    
    def display_chirho(self):
        names = ['x', 'y', 'z']
        def vals(v):
            s = ''
            if v & VAL_A: s += 'a'
            if v & VAL_B: s += 'b'
            if v & VAL_C: s += 'c'
            return s or '∅'
        
        status = "(FAILED)" if self.failed_chirho() else ""
        print(f"State {status}:")
        for i in range(3):
            eq_str = ""
            if bin(self.equals[i]).count('1') > 1:
                eq_vars = [names[j] for j in range(3) if (self.equals[i] >> j) & 1 and j != i]
                eq_str = f"  [≡ {' '.join(eq_vars)}]"
            print(f"  {names[i]} ∈ {{{vals(self.possible[i])}}}{eq_str}")
    
    def matrix_view_chirho(self):
        """Show state as actual bit matrix"""
        print("       a b c")
        names = ['x', 'y', 'z']
        for i, name in enumerate(names):
            bits = ' '.join(str((self.possible[i] >> b) & 1) for b in range(3))
            print(f"    {name} [{bits}]")


def main():
    print("=== 1-Bit Matrix Unification ☧ ===\n")
    
    # Example 1: Simple unification
    print("--- Example 1: x = a ---")
    s1 = StateChirho().unify_value_chirho(VAR_X, VAL_A)
    s1.display_chirho()
    
    # Example 2: Variable equality
    print("\n--- Example 2: x = y, then x = a ---")
    s2 = (StateChirho()
          .unify_vars_chirho(VAR_X, VAR_Y)
          .unify_value_chirho(VAR_X, VAL_A))
    s2.display_chirho()
    print("  ^ y is also 'a' because x ≡ y!")
    
    # Example 3: Transitive equality
    print("\n--- Example 3: x = y, y = z, z = b ---")
    s3 = (StateChirho()
          .unify_vars_chirho(VAR_X, VAR_Y)
          .unify_vars_chirho(VAR_Y, VAR_Z)
          .unify_value_chirho(VAR_Z, VAL_B))
    s3.display_chirho()
    
    # Example 4: Failure
    print("\n--- Example 4: x = a, x = b (contradiction) ---")
    s4 = (StateChirho()
          .unify_value_chirho(VAR_X, VAL_A)
          .unify_value_chirho(VAR_X, VAL_B))
    s4.display_chirho()
    
    # Example 5: conde (disjunction) - THE KEY PART
    print("\n--- Example 5: conde(x=a, x=b) ---")
    print("This is where we get MULTIPLE states (branching):")
    s5 = StateChirho()
    branches = s5.conde_chirho(
        lambda s: s.unify_value_chirho(VAR_X, VAL_A),
        lambda s: s.unify_value_chirho(VAR_X, VAL_B),
    )
    for i, state in enumerate(branches):
        print(f"Branch {i}:")
        state.display_chirho()
    
    # Example 6: Combined
    print("\n--- Example 6: x=y, conde(y=a, y=b), z=c ---")
    s6 = StateChirho().unify_vars_chirho(VAR_X, VAR_Y)
    branches6 = s6.conde_chirho(
        lambda s: s.unify_value_chirho(VAR_Y, VAL_A),
        lambda s: s.unify_value_chirho(VAR_Y, VAL_B),
    )
    for i, state in enumerate(branches6):
        final = state.unify_value_chirho(VAR_Z, VAL_C)
        print(f"Branch {i}:")
        final.display_chirho()
    
    # Matrix view
    print("\n=== Matrix View ===")
    print("State as bit matrix (branch 0):")
    branches6[0].unify_value_chirho(VAR_Z, VAL_C).matrix_view_chirho()
    
    print("\n=== The Key Insight ===")
    print("""
    Unification = AND (bitwise)
    Disjunction = multiple states (parallel worlds)
    Propagation = fixed-point iteration
    
    For GPU: states could be rows in a batch
    Each 'conde' multiplies the batch size
    Constraint propagation = matrix ops on the batch
    
    Next steps:
    1. Pack multiple states into single tensor
    2. Parallel propagation across all states
    3. Prune failed states (rows of zeros)
    """)


if __name__ == "__main__":
    main()
