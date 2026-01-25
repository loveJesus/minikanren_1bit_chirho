# miniKanren 1-Bit Integration Audit Skill

## Skill: /audit-minikanren

When invoked, perform a comprehensive audit of the miniKanren 1-bit project to ensure all components are integrated and follow the framework rules.

---

## Project Architecture Map

### Core Principle
**miniKanren search = sparse Boolean tensor network contraction**

All code must serve this equation. Every feature must integrate with the 1-bit domain representation.

---

## Component Layers

### Layer 1: Foundation (MUST be bit-parallel)
| Component | Rust Location | Python Location | Hardware |
|-----------|---------------|-----------------|----------|
| Term Store | `reference_chirho/terms_chirho.rs` | `hashcons_chirho.py` | `hashcons_chirho.futil` |
| Union-Find | `reference_chirho/union_find_chirho.rs` | `var_propagation_chirho.py` | CAM lookup |
| Unification | `reference_chirho/unify_chirho.rs` | `unify_bits_chirho.py` | AND gates |

### Layer 2: Domains (Scaling strategy)
| Domain Type | Values | Rust Module | Use Case |
|-------------|--------|-------------|----------|
| BitVec64Chirho | ≤64 | `hardware_chirho.rs` | Fastest, FPGA native |
| Hierarchical4kChirho | ≤4096 | `hierarchical_chirho.rs` | Medium scale |
| Hierarchical256kChirho | ≤262144 | `hierarchical_chirho.rs` | Large scale |
| SymbolicChirho | ∞ | `symbolic_chirho.rs` | Infinite domains |
| DiffHierarchical4kChirho | ≤4096 | `diff_hierarchical_chirho.rs` | Learning mode |

### Layer 3: Goals & Relations
| Feature | Rust | Python | Status |
|---------|------|--------|--------|
| eq (==) | `goals_chirho.rs` | `minikanren_proper_chirho.py` | Core |
| conde (or) | `goals_chirho.rs` | `minikanren_proper_chirho.py` | Core |
| fresh | `goals_chirho.rs` | `minikanren_proper_chirho.py` | Core |
| not | `goals_chirho.rs` | TBD | Extended |
| conda | `goals_chirho.rs` | TBD | Extended |
| condu | `goals_chirho.rs` | TBD | Extended |
| diseq (!=) | `goals_chirho.rs` | TBD | Extended |
| project | `goals_chirho.rs` | TBD | Extended |

### Layer 4: Search Strategies
| Strategy | Location | Integration |
|----------|----------|-------------|
| Stream-based | `stream_chirho.rs` | Reference implementation |
| Tabling | `tabling_chirho.rs` | Mode-driven reordering |
| Arc Consistency | `constraint_chirho.rs` | Domain pruning |
| Contraction Order | `contraction_chirho.rs` | Query optimization |

### Layer 5: Acceleration
| Target | Location | Status |
|--------|----------|--------|
| SIMD (AVX2) | `simd_chirho.rs` | Working |
| GPU (WebGPU) | `gpu_chirho.rs` | Working |
| FPGA (Calyx) | `calyx_chirho/` | Verified |
| FPGA (Clash) | `clash_chirho/` | Verified |

### Layer 6: Differentiable
| Component | Location | Integration |
|-----------|----------|-------------|
| Semirings | `semiring_chirho.rs` | Bool/Prob/Tropical/Count |
| DiffSemiring | `diff_semiring_chirho.rs` | Gradient flow |
| DiffHierarchical | `diff_hierarchical_chirho.rs` | Soft 1-bit domains |
| Gumbel-Softmax | `diff_semiring_chirho.rs` | Branching gradients |

---

## Audit Checklist

### 1. Naming Convention Compliance
Run grep to find violations:
```bash
# Find Rust identifiers missing _chirho suffix
grep -rn "pub fn [a-z_]*[^_chirho]\s*(" rust_chirho/src/ --include="*.rs" | grep -v "fn new\|fn default\|fn from\|fn into\|fn clone\|fn fmt\|fn eq\|fn hash\|fn cmp\|fn partial_cmp"

# Find Python functions missing _chirho
grep -rn "def [a-z_]*[^_chirho]\s*(" *.py --include="*.py" | grep -v "def __\|venv_chirho"
```

### 2. Integration Validation
Every new feature MUST:
- [ ] Use existing domain types (BitVec64/Hierarchical/Symbolic)
- [ ] Support bit-parallel operations (AND, OR, NOT)
- [ ] Have corresponding test in `rust_chirho/tests/` or `tests_chirho/`
- [ ] Be documented in AGENTS.md if significant
- [ ] Appear in README.md performance tables if benchmarked
- [ ] Be reflected in paper_chirho.tex if publishable

### 3. Cross-Component Sync
Check these stay aligned:
- [ ] `rust_chirho/Cargo.toml` version matches README
- [ ] Benchmark results in README match `benches/` output
- [ ] Paper tables match actual measured performance
- [ ] AGENTS.md "Current Implementations" matches actual modules

### 4. Framework Rule Compliance
New code MUST NOT:
- [ ] Create separate floating-point systems (use DiffHierarchical)
- [ ] Add heap-based containers in hot paths
- [ ] Bypass the domain abstraction
- [ ] Create "helper libraries" disconnected from core

New code MUST:
- [ ] Integrate through AdaptiveDomainChirho if adding new domain type
- [ ] Use existing semiring abstraction for new algebraic operations
- [ ] Maintain FPGA-friendliness (fixed-size arrays, no dynamic allocation)

---

## Quick Commands

### Run Full Audit
```bash
cd rust_chirho && cargo test && cargo clippy
cd .. && python3 -m pytest tests_chirho/ -v
```

### Check Naming Violations
```bash
# This finds potential violations
grep -rn "pub \(fn\|struct\|enum\|type\) [A-Za-z_]*[^o]$\|[^_]$" rust_chirho/src/ --include="*.rs" | head -20
```

### Regenerate Documentation
```bash
cd paper_chirho && pdflatex paper_chirho.tex && bibtex paper_chirho && pdflatex paper_chirho.tex && pdflatex paper_chirho.tex
```

### Run Benchmarks
```bash
cd rust_chirho && cargo bench -- --save-baseline current
```

---

## Framework Rules Summary

1. **Bit-Parallel First**: All operations must be expressible as bitwise ops
2. **Domain Hierarchy**: BitVec64 → Hierarchical → Symbolic (in order of preference)
3. **Integration Over Extension**: Enhance AdaptiveDomainChirho, don't create parallel systems
4. **Semiring Abstraction**: Use existing Bool/Prob/Tropical for different interpretations
5. **FPGA-Friendly**: No dynamic allocation, fixed-size structures preferred
6. **_chirho Everywhere**: All custom identifiers carry the Chi-Rho suffix

---

*Soli Deo Gloria*
