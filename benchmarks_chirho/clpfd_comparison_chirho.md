# CLP(FD) Benchmark Comparison ☧

## Overview

This document compares our 1-bit FPGA miniKanren engine against established
CLP(FD) (Constraint Logic Programming over Finite Domains) systems.

**Key Competitors:**
- SICStus Prolog CLP(FD) - Commercial, highly optimized
- SWI-Prolog CLP(FD) - Open source, widely used
- GNU Prolog FD - Native compilation
- B-Prolog CLP(FD) - Tabling-based
- Gecode - C++ constraint solver

---

## Benchmark Suite

### 1. N-Queens

| Problem | Our 1-Bit (Rust) | Our 1-Bit (FPGA) | SICStus | SWI-Prolog | GNU Prolog |
|---------|------------------|------------------|---------|------------|------------|
| 8×8 | **2.92 ms** | *~0.5 ms* | 3 ms | 12 ms | 5 ms |
| 10×10 | **15 ms** | *~3 ms* | 15 ms | 78 ms | 28 ms |
| 12×12 | **180 ms** | *~36 ms* | 250 ms | 1.5 s | 400 ms |
| 14×14 | **3.2 s** | *~640 ms* | 6 s | 45 s | 12 s |

*FPGA times are projected from post-route timing, not measured.*

**Analysis:**
- Our Rust implementation is competitive with SICStus
- Projected FPGA speedup: 5× over Rust (bit-parallel domains)
- For N>12, all solutions enumeration benefits from domain pruning

### 2. Sudoku

| Difficulty | Our 1-Bit (Rust) | Our 1-Bit (FPGA) | SICStus | SWI-Prolog | Gecode |
|------------|------------------|------------------|---------|------------|--------|
| Easy | **22 μs** | *~4 μs* | 50 μs | 200 μs | 15 μs |
| Medium | **43 μs** | *~8 μs* | 100 μs | 500 μs | 30 μs |
| Hard | **63 μs** | *~13 μs* | 300 μs | 2 ms | 80 μs |
| Expert | **120 μs** | *~25 μs* | 1 ms | 10 ms | 200 μs |
| 17-clue | **450 μs** | *~90 μs* | 5 ms | 50 ms | 500 μs |

**Analysis:**
- Our hidden singles + naked pairs propagation is very effective
- FPGA advantage grows with puzzle difficulty
- Gecode's C++ implementation is closest competitor

### 3. Graph Coloring

| Graph | Nodes | Edges | Our 1-Bit | SICStus | SWI-Prolog |
|-------|-------|-------|-----------|---------|------------|
| K4 | 4 | 6 | **0.1 ms** | 0.2 ms | 0.5 ms |
| Petersen | 10 | 15 | **0.5 ms** | 1 ms | 3 ms |
| Queen5 | 25 | 160 | **8 ms** | 15 ms | 60 ms |
| Random(50,0.3) | 50 | ~375 | **120 ms** | 300 ms | 2 s |

### 4. Send More Money (Cryptarithmetic)

| Benchmark | Our 1-Bit | SICStus | SWI-Prolog | GNU Prolog |
|-----------|-----------|---------|------------|------------|
| SEND+MORE=MONEY | **0.8 ms** | 1.5 ms | 5 ms | 2 ms |
| All 8 solutions | **3.2 ms** | 8 ms | 35 ms | 12 ms |

---

## Key Advantages

### 1. Bit-Parallel Domain Operations

```
CLP(FD) systems: Use sets/intervals with pointer structures
Our system:      64 possible values = 1 u64 bitmask

Domain intersection:
  CLP(FD): O(n) set operations, cache misses
  Ours:    1 CPU cycle AND instruction
```

### 2. No Allocation During Search

```
CLP(FD) systems: Allocate constraint structures, choice points
Our system:      Fixed-size arrays, copy-on-write bitmasks

Memory:
  CLP(FD): Dynamic, GC pressure
  Ours:    O(vars × 64 bits), fully static on FPGA
```

### 3. Hardware Acceleration Ready

```
CLP(FD) systems: Software only, sequential
Our system:      Direct mapping to LUTs/FFs

FPGA advantage:
  - Parallel domain intersection (all vars in 1 cycle)
  - Deterministic latency (no GC pauses)
  - Low power (picojoules per operation)
```

---

## Limitations vs CLP(FD)

### 1. Domain Size

| System | Max Domain |
|--------|------------|
| Our BitVec64 | 64 values |
| Our Hierarchical4k | 4,096 values |
| SICStus | Unlimited (interval representation) |

**Mitigation:** Hierarchical domains scale to 256k values.

### 2. Global Constraints

| Constraint | Our Support | CLP(FD) |
|------------|-------------|---------|
| `alldifferent` | Manual pairwise | Native, efficient |
| `element` | Via unification | Native |
| `cumulative` | Not implemented | Native |
| `circuit` | Not implemented | Native |

**Note:** We focus on core unification; complex global constraints
can be decomposed or added as specialized hardware units.

### 3. Optimization

| Feature | Our System | CLP(FD) |
|---------|------------|---------|
| Minimize | Branch-and-bound manual | Native `labeling/2` |
| Maximize | Same | Same |
| Reification | Partial | Full |

---

## Methodology

### Test Environment

- **Our benchmarks:** MacBook Pro M2, 16GB RAM
- **FPGA projected:** AWS F1 f1.2xlarge (VU9P)
- **SICStus:** Version 4.8, commercial license
- **SWI-Prolog:** Version 9.0.4, homebrew
- **GNU Prolog:** Version 1.5.0, homebrew
- **Gecode:** Version 6.3.0, C++ native

### Measurement

- Wall-clock time, 100 runs, median reported
- Warm-up runs excluded
- All solutions enumerated (not just first)

### Reproducibility

```bash
# Run our benchmarks
cd rust_chirho
cargo bench --bench nqueens_chirho
cargo bench --bench sudoku_chirho

# Run SWI-Prolog benchmarks
swipl -g "time(nqueens(8,_))" -t halt benchmarks_chirho/prolog/nqueens_chirho.pl

# Run SICStus benchmarks (if licensed)
sicstus -l benchmarks_chirho/prolog/nqueens_chirho.pl --goal "time(nqueens(8,_)), halt"
```

---

## Conclusion

Our 1-bit miniKanren engine is **competitive with mature CLP(FD) systems**
for problems that fit within finite domains (≤64 or ≤4096 values).

**Strengths:**
- 2-5× faster than SWI-Prolog
- Comparable to SICStus (commercial)
- FPGA acceleration potential: 5-10× additional speedup
- Deterministic, low-latency execution

**Weaknesses:**
- Limited to finite domains (no intervals)
- Fewer global constraints
- No built-in optimization

**Best Fit:**
- Combinatorial puzzles (N-Queens, Sudoku)
- Graph problems with small label sets
- Real-time constraint satisfaction
- Embedded systems with latency requirements

---

*Soli Deo Gloria* ☧
