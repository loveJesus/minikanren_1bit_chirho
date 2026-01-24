# Theoretical Foundations ☧

## 1. The Domain-Bitmask Correspondence

### Finding 1.1: Variables as Bitvectors

A logic variable's domain (set of possible values) maps directly to a bitmask:

```
Domain {0, 2, 5} ↔ Bitmask 0b100101
         ↑              ↑↑  ↑
       values        bit positions
```

**Theorem:** For a finite domain D ⊆ {0, ..., n-1}, the characteristic function 
χ_D : {0,...,n-1} → {0,1} is exactly a bitmask representation.

### Finding 1.2: Unification as Bitwise AND

Unifying two variables computes their domain intersection:

```
x ∈ {0,2,5}  →  0b100101
y ∈ {2,3,5}  →  0b101100
─────────────────────────
x = y        →  0b100100  =  {2,5}
```

**Theorem:** For domain bitmasks D₁, D₂:
```
unify(D₁, D₂) = D₁ ∧ D₂  (bitwise AND)
```

This is O(1) for fixed-width bitvectors, parallelizable across all bits.

### Finding 1.3: Failure Detection

Unification fails iff the result domain is empty:

```
failed(D) ⟺ D = 0  (all bits zero)
```

Hardware: Single zero-detect comparator.

## 2. Relations as Sparse Tensors

### Finding 2.1: Binary Relations

A binary relation R ⊆ A × B maps to a Boolean matrix M where:
```
M[i,j] = 1 ⟺ (i,j) ∈ R
```

Example: "less-than" on {0,1,2}:
```
    0 1 2
  ┌───────┐
0 │ 0 1 1 │  0 < 1, 0 < 2
1 │ 0 0 1 │  1 < 2
2 │ 0 0 0 │  
  └───────┘
```

### Finding 2.2: N-ary Relations as Tensors

An N-ary relation R ⊆ A₁ × ... × Aₙ maps to an N-dimensional Boolean tensor:

```
appendo(L, S, Out) → 3D tensor T where
T[l,s,o] = 1 ⟺ L ++ S = Out
```

For lists up to length 3 over alphabet {a,b}:
- Dimension: ~100 × 100 × 100 = 10⁶ entries
- Sparsity: Most entries are 0 (lists don't concatenate arbitrarily)
- Storage: COO format (list of (l,s,o) triples)

### Finding 2.3: Constraint Propagation via Tensor Slicing

Given partial knowledge (e.g., L = [a]), extract valid (S, Out) pairs:

```python
valid_pairs = T[hash([a]), :, :]  # 2D slice
```

This is a single memory access pattern, parallelizable.

## 3. Goal Composition as Tensor Contraction

### Finding 3.1: Conjunction is Tensor Product + Contraction

```
(== x y) ∧ (== y z)
```

Corresponds to:
1. Take identity matrix I for (== x y): I[x,y] = 1 iff x = y
2. Take identity matrix I for (== y z): I[y,z] = 1 iff y = z
3. Contract on shared variable y: R[x,z] = ∨_y (I[x,y] ∧ I[y,z])

Result: R[x,z] = 1 iff x = z (transitivity!)

### Finding 3.2: The (OR, AND) Semiring

Tensor contraction uses the Boolean semiring:
- Addition: OR (∨)
- Multiplication: AND (∧)
- Zero: false
- One: true

**Theorem:** Goal composition under (∨, ∧) semiring exactly computes 
the relational semantics of miniKanren.

### Finding 3.3: Connection to Quantum Tensor Networks

The structure is isomorphic to tensor network contraction in quantum computing:
- Variables ↔ tensor indices
- Goals ↔ tensors
- Conjunction ↔ contraction
- Disjunction ↔ tensor direct sum

Contraction order optimization is NP-hard in both domains (same problem!).

## 4. Occurs Check via Transitive Closure

### Finding 4.1: Substitution as Directed Graph

A substitution {x → y, y → z, z → cons(a, x)} forms a directed graph:
```
x → y → z → cons(a, x)
          ↑_________|
```

The occurs check detects cycles in this graph.

### Finding 4.2: Boolean Matrix Reachability

Represent substitution as adjacency matrix A where A[i,j] = 1 iff var i 
depends on var j. Then:

```
Transitive closure: A* = I ∨ A ∨ A² ∨ A³ ∨ ...
```

**Theorem:** Variable i occurs in term t iff A*[i,j] = 1 for some j in t's variables.

### Finding 4.3: Repeated Squaring

Compute A* in O(log n) matrix multiplications:
```
A* = (I ∨ A ∨ A² ∨ ... ∨ A^(2^k)) for k = ⌈log n⌉
```

Each multiplication is O(n³) naively, but for Boolean matrices:
- Strassen: O(n^2.807)
- Hardware: O(1) with sufficient parallelism

## 5. Hash Consing and Infinite Domains

### Finding 5.1: Demand-Driven Term Creation

Hash consing enables "infinite" domains by creating terms on demand:
```
Term 0: nil
Term 1: cons(a, 0) = [a]
Term 2: cons(b, 0) = [b]
Term 3: cons(a, 1) = [a, a]
...
```

### Finding 5.2: Structural Sharing

Hash consing ensures `cons(a, cons(b, nil))` created twice yields same ID:
```
intern(cons(a, intern(cons(b, intern(nil)))))
     = intern(cons(a, 2))  // assuming cons(b,nil) → 2
     = 5                    // same ID each time
```

### Finding 5.3: Hardware Hash Consing

Requires:
1. Term memory (RAM): stores (tag, child1, child2)
2. CAM: parallel lookup for deduplication
3. Free pointer: next available ID

Estimated cost: ~1000 additional LUTs for 256-entry term store.

## 6. Semiring Generalization

### Finding 6.1: Beyond Boolean

The (OR, AND) semiring can be replaced with:

| Semiring | ⊕ | ⊗ | Use Case |
|----------|---|---|----------|
| Boolean | ∨ | ∧ | Standard logic |
| Counting | + | × | Count solutions |
| Probability | + | × | Weighted inference |
| Tropical | min | + | Shortest path |
| Viterbi | max | × | Most likely |

### Finding 6.2: Differentiable Logic

Using probability semiring with gradient tracking enables:
- Learning relation weights from examples
- Differentiable program synthesis
- Neuro-symbolic integration

This connects to Scallop (differentiable Datalog) and DeepProbLog.

---

*Soli Deo Gloria* ☧
