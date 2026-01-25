# PoC 2: OCanren/faster-miniKanren Comparison ☧

## Goal

Direct benchmark comparison with optimized miniKanren implementations to validate our speedup claims.

---

## Comparison Targets

### 1. OCanren (OCaml)
- Repository: https://github.com/JetBrains-Research/OCanren
- State-of-the-art typed relational programming
- Compilation to OCaml for performance

### 2. faster-miniKanren (Scheme)
- Repository: https://github.com/michaelballantyne/faster-miniKanren
- Optimized Scheme implementation
- Used in academic benchmarks

### 3. core.logic (Clojure)
- Repository: https://github.com/clojure/core.logic
- Popular in production
- Constraint logic programming extensions

### 4. miniKanren.rs (Rust)
- Various Rust implementations
- Direct comparison (same language)

---

## Benchmark Suite

### Benchmark 1: appendo

```
(run* (q) (appendo '(1 2 3) '(4 5 6) q))
→ ((1 2 3 4 5 6))

(run* (q) (fresh (a b) (appendo a b '(1 2 3 4 5)) (== q (list a b))))
→ All 6 splits
```

| Implementation | Expected Time | Our Target |
|----------------|---------------|------------|
| OCanren | ~1ms | <0.1ms |
| faster-miniKanren | ~2ms | <0.1ms |
| Ours (1-bit) | — | <0.1ms |

### Benchmark 2: N-Queens

```
(run* (q) (n-queens 8 q))
→ 92 solutions
```

| Implementation | Expected Time | Our Target |
|----------------|---------------|------------|
| OCanren | ~50ms | <10ms |
| core.logic | ~100ms | <10ms |
| Ours (1-bit) | — | <5ms |

### Benchmark 3: Type Inference (Simply Typed Lambda)

```
(run* (t) (type-of '(lambda (x) (lambda (y) (x y))) t))
→ ((→ (→ A B) (→ A B)))
```

| Implementation | Expected Time | Our Target |
|----------------|---------------|------------|
| OCanren | ~10ms | <1ms |
| Ours (1-bit) | — | <1ms |

### Benchmark 4: Quine Generation

```
(run 1 (q) (eval-expo q '() q))
→ First quine
```

| Implementation | Expected Time | Our Target |
|----------------|---------------|------------|
| OCanren | ~500ms | <100ms |
| faster-miniKanren | ~200ms | <100ms |
| Ours (1-bit) | — | <50ms |

---

## Implementation Plan

### Step 1: Install Comparison Systems

```bash
# OCanren
opam install OCanren

# faster-miniKanren
git clone https://github.com/michaelballantyne/faster-miniKanren
# Requires Chez Scheme

# core.logic
# Add to deps.edn: org.clojure/core.logic {:mvn/version "1.0.1"}
```

### Step 2: Port Benchmarks

Create equivalent implementations:

```
benchmarks_chirho/
├── appendo/
│   ├── ocanren_chirho.ml
│   ├── faster_chirho.scm
│   ├── corelogic_chirho.clj
│   └── ours_chirho.rs
├── nqueens/
│   └── ...
├── typecheck/
│   └── ...
└── quine/
    └── ...
```

### Step 3: Measurement Script

```bash
#!/bin/bash
# benchmark_all_chirho.sh

echo "=== appendo benchmark ==="
echo "OCanren:"
time ./ocanren_appendo_chirho
echo "faster-miniKanren:"
time chez --script faster_appendo_chirho.scm
echo "Ours:"
cargo bench --bench appendo_bench_chirho

# ... repeat for other benchmarks
```

---

## Files to Create

| File | Purpose |
|------|---------|
| `benchmarks_chirho/ocanren/` | OCanren benchmark implementations |
| `benchmarks_chirho/faster/` | faster-miniKanren implementations |
| `benchmarks_chirho/scripts/run_comparison_chirho.sh` | Automated comparison |
| `rust_chirho/benches/comparison_bench_chirho.rs` | Our implementations |

---

## Success Criteria

1. **Apples-to-apples:** Same problems, same hardware
2. **Speedup:** Demonstrate ≥10× on at least 2 benchmarks
3. **Correctness:** Verify output equivalence

---

## Expected Results Table (for paper)

| Benchmark | OCanren | faster-mk | core.logic | Ours | Speedup |
|-----------|---------|-----------|------------|------|---------|
| appendo (backward) | 2ms | 3ms | 5ms | 0.1ms | **20-50×** |
| N-Queens 8 | 50ms | 80ms | 100ms | 3ms | **15-30×** |
| Type infer | 10ms | 15ms | 20ms | 0.5ms | **20-40×** |
| Quine gen | 500ms | 200ms | 1s | 30ms | **7-30×** |

---

*Soli Deo Gloria* ☧
