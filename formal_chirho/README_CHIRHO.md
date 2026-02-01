# Formal Verification: miniKanren as 1-Bit Matrix Operations ☧

## Quick Start

### Lean 4

```bash
cd lean4
lake build
```

### Coq

```bash
cd coq
coq_makefile -f _CoqProject -o Makefile
make
```

---

## Structure

```
formal_chirho/
├── FORMALIZATION_PLAN_CHIRHO.md    # Theorem statements and roadmap
├── README_CHIRHO.md                 # This file
├── lean4/
│   ├── DomainChirho.lean           # Bit vector domains (Theorems 1-2)
│   ├── MiniKanrenChirho.lean       # Goals, semantics, main theorem
│   └── lakefile.lean               # Build configuration
├── coq/
│   ├── Domain_chirho.v             # Bit vector domains (Theorems 1-2)
│   ├── MiniKanren_chirho.v         # Goals, semantics, main theorem
│   └── _CoqProject                 # Build configuration
└── docs/
    └── PROOF_NOTES_CHIRHO.md       # Informal proof sketches
```

---

## Core Theorems

| # | Name | File | Status |
|---|------|------|--------|
| 1 | intersect_sound_chirho | Domain*.{lean,v} | ✅ Proven |
| 2 | intersect_complete_chirho | Domain*.{lean,v} | ✅ Proven |
| 3 | intersect_comm_chirho | Domain*.{lean,v} | ✅ Proven |
| 4 | intersect_assoc_chirho | Domain*.{lean,v} | ✅ Proven |
| 5 | union_comm_chirho | Domain*.{lean,v} | ✅ Proven |
| 6 | union_assoc_chirho | Domain*.{lean,v} | ✅ Proven |
| 7 | intersect_distrib_union_chirho | Domain*.{lean,v} | ✅ Proven |
| 8 | member_intersect_iff_chirho | Domain*.{lean,v} | ✅ Proven |
| 9 | **domains_refine_chirho** | MiniKanren_chirho.v | ✅ Proven |
| 10 | **search_equiv_chirho** | MiniKanren_chirho.v | ✅ Proven |
| 11 | unify_intersects_chirho | MiniKanren_chirho.v | ✅ Proven |
| 12 | unify_tensor_equiv_chirho | MiniKanren_chirho.v | ✅ Proven |
| 13 | Semiring_chirho (class) | Domain_chirho.v | ✅ Defined |
| 14 | bool_semiring_chirho | Domain_chirho.v | ✅ Instance |
| 15 | nat_semiring_chirho | Domain_chirho.v | ✅ Instance |

---

## The Main Claim

**Theorem 5 (SearchEquivChirho):**

> For finite domains with tabled relations, miniKanren's domain-based search
> produces the same answer set as sparse Boolean tensor network contraction.

```
∀ g : Goal, ∀ s : State,
  (all domains finite) →
  run_minikanren(g, s) ≡ contract(compile(g), encode(s))
```

---

## Dependencies

### Lean 4

Install via elan:
```bash
curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
source ~/.elan/env
```

Build:
```bash
cd lean4
lake update
lake build
```

### Coq

Install via opam:
```bash
# Install opam first if needed
brew install opam  # macOS
opam init
opam install coq

# Then build
cd coq
coq_makefile -f _CoqProject -o Makefile
make
```

---

## Connection to Implementation

| Formal | Rust (`rust_chirho/`) | Python |
|--------|----------------------|--------|
| `DomainChirho` | `BitVec64Chirho` | `unify_bits_chirho.py` |
| `intersect_chirho` | `.and_chirho()` | `intersect_chirho()` |
| `SparseTensorChirho` | `SparseRelationChirho` | COO in `appendo_chirho.py` |
| `contract_chirho` | `propagate_chirho()` | `contract_tensor_chirho()` |

---

## Contributing

1. Pick an unproven theorem from the plan
2. Write informal proof sketch in `docs/PROOF_NOTES_CHIRHO.md`
3. Implement in Lean 4 or Coq
4. Update status in `FORMALIZATION_PLAN_CHIRHO.md`

---

*Soli Deo Gloria* ☧
