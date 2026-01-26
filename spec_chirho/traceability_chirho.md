# Paper Claim → Artifact Traceability Matrix ☧

This document maps every numeric claim in the paper to reproducible commands.

**Benchmark Filter Syntax:** `cargo bench -- <group>/<name>` or `cargo bench -- <name>`

## Table: Bit Operations (Hardware Optics)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Domain AND: 420 ps | §5.1 | `cd rust_chirho && cargo bench -- "BitOps/BitVec64_AND"` | ~420ps per op |
| BitVec256 AND | §5.1 | `cd rust_chirho && cargo bench -- "BitOps/BitVec256_AND"` | ~1.8ns |
| Mass intersect n=10: 1.2 ns | §5.1 | `cd rust_chirho && cargo bench --bench optics_bench_chirho -- "mass_intersect/hw/10"` | ~1.2ns |
| Mass intersect n=1000: 56 ns | §5.1 | `cd rust_chirho && cargo bench --bench optics_bench_chirho -- "mass_intersect/hw/1000"` | ~56ns |
| Heap HashSet n=1000: 223 μs | §5.1 | `cd rust_chirho && cargo bench --bench optics_bench_chirho -- "mass_intersect/heap_hashset/1000"` | ~223μs |
| Speedup: 2,500-4,000× | §5.1 | `cd rust_chirho && cargo run --release --example profile_breakdown_chirho` | Shows speedup ratio |

## Table: N-Queens (Table 3)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| N-Queens 8 (Rust): 3.7 μs | §5.2 | `cd rust_chirho && cargo bench -- "NQueens/8_count"` | ~3.7μs |
| N-Queens 12 (Rust): 3.9 ms | §5.2 | `cd rust_chirho && cargo bench -- "NQueens/12_count"` | ~3.9ms |
| N-Queens 20 (first): 923 μs | §5.2 | `cd rust_chirho && cargo bench -- "NQueens/20_one"` | ~0.9ms |
| N-Queens 32 (first): large | §5.2 | `cd rust_chirho && cargo bench -- "NQueens/32_one"` | First solution only |

## Table: Sudoku (Table 4)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Easy Sudoku: 3.1 μs | §5.2 | `cd rust_chirho && cargo bench -- "Sudoku/adaptive/easy"` | ~3.1μs |
| Hard 17-clue: 9.4 μs | §5.2 | `cd rust_chirho && cargo bench -- "Sudoku/adaptive/hard_17clue"` | ~9.4μs |
| Escargot: 48 μs | §5.2 | `cd rust_chirho && cargo bench -- "Sudoku/adaptive/escargot"` | ~48μs |

## Table: Unification (Table 5)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Simple unify (equal ints): 40 ns | §5.2 | `cd rust_chirho && cargo bench -- "Unify/equal_ints"` | ~40ns |
| Var to int binding | §5.2 | `cd rust_chirho && cargo bench -- "Unify/var_to_int"` | ~50ns |
| List unification | §5.2 | `cd rust_chirho && cargo bench -- "Unify/equal_lists"` | varies by length |

## Table: Goal Chains (Table 6)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Conjunction 2 goals: 78 ns | §5.2 | `cd rust_chirho && cargo bench --bench goal_bench_chirho -- "ConjChain/hardware_1bit/2"` | ~78ns |
| Conjunction 4 goals: 144 ns | §5.2 | `cd rust_chirho && cargo bench --bench goal_bench_chirho -- "ConjChain/hardware_1bit/4"` | ~144ns |
| Conjunction 8 goals: 288 ns | §5.2 | `cd rust_chirho && cargo bench --bench goal_bench_chirho -- "ConjChain/hardware_1bit/8"` | ~288ns |

## Table: Datalog Comparison (Table 7)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| BitMatrix TC | §6 | `cd rust_chirho && cargo bench --bench datalog_bench_chirho` | ~3.9ms for 1K edges |
| Speedup vs Soufflé: 119× | §6 | `cd benchmarks_chirho/souffle_chirho && ./run_benchmark_chirho.sh` | Compare times |

## Table: SyGuS Synthesis (Table 8)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| max2: 12 μs | §7 | `cd rust_chirho && cargo run --release --example synthesis_chirho` | ~12μs |
| abs: 8 μs | §7 | `cd rust_chirho && cargo run --release --example synthesis_chirho` | ~8μs |

## Table: OCanren Comparison (Table 9)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Type inference: 15 μs | §8 | `cd rust_chirho && cargo run --release --example type_infer_chirho` | ~15μs |

## Table: GPU Acceleration (Table 10)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| GPU ops | §9 | Requires WebGPU device; see `rust_chirho/benches/gpu_bench_chirho.rs` | ~280ns/op at 100K |

**Note:** GPU benchmarks require `--features gpu_chirho` and a compatible GPU.

## Table: Domain Composition (Table 12)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| All domain types | §11 | `cd rust_chirho && cargo bench --bench domain_composition_bench_chirho` | See table in paper |
| Differentiable domains | §11 | `cd rust_chirho && cargo bench --bench differentiable_bench_chirho` | ~1.26μs for soft 4k |

## Table: Gradient Attenuation (Table 3)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Gradient attenuation demo | §4.4 | `cd rust_chirho && cargo run --release --example gradient_table_chirho` | Gradient table |
| Deep learning gradients | §4.4 | `cd rust_chirho && cargo run --release --example learn_deep_chirho` | Shows attenuation |

## Symbolic Addition (MNIST-Addition Concept)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Symbolic addition (finite-diff) | §10 | `cd rust_chirho && cargo run --release --example symbolic_addition_chirho` | 100% accuracy |
| Symbolic addition (analytic) | §10 | `cd rust_chirho && cargo run --release --example symbolic_addition_analytic_chirho` | 100% accuracy |
| Gradient method comparison | §10 | `cd rust_chirho && cargo bench --bench symbolic_addition_bench_chirho` | Analytic ~65× faster |

**Note:** Two gradient implementations:
- `symbolic_addition_chirho.rs` - Finite-difference gradients (~8.4μs/step)
- `symbolic_addition_analytic_chirho.rs` - Analytic backprop through tensor contraction (~129ns/step, 65× faster)

## Profile Breakdown (Interning Tax)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Interning overhead: 0.04% | §3.4 | `cd rust_chirho && cargo run --release --example profile_breakdown_chirho` | Shows breakdown |

## Additional Case Studies

| Case Study | Command | Notes |
|------------|---------|-------|
| Type Inference (HM) | `cargo run --release --example type_infer_chirho` | Hindley-Milner |
| Zebra Puzzle | `cargo run --release --example zebra_chirho` | Einstein's riddle |
| Graph Coloring | `cargo run --release --example graph_color_chirho` | CSP example |
| Family Relations | `cargo run --release --example family_chirho` | Relational example |

## FPGA Verification

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Calyx simulation: 8 cycles | §4.2 | `cd calyx_chirho && ./simulate_chirho.sh` | "8 clock cycles" |
| LUT estimate: ~2000 | §4.2 | Synthesis (BLOCKED - needs hardware) | ~2000 LUTs |
| QuickCheck properties | §4.2 | `cd clash_chirho && cabal test` | Requires GHC 9.6.4 |

**Note:** QuickCheck property tests exist in `clash_chirho/test/QuickCheckChirho.hs`; execution requires GHC 9.6.4.

---

## Quick Reproduction Script

```bash
#!/bin/bash
# Run all paper benchmarks
cd rust_chirho

# Core benchmarks (Criterion)
cargo bench --bench criterion_bench_chirho 2>/dev/null | tee results_chirho.txt

# Examples (release mode for accurate timing)
cargo run --release --example profile_breakdown_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example type_infer_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example learn_deep_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example gradient_table_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example synthesis_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example symbolic_addition_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example symbolic_addition_analytic_chirho 2>/dev/null >> results_chirho.txt
cargo run --release --example zebra_chirho 2>/dev/null >> results_chirho.txt

echo "Results saved to results_chirho.txt"
```

---

*Soli Deo Gloria* ☧
