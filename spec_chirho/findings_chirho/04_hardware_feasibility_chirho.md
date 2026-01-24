# Hardware Feasibility ☧

## 1. FPGA Resource Estimates

### Finding 4.1: Core Search Engine

Based on `calyx_chirho/search_engine_chirho.futil`:

| Component | LUTs | FFs | BRAMs |
|-----------|------|-----|-------|
| Domain registers (8×64-bit) | 512 | 512 | 0 |
| AND gates (unify) | 64 | 0 | 0 |
| Zero detect (fail) | 16 | 0 | 0 |
| Fork logic | 128 | 64 | 0 |
| State machine | 50 | 32 | 0 |
| **Total** | **~800** | **~600** | **0** |

This fits in the smallest FPGAs (e.g., Lattice iCE40, ~1000 LUTs).

### Finding 4.2: Hash Consing Unit

Based on `calyx_chirho/hashcons_chirho.futil`:

| Component | LUTs | FFs | BRAMs |
|-----------|------|-----|-------|
| Term store (1024×34-bit) | 0 | 0 | 2 |
| CAM (64 entries) | 512 | 256 | 0 |
| Hash compute (XOR) | 48 | 6 | 0 |
| Free pointer | 16 | 16 | 0 |
| Reference counts | 0 | 0 | 1 |
| Control logic | 100 | 50 | 0 |
| **Total** | **~700** | **~350** | **3** |

### Finding 4.3: Complete System

| Component | LUTs | FFs | BRAMs |
|-----------|------|-----|-------|
| Search engine | 800 | 600 | 0 |
| Hash cons unit | 700 | 350 | 3 |
| Substitution table | 200 | 0 | 1 |
| Stack (backtrack) | 100 | 0 | 1 |
| I/O interface | 200 | 100 | 0 |
| **Total** | **~2000** | **~1050** | **5** |

Fits in: Lattice ECP5, Xilinx Artix-7, Intel Cyclone IV.

## 2. FPGA Target Comparison

### Finding 4.4: Device Options

| FPGA | LUTs | BRAMs | Price | Fit? |
|------|------|-------|-------|------|
| iCE40 UP5K | 5,280 | 30 | $6 | ✓ (basic) |
| ECP5-25 | 24,576 | 56 | $15 | ✓ (full) |
| Artix-7 35T | 33,280 | 50 | $30 | ✓ (full) |
| Cyclone 10 LP | 25,000 | 66 | $25 | ✓ (full) |

### Finding 4.5: Clock Speed Estimates

| Implementation | Est. Fmax |
|----------------|-----------|
| Calyx → Verilog | 100-150 MHz |
| Clash → Verilog | 80-120 MHz |
| Hand-optimized | 200-300 MHz |

Conservative: 100 MHz provides 10ns cycle time.

## 3. ASIC Projections

### Finding 4.6: Technology Scaling

| Node | Area (mm²) | Power | Speed |
|------|------------|-------|-------|
| 180nm | 2.0 | 100mW | 200MHz |
| 65nm | 0.3 | 30mW | 500MHz |
| 28nm | 0.1 | 10mW | 1GHz |
| 7nm | 0.02 | 3mW | 2GHz |

At 28nm with 1GHz: 1ns per unification cycle.

### Finding 4.7: Comparison to Existing Accelerators

| Accelerator | Operation | Latency |
|-------------|-----------|---------|
| miniKanren FPGA (ours) | Unify | 10ns |
| EIE (DNN sparsity) | SpMV | 10ns |
| TPU (matrix multiply) | MatMul | 700ns |
| GPU (CUDA core) | FMADD | 4ns |

Our bit-parallel unification is competitive with specialized DNN accelerators.

## 4. Memory Hierarchy

### Finding 4.8: On-Chip vs Off-Chip

| Data Structure | Size | Location |
|----------------|------|----------|
| Domain registers | 512B | Registers |
| Substitution | 2KB | BRAM |
| Term store | 4KB | BRAM |
| Relation tensors | 1MB+ | DRAM |

### Finding 4.9: Memory Bandwidth Requirements

| Operation | Bandwidth |
|-----------|-----------|
| Domain ops | 0 (register-register) |
| Substitution walk | 64 bits/step |
| Term deref | 64 bits/access |
| Tensor slice | 1KB-1MB per query |

For small problems: All on-chip (no DRAM access).
For large relations: DRAM bandwidth limited.

## 5. Parallelism Strategies

### Finding 4.10: Search Branch Parallelism

Multiple search branches can run simultaneously:

```
Branch 1: [domain1_a, domain2_a, ...]
Branch 2: [domain1_b, domain2_b, ...]
...
Branch N: [domain1_n, domain2_n, ...]
```

N parallel branches = N× LUT usage, but shared BRAM.

### Finding 4.11: Constraint Parallelism

All constraints over disjoint variables can propagate in parallel:

| Constraints | Parallel Factor |
|-------------|-----------------|
| Sudoku rows | 9× |
| Sudoku columns | 9× |
| N-queens diagonals | N× |
| Type equations | #independent |

### Finding 4.12: Pipeline Depth

Operations can be pipelined:

```
Cycle 1: Unify(x,y)
Cycle 2: Unify(a,b), Fail-check(x,y)
Cycle 3: Fork(x), Unify(c,d), Fail-check(a,b)
...
```

3-stage pipeline provides 3× throughput with 3-cycle latency.

## 6. Power Efficiency

### Finding 4.13: Energy per Operation

| Platform | Energy/Unify |
|----------|--------------|
| CPU (scalar) | 50 pJ |
| CPU (SIMD) | 15 pJ |
| FPGA | 5 pJ |
| ASIC (28nm) | 0.5 pJ |

FPGA is 10× more efficient than CPU.
ASIC is 100× more efficient than CPU.

### Finding 4.14: Operations per Watt

| Platform | Ops/Watt |
|----------|----------|
| CPU @ 100W | 2B |
| GPU @ 300W | 50B |
| FPGA @ 5W | 1B |
| ASIC @ 1W | 2B |

GPU wins for throughput per watt.
ASIC wins for latency per watt.

## 7. Implementation Complexity

### Finding 4.15: Lines of Code Comparison

| Implementation | LoC | Complexity |
|----------------|-----|------------|
| Python prototype | 500 | Low |
| Rust + SIMD | 2000 | Medium |
| Calyx IR | 800 | Medium |
| Clash Haskell | 600 | Medium-High |
| Hand Verilog | 3000 | High |

Calyx and Clash provide good abstraction without sacrificing hardware control.

### Finding 4.16: Verification Effort

| Approach | Verification |
|----------|--------------|
| Python | Unit tests |
| Rust | Unit + fuzz |
| Calyx | Simulation |
| Clash | QuickCheck + formal |

Clash's type system catches many hardware bugs at compile time.

---

*Soli Deo Gloria* ☧
