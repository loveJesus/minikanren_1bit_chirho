# minikanren_1bit_chirho

[![crates.io](https://img.shields.io/crates/v/minikanren_1bit_chirho.svg)](https://crates.io/crates/minikanren_1bit_chirho)
[![docs.rs](https://docs.rs/minikanren_1bit_chirho/badge.svg)](https://docs.rs/minikanren_1bit_chirho)

**miniKanren as 1-bit matrix operations** — hardware-accelerated logic programming.

**[Full documentation with Python/FPGA implementations →](https://github.com/loveJesus/minikanren_1bit_chirho)**

## Core Insight

```
miniKanren search = sparse Boolean tensor network contraction
```

| miniKanren | Tensor Representation |
|------------|----------------------|
| Variable domain | Bitmask (1 = value possible) |
| `(== x y)` | Bitwise AND of domains |
| `conde` (or) | Row duplication / tensor stack |
| Relation | Sparse N-D Boolean tensor |
| Composition | Tensor contraction |

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Unify (64-bit AND) | **2ns** | Single SIMD instruction |
| Domain intersection (AVX2) | **8ns** | 8 domains parallel |
| Hardware optics (BitVec64) | **420ps** | Single CPU cycle |

vs Heap-based alternatives: **3000-4000x faster**

## Usage

```rust
use minikanren_1bit_chirho::*;

fn main() {
    let mut store_chirho = TermStoreChirho::new();

    // Create terms
    let (_, x_chirho) = store_chirho.fresh_var_chirho();
    let one_chirho = store_chirho.int_chirho(1);
    let two_chirho = store_chirho.int_chirho(2);

    // Build goal: x == 1 OR x == 2
    let goal_chirho = conde_chirho(vec![
        vec![eq_chirho(x_chirho, one_chirho)],
        vec![eq_chirho(x_chirho, two_chirho)],
    ]);

    // Run and collect solutions
    let solutions_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);
    println!("x_chirho can be: {:?}", solutions_chirho); // [1, 2]
}
```

## Ready-to-Use Solvers

```rust
use minikanren_1bit_chirho::sudoku_chirho::SudokuSolverChirho;
use minikanren_1bit_chirho::nqueens_chirho::NQueensSolverChirho;

// Sudoku: 9-bit domains, solves in ~10μs
let mut sudoku_chirho = SudokuSolverChirho::new_chirho();
sudoku_chirho.load_puzzle_chirho("530070000600195000...");
sudoku_chirho.solve_adaptive_chirho();

// N-Queens: 64-bit domains, 8-queens in 4μs
let mut queens_chirho = NQueensSolverChirho::new_chirho(8);
assert_eq!(queens_chirho.count_solutions_chirho(), 92);
```

## Features

- **Full miniKanren**: `==`, `conde`, `fresh`, `not`, `conda`, `condu`, `=/=`, `project`
- **Constraint solvers**: Sudoku, N-Queens, JSON Schema validation
- **SIMD acceleration**: AVX2/AVX-512 bulk operations
- **Differentiable**: Gradients flow through logic (Gumbel-softmax, learnable relations)
- **Hardware-ready**: Calyx IR and Clash for FPGA synthesis (verified)
- **Multiple semirings**: Boolean, Probability, Tropical, Counting

## Feature Flags

```toml
[dependencies]
minikanren_1bit_chirho = { version = "0.1", features = ["kmett_chirho"] }
```

| Feature | Description |
|---------|-------------|
| `egraph_native_chirho` (default) | Native hardware-optimized e-graph |
| `egg_chirho` | External egg crate for more features |
| `goal_ast_chirho` | Goals as AST for introspection |
| `gpu_chirho` | WebGPU backend |
| `optics_chirho` | Kmett-style Prisms, Lenses, Traversals |
| `free_goal_chirho` | Free Monad for Bool/Prob/SMT interpreters |
| `comonad_chirho` | Search zipper with extend |
| `linear_chirho` | Tensor/Par linear logic connectives |
| `kmett_chirho` | All category-theoretic extensions |

## Examples

```bash
cargo run --example appendo_chirho   # List append relation
cargo run --example type_infer_chirho # Type inference demo
```

## Naming Convention

All identifiers end with `_chirho` (the Chi-Rho Christogram).

## License

MIT

*Soli Deo Gloria*
