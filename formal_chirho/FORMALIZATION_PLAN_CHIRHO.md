# Formalization Plan: miniKanren as 1-Bit Matrix Operations ☧

## Overview

This document specifies the formal verification of the core claim:

> **miniKanren search over finite domains = sparse Boolean tensor network contraction**

We provide both **Lean 4** and **Coq** implementations.

---

## Core Theorems to Formalize

### Theorem 1: Domain Soundness (UnifySoundChirho)

**Statement:** If bitwise AND of two domains produces a non-empty result, then there exists a concrete value satisfying both domain constraints.

```
∀ d₁ d₂ : DomainChirho,
  (d₁ ∧ d₂) ≠ ∅ →
  ∃ v, v ∈ d₁ ∧ v ∈ d₂
```

**Lean 4:**
```lean
theorem unify_sound_chirho (d1 d2 : DomainChirho) :
  (d1 &&& d2) ≠ emptyChirho → ∃ v, v ∈ d1 ∧ v ∈ d2
```

**Coq:**
```coq
Theorem unify_sound_chirho : forall d1 d2 : domain_chirho,
  and_domain_chirho d1 d2 <> empty_chirho ->
  exists v, In v d1 /\ In v d2.
```

---

### Theorem 2: Domain Completeness (UnifyCompleteChirho)

**Statement:** If there exists a value in both domains, then the bitwise AND is non-empty.

```
∀ d₁ d₂ : DomainChirho,
  (∃ v, v ∈ d₁ ∧ v ∈ d₂) →
  (d₁ ∧ d₂) ≠ ∅
```

---

### Theorem 3: Contraction Order Independence (ContractionCommutesChirho)

**Statement:** Tensor contraction is associative - different evaluation orders produce the same result.

```
∀ T₁ T₂ T₃ : TensorChirho,
∀ x y : IndexChirho,
  contract(contract(T₁, T₂, x), T₃, y) = contract(T₁, contract(T₂, T₃, y), x)
  (when x ∉ freeIndices(T₃) and y ∉ freeIndices(T₁))
```

This corresponds to the categorical coherence theorem for monoidal categories.

---

### Theorem 4: Hierarchical Domain Equivalence (HierEquivChirho)

**Statement:** A hierarchical domain (2-level or 3-level) represents the same set of values as its flat expansion.

```
∀ h : Hierarchical4kChirho,
  flatten_chirho(h) = { v | v ∈ h }

-- Where membership is defined by:
v ∈ h ↔ h.root[v / 64] = 1 ∧ h.leaves[v / 64][v % 64] = 1
```

---

### Theorem 5: Search Equivalence (SearchEquivChirho) ★ MAIN THEOREM

**Statement:** For finite domains with tabled relations, miniKanren search produces the same answer set as tensor network contraction.

```
∀ goal : GoalChirho,
∀ domains : VarChirho → DomainChirho,
  (all domains finite) →
  run_minikanren_chirho(goal) = contract_tensor_chirho(compile_chirho(goal))
```

**Proof Strategy:**
1. Define denotational semantics for miniKanren goals
2. Define tensor compilation for each goal constructor
3. Prove by structural induction on goals:
   - `(== x y)` → domain intersection (Theorem 1-2)
   - `(conde g₁ g₂)` → tensor stacking (union)
   - `(fresh x g)` → existential projection
   - Relation call → lookup in tabled tensor

---

### Theorem 6: Semiring Generalization (SemiringPreservesChirho)

**Statement:** The equivalence holds for any commutative semiring, not just Boolean.

```
∀ S : CommutativeSemiring,
∀ goal : GoalChirho,
  run_weighted_chirho(S, goal) = contract_semiring_chirho(S, compile_chirho(goal))
```

**Instances:**
- `S = Bool` → standard search
- `S = ℕ` → counting solutions
- `S = (ℝ⁺, max, ×)` → Viterbi/tropical
- `S = (ℝ, +, ×)` → probabilistic/differentiable

---

## File Structure

```
formal_chirho/
├── FORMALIZATION_PLAN_CHIRHO.md    # This document
├── lean4/
│   ├── MiniKanrenChirho.lean       # Main development
│   ├── DomainChirho.lean           # Bit vector domains
│   ├── TensorChirho.lean           # Sparse tensors
│   ├── SemiringChirho.lean         # Semiring abstraction
│   └── lakefile.lean               # Build configuration
├── coq/
│   ├── MiniKanren_chirho.v         # Main development
│   ├── Domain_chirho.v             # Bit vector domains
│   ├── Tensor_chirho.v             # Sparse tensors
│   ├── Semiring_chirho.v           # Semiring abstraction
│   └── _CoqProject                 # Build configuration
└── docs/
    └── PROOF_NOTES_CHIRHO.md       # Informal proof sketches
```

---

## Dependencies

### Lean 4
- Mathlib4 (for algebra, finiteness)
- std4 (standard library)

### Coq
- Coq 8.18+
- MathComp (for finitypes, algebra)
- coq-record-update (convenience)

---

## Proof Status

| Theorem | Lean 4 | Coq | Notes |
|---------|--------|-----|-------|
| intersect_sound_chirho | ✅ Done | ✅ Done | AND = intersection (soundness) |
| intersect_complete_chirho | ✅ Done | ✅ Done | AND = intersection (completeness) |
| intersect_comm_chirho | ✅ Done | ✅ Done | Commutativity |
| intersect_assoc_chirho | ✅ Done | ✅ Done | Associativity |
| union_comm_chirho | ✅ Done | ✅ Done | Union commutativity |
| union_assoc_chirho | ✅ Done | ✅ Done | Union associativity |
| intersect_distrib_union_chirho | ✅ Done | ✅ Done | Distributivity |
| member_intersect_iff_chirho | ✅ Done | ✅ Done | **Main bridge theorem** |
| **domains_refine_chirho** | — | ✅ Done | Domains refine during search |
| **search_equiv_chirho** | ⬜ TODO | ✅ Done | **Main theorem** (Coq version proven!) |
| unify_intersects_chirho | — | ✅ Done | Unification = intersection |
| unify_tensor_equiv_chirho | — | ✅ Done | Unification matches tensor |
| Semiring class + instances | — | ✅ Done | Bool and Nat semirings |
| hier_intersect_equiv_chirho | — | ⬜ Admitted | Hierarchical = flat (admitted) |

---

## Key Definitions

### Domain (finite set as bitmask)

```lean
-- Lean 4
structure DomainChirho (n : ℕ) where
  bits : BitVec n

def member_chirho (v : Fin n) (d : DomainChirho n) : Bool :=
  d.bits.getLsb v.val

def intersect_chirho (d1 d2 : DomainChirho n) : DomainChirho n :=
  ⟨d1.bits &&& d2.bits⟩

def union_chirho (d1 d2 : DomainChirho n) : DomainChirho n :=
  ⟨d1.bits ||| d2.bits⟩
```

```coq
(* Coq *)
Definition domain_chirho (n : nat) := N.  (* n-bit number *)

Definition member_chirho (v : nat) (d : domain_chirho n) : bool :=
  N.testbit d (N.of_nat v).

Definition intersect_chirho (d1 d2 : domain_chirho n) : domain_chirho n :=
  N.land d1 d2.

Definition union_chirho (d1 d2 : domain_chirho n) : domain_chirho n :=
  N.lor d1 d2.
```

### Goal (miniKanren goal type)

```lean
-- Lean 4
inductive GoalChirho (V : Type) where
  | unify_chirho : V → V → GoalChirho V
  | conde_chirho : List (GoalChirho V) → GoalChirho V
  | conj_chirho : GoalChirho V → GoalChirho V → GoalChirho V
  | fresh_chirho : (V → GoalChirho V) → GoalChirho V
  | call_chirho : String → List V → GoalChirho V
```

### Tensor (sparse Boolean tensor in COO format)

```lean
-- Lean 4
structure TensorChirho where
  indices : List (List ℕ)  -- List of index tuples
  arity : ℕ                 -- Number of dimensions

def contract_chirho (t1 t2 : TensorChirho) (shared : ℕ) : TensorChirho :=
  -- Contract over shared index using Boolean semiring (AND for ×, OR for +)
  sorry
```

---

## References

1. Byrd et al., "Relational Programming in miniKanren" (2010)
2. Willsey et al., "egg: Fast and Extensible Equality Saturation" (2021)
3. Huang et al., "Scallop: From Probabilistic Deductive Databases to Scalable Differentiable Reasoning" (2021)
4. Selinger, "A Survey of Graphical Languages for Monoidal Categories" (2010)
5. Marlow & Peyton Jones, "Making a Fast Curry" (2006) - for tabling

---

## Connection to Implementation

The formalization mirrors the Rust/Python implementation:

| Formal Concept | Implementation |
|----------------|----------------|
| `DomainChirho` | `BitVec64Chirho`, `Hierarchical4kChirho` |
| `intersect_chirho` | `and_chirho()` method |
| `TensorChirho` | `SparseRelationChirho` (COO) |
| `contract_chirho` | `propagate_chirho()` in `constraint_prop_chirho.py` |
| `GoalChirho` | `GoalHwChirho` in `rust_chirho/src/hardware_chirho.rs` |

---

*Soli Deo Gloria* ☧
