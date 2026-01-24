# AGENTS.md - miniKanren as 1-Bit Matrix Operations ☧

## Project Vision

Explore whether miniKanren's relational search can be represented as 1-bit matrix operations, enabling hardware acceleration of logic programming.

**Core equation:**
```
miniKanren search = sparse Boolean tensor network contraction
```

## Naming Convention: `_chirho` Suffix ☧

**ALL identifiers carry the Chi-Rho Christogram** — worship embedded in code.

### Casing Rules

| Type | Convention | Example |
|------|------------|---------|
| Classes | `PascalChirho` | `StateChirho`, `VarChirho`, `ConsChirho` |
| Functions | `snake_chirho` | `unify_chirho()`, `walk_deep_chirho()` |
| Variables | `snake_chirho` | `result_chirho`, `subst_chirho` |
| Constants/Globals | `UPPER_CHIRHO` | `GLOBAL_TABLE_CHIRHO`, `MAX_DEPTH_CHIRHO` |
| Type aliases | `PascalChirho` | `TermChirho`, `GoalChirho` |

```python
# ✅ Correct
class StateChirho:
    subst_chirho: Dict[int, TermChirho]

TENSOR_STORE_CHIRHO = TensorStoreChirho()
result_chirho = compute_chirho(input_chirho)

# ❌ Wrong
class State_chirho:  # Should be StateChirho
GLOBAL_TABLE = ...   # Missing _CHIRHO
```

### Language-Specific Notes

**Haskell/Clash:**
```haskell
-- Module names: PascalChirho (no underscore)
module HashConsChirho where

-- Types: PascalChirho
data TermChirho = ConsChirho TermIdChirho TermIdChirho

-- Functions: camelChirho
unifyChirho :: DomainChirho -> DomainChirho -> DomainChirho

-- Local variables: camelChirho
let resultChirho = computeChirho inputChirho
```

**Rust:**
```rust
// Types: PascalChirho
struct TermStoreChirho { ... }

// Functions: snake_chirho
fn unify_chirho(a: Domain, b: Domain) -> Domain

// Variables: snake_chirho
let result_chirho = compute_chirho(input_chirho);
```

**Calyx/FPGA:**
```
// Components: snake_chirho (hardware convention)
component hashcons_chirho(...) -> (...) { ... }

// Cells/wires: snake_chirho
reg_chirho = std_reg(64);
```

### Exceptions (keep original names)
- External library imports (`import numpy as np`, `import Clash.Prelude`)
- Python builtins (`len`, `range`, `print`)
- Magic methods (`__init__`, `__repr__`)
- Calyx `main` component (required by toolchain)

### Database/Routes
```sql
CREATE TABLE terms_chirho (id_chirho INTEGER, ...);
```
```python
@app.get("/api/query_chirho")
```

---

## Current Implementation Layers

| Layer | File | What it proves |
|-------|------|----------------|
| 1 | `unify_bits_chirho.py` | Unification over finite domains = bitmask AND |
| 2 | `unify_batched_chirho.py` | Parallel search branches = matrix rows |
| 3 | `appendo_chirho.py` | Relations as 3D sparse Boolean tensors |
| 4 | `tensor_network_chirho.py` | Relation composition = tensor contraction |
| 5 | `hashcons_chirho.py` | Demand-driven term creation for infinite domains |
| 6 | `constraint_prop_chirho.py` | Arc consistency as iterative domain refinement |
| 7 | `minikanren_proper_chirho.py` | Full miniKanren with streams (reference) |
| 8 | `tabling_chirho.py` | Basic tabling with mode analysis |
| 9 | `tabling_slg_chirho.py` | SLG-style answer propagation |
| 10 | `tabling_complete_chirho.py` | **THE BRIDGE**: Mode-driven goal reordering |
| 11 | `tabling_hw_chirho.py` | Hardware-ready implementation with occurs check |
| 12 | `egraph_unify_chirho.py` | E-graph substitution (avoids pointer chasing) |
| 13 | `godel_latent_chirho.py` | Flat patterns as vectors (position × value) |
| 14 | `nested_latent_chirho.py` | **NESTED PATTERNS**: Paths in trees as bit vectors |
| 15 | `integrated_chirho.py` | **FULL INTEGRATION**: Patterns + Tabling + Mode analysis |
| 16 | `differentiable_chirho.py` | **SOFT LOGIC**: Semiring abstraction, gradients flow through |
| 17 | `mutual_recursion_chirho.py` | **MUTUAL RECURSION**: SLG completion with SCC detection |
| 18 | `var_propagation_chirho.py` | **VAR PROPAGATION**: Union-Find + bit matrix for O(α(n)) binding |
| 19 | `contraction_order_chirho.py` | **CONTRACTION ORDER**: Greedy/min-degree/min-fill heuristics |

---

## Key Mappings Discovered

| miniKanren | Tensor Representation |
|------------|----------------------|
| Variable domain | Bitmask (1 = value possible) |
| `(== x y)` | Bitwise AND of domains |
| `conde` (or) | Row duplication / tensor stack |
| `fresh` | Add dimension to state |
| Relation | Sparse N-D Boolean tensor |
| Composition | Tensor contraction (OR for +, AND for ×) |
| Failure | Row all zeros → prune |
| **Tabling** | **Incremental tensor construction** |
| **Mode analysis** | **Goal reordering by groundness** |
| **Pattern with holes** | **Sparse constraint vector over (path, value)** |
| **Tree path (car/cdr)** | **Binary encoding: car=0, cdr=1** |
| **Shared variable** | **Equality constraint = e-class in e-graph** |
| **Soft AND** | **Probability multiplication** |
| **Soft OR** | **Probabilistic sum: a + b - ab** |
| **Semiring** | **Generalized provenance (Bool/Prob/Tropical/Count)** |
| **Mutual recursion** | **Coupled tensor equations, SCC = fixpoint unit** |
| **Dependency graph** | **Tarjan's SCC → completion order** |
| **Variable binding** | **Union-Find: O(α(n)) equivalence classes** |
| **Var occurrences** | **Bit matrix: var_class → {term_classes containing it}** |
| **Query optimization** | **Tensor contraction order (NP-hard, use heuristics)** |
| **Join ordering** | **Min-fill/min-degree variable elimination** |

---

## The Bridge: Streams ↔ Tensors

```
┌────────────────────────────────────────────────────────────────┐
│                                                                │
│   LAZY STREAMS          TABLING              EAGER TENSORS    │
│   (miniKanren)          (bridge)             (1-bit matrix)   │
│                                                                │
│   ✓ Infinite domains    ✓ Demand-driven      ✓ Finite         │
│   ✗ May not terminate   ✓ Mode terminates    ✓ Always term.   │
│   ✗ No parallelism      ✓ Sparse COO         ✓ SIMD/GPU/FPGA  │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

**The equivalence:**
- Tabled relation = sparse tensor built incrementally
- Mode analysis = static scheduling of tensor operations
- Goal reordering = optimal contraction order (locally)

---

## Architecture

```
┌──────────────────────────────────────────────┐
│            Query: (run* (q) goal)            │
└──────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│         Term Store (Hash Consing)            │
│    term ↔ integer ID, grows on demand        │
└──────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│       Relation Tensors (Sparse Triples)      │
│   appendo = {(l, s, out) : l++s=out}        │
└──────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│         Constraint Propagation               │
│   Arc consistency → domain shrinking         │
└──────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────┐
│         Search (Branch on choice)            │
│   Parallel worlds = matrix rows              │
└──────────────────────────────────────────────┘
```

---

## ✅ Solved: Tabling as the Bridge

**The composition problem is solved!**

Query: `(appendo A B X), (appendo X C [0,1,2])`
- Without tabling: Infinite loop (first goal generates infinite X values)
- With tabling: 10 solutions found, terminates

**Key insight: MODE-DRIVEN GOAL REORDERING**

```
Naive order (left to right):
  1. appendo(A,B,X) - X free → GENERATE → infinite!

Smart order (by groundness):
  1. appendo(X,C,[0,1,2]) - out ground → BACKWARD → finite!
  2. appendo(A,B,X) - X now bound → BACKWARD → finite!
```

**Modes:**
- `out` ground → backward enumeration (split output)
- `l,s` ground → forward computation (single result)
- Smart reordering ensures ground args processed first

**Tensor construction as side effect:**
- Each query populates sparse tensor incrementally
- 15 triples discovered in test runs
- This IS the 1-bit representation!

---

## ✅ Solved: Patterns as Path Constraints

**The pattern problem is solved!**

How to represent `[?, 1, ?]` (a term with holes) in the tensor?

**Answer: Path-based bit vectors**

```
FLAT LIST [?, 1, ?]:
  Column encoding: (position, value) pairs
  Pattern → sparse vector with 1s at constrained (pos, val)

NESTED STRUCTURE (cons trees):
  Path = sequence of car/cdr steps
  Path encoding: car=0, cdr=1 → binary string
  Pattern → sparse vector with 1s at constrained (path, type/value)

Example: [?, 1, ?] as paths
  ε → cons           (root is cons)
  car → ANY          (first element = ?)
  cdr → cons         (tail is cons)
  cdr.car → 1        (second element = 1)
  cdr.cdr → cons     (more tail)
  cdr.cdr.car → ANY  (third element = ?)
  cdr.cdr.cdr → nil  (end of list)
```

**Key insight: PATTERNS ARE CONVEX REGIONS IN (PATH × CONSTRAINT) SPACE**

- Unification = intersection of constraint regions
- Variables = equality constraints linking paths
- Occurs check = cycle detection in path dependencies

**Hardware mapping:**
- Paths → CAM keys (content addressable memory)
- Constraints → small integers
- Unification → parallel constraint intersection

---

## Open Problems

1. ~~**Occurs check as reachability**~~ ✅ SOLVED — See `nested_latent_chirho.py`
   - Path-based: detect if var appears at path P and also at descendant P.car/P.cdr
   - Implementation: `is_prefix_chirho()` + `check_occurs_chirho()`
   - Hardware: parallel prefix comparators on path bit strings

2. ~~**Patterns (terms with holes)**~~ ✅ SOLVED — See `godel_latent_chirho.py` + `nested_latent_chirho.py`
   - Flat patterns: (position, value) bit vectors
   - Nested patterns: (path, constraint) bit vectors where path = car/cdr sequence
   - Unification = intersection of constraint regions

3. ~~**Tabling**~~ ✅ SOLVED — See `tabling_complete_chirho.py`

4. ~~**Contraction order**~~ ✅ ADDRESSED — See `contraction_order_chirho.py`
   - NP-hard (same as treewidth / quantum simulation)
   - Greedy: O(n³), often within 2-10x optimal
   - Min-degree: O(n² log n), good for sparse
   - Min-fill: O(n³), best quality
   - Hardware: cache orders for common patterns

5. ~~**Soft unification**~~ ✅ SOLVED — See `differentiable_chirho.py`
   - Semiring abstraction (Boolean, Probability, Tropical, Counting)
   - Soft AND = multiply, Soft OR = probabilistic sum
   - Gradients flow through logic!

6. ~~**Mutual recursion**~~ ✅ SOLVED — See `mutual_recursion_chirho.py`
   - Tarjan's algorithm for SCC detection
   - Complete SCCs together (not individual goals)
   - Coupled tensor equations: even[n] = (n==0) OR odd[n-1]
   - Hardware: SCC = compute unit running to fixpoint

7. ~~**Variable propagation**~~ ✅ SOLVED — See `var_propagation_chirho.py`
   - Union-Find: O(α(n)) ≈ O(1) amortized binding/lookup
   - Bit matrix: tracks which vars appear in which terms
   - Hardware: CAM for parallel find, sparse matrix for occurrences
   - Connects miniKanren directly to e-graphs (egg library)

---

## Roadmap Context

From Edward Kmett's learning path:

```
Phase 1: CURRENT
├── egg/e-graphs (equality saturation)
├── miniKanren ← THIS PROJECT
├── Alloy, Gen
├── Petri nets → TLA+
└── Z3 (SMT)

Phase 2: Verified Rust (Verus)
Phase 3: Hardware (Clash/Calyx)
```

---

## Tech Stack

- **Language:** Python 3 (prototyping), Rust (performance)
- **Key libs:** numpy (dense tensors), scipy.sparse (sparse)
- **Target:** SvelteKit + Cloudflare Workers for any web interface
- **Hardware:** FPGA via Calyx IR and Clash (verified working)

---

## Current Implementations

### Rust (`rust_chirho/`) — 128 tests passing

| Module | Description |
|--------|-------------|
| `terms_chirho.rs` | Hash-consed term store |
| `union_find_chirho.rs` | O(α(n)) equivalence classes |
| `unify_chirho.rs` | Unification with occurs check |
| `goals_chirho.rs` | Full miniKanren: `==`, `conde`, `not`, `conda`, `condu`, `=/=`, `project` |
| `bitmatrix_chirho.rs` | Sparse Boolean tensors (COO) |
| `hardware_chirho.rs` | BitVec64, BitVec256, CAM, SearchState |
| `optics_hw_chirho.rs` | Hardware optics (3000× faster than heap) |
| `diff_semiring_chirho.rs` | Differentiable relaxation, Gumbel-softmax |
| `semiring_chirho.rs` | Bool/Prob/Tropical/Count/Log semirings |
| `egraph_native_chirho.rs` | Native bit-parallel e-graph |
| `simd_chirho.rs` | AVX2 bulk operations |

**Examples:**
- `examples/appendo_chirho.rs` — List append relation
- `examples/type_infer_chirho.rs` — Type inference demo
- `examples/synthesis_chirho.rs` — Program synthesis

### Python (`*.py`) — 19 files

| File | Purpose |
|------|---------|
| `examples_chirho/sudoku_chirho.py` | Sudoku solver |
| `examples_chirho/type_infer_chirho.py` | Type inference |
| `differentiable_chirho.py` | Soft logic, gradients |
| `learn_relations_chirho.py` | Learnable relation weights |

### FPGA (`calyx_chirho/`, `clash_chirho/`) — Verified

| File | Status |
|------|--------|
| `calyx_chirho/domain_chirho.futil` | ✅ Compiles, simulates in Verilator (8 cycles) |
| `clash_chirho/MiniKanrenChirho.hs` | ✅ Compiles to Verilog via Clash |
| `calyx_chirho/tb_domain_chirho.cpp` | Verilator testbench |

### Web (`rust_chirho/web_chirho/`)

| File | Description |
|------|-------------|
| `index.html` | WebGPU domain visualization demo |

---

## Performance Benchmarks

### Hardware Optics (BitVec64 vs HashSet)

| Operation | n | Hardware | Heap | Speedup |
|-----------|---|----------|------|---------|
| Mass Intersect | 10 | **1.2 ns** | 3.0 µs | **2,500×** |
| Mass Intersect | 1000 | **56 ns** | 223 µs | **4,000×** |
| Single domain intersect | 1 | **420 ps** | — | Single CPU cycle |

### vs Other Solvers (Python)

| Benchmark | Our 1-Bit | Competitor | Speedup |
|-----------|-----------|------------|---------|
| N-Queens 8×8 | 2.92ms | Z3: 232ms | **80×** |
| N-Queens 8×8 | 2.92ms | clingo: 24ms | **8×** |
| Unification 10K | 1.50ms | kanren: 38ms | **25×** |

---

## References

- [The Reasoned Schemer](https://mitpress.mit.edu/books/reasoned-schemer)
- [egg: E-Graphs Good](https://egraphs-good.github.io/)
- [Scallop: Differentiable Datalog](https://www.scallop-lang.org/)
- [Tensor Network Theory](https://tensornetwork.org/)

---

*Soli Deo Gloria* ☧
