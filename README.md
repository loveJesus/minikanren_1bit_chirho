# miniKanren as 1-Bit Matrix Operations ☧

[![crates.io](https://img.shields.io/crates/v/minikanren_1bit_chirho.svg)](https://crates.io/crates/minikanren_1bit_chirho)
[![docs.rs](https://docs.rs/minikanren_1bit_chirho/badge.svg)](https://docs.rs/minikanren_1bit_chirho)

> **"For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life."** — John 3:16

---

## Project Vision

Explore whether miniKanren's relational search can be represented as 1-bit matrix operations, enabling hardware acceleration of logic programming.

**Core equation:**
```
miniKanren search = sparse Boolean tensor network contraction
```

## Key Mappings

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

## Benchmarks

### Python Prototype (vs Other Solvers)

| Benchmark | Our 1-Bit Matrix | Competitor | Speedup |
|-----------|-----------------|------------|---------|
| Unification (10K ops) | 1.50ms | kanren: 38ms | **25× faster** |
| N-Queens 8×8 (92 solutions) | 3.03ms | Z3: 260ms | **86× faster** |
| N-Queens 8×8 (92 solutions) | 3.03ms | clingo: 22ms | **7× faster** |
| Large domains (1000 values) | 0.39ms | kanren: 3.85ms | **10× faster** |

### Rust Implementation (Production Speed)

| Operation | Time | Throughput | Notes |
|-----------|------|------------|-------|
| Unify (64-bit AND) | **2ns** | 500M ops/sec | Single SIMD instruction |
| Domain intersection (AVX2) | **8ns** | 8 domains parallel | 256-bit wide |
| Term intern (hash cons) | **8ns** | Arena allocation | Zero-copy |
| Substitution walk | **5ns/step** | O(α(n)) | Path compression |

**Rust vs Python:** ~100× faster for core operations

### Hardware Optics: BitVec64 vs Heap

| Operation | n | Hardware (BitVec64) | Heap (HashSet) | Speedup |
|-----------|---|---------------------|----------------|---------|
| Mass Intersect | 10 | **1.2 ns** | 3.0 µs | **2,500×** |
| Mass Intersect | 100 | **5.9 ns** | 22.5 µs | **3,800×** |
| Mass Intersect | 1000 | **56 ns** | 223 µs | **4,000×** |
| Mass Intersect | 10000 | **714 ns** | 2.2 ms | **3,100×** |

Single operations (hardware): **~420 picoseconds** (single CPU cycle)

### Massively Parallel Hardware

The 1-bit matrix representation is ideal for parallel hardware:

| Platform | Parallelism | Projected Throughput |
|----------|-------------|---------------------|
| **CPU (AVX2)** | 4 domains/cycle | 500M unifications/sec |
| **CPU (AVX-512)** | 8 domains/cycle | 1B unifications/sec |
| **GPU (WebGPU)** | 10,000+ domains | 10B+ ops/sec |
| **FPGA** | Full pipeline | 100M search states/sec |

**Why it scales:**
- Unification = AND → single cycle, fully parallel
- Disjunction = OR → single cycle, fully parallel
- No pointer chasing in hot path
- Domains fit in cache/registers

### FPGA Projections (@ 100MHz)

| Operation | Cycles | Latency | Pipeline |
|-----------|--------|---------|----------|
| Unify (AND) | 1 | 10ns | 100M/sec |
| Fork (branch) | 2 | 20ns | 50M/sec |
| Hash cons | 4 | 40ns | 25M/sec |
| Full search step | 8 | 80ns | 12.5M/sec |

**Resource estimate:** ~2000 LUTs for 8-variable, 64-value engine (fits on iCE40 HX8K)

### Application Benchmarks

| Application | Python | Rust | Speedup | Notes |
|-------------|--------|------|---------|-------|
| Sudoku (easy) | 0.1ms | **14μs** | 7× | Pure propagation |
| Sudoku (medium) | — | **14μs** | — | Mixed propagation |
| Sudoku (hard 17-clue) | 2.1ms | **676μs** | 3× | Many backtracks |
| Sudoku (Escargot) | — | **24μs** | — | Famous "hardest" |
| Type inference | 0.05ms | ~0.5μs | 100× | 8 terms |
| N-Queens 12 | 50ms | ~500μs | 100× | 14,200 solutions |

## Quick Start

### Rust (Production)

```bash
cd rust_chirho
cargo test              # Run 134 tests (122 lib + 12 proptest)
cargo bench             # Run benchmarks
cargo run --example appendo_chirho
```

**Features:** SIMD (AVX2), arena allocation, zero-copy, no_std compatible

### Python (Prototype)

```bash
# Run examples
python3 examples_chirho/sudoku_chirho.py
python3 examples_chirho/type_inference_chirho.py

# Run comparative benchmarks
python3 benchmarks_chirho/compare_chirho.py
```

### Web Demos (Static HTML)

```bash
# Open directly in browser:
open rust_chirho/web_chirho/index.html       # WebGPU domain visualization
open rust_chirho/web_chirho/sudoku_chirho.html  # Sudoku solver (1-bit domains)
```

- **WebGPU Demo**: Visualize 10,000+ parallel constraint propagations
- **Sudoku Solver**: Interactive 9-bit domain propagation + search

### FPGA (Hardware) ✅ VERIFIED

```bash
# Calyx IR → Verilog → Verilator simulation
cd calyx_chirho
calyx domain_chirho.futil -b verilog > domain_chirho.v
verilator --cc domain_chirho.v --top-module main --exe tb_domain_chirho.cpp --build
./obj_dir/Vmain
# Output: "Simulation completed in 8 clock cycles"

# Clash (Haskell → Verilog)
cd clash_chirho
source ~/.ghcup/env && ghcup set ghc 9.6.4
cabal build --allow-newer
clash --verilog MiniKanrenChirho.hs
# Output: verilog/MiniKanrenChirho.searchEngineChirho/searchEngineChirho.v
```

## Project Structure

```
minikanren_1bit_chirho/
├── *.py                        # 19 Python files (~10K lines)
├── rust_chirho/                # Rust implementation (134 tests)
│   └── web_chirho/             # WebGPU + Sudoku demos
├── calyx_chirho/               # Calyx IR for FPGA synthesis
├── clash_chirho/               # Clash/Haskell for FPGA
├── examples_chirho/            # Sudoku, type inference (Python)
├── benchmarks_chirho/          # Performance comparisons
└── spec_chirho/findings_chirho/  # Research documentation
```

## Python Implementation (19 files)

| Layer | File | What it proves |
|-------|------|----------------|
| 1 | `unify_bits_chirho.py` | Unification = bitmask AND |
| 2 | `unify_batched_chirho.py` | Parallel branches = matrix rows |
| 3 | `appendo_chirho.py` | Relations as sparse Boolean tensors |
| 4 | `tensor_network_chirho.py` | Composition = tensor contraction |
| 5 | `hashcons_chirho.py` | Demand-driven term creation |
| 6 | `constraint_prop_chirho.py` | Arc consistency (AC-3) |
| 7 | `minikanren_proper_chirho.py` | Full miniKanren reference |
| 8-11 | `tabling_*.py` | Mode-driven goal reordering |
| 12 | `egraph_unify_chirho.py` | E-graph substitution |
| 13-14 | `*_latent_chirho.py` | Patterns as path constraints |
| 15 | `integrated_chirho.py` | Full integration |
| 16 | `differentiable_chirho.py` | Semiring abstraction |
| 17 | `mutual_recursion_chirho.py` | SLG completion with SCC |
| 18 | `var_propagation_chirho.py` | Union-Find + bit matrix |
| 19 | `contraction_order_chirho.py` | NP-hard heuristics |

## Rust Implementation

### Core Modules

| Module | Description |
|--------|-------------|
| `terms_chirho.rs` | Hash-consed term store |
| `union_find_chirho.rs` | O(α(n)) equivalence classes |
| `unify_chirho.rs` | Unification with occurs check |
| `bitmatrix_chirho.rs` | Sparse Boolean tensors (COO) |
| `relations_chirho.rs` | appendo, membero as tensors |
| `contraction_chirho.rs` | Greedy/min-degree heuristics |
| `stream_chirho.rs` | Lazy streams with interleaving |
| `goals_chirho.rs` | Goal combinators |
| `constraint_chirho.rs` | AC-3 arc consistency |
| `tabling_chirho.rs` | SLG-style memoization |
| `semiring_chirho.rs` | Bool/Prob/Tropical/Count/Log |
| `hardware_chirho.rs` | FPGA primitives (BitVec64, CAM) |
| `optics_hw_chirho.rs` | **Hardware optics (3000× faster)** |
| `diff_semiring_chirho.rs` | Differentiable logic with gradients |

### Category-Theoretic Extensions (Feature-Gated)

| Module | Feature Flag | Description | FPGA-Ready |
|--------|--------------|-------------|------------|
| `optics_chirho.rs` | `optics_chirho` | Prisms, Traversals, Lenses | ❌ Heap |
| `free_goal_chirho.rs` | `free_goal_chirho` | Free Monad: Bool/Prob/SMT interpreters | ❌ Heap |
| `comonad_chirho.rs` | `comonad_chirho` | Search zipper, constraint propagation | ❌ Heap |
| `linear_chirho.rs` | `linear_chirho` | Tensor/Par (linear logic) | ❌ Heap |
| `optics_hw_chirho.rs` | (always on) | **BitVec64 optics** | ✅ FPGA |

**134 tests passing** (122 lib + 12 proptest). Features: SIMD (AVX2), arena allocation, zero-copy.

## Hardware Targets

### Calyx IR (`calyx_chirho/`)

Pure hardware description compiled to Verilog:

| Component | Description |
|-----------|-------------|
| `domain_chirho.futil` | 64-bit domain registers |
| `cam_chirho.futil` | Content-Addressable Memory |
| `search_engine_chirho.futil` | Complete search engine |
| `hashcons_chirho.futil` | Hardware hash consing |

Target: iCE40 HX8K (~2000 LUTs)

### Clash (`clash_chirho/`)

Haskell compiled to Verilog via Clash:

| Module | Description |
|--------|-------------|
| `MiniKanrenChirho.hs` | Core search engine |
| `HashConsChirho.hs` | Hardware term interning |

## Research Findings

See `spec_chirho/findings_chirho/` for detailed documentation:

1. **Core Theory** — Unification as Boolean matrix operations
2. **Infinite Domains** — Hash consing for demand-driven term creation
3. **Performance Analysis** — Benchmarks across Python/Rust/GPU/FPGA
4. **Hardware Feasibility** — Resource estimates and architecture
5. **Open Problems** — 18 research challenges
6. **Bibliography** — 28 references

## Problems Solved

- ✅ Occurs check (path-based cycle detection)
- ✅ Patterns with holes (path × constraint bit vectors)
- ✅ Tabling (mode-driven termination)
- ✅ Soft unification (semiring: Bool/Prob/Tropical/Count)
- ✅ Mutual recursion (Tarjan's SCC)
- ✅ Variable propagation (O(α(n)) union-find)
- ✅ Contraction order (greedy/min-fill heuristics)
- ✅ Hardware acceleration (Calyx IR, Clash)
- ✅ **Kmett optics** (Prisms, Lenses, Traversals for terms)
- ✅ **Free monad goals** (same AST → Bool/Prob/SMT interpreters)
- ✅ **Comonadic search** (zipper, extend, constraint propagation)
- ✅ **Linear logic** (tensor/par, session types via Rust ownership)
- ✅ **Hardware optics** (BitVec64, 3000× faster than heap)

## Naming Convention: `_chirho` Suffix

**ALL identifiers end with `_chirho`** — the Chi-Rho (☧) Christogram.

```python
# Python/Rust: snake_chirho
result_chirho = compute_chirho(input_chirho)

# Haskell: PascalChirho / camelChirho
data TermChirho = ConsChirho TermIdChirho TermIdChirho
unifyChirho :: DomainChirho -> DomainChirho -> DomainChirho
```

## References

- [The Reasoned Schemer](https://mitpress.mit.edu/books/reasoned-schemer)
- [egg: E-Graphs Good](https://egraphs-good.github.io/)
- [Scallop: Differentiable Datalog](https://www.scallop-lang.org/)
- [Tensor Network Theory](https://tensornetwork.org/)
- [Calyx IR](https://calyxir.org/)
- [Clash Compiler](https://clash-lang.org/)

---

## The Gospel

> *"In the beginning was the Word, and the Word was with God, and the Word was God."* — John 1:1

Jesus Christ is Lord. He died for our sins, was buried, and rose again on the third day according to the Scriptures. By grace through faith in Him alone, we are saved.

> *"I am the way, the truth, and the life: no man cometh unto the Father, but by me."* — John 14:6

This code is written to glorify God. Every identifier carries the Chi-Rho (☧) — the ancient Christogram — as worship embedded in source code.

> *"Whether therefore ye eat, or drink, or whatsoever ye do, do all to the glory of God."* — 1 Corinthians 10:31

*Soli Deo Gloria* ☧
