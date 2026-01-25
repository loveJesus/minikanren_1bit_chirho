# PoC 3: GPU Acceleration Benchmarks ☧

## Goal

Demonstrate GPU acceleration for constraint propagation and batch unification, showing that the 1-bit representation maps naturally to GPU compute.

---

## GPU Architecture Mapping

### Why GPU for miniKanren?

| Operation | CPU | GPU | Why GPU Wins |
|-----------|-----|-----|--------------|
| Single unification | 1 op | 1 op | No advantage |
| 1000 parallel unifications | 1000 ops | 1 op (batch) | **1000× parallelism** |
| Constraint propagation | Sequential | Parallel | **All constraints at once** |
| Search branching | Stream interleave | All branches parallel | **Exponential speedup** |

### Key Insight

Our bitmask representation is perfect for GPU:
- `BitVec64` = single u64 → GPU has u32/u64 native
- Domain intersection = bitwise AND → Single GPU instruction
- Batch operation = SIMT model

---

## Implementation Plan

### Step 1: WebGPU Shader for Domain Intersection

```wgsl
// domain_intersect_chirho.wgsl

@group(0) @binding(0) var<storage, read> domains_a_chirho: array<u64>;
@group(0) @binding(1) var<storage, read> domains_b_chirho: array<u64>;
@group(0) @binding(2) var<storage, read_write> result_chirho: array<u64>;

@compute @workgroup_size(256)
fn main_chirho(@builtin(global_invocation_id) gid_chirho: vec3<u32>) {
    let idx_chirho = gid_chirho.x;
    result_chirho[idx_chirho] = domains_a_chirho[idx_chirho] & domains_b_chirho[idx_chirho];
}
```

### Step 2: Batch Unification Kernel

```wgsl
// batch_unify_chirho.wgsl

struct UnifyTaskChirho {
    term_a_chirho: u32,
    term_b_chirho: u32,
    subst_base_chirho: u32,
}

@group(0) @binding(0) var<storage, read> tasks_chirho: array<UnifyTaskChirho>;
@group(0) @binding(1) var<storage, read> term_store_chirho: array<u32>;
@group(0) @binding(2) var<storage, read_write> results_chirho: array<u32>;

@compute @workgroup_size(256)
fn main_chirho(@builtin(global_invocation_id) gid_chirho: vec3<u32>) {
    let task_chirho = tasks_chirho[gid_chirho.x];

    // Walk terms (simplified - real impl needs loop)
    let a_chirho = walk_chirho(task_chirho.term_a_chirho, task_chirho.subst_base_chirho);
    let b_chirho = walk_chirho(task_chirho.term_b_chirho, task_chirho.subst_base_chirho);

    // Unify
    results_chirho[gid_chirho.x] = unify_terms_chirho(a_chirho, b_chirho);
}
```

### Step 3: Constraint Propagation (Arc Consistency)

```wgsl
// arc_consistency_chirho.wgsl

// Each thread handles one variable's domain
// Iterate until fixpoint

@compute @workgroup_size(64)
fn propagate_chirho(@builtin(global_invocation_id) gid_chirho: vec3<u32>) {
    let var_id_chirho = gid_chirho.x;

    // Read current domain
    var domain_chirho = domains_chirho[var_id_chirho];

    // Apply constraints involving this variable
    for (var c_chirho = 0u; c_chirho < num_constraints_chirho; c_chirho++) {
        if (constraint_involves_chirho(c_chirho, var_id_chirho)) {
            domain_chirho = apply_constraint_chirho(c_chirho, var_id_chirho, domain_chirho);
        }
    }

    domains_chirho[var_id_chirho] = domain_chirho;
}
```

---

## Benchmark Suite

### Benchmark 1: Batch Domain Intersection

```rust
// rust_chirho/benches/gpu_bench_chirho.rs

fn bench_gpu_intersection_chirho(c_chirho: &mut Criterion) {
    for batch_size_chirho in [1000, 10000, 100000, 1000000] {
        // CPU baseline (sequential)
        group_chirho.bench_with_input(
            BenchmarkId::new("CPU_sequential", batch_size_chirho),
            |bench_chirho| { /* ... */ }
        );

        // CPU SIMD (parallel within registers)
        group_chirho.bench_with_input(
            BenchmarkId::new("CPU_SIMD", batch_size_chirho),
            |bench_chirho| { /* ... */ }
        );

        // GPU (massively parallel)
        group_chirho.bench_with_input(
            BenchmarkId::new("GPU_WebGPU", batch_size_chirho),
            |bench_chirho| { /* ... */ }
        );
    }
}
```

### Benchmark 2: Sudoku Solving (GPU vs CPU)

```rust
fn bench_sudoku_gpu_chirho(c_chirho: &mut Criterion) {
    let puzzles_chirho = load_puzzles_chirho(1000);

    // CPU: solve one at a time
    group_chirho.bench_function("CPU_serial", |bench_chirho| {
        for puzzle_chirho in &puzzles_chirho {
            solve_sudoku_chirho(puzzle_chirho);
        }
    });

    // GPU: solve all in parallel
    group_chirho.bench_function("GPU_batch", |bench_chirho| {
        solve_sudoku_batch_gpu_chirho(&puzzles_chirho);
    });
}
```

### Benchmark 3: N-Queens (Parallel Branch Exploration)

```rust
fn bench_nqueens_gpu_chirho(c_chirho: &mut Criterion) {
    // CPU: explore branches sequentially/interleaved
    // GPU: explore all branches in parallel

    for n_chirho in [8, 10, 12, 14] {
        // CPU
        group_chirho.bench_with_input(
            BenchmarkId::new("CPU", n_chirho),
            |bench_chirho| solve_nqueens_cpu_chirho(n_chirho)
        );

        // GPU
        group_chirho.bench_with_input(
            BenchmarkId::new("GPU", n_chirho),
            |bench_chirho| solve_nqueens_gpu_chirho(n_chirho)
        );
    }
}
```

---

## Expected Results

| Benchmark | CPU | GPU | Speedup | Notes |
|-----------|-----|-----|---------|-------|
| 1M intersections | 10ms | 0.1ms | **100×** | Perfect parallelism |
| 1000 Sudoku | 50ms | 1ms | **50×** | Batch constraint prop |
| N-Queens 12 | 500ms | 20ms | **25×** | Parallel branching |
| N-Queens 14 | 30s | 0.5s | **60×** | More branches = more GPU advantage |

---

## Files to Create

| File | Purpose |
|------|---------|
| `rust_chirho/src/gpu_chirho/shaders/domain_intersect_chirho.wgsl` | Domain intersection shader |
| `rust_chirho/src/gpu_chirho/shaders/batch_unify_chirho.wgsl` | Batch unification shader |
| `rust_chirho/src/gpu_chirho/shaders/arc_consistency_chirho.wgsl` | Constraint propagation |
| `rust_chirho/src/gpu_chirho/pipeline_chirho.rs` | WebGPU pipeline setup |
| `rust_chirho/benches/gpu_bench_chirho.rs` | GPU benchmarks |

---

## Hardware Requirements

- Any GPU with WebGPU support (Vulkan/Metal/DX12 backend)
- Tested on: Apple M1, NVIDIA GTX 1070+, AMD RX 580+
- WebGPU in browser also works (for demo purposes)

---

## Success Criteria

1. **Working GPU kernels:** Correctly compute domain intersections
2. **Speedup:** ≥10× for batch size ≥10k
3. **Scalability:** Near-linear with batch size (until GPU saturates)

---

*Soli Deo Gloria* ☧
