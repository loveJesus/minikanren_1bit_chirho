# Proof Notes: miniKanren as 1-Bit Matrix Operations ☧

## Overview

This document provides informal proof sketches for the theorems in our formalization.
The goal is to establish:

> **For finite domains, miniKanren search = sparse Boolean tensor network contraction**

---

## Theorem 1: Intersection Soundness (UnifySoundChirho)

**Statement:**
```
∀ d₁ d₂ : Domain, ∀ v : Value,
  v ∈ (d₁ ∧ d₂) → (v ∈ d₁ ∧ v ∈ d₂)
```

**Proof Sketch:**

1. Membership in a domain is defined as: `v ∈ d ↔ testBit(d, v) = 1`
2. By definition: `(d₁ ∧ d₂)[v] = d₁[v] AND d₂[v]`
3. If `(d₁ ∧ d₂)[v] = 1`, then both `d₁[v] = 1` and `d₂[v] = 1`
4. Therefore `v ∈ d₁` and `v ∈ d₂` ∎

**Coq:** Direct application of `andb_true_iff`.

---

## Theorem 2: Intersection Completeness (UnifyCompleteChirho)

**Statement:**
```
∀ d₁ d₂ : Domain, ∀ v : Value,
  (v ∈ d₁ ∧ v ∈ d₂) → v ∈ (d₁ ∧ d₂)
```

**Proof Sketch:**

1. Given `v ∈ d₁` means `d₁[v] = 1`
2. Given `v ∈ d₂` means `d₂[v] = 1`
3. By definition: `(d₁ ∧ d₂)[v] = 1 AND 1 = 1`
4. Therefore `v ∈ (d₁ ∧ d₂)` ∎

**Combined with Theorem 1:**
```
v ∈ (d₁ ∧ d₂) ↔ (v ∈ d₁ ∧ v ∈ d₂)
```

This establishes that bitwise AND exactly computes set intersection.

---

## Theorem 3: Contraction Order Independence (ContractionCommutesChirho)

**Statement:**
```
∀ T₁ T₂ T₃ : Tensor, ∀ x y : Index,
  (x ∉ freeIndices(T₃)) ∧ (y ∉ freeIndices(T₁)) →
  contract(contract(T₁, T₂, x), T₃, y) = contract(T₁, contract(T₂, T₃, y), x)
```

**Proof Sketch:**

This follows from the associativity of the Boolean semiring:

1. Tensor contraction is defined as:
   ```
   (T₁ ⊗ T₂)[i,k] = Σⱼ T₁[i,j] × T₂[j,k]
   ```
   where Σ is OR and × is AND in the Boolean semiring.

2. For three tensors:
   ```
   ((T₁ ⊗ T₂) ⊗ T₃)[i,l] = Σₖ (Σⱼ T₁[i,j] × T₂[j,k]) × T₃[k,l]
   ```

3. By distributivity of AND over OR:
   ```
   = Σₖ Σⱼ (T₁[i,j] × T₂[j,k] × T₃[k,l])
   ```

4. By commutativity and associativity:
   ```
   = Σⱼ Σₖ (T₁[i,j] × T₂[j,k] × T₃[k,l])
   = Σⱼ T₁[i,j] × (Σₖ T₂[j,k] × T₃[k,l])
   = (T₁ ⊗ (T₂ ⊗ T₃))[i,l]
   ```

5. The index condition ensures we're not contracting the same index twice. ∎

**Connection to Category Theory:**

This is the coherence theorem for monoidal categories. String diagrams
commute when the "wiring" is topologically equivalent.

---

## Theorem 4: Hierarchical Domain Equivalence (HierEquivChirho)

**Statement:**
```
∀ h : Hierarchical4k,
  toSet(flatten(h)) = toSet(h)
```

**Proof Sketch:**

For a 2-level hierarchy (4k = 64 × 64):

1. Define membership:
   ```
   v ∈ h ↔ h.root[v/64] = 1 ∧ h.leaves[v/64][v%64] = 1
   ```

2. Define flattening:
   ```
   flatten(h)[v] = h.root[v/64] AND h.leaves[v/64][v%64]
   ```

3. By construction: `v ∈ flatten(h) ↔ flatten(h)[v] = 1`

4. Expanding: `flatten(h)[v] = 1 ↔ h.root[v/64] = 1 ∧ h.leaves[v/64][v%64] = 1`

5. Therefore: `v ∈ flatten(h) ↔ v ∈ h` ∎

**Key Insight for Efficiency:**

When intersecting two hierarchical domains:
```
(h₁ ∩ h₂).root = h₁.root AND h₂.root
```

If a bit in `(h₁ ∩ h₂).root` is 0, we skip the corresponding leaves entirely.
For sparse domains (typical in constraint solving), this provides exponential speedup.

---

## Theorem 5: Search Equivalence (SearchEquivChirho) ★ MAIN

**Statement:**
```
∀ g : Goal, ∀ s : State,
  (all domains in s are finite) →
  solutions(run_minikanren(g, s)) = solutions(contract(compile(g), encode(s)))
```

**Proof Strategy:**

By structural induction on goals:

### Base Case: Succeed
```
[[succeed]](s) = {s}
T[[succeed]] = identity tensor
```
Both produce exactly the input state.

### Base Case: Fail
```
[[fail]](s) = ∅
T[[fail]] = empty tensor
```
Both produce no solutions.

### Case: Unify(x, y)
```
[[x == y]](s) = if s.domain(x) ∩ s.domain(y) = ∅ then ∅
                else {s[x ↦ s.domain(x) ∩ s.domain(y),
                       y ↦ s.domain(x) ∩ s.domain(y)]}

T[[x == y]] = {(v, v) | v ∈ domain_x ∩ domain_y}
```

By Theorems 1-2, these are equivalent:
- Domain semantics produces states where x,y have intersected domains
- Tensor has entries exactly for values in the intersection

### Case: Conj(g₁, g₂)
```
[[g₁ ∧ g₂]](s) = ⋃_{s' ∈ [[g₁]](s)} [[g₂]](s')

T[[g₁ ∧ g₂]] = T[[g₁]] ⊗ T[[g₂]]  (contracted on shared variables)
```

By induction hypothesis on g₁ and g₂:
- [[g₁]] ≡ T[[g₁]]
- [[g₂]] ≡ T[[g₂]]

Tensor contraction corresponds to sequential constraint application.

### Case: Disj(g₁, g₂)
```
[[g₁ ∨ g₂]](s) = [[g₁]](s) ∪ [[g₂]](s)

T[[g₁ ∨ g₂]] = T[[g₁]] ⊕ T[[g₂]]  (tensor sum/stacking)
```

Union of solution sets = disjoint union of tensor entries.

### Case: Fresh(x, g)
```
[[fresh x. g]](s) = [[g]](s[x ↦ full_domain])

T[[fresh x. g]] = project_x(T[[g]])  (existential projection)
```

Fresh variable gets full domain; projection removes the index from tensor.

### Case: Call(rel, args)

This requires **tabling**:
- Pre-compute T[[rel]] as a sparse tensor
- Store in a table indexed by relation name
- Lookup and contract at call site

The key insight is that tabled relations are **static** tensors that can be
precomputed and stored. ∎

---

## Theorem 6: Semiring Generalization (SemiringPreservesChirho)

**Statement:**
```
∀ S : CommutativeSemiring, ∀ g : Goal,
  run_weighted(S, g) = contract_semiring(S, compile(g))
```

**Proof Sketch:**

The proof of Theorem 5 only uses:
- Associativity and commutativity of ⊕ (OR generalized)
- Associativity of ⊗ (AND generalized)
- Distributivity: a ⊗ (b ⊕ c) = (a ⊗ b) ⊕ (a ⊗ c)
- Identity: 1 ⊗ a = a, 0 ⊕ a = a
- Absorption: 0 ⊗ a = 0

These are exactly the semiring axioms. Therefore the equivalence holds
for any semiring, not just Boolean. ∎

**Applications:**

| Semiring | ⊕ | ⊗ | Application |
|----------|---|---|-------------|
| Boolean | OR | AND | Standard search |
| ℕ | + | × | Count solutions |
| (ℝ⁺, max, ×) | max | × | Viterbi / MAP |
| (ℝ, +, ×) | + | × | Probabilistic / differentiable |
| (ℝ⁺ ∪ {∞}, min, +) | min | + | Shortest path / tropical |

---

## Open Questions

1. **Infinite Domains:** Can we extend the equivalence to lazy/symbolic domains?
   - Hypothesis: Yes, via demand-driven tensor construction
   - Key: Hash-consing creates terms on-demand

2. **Negation:** How does `=/=` (disequality) map to tensors?
   - Hypothesis: Complement operation, requires "full" tensor

3. **Occurs Check:** How does cycle detection map to tensors?
   - Hypothesis: Path-based constraints (see `nested_latent_chirho.py`)

4. **Optimal Contraction Order:** NP-hard in general
   - Practice: Greedy heuristics work well for typical queries
   - Theory: Connected to treewidth of query hypergraph

---

## References

1. Byrd et al., "Relational Programming in miniKanren" (2010)
2. Willsey et al., "egg: Fast and Extensible Equality Saturation" (2021)
3. Huang et al., "Scallop: From Probabilistic Deductive Databases..." (2021)
4. Selinger, "A Survey of Graphical Languages for Monoidal Categories" (2010)
5. Orús, "A practical introduction to tensor networks" (2014)

---

*Soli Deo Gloria* ☧
