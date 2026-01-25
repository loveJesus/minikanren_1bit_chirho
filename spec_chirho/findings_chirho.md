# Implementation Findings ☧

## Summary

This project contains **two distinct implementations** with different capabilities:

| Aspect | `hardware_chirho` (BitVec64) | `reference_chirho` (Streams) |
|--------|------------------------------|------------------------------|
| What it is | Finite-domain constraint solver | Full miniKanren |
| Domain size | 64 values max | Infinite (lazy) |
| Terms | Integers 0-63 | Nested cons/var/int/sym |
| Unification | Bitwise AND | Structural recursion |
| Speed | 3.7μs (N-Queens 8) | 25μs (6-8× slower) |

---

## Finding 1: Hardware Is Not Full miniKanren

The `hardware_chirho` module with `BitVec64Chirho` is a **finite-domain constraint solver**, not full miniKanren.

### What's Missing

```
Full miniKanren                    BitVec64 Hardware
─────────────────────────────────────────────────────
Nested terms: (cons 1 (cons 2 nil))   ✗ Not supported
Symbolic unification                   ✗ Not supported
Occurs check                           ✗ Not needed (no nesting)
Infinite domains                       ✗ Limited to 64 values
```

### What It Actually Does

```rust
// Variable x can be values 0-63, represented as bits
let domain_x: BitVec64Chirho = BitVec64Chirho(0b1110);  // x ∈ {1,2,3}

// Unification = bitwise AND
domain_x.and_chirho(domain_y)  // Intersection of possible values
```

This is **arc consistency over finite integer domains** - the same algorithm used in:
- Sudoku solvers
- N-Queens
- Graph coloring
- Scheduling problems

### Accuracy of Paper Claims

The paper title "miniKanren as 1-Bit Matrix Operations" is technically **overstated**. More accurate:

> "Finite-Domain Constraint Solving as 1-Bit Matrix Operations"

The benchmarks (N-Queens, Sudoku) are valid because they fit in 64 values. The 70,000× speedup vs Z3 is real for these problems.

---

## Finding 2: Reference IS Full miniKanren

The `reference_chirho` module is a proper miniKanren implementation:

```rust
pub enum TermChirho {
    VarChirho(VarIdChirho),                    // Logic variables
    IntChirho(i64),                             // Integers
    NilChirho,                                  // Empty list
    ConsChirho(TermIdChirho, TermIdChirho),    // Nested cons cells
    SymChirho(u32),                             // Symbols
}
```

### Capabilities

- **Nested structure**: `(cons (cons 1 2) (cons 3 nil))`
- **Symbolic unification**: `(cons ?x ?y)` matches `(cons 1 2)` → x=1, y=2
- **Infinite domains**: Variables can be any term
- **Lazy streams**: Fair interleaving for infinite result sets
- **Full operators**: `==`, `conde`, `fresh`, `not`, `conda`, `condu`, `=/=`, `project`

---

## Finding 3: Hash Consing Is Unusual for miniKanren

Our reference uses **hash consing** (term interning), which is NOT standard:

| Implementation | Term Representation | Equality Check |
|----------------|---------------------|----------------|
| Standard miniKanren | Direct cons cells | Structural walk O(n) |
| µKanren | Direct cons cells | Structural walk O(n) |
| **Our reference** | Hash-consed to IDs | Integer compare O(1) |
| E-graphs (egg) | Hash-consed | Integer compare O(1) |

### Why This Matters

Hash consing is the **bridge** between traditional miniKanren and bit-vector operations:

```
Standard miniKanren → Hash-consed miniKanren → Bit-vector domains
     (trees)              (integer IDs)           (bitmasks)
```

Once terms are integers, they could index into bit vectors. This is the path to hardware.

### Who Else Hash-Conses

- E-graphs (egg library) - yes
- BDDs (Binary Decision Diagrams) - yes
- Some Prolog systems - yes
- Most miniKanren implementations - **no**

---

## Finding 4: The Bridge Is Incomplete

The current implementation has two separate systems:

```
┌─────────────────────┐      ┌─────────────────────┐
│   reference_chirho  │      │   hardware_chirho   │
│   (full miniKanren) │      │  (finite domains)   │
│                     │      │                     │
│   TermChirho        │  ?   │   BitVec64Chirho    │
│   StreamChirho      │ ──── │   SearchStateHwChirho│
│   SubstChirho       │      │   GoalHwChirho      │
└─────────────────────┘      └─────────────────────┘
```

The "?" connection is missing. To truly unify them:

1. **Finite terms** (ground, no variables) could be compiled to bit vectors
2. **Tabling** could cache relation results as sparse tensors
3. **Mode analysis** could determine when hardware path is safe

---

## Finding 5: Benchmarks Are Valid But Scoped

The benchmark claims are accurate **within their scope**:

| Claim | Valid For | Not Valid For |
|-------|-----------|---------------|
| 70,000× vs Z3 | N-Queens (integers 0-N) | Symbolic unification |
| 4000× vs heap | Bit operations | Nested term manipulation |
| 6-8× vs streams | Finite domain goals | Infinite domain search |

---

## Recommendations

### Short Term
1. Rename paper section: "Hardware-Accelerated Finite-Domain Constraints"
2. Document the 64-value limitation prominently
3. Add `BitVec256Chirho` for problems needing 256 values

### Medium Term
1. Build the bridge: compile finite-domain subsets of miniKanren to hardware
2. Mode inference: automatically detect when hardware path is safe
3. Hybrid execution: reference for structure, hardware for propagation

### Long Term
1. Extend bit representation to handle simple cons structures
2. CAM-based hash consing in hardware (FPGA)
3. GPU backend for massive parallel domain propagation

---

## Conclusion

The project demonstrates that **finite-domain constraint solving** maps beautifully to 1-bit operations with massive speedups. The claim that **full miniKanren** maps to 1-bit operations is aspirational - the bridge exists conceptually but not yet in implementation.

The reference implementation is a solid, hash-consed miniKanren. The hardware implementation is a fast finite-domain solver. The connection between them is the interesting research direction.

*Soli Deo Gloria* ☧
