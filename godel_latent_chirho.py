#!/usr/bin/env python3
"""
Gödelian Latent Patterns for miniKanren ☧

Idea: Encode patterns (terms with holes) as vectors/matrices.

Traditional Gödel numbering:
    encode(cons(a, cons(b, nil))) = 2^a * 3^b * 5^0 = 2^a * 3^b

But we want PATTERNS (terms with holes):
    [?, 1, ?]  = "list of length 3, second element is 1"

Latent representation:
    Instead of a single number, use a VECTOR that encodes:
    - What positions are known
    - What values are at known positions
    - Structural constraints

This is essentially: patterns as points in a structured space
Unification = intersection of constraint regions
"""

from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional, Set, Any
import math


# === Gödel-style Prime Encoding ===

PRIMES_CHIRHO = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]


def godel_encode_list_chirho(lst_chirho: List[int], max_val_chirho: int = 10) -> int:
    """
    Encode a list as a Gödel number.

    encode([a, b, c]) = 2^(a+1) * 3^(b+1) * 5^(c+1)

    +1 to handle zeros (since p^0 = 1)
    """
    if not lst_chirho:
        return 1  # Empty list = 1

    result_chirho = 1
    for i_chirho, val_chirho in enumerate(lst_chirho):
        if i_chirho >= len(PRIMES_CHIRHO):
            raise ValueError(f"List too long: {len(lst_chirho)}")
        result_chirho *= PRIMES_CHIRHO[i_chirho] ** (val_chirho + 1)

    return result_chirho


def godel_decode_list_chirho(n_chirho: int, length_chirho: int) -> List[int]:
    """Decode a Gödel number back to a list"""
    result_chirho = []
    for i_chirho in range(length_chirho):
        p_chirho = PRIMES_CHIRHO[i_chirho]
        exp_chirho = 0
        while n_chirho % p_chirho == 0:
            n_chirho //= p_chirho
            exp_chirho += 1
        result_chirho.append(exp_chirho - 1)  # -1 to undo the +1
    return result_chirho


# === Pattern Representation ===

@dataclass
class PatternChirho:
    """
    A pattern is a partial specification of a list.

    Known positions have concrete values.
    Unknown positions (holes) are None.

    Example: [?, 1, ?] = PatternChirho([None, 1, None])
    """
    elements_chirho: List[Optional[int]]

    def __repr__(self_chirho):
        parts_chirho = []
        for e_chirho in self_chirho.elements_chirho:
            if e_chirho is None:
                parts_chirho.append("?")
            else:
                parts_chirho.append(str(e_chirho))
        return "[" + ", ".join(parts_chirho) + "]"

    @property
    def length_chirho(self_chirho) -> int:
        return len(self_chirho.elements_chirho)

    def is_ground_chirho(self_chirho) -> bool:
        return all(e_chirho is not None for e_chirho in self_chirho.elements_chirho)

    def matches_chirho(self_chirho, lst_chirho: List[int]) -> bool:
        """Check if a concrete list matches this pattern"""
        if len(lst_chirho) != self_chirho.length_chirho:
            return False
        for p_chirho, v_chirho in zip(self_chirho.elements_chirho, lst_chirho):
            if p_chirho is not None and p_chirho != v_chirho:
                return False
        return True


# === Latent Vector Representation ===

class LatentPatternChirho:
    """
    Encode a pattern as a latent vector.

    For a pattern of max length L with values in [0, V):
    - Vector has L * (V + 1) dimensions
    - Position (i, v) encodes "position i has value v"
    - Position (i, V) encodes "position i is unknown"

    Actually, simpler approach:
    - Vector of length L
    - Entry i = value at position i, or -1 for unknown
    - Plus a "mask" vector: 1 if known, 0 if unknown

    This is essentially one-hot encoding per position.
    """

    def __init__(self_chirho, length_chirho: int, max_val_chirho: int = 10):
        self_chirho.length_chirho = length_chirho
        self_chirho.max_val_chirho = max_val_chirho

        # values[i] = value at position i (or 0 if unknown)
        self_chirho.values_chirho: List[int] = [0] * length_chirho

        # mask[i] = 1 if position i is known, 0 if unknown
        self_chirho.mask_chirho: List[int] = [0] * length_chirho

    @classmethod
    def from_pattern_chirho(cls_chirho, pattern_chirho: PatternChirho, max_val_chirho: int = 10) -> 'LatentPatternChirho':
        """Create latent representation from pattern"""
        lp_chirho = cls_chirho(pattern_chirho.length_chirho, max_val_chirho)
        for i_chirho, elem_chirho in enumerate(pattern_chirho.elements_chirho):
            if elem_chirho is not None:
                lp_chirho.values_chirho[i_chirho] = elem_chirho
                lp_chirho.mask_chirho[i_chirho] = 1
        return lp_chirho

    @classmethod
    def from_list_chirho(cls_chirho, lst_chirho: List[int], max_val_chirho: int = 10) -> 'LatentPatternChirho':
        """Create latent representation from ground list"""
        lp_chirho = cls_chirho(len(lst_chirho), max_val_chirho)
        for i_chirho, val_chirho in enumerate(lst_chirho):
            lp_chirho.values_chirho[i_chirho] = val_chirho
            lp_chirho.mask_chirho[i_chirho] = 1
        return lp_chirho

    def to_pattern_chirho(self_chirho) -> PatternChirho:
        """Convert back to pattern"""
        elements_chirho = []
        for i_chirho in range(self_chirho.length_chirho):
            if self_chirho.mask_chirho[i_chirho]:
                elements_chirho.append(self_chirho.values_chirho[i_chirho])
            else:
                elements_chirho.append(None)
        return PatternChirho(elements_chirho)

    def unify_chirho(self_chirho, other_chirho: 'LatentPatternChirho') -> Optional['LatentPatternChirho']:
        """
        Unify two latent patterns.

        Result has:
        - Known positions from either pattern
        - Fails if same position has different known values

        This is the KEY operation!
        """
        if self_chirho.length_chirho != other_chirho.length_chirho:
            return None

        result_chirho = LatentPatternChirho(self_chirho.length_chirho, self_chirho.max_val_chirho)

        for i_chirho in range(self_chirho.length_chirho):
            self_known_chirho = self_chirho.mask_chirho[i_chirho]
            other_known_chirho = other_chirho.mask_chirho[i_chirho]

            if self_known_chirho and other_known_chirho:
                # Both known - must match
                if self_chirho.values_chirho[i_chirho] != other_chirho.values_chirho[i_chirho]:
                    return None  # Conflict!
                result_chirho.values_chirho[i_chirho] = self_chirho.values_chirho[i_chirho]
                result_chirho.mask_chirho[i_chirho] = 1
            elif self_known_chirho:
                result_chirho.values_chirho[i_chirho] = self_chirho.values_chirho[i_chirho]
                result_chirho.mask_chirho[i_chirho] = 1
            elif other_known_chirho:
                result_chirho.values_chirho[i_chirho] = other_chirho.values_chirho[i_chirho]
                result_chirho.mask_chirho[i_chirho] = 1
            # else: both unknown, result is unknown

        return result_chirho

    def to_one_hot_chirho(self_chirho) -> List[List[int]]:
        """
        Convert to one-hot matrix representation.

        Matrix[i][v] = 1 if position i could have value v
        For known positions: only one 1
        For unknown positions: all 1s
        """
        matrix_chirho = []
        for i_chirho in range(self_chirho.length_chirho):
            row_chirho = [0] * self_chirho.max_val_chirho
            if self_chirho.mask_chirho[i_chirho]:
                # Known: only this value
                row_chirho[self_chirho.values_chirho[i_chirho]] = 1
            else:
                # Unknown: any value possible
                row_chirho = [1] * self_chirho.max_val_chirho
            matrix_chirho.append(row_chirho)
        return matrix_chirho

    def __repr__(self_chirho):
        return f"Latent({self_chirho.to_pattern_chirho()})"


# === Latent Matrix for Pattern Sets ===

class PatternMatrixChirho:
    """
    Represent a SET of patterns as a matrix.

    Each row is a latent pattern vector.
    Rows can be combined (OR) or filtered (AND with constraint).

    This is the "latent matrix" idea!
    """

    def __init__(self_chirho, length_chirho: int, max_val_chirho: int = 10):
        self_chirho.length_chirho = length_chirho
        self_chirho.max_val_chirho = max_val_chirho
        self_chirho.patterns_chirho: List[LatentPatternChirho] = []

    def add_pattern_chirho(self_chirho, pattern_chirho: LatentPatternChirho):
        self_chirho.patterns_chirho.append(pattern_chirho)

    def add_list_chirho(self_chirho, lst_chirho: List[int]):
        self_chirho.patterns_chirho.append(
            LatentPatternChirho.from_list_chirho(lst_chirho, self_chirho.max_val_chirho)
        )

    def unify_with_chirho(self_chirho, constraint_chirho: LatentPatternChirho) -> 'PatternMatrixChirho':
        """
        Unify all patterns with a constraint.
        Returns new matrix with only compatible patterns.

        This is FILTERING: like tensor slicing!
        """
        result_chirho = PatternMatrixChirho(self_chirho.length_chirho, self_chirho.max_val_chirho)

        for pattern_chirho in self_chirho.patterns_chirho:
            unified_chirho = pattern_chirho.unify_chirho(constraint_chirho)
            if unified_chirho is not None:
                result_chirho.add_pattern_chirho(unified_chirho)

        return result_chirho

    def to_bit_matrix_chirho(self_chirho) -> List[List[int]]:
        """
        Convert to bit matrix where each row is a pattern
        and columns encode position-value pairs.

        Column index = position * max_val + value
        Entry = 1 if that position has that value in the pattern

        This IS the 1-bit representation!
        """
        num_cols_chirho = self_chirho.length_chirho * self_chirho.max_val_chirho
        matrix_chirho = []

        for pattern_chirho in self_chirho.patterns_chirho:
            row_chirho = [0] * num_cols_chirho
            for pos_chirho in range(self_chirho.length_chirho):
                if pattern_chirho.mask_chirho[pos_chirho]:
                    col_chirho = pos_chirho * self_chirho.max_val_chirho + pattern_chirho.values_chirho[pos_chirho]
                    row_chirho[col_chirho] = 1
            matrix_chirho.append(row_chirho)

        return matrix_chirho

    def display_chirho(self_chirho):
        print(f"PatternMatrix ({len(self_chirho.patterns_chirho)} patterns):")
        for i_chirho, p_chirho in enumerate(self_chirho.patterns_chirho):
            print(f"  {i_chirho}: {p_chirho.to_pattern_chirho()}")


# === Demo ===

def main():
    print("=== Gödelian Latent Patterns ☧ ===\n")

    # === Gödel encoding demo ===
    print("="*60)
    print("GÖDEL ENCODING")
    print("="*60)

    test_lists_chirho = [[0], [1], [0, 1], [1, 0], [0, 1, 2]]

    print("\nEncoding lists as Gödel numbers:")
    for lst_chirho in test_lists_chirho:
        g_chirho = godel_encode_list_chirho(lst_chirho)
        decoded_chirho = godel_decode_list_chirho(g_chirho, len(lst_chirho))
        print(f"  {lst_chirho} → {g_chirho} → {decoded_chirho}")

    print("\nNote: Numbers grow VERY fast (exponential in values)")
    print(f"  [5, 5, 5] → {godel_encode_list_chirho([5, 5, 5])}")

    # === Pattern representation ===
    print("\n" + "="*60)
    print("PATTERN REPRESENTATION")
    print("="*60)

    p1_chirho = PatternChirho([None, 1, None])  # [?, 1, ?]
    p2_chirho = PatternChirho([0, None, 2])      # [0, ?, 2]

    print(f"\nPattern 1: {p1_chirho}")
    print(f"Pattern 2: {p2_chirho}")

    lp1_chirho = LatentPatternChirho.from_pattern_chirho(p1_chirho)
    lp2_chirho = LatentPatternChirho.from_pattern_chirho(p2_chirho)

    print(f"\nLatent 1: values={lp1_chirho.values_chirho}, mask={lp1_chirho.mask_chirho}")
    print(f"Latent 2: values={lp2_chirho.values_chirho}, mask={lp2_chirho.mask_chirho}")

    # Unify patterns
    unified_chirho = lp1_chirho.unify_chirho(lp2_chirho)
    if unified_chirho:
        print(f"\nUnified: {unified_chirho.to_pattern_chirho()}")
        print(f"  values={unified_chirho.values_chirho}, mask={unified_chirho.mask_chirho}")

    # === Conflict detection ===
    print("\n" + "="*60)
    print("CONFLICT DETECTION")
    print("="*60)

    p3_chirho = PatternChirho([1, None, None])  # [1, ?, ?]
    p4_chirho = PatternChirho([2, None, None])  # [2, ?, ?]

    lp3_chirho = LatentPatternChirho.from_pattern_chirho(p3_chirho)
    lp4_chirho = LatentPatternChirho.from_pattern_chirho(p4_chirho)

    conflict_chirho = lp3_chirho.unify_chirho(lp4_chirho)
    print(f"\n{p3_chirho} unify {p4_chirho} = {'FAIL' if conflict_chirho is None else conflict_chirho.to_pattern_chirho()}")

    # === Pattern matrix ===
    print("\n" + "="*60)
    print("PATTERN MATRIX (the latent matrix)")
    print("="*60)

    pm_chirho = PatternMatrixChirho(length_chirho=3, max_val_chirho=5)

    # Add some lists
    for lst_chirho in [[0, 1, 2], [1, 1, 1], [0, 2, 1], [2, 1, 0]]:
        pm_chirho.add_list_chirho(lst_chirho)

    print("\nOriginal matrix:")
    pm_chirho.display_chirho()

    # Filter with constraint: second element = 1
    constraint_chirho = LatentPatternChirho.from_pattern_chirho(PatternChirho([None, 1, None]))
    filtered_chirho = pm_chirho.unify_with_chirho(constraint_chirho)

    print(f"\nFiltered by {constraint_chirho.to_pattern_chirho()}:")
    filtered_chirho.display_chirho()

    # === Bit matrix representation ===
    print("\n" + "="*60)
    print("BIT MATRIX (1-bit representation)")
    print("="*60)

    bit_matrix_chirho = filtered_chirho.to_bit_matrix_chirho()
    print(f"\nShape: {len(bit_matrix_chirho)} rows × {len(bit_matrix_chirho[0]) if bit_matrix_chirho else 0} cols")
    print("Columns encode (position, value) pairs")
    print("\nMatrix:")
    for i_chirho, row_chirho in enumerate(bit_matrix_chirho):
        # Show which (pos, val) pairs are set
        pairs_chirho = []
        for j_chirho, bit_chirho in enumerate(row_chirho):
            if bit_chirho:
                pos_chirho = j_chirho // 5
                val_chirho = j_chirho % 5
                pairs_chirho.append(f"({pos_chirho},{val_chirho})")
        print(f"  {i_chirho}: {pairs_chirho}")

    # === One-hot per position ===
    print("\n" + "="*60)
    print("ONE-HOT ENCODING (per position)")
    print("="*60)

    p5_chirho = PatternChirho([0, None, 2])
    lp5_chirho = LatentPatternChirho.from_pattern_chirho(p5_chirho, max_val_chirho=5)
    one_hot_chirho = lp5_chirho.to_one_hot_chirho()

    print(f"\nPattern: {p5_chirho}")
    print("One-hot matrix (rows=positions, cols=values):")
    for i_chirho, row_chirho in enumerate(one_hot_chirho):
        print(f"  pos {i_chirho}: {row_chirho}")

    print("\nNote: Unknown positions have all 1s (any value possible)")

    # === The connection ===
    print("\n" + "="*60)
    print("THE LATENT MATRIX INSIGHT")
    print("="*60)
    print("""
    Patterns as vectors/matrices:

    1. Each pattern = point in high-dimensional space
    2. Unknown positions = "any" = spans a hyperplane
    3. Unification = intersection of constraint regions

    Bit matrix view:
    - Row = pattern
    - Column = (position, value) pair
    - Entry = 1 if pattern constrains that position to that value

    Operations:
    - Filter by constraint: keep rows where masked positions match
    - This IS tensor slicing!

    Connection to neural/probabilistic:
    - Could relax to probabilities: P(pos i = val v)
    - Unification becomes Bayesian update
    - This is how Scallop does differentiable Datalog

    The Gödel connection:
    - Classic Gödel: term → single integer (via prime factorization)
    - Latent Gödel: term → vector (via position encoding)
    - Patterns naturally extend to vectors with "don't care" positions

    Is this useful?

    ✓ Natural representation for patterns with holes
    ✓ Unification as vector intersection (parallelizable)
    ✓ Connects to one-hot encoding / sparse matrices
    ✓ Path to differentiable relaxation

    ✗ Doesn't handle structure (nested cons cells)
    ✗ Grows with max value range
    ✗ Not clear how to handle variables across positions

    The real insight:

    PATTERNS ARE CONVEX REGIONS IN (POSITION × VALUE) SPACE

    And unification is intersection of convex regions!
    """)


if __name__ == "__main__":
    main()
