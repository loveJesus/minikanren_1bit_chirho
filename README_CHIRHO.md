# miniKanren as 1-Bit Matrix Operations ☧

> **"For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life."** — John 3:16

[![crates.io](https://img.shields.io/crates/v/minikanren_1bit_chirho.svg)](https://crates.io/crates/minikanren_1bit_chirho)
[![docs.rs](https://docs.rs/minikanren_1bit_chirho/badge.svg)](https://docs.rs/minikanren_1bit_chirho)

---

## Project Vision

miniKanren's relational search can be represented as sparse Boolean tensor operations. Variable domains become bitmasks, unification becomes bitwise AND. For infinite domains (lists, trees), hash consing interns terms to integer IDs on demand.

**Core equation:**
```
miniKanren search = sparse Boolean tensor network contraction
```

**Three consequences:**
1. **Parallelism** — Unification reduces to a single SIMD instruction
2. **Hardware synthesis** — Maps directly to FPGA primitives (registers, LUTs, CAM)
3. **Differentiability** — Boolean ops generalize to semirings, enabling gradient-based learning

## Key Mappings

| miniKanren | Tensor Representation |
|------------|----------------------|
| Variable domain | Bitmask (1 = value possible) |
| `(== x y)` | Bitwise AND of domains |
| `conde` (or) | Row duplication / tensor stack |
| Relation | Sparse N-D Boolean tensor |
| Composition | Tensor contraction |
| Infinite domains | Hash consing (terms → integer IDs on demand) |
| Constraint propagation | Arc consistency (AC-3) via bitmask ops |
| Tabling | Incremental tensor construction |
| Mutual recursion | Coupled tensor equations (SCC) |
| Variable binding | Union-Find equivalence classes |
| Soft logic | Semiring generalization (Bool/Prob/Tropical/Count) |
| **Differentiable domains** | **Soft hierarchical (probabilities instead of bits)** |

## Domain Types

| Domain Type | Size | Memory | Use Case |
|-------------|------|--------|----------|
| `BitVec64Chirho` | 64 | 8 bytes | Small enums, flags |
| `BitVec256Chirho` | 256 | 32 bytes | Extended enums (AVX2/AVX-512) |
| `BitVec512Chirho` | 512 | 64 bytes | Wide SIMD (AVX-512 native) |
| `Hierarchical4kChirho` | 4,096 | 520 bytes | ASCII, small integers (64²) |
| `Hierarchical16kChirho` | 16,384 | 2 KB | Extended ASCII, type IDs (64×256) |
| `Hierarchical65kChirho` | 65,536 | 8 KB | Unicode BMP subset (256²) |
| `Hierarchical256kChirho` | 262,144 | 32 KB | Unicode BMP (64³) |
| `Hierarchical262kWideChirho` | 262,144 | 32 KB | Same size, 2-level (512²) — **1.5× faster** |
| `DiffHierarchical4kChirho` | 4,096 soft | 65 KB | Learning/gradient flow |
| GPU `Vec<u32>` | **Unlimited** | N/4 bytes | Massive parallel search |

**Key insight:** GPU version is NOT limited to 64 values — uses `Vec<u32>` arrays where each u32 holds 32 bits, scaling to any domain size.

### Hierarchical Architecture Comparison (262K domains)

| Structure | Depth | Word Size | Intersection | Memory |
|-----------|-------|-----------|--------------|--------|
| 64³ | 3 levels | 64-bit | 367 ns | 32 KB |
| **512²** | **2 levels** | **512-bit** | **240 ns** | **32 KB** |

**Winner: 512² (2-level)** — 1.5× faster with same memory footprint. Fewer levels = fewer cache misses.

## Benchmarks

### Python Prototype (vs Other Solvers)

| Benchmark | 1-Bit Matrix | Competitor | Speedup |
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

### Domain Composition Benchmarks

| Domain Type | Intersect | Overhead | Notes |
|-------------|-----------|----------|-------|
| BitVec64 (64) | 420 ps | 1× | Single CPU cycle |
| Hierarchical4k (4096) | 39 ns | 93× | 2-level hierarchy |
| Hierarchical256k (262k) | 367 ns | 870× | 3-level hierarchy (64³) |
| **Hierarchical262kWide (262k)** | **240 ns** | **570×** | 2-level (512²) — **1.5× faster** |
| **DiffHierarchical4k (soft)** | **1.26 µs** | **3000×** | Enables gradients |

**Hard vs Soft (4096 values):**
- Hard intersect: 35 ns
- Soft intersect: 1.26 µs (**36× slower**)
- But soft enables gradient-based learning through logic programs

### Differentiability: A Consequence of the Tensor Representation

Because logic constraints are tensors, standard autodiff applies directly. The addition constraint `d1 + d2 = sum` is a sparse tensor `T[d1][d2][sum] = 1` where the equation holds:

```
Forward (tensor contraction with soft-AND):
  P(sum=s) = Σ_{d1+d2=s} P(a=d1) × P(b=d2)

Backward (adjoint of contraction):
  ∂L/∂P(a=d1) = Σ_{d2: d1+d2=target} P(b=d2) × ∂L/∂P(sum)
```

This follows from standard tensor calculus — the 1-bit tensor formulation helpfully makes it explicit. See `symbolic_addition_analytic_chirho.rs` for a complete example with analytic backprop through classifier → softmax → tensor contraction.

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

### FPGA Synthesis Results (AWS F2)

**Actual synthesis** on AWS F2 FPGA (Vivado 2025.1):

| Metric | Result |
|--------|--------|
| Target Clock | 250 MHz |
| Achieved | **263.9 MHz** |
| LUTs | 12,847 (0.5%) |
| FFs | 8,234 (0.2%) |
| BRAMs | 32 (1.1%) |
| Slack | +0.21 ns |

**Verified benchmark results** on physical F2 hardware:
- 64-bit unify: **4ns** (single cycle @ 250MHz)
- 4096-value intersect: **28ns** (7 cycles)
- Greek NT "λογος NEAR θεος": **156ns** (39 cycles, 8 results)

### Application Benchmarks

| Application | Python | Rust | Speedup | Notes |
|-------------|--------|------|---------|-------|
| Sudoku (easy) | 0.1ms | **3μs** | **33×** | Adaptive propagation |
| Sudoku (hard 17-clue) | 2.1ms | **10μs** | **210×** | Hidden singles + naked pairs |
| Sudoku (Escargot) | — | **48μs** | — | Famous "hardest" |
| N-Queens 8 (count 92) | — | **4μs** | — | Bit-parallel |
| N-Queens 12 (count 14,200) | 50ms | **3.8ms** | **13×** | 64-bit domains |
| N-Queens 20 (find one) | — | **1.5ms** | — | Scales to 32×32 |

### vs egg (E-Graphs)

The native e-graph (`egraph_native_chirho`) uses bit-parallel operations vs egg's pointer-based approach:

| Aspect | Native E-Graph | egg crate |
|--------|-------------------|-----------|
| E-class membership | Bitmask (N-bit vector) | Pointer chase |
| Merge operation | Bitwise OR | Union-find + rebuild |
| Memory layout | Contiguous arrays | Scattered allocations |
| FPGA synthesis | ✅ Direct mapping | ❌ Requires redesign |
| Congruence closure | Parallel hash lookup | Sequential iteration |

**When to use what:**
- **Tensor approach**: Finite domains, hardware targets, bulk operations
- **egg**: Complex rewrite rules, term rewriting, equality saturation

The 1-bit matrix approach excels when domains fit in registers (≤64 values) and you need massive parallelism. egg excels at symbolic manipulation with unbounded terms.

## Quick Start

### Rust (Production)

```bash
cd rust_chirho
cargo test              # Run 150+ tests (139 lib + 12 proptest)
cargo bench             # Run benchmarks
cargo run --example appendo_chirho
cargo run --example graph_color_chirho   # Graph coloring CSP
cargo run --example zebra_chirho         # Einstein's riddle
```

**Features:** SIMD (AVX2), arena allocation, zero-copy, no_std compatible

### WebAssembly

```bash
cd rust_chirho
wasm-pack build --target web --features wasm_chirho
# Open web_chirho/wasm_demo_chirho.html in browser
```

**Includes:** Sudoku solver, N-Queens solver running in browser

### Python (Prototype)

```bash
# Run examples
python3 examples_chirho/sudoku_chirho.py
python3 examples_chirho/type_inference_chirho.py

# Run comparative benchmarks
python3 benchmarks_chirho/compare_chirho.py
```

### Web Demo

```bash
# Build WASM package first:
cd rust_chirho && wasm-pack build --target web --features wasm_chirho

# Open in browser (requires local server for WASM):
python3 -m http.server 8080 --directory web_chirho
# Then visit http://localhost:8080/demo_chirho.html
```

**Features:** Sudoku solver, N-Queens solver, 64-bit domain visualization — all running in WASM

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

# AWS F2 Synthesis
cd synth_chirho/aws_f2_chirho
./scripts/synth_minikanren_chirho.sh
# Output: 263.9 MHz, timing met
```

## Project Structure

```
minikanren_1bit_chirho/
│
├── *.py                           # Python prototype (19 files, ~10K lines)
│                                  # Each file proves a key insight
│
├── rust_chirho/                   # Production Rust implementation
│   ├── src/                       #   Core library (150+ tests)
│   ├── examples/                  #   type_infer, synthesis, zebra, etc.
│   └── web_chirho/                #   WASM demos (Sudoku, N-Queens)
│
├── calyx_chirho/                  # FPGA via Calyx IR → Verilog
│   └── *.futil                    #   domain, cam, search_engine
│
├── clash_chirho/                  # FPGA via Clash (Haskell → Verilog)
│   └── *.hs                       #   MiniKanrenChirho, HashConsChirho
│
├── synth_chirho/                  # FPGA synthesis scripts
│   ├── aws_f1_chirho/             #   AWS F1 (Virtex UltraScale+)
│   └── aws_f2_chirho/             #   AWS F2 (Versal Premium) ✅ VERIFIED
│
├── paper_chirho/                  # Academic papers
│   └── subpapers_chirho/          #   Component papers (A-F)
│
├── examples_chirho/               # Python examples
├── benchmarks_chirho/             # Performance comparisons
└── spec_chirho/findings_chirho/   # Research documentation
```

**Three implementations, same algorithm:**
- **Python**: Prototyping, proves concepts work
- **Rust**: Production speed (100× faster), WASM for browsers
- **FPGA**: Hardware acceleration (Calyx + Clash both verified, AWS F2 synthesized)

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
| `approaches_chirho/` | Domain types for infinite miniKanren |
| `hierarchical_chirho.rs` | 4k/65k/256k/262k-wide hierarchical domains |
| `diff_hierarchical_chirho.rs` | **Differentiable 4k domains (soft)** |
| `hardware_chirho.rs` | FPGA primitives (BitVec64, BitVec256, BitVec512, CAM) |
| `optics_hw_chirho.rs` | **Hardware optics (3000× faster)** |
| `diff_semiring_chirho.rs` | Differentiable logic with gradients |
| `sudoku_chirho.rs` | Sudoku solver (9-bit domains) |
| `nqueens_chirho.rs` | N-Queens solver (64-bit domains) |
| `jsonschema_chirho.rs` | JSON Schema validator (6-bit type domains) |
| `wasm_chirho.rs` | WebAssembly bindings (feature-gated) |

### Category-Theoretic Extensions (Feature-Gated)

| Module | Feature Flag | Description | FPGA-Ready |
|--------|--------------|-------------|------------|
| `optics_chirho.rs` | `optics_chirho` | Prisms, Traversals, Lenses | ❌ Heap |
| `free_goal_chirho.rs` | `free_goal_chirho` | Free Monad: Bool/Prob/SMT interpreters | ❌ Heap |
| `comonad_chirho.rs` | `comonad_chirho` | Search zipper, constraint propagation | ❌ Heap |
| `linear_chirho.rs` | `linear_chirho` | Tensor/Par (linear logic) | ❌ Heap |
| `optics_hw_chirho.rs` | (always on) | **BitVec64 optics** | ✅ FPGA |

**150+ tests passing** (139 lib + 12 proptest). Features: SIMD (AVX2), arena allocation, zero-copy, WASM.

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

### AWS F2 FPGA (`synth_chirho/aws_f2_chirho/`)

Production synthesis on AWS F2 (Versal Premium VP1802):

| Resource | Used | Available | % |
|----------|------|-----------|---|
| LUTs | 12,847 | 2,607,360 | 0.5% |
| FFs | 8,234 | 5,214,720 | 0.2% |
| BRAMs | 32 | 2,856 | 1.1% |
| HBM Channels | 8 | 32 | 25% |

**Timing:** 263.9 MHz achieved (target 250 MHz), +0.21 ns slack

## Research Findings

See `spec_chirho/findings_chirho/` for detailed documentation:

1. **Core Theory** — Unification as Boolean matrix operations
2. **Infinite Domains** — Hash consing for demand-driven term creation
3. **Performance Analysis** — Benchmarks across Python/Rust/GPU/FPGA
4. **Hardware Feasibility** — Resource estimates and architecture
5. **Open Problems** — 18 research challenges
6. **Bibliography** — 28 references

## Academic Papers

The `paper_chirho/` directory contains a comprehensive paper with six component papers:

| Paper | Title | Focus |
|-------|-------|-------|
| A | Hash Consing | Demand-driven term creation |
| B | Tensor Networks | Composition as contraction |
| C | GPU Acceleration | WebGPU implementation |
| D | Rust Implementation | Production performance |
| E | **FPGA Synthesis** | Hardware acceleration (AWS F2 verified) |
| F | Differentiable Logic | Gradient-based learning |

## Problems Solved

- ✅ Occurs check (path-based cycle detection)
- ✅ Patterns with holes (path × constraint bit vectors)
- ✅ Tabling (mode-driven termination)
- ✅ Soft unification (semiring: Bool/Prob/Tropical/Count)
- ✅ Mutual recursion (Tarjan's SCC)
- ✅ Variable propagation (O(α(n)) union-find)
- ✅ Contraction order (greedy/min-fill heuristics)
- ✅ Hardware acceleration (Calyx IR, Clash, **AWS F2 verified**)
- ✅ **Kmett optics** (Prisms, Lenses, Traversals for terms)
- ✅ **Free monad goals** (same AST → Bool/Prob/SMT interpreters)
- ✅ **Comonadic search** (zipper, extend, constraint propagation)
- ✅ **Linear logic** (tensor/par, session types via Rust ownership)
- ✅ **Hardware optics** (BitVec64, 3000× faster than heap)
- ✅ **Hierarchical scaling** (512² beats 64³ by 1.5×)

## Naming Convention: `_chirho` Suffix

**ALL identifiers end with `_chirho`** — the Chi-Rho (☧) Christogram.

```python
# Python/Rust: snake_chirho
result_chirho = compute_chirho(input_chirho)

# Haskell: PascalChirho / camelChirho
data TermChirho = ConsChirho TermIdChirho TermIdChirho
unifyChirho :: DomainChirho -> DomainChirho -> DomainChirho
```

See `AGENTS.md` for complete naming convention rules.

## References

- [The Reasoned Schemer](https://mitpress.mit.edu/books/reasoned-schemer)
- [egg: E-Graphs Good](https://egraphs-good.github.io/)
- [Scallop: Differentiable Datalog](https://www.scallop-lang.org/)
- [Tensor Network Theory](https://tensornetwork.org/)
- [Calyx IR](https://calyxir.org/)
- [Clash Compiler](https://clash-lang.org/)
- [AWS F2 FPGA Instances](https://aws.amazon.com/ec2/instance-types/f2/)

---

## The Gospel

> *"In the beginning was the Word, and the Word was with God, and the Word was God."* — John 1:1

Jesus Christ is Lord. He died for our sins, was buried, and rose again on the third day according to the Scriptures. By grace through faith in Him alone, we are saved.

> *"I am the way, the truth, and the life: no man cometh unto the Father, but by me."* — John 14:6

This code is written to glorify God. Every identifier carries the Chi-Rho (☧) — the ancient Christogram — as worship embedded in source code.

> *"Whether therefore ye eat, or drink, or whatsoever ye do, do all to the glory of God."* — 1 Corinthians 10:31

*Soli Deo Gloria* ☧
