# Paper Claim → Artifact Traceability Matrix ☧

This document maps every numeric claim in the paper to reproducible commands.

## Table: Microbenchmarks (Table 2 - Hardware Optics vs Heap)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Single intersect: 420 ps | §5.1 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- domain_and` | ~420ps per op |
| Mass intersect n=10: 1.2 ns | §5.1 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- mass_intersect/10` | ~1.2ns |
| Mass intersect n=1000: 56 ns | §5.1 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- mass_intersect/1000` | ~56ns |
| Speedup: 2,500-4,000× | §5.1 | `cd rust_chirho && cargo run --example profile_breakdown_chirho` | Shows speedup ratio |

## Table: N-Queens (Table 3)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| N-Queens 8 (Rust): 3.7 μs | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- nqueens/8` | ~3.7μs |
| N-Queens 12 (Rust): 3.9 ms | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- nqueens/12` | ~3.9ms |
| N-Queens 20 first: 923 μs | §5.2 | `cd rust_chirho && cargo run --example nqueens_chirho -- 20` | ~0.9ms for first |

## Table: Sudoku (Table 4)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Easy Sudoku: 3.1 μs | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- sudoku/easy` | ~3.1μs |
| Hard 17-clue: 9.4 μs | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- sudoku/hard` | ~9.4μs |
| Escargot: 48 μs | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- sudoku/escargot` | ~48μs |

## Table: Unification (Table 5)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Simple unify (Rust HW): 40 ns | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- unify` | ~40ns |
| Domain AND: 423 ps | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- domain_and` | ~420ps |

## Table: Goal Chains (Table 6)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Conjunction 2 goals: 78 ns | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- goals/2` | ~78ns |
| Conjunction 4 goals: 144 ns | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- goals/4` | ~144ns |
| Conjunction 8 goals: 288 ns | §5.2 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- goals/8` | ~288ns |

## Table: Datalog Comparison (Table 7)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| BitMatrix TC (1K edges): 3.9 ms | §6 | `cd rust_chirho && cargo bench --bench datalog_bench_chirho -- tc/1000` | ~3.9ms |
| Speedup vs Soufflé: 119× | §6 | `cd benchmarks_chirho/souffle_chirho && ./run_benchmark_chirho.sh` | Compare times |

## Table: SyGuS Synthesis (Table 8)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| max2: 12 μs | §7 | `cd rust_chirho && cargo run --example synthesis_chirho -- max2` | ~12μs |
| abs: 8 μs | §7 | `cd rust_chirho && cargo run --example synthesis_chirho -- abs` | ~8μs |
| min2: 11 μs | §7 | `cd rust_chirho && cargo run --example synthesis_chirho -- min2` | ~11μs |

## Table: OCanren Comparison (Table 9)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| appendo forward: 1.2 μs | §8 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- appendo/forward` | ~1.2μs |
| appendo backward: 8.5 μs | §8 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- appendo/backward` | ~8.5μs |
| Type inference: 15 μs | §8 | `cd rust_chirho && cargo run --example type_infer_chirho` | ~15μs |

## Table: GPU Acceleration (Table 10)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Bulk AND 1K: 180 ns | §9 | GPU requires WebGPU device | ~180ns |
| Bulk AND 100K: 280 ns | §9 | GPU requires WebGPU device | ~280ns |
| Transitive Closure 64×64: 18 μs | §9 | GPU requires WebGPU device | ~18μs |

## Table: Domain Composition (Table 12)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| BitVec64: 420 ps | §11 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- domain_and` | ~420ps |
| Hierarchical4k: 39 ns | §11 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- hierarchical4k` | ~39ns |
| Hierarchical256k: 367 ns | §11 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- hierarchical256k` | ~367ns |
| DiffHierarchical4k (soft): 1.26 μs | §11 | `cd rust_chirho && cargo bench --bench criterion_bench_chirho -- diff_hierarchical4k` | ~1.26μs |

## Table: Gradient Attenuation (Table 3)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| 1-hop gradient L2: 2.0e-1 | §4.4 | `cd rust_chirho && cargo run --example learn_deep_chirho` | Gradient table |
| 2-hop gradient L2: 1.9e-2 | §4.4 | `cd rust_chirho && cargo run --example learn_deep_chirho` | ~10× attenuation |
| 3-hop gradient L2: 1.6e-3 | §4.4 | `cd rust_chirho && cargo run --example learn_deep_chirho` | ~10× per hop |
| 4-hop gradient L2: 1.2e-4 | §4.4 | `cd rust_chirho && cargo run --example learn_deep_chirho` | Stable attenuation |

## Profile Breakdown (Interning Tax)

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Interning overhead: 0.04% | §3.4 | `cd rust_chirho && cargo run --example profile_breakdown_chirho` | Shows breakdown |

## FPGA Verification

| Paper Claim | Section | Command | Expected Output |
|-------------|---------|---------|-----------------|
| Calyx simulation: 8 cycles | §4.2 | `cd calyx_chirho && ./simulate_chirho.sh` | "8 clock cycles" |
| LUT estimate: ~2000 | §4.2 | Synthesis (BLOCKED - needs hardware) | ~2000 LUTs |

---

## Quick Reproduction Script

```bash
#!/bin/bash
# Run all paper benchmarks
cd rust_chirho

# Core benchmarks
cargo bench --bench criterion_bench_chirho 2>/dev/null | tee results_chirho.txt

# Examples
cargo run --example profile_breakdown_chirho 2>/dev/null >> results_chirho.txt
cargo run --example type_infer_chirho 2>/dev/null >> results_chirho.txt
cargo run --example learn_deep_chirho 2>/dev/null >> results_chirho.txt
cargo run --example synthesis_chirho 2>/dev/null >> results_chirho.txt

echo "Results saved to results_chirho.txt"
```

---

*Soli Deo Gloria* ☧
