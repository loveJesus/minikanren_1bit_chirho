# PoC 1: SyGuS Program Synthesis Benchmark ☧

## Goal

Solve a real SyGuS (Syntax-Guided Synthesis) competition problem faster than existing tools, demonstrating practical value of our 1-bit representation.

---

## Target Problem: Conditional Max

**SyGuS Specification:**
```lisp
(synth-fun max2 ((x Int) (y Int)) Int
  ((Start Int) (StartBool Bool))
  ((Start Int (x y (ite StartBool Start Start)))
   (StartBool Bool ((<= x y) (<= y x)))))

(constraint (= (max2 0 1) 1))
(constraint (= (max2 1 0) 1))
(constraint (= (max2 3 5) 5))
```

**Expected Output:**
```lisp
(define-fun max2 ((x Int) (y Int)) Int
  (ite (<= x y) y x))
```

---

## Implementation Plan

### Step 1: Grammar as Domain

Encode grammar productions as domain values:

```rust
// Grammar productions for max2
// Terminal: x → ID 0
// Terminal: y → ID 1
// ITE: ite(b, t, e) → ID 2 + (b * 4 + t * 2 + e)
// LE: (<= a b) → ID 100 + (a * 2 + b)

struct GrammarDomainChirho {
    /// Valid productions at each AST position
    valid_chirho: Hierarchical4kChirho,
}
```

### Step 2: Constraint Propagation

For each I/O example, propagate constraints:

```rust
fn propagate_example_chirho(
    grammar_chirho: &GrammarDomainChirho,
    input_chirho: (i32, i32),
    output_chirho: i32,
) -> GrammarDomainChirho {
    // Top-down: output type must be Int
    // Bottom-up: evaluate partial programs, prune invalid
}
```

### Step 3: Search with Domains

```rust
fn synthesize_chirho(
    spec_chirho: &SygusSpecChirho,
) -> Option<ExprChirho> {
    let mut domains_chirho = initialize_domains_chirho(&spec_chirho.grammar);

    // Propagate all examples
    for (input_chirho, output_chirho) in &spec_chirho.examples {
        domains_chirho = propagate_example_chirho(&domains_chirho, *input_chirho, *output_chirho);
    }

    // If any domain empty → UNSAT
    if domains_chirho.any_empty_chirho() {
        return None;
    }

    // Enumerate from pruned domain
    enumerate_from_domain_chirho(&domains_chirho)
}
```

---

## Baseline Comparisons

| Tool | Type | Expected Time | Notes |
|------|------|---------------|-------|
| CVC5 | SMT-based SyGuS | ~100ms | State of the art |
| EUSolver | Enumerative | ~50ms | Good on small grammars |
| Our approach | Domain-pruned | Target: <10ms | Bit-parallel filtering |

---

## Files to Create

| File | Purpose |
|------|---------|
| `rust_chirho/src/sygus_chirho.rs` | SyGuS parser and representation |
| `rust_chirho/src/synthesis_chirho.rs` | Domain-based synthesis engine |
| `rust_chirho/benches/sygus_bench_chirho.rs` | Comparison benchmarks |
| `benchmarks_chirho/sygus/max2.sl` | Test problem in SyGuS format |

---

## Success Criteria

1. **Correctness:** Synthesizes correct `max2` function
2. **Performance:** <10ms (10× faster than CVC5)
3. **Scalability:** Works on 5+ SyGuS benchmarks from competition

---

## SyGuS Benchmark Sources

- SyGuS-Comp benchmarks: https://sygus.org/comp/
- Specific tracks:
  - **CLIA** (Conditional Linear Integer Arithmetic) - good fit
  - **INV** (Invariant synthesis) - stretch goal
  - **PBE** (Programming by Example) - very good fit

---

*Soli Deo Gloria* ☧
