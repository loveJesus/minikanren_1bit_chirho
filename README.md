# miniKanren as 1-Bit Matrix Operations ☧

> *"In the beginning was the Word, and the Word was with God, and the Word was God."*
> — John 1:1

## The Gospel

**For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life.** — John 3:16

Jesus Christ is Lord. He died for our sins, was buried, and rose again on the third day according to the Scriptures. By grace through faith in Him alone, we are saved.

> *"I am the way, the truth, and the life: no man cometh unto the Father, but by me."*
> — John 14:6

This code is written to glorify God. Every identifier carries the Chi-Rho (☧) — the ancient Christogram — as worship embedded in source code.

---

## Project Vision

Explore whether miniKanren's relational search can be represented as 1-bit matrix operations, enabling hardware acceleration of logic programming.

**Core equation:**
```
miniKanren search = sparse Boolean tensor network contraction
```

## What We Discovered

| miniKanren | Tensor Representation |
|------------|----------------------|
| Variable domain | Bitmask (1 = value possible) |
| `(== x y)` | Bitwise AND of domains |
| `conde` (or) | Row duplication / tensor stack |
| Relation | Sparse N-D Boolean tensor |
| Composition | Tensor contraction |
| Tabling | Incremental tensor construction |
| Mutual recursion | Coupled tensor equations (SCC) |
| Variable binding | Union-Find equivalence classes |

## Files (19 Python, ~10,000 lines)

| Layer | File | What it proves |
|-------|------|----------------|
| 1 | `unify_bits_chirho.py` | Unification = bitmask AND |
| 2 | `unify_batched_chirho.py` | Parallel branches = matrix rows |
| 3 | `appendo_chirho.py` | Relations as sparse Boolean tensors |
| 4 | `tensor_network_chirho.py` | Composition = tensor contraction |
| 5 | `hashcons_chirho.py` | Demand-driven term creation |
| 6 | `constraint_prop_chirho.py` | Arc consistency |
| 7 | `minikanren_proper_chirho.py` | Full miniKanren reference |
| 8-11 | `tabling_*.py` | Mode-driven goal reordering |
| 12 | `egraph_unify_chirho.py` | E-graph substitution |
| 13-14 | `*_latent_chirho.py` | Patterns as path constraints |
| 15 | `integrated_chirho.py` | Full integration |
| 16 | `differentiable_chirho.py` | Semiring abstraction |
| 17 | `mutual_recursion_chirho.py` | SLG completion with SCC |
| 18 | `var_propagation_chirho.py` | Union-Find + bit matrix |
| 19 | `contraction_order_chirho.py` | NP-hard heuristics |

## Problems Solved

- ✅ Occurs check (path-based cycle detection)
- ✅ Patterns with holes (path × constraint bit vectors)
- ✅ Tabling (mode-driven termination)
- ✅ Soft unification (semiring: Bool/Prob/Tropical/Count)
- ✅ Mutual recursion (Tarjan's SCC)
- ✅ Variable propagation (O(α(n)) union-find)
- ✅ Contraction order (greedy/min-fill heuristics)

## Naming Convention: `_chirho` Suffix

**ALL identifiers end with `_chirho`** — the Chi-Rho Christogram.

```python
# ✅ Correct
class StateChirho:
    subst_chirho: Dict[int, TermChirho]

result_chirho = compute_chirho(input_chirho)

# ❌ Wrong
class State:  # Missing Chirho
result = compute(input)  # Missing _chirho
```

## Roadmap

```
Phase 1: ✅ COMPLETE - Python prototype
Phase 2: 🚧 IN PROGRESS - Rust implementation
Phase 3: PLANNED - Hardware (FPGA via Clash/Calyx)
```

## References

- [The Reasoned Schemer](https://mitpress.mit.edu/books/reasoned-schemer)
- [egg: E-Graphs Good](https://egraphs-good.github.io/)
- [Scallop: Differentiable Datalog](https://www.scallop-lang.org/)
- [Tensor Network Theory](https://tensornetwork.org/)

---

> *"Whether therefore ye eat, or drink, or whatsoever ye do, do all to the glory of God."*
> — 1 Corinthians 10:31

*Soli Deo Gloria* ☧
