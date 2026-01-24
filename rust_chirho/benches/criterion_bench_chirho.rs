//! Criterion Benchmarks for miniKanren 1-bit ☧
//!
//! Standard benchmark suite comparing to state-of-the-art.
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::*;

/// Benchmark core bit operations (these map to single CPU instructions)
fn bench_bit_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BitOps");

    group_chirho.bench_function("BitVec64_AND", |b| {
        let a_chirho = BitVec64Chirho(0xFFFF_0000_FFFF_0000);
        let b_chirho = BitVec64Chirho(0x0F0F_0F0F_0F0F_0F0F);
        b.iter(|| black_box(a_chirho.and_chirho(b_chirho)))
    });

    group_chirho.bench_function("BitVec64_popcount", |b| {
        let d_chirho = BitVec64Chirho(0xDEAD_BEEF_CAFE_BABE);
        b.iter(|| black_box(d_chirho.popcount_chirho()))
    });

    group_chirho.bench_function("BitVec256_AND", |b| {
        let a_chirho = BitVec256Chirho([0xFFFF_FFFF; 4]);
        let b_chirho = BitVec256Chirho([0x0F0F_0F0F; 4]);
        b.iter(|| black_box(a_chirho.and_chirho(b_chirho)))
    });

    group_chirho.finish();
}

/// Benchmark union-find operations
fn bench_union_find_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("UnionFind");

    for size_chirho in [10, 100, 1000].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("chain", size_chirho),
            size_chirho,
            |b, &n| {
                b.iter(|| {
                    let mut uf_chirho = UnionFindChirho::new();
                    for i_chirho in 0..n {
                        uf_chirho.make_set_chirho(i_chirho);
                    }
                    for i_chirho in 0..(n - 1) {
                        uf_chirho.union_chirho(i_chirho, i_chirho + 1);
                    }
                    black_box(uf_chirho.find_chirho(0))
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark unification
fn bench_unify_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Unify");

    group_chirho.bench_function("equal_ints", |b| {
        let mut store_chirho = TermStoreChirho::new();
        let a_chirho = store_chirho.int_chirho(42);
        let b_term_chirho = store_chirho.int_chirho(42);
        b.iter(|| {
            let mut subst_chirho = SubstChirho::new();
            black_box(unify_chirho(a_chirho, b_term_chirho, &mut subst_chirho, &store_chirho))
        })
    });

    group_chirho.bench_function("var_to_int", |b| {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let val_chirho = store_chirho.int_chirho(42);
        b.iter(|| {
            let mut subst_chirho = SubstChirho::new();
            black_box(unify_chirho(x_chirho, val_chirho, &mut subst_chirho, &store_chirho))
        })
    });

    for len_chirho in [5, 10, 20].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("equal_lists", len_chirho),
            len_chirho,
            |b, &n| {
                let mut store_chirho = TermStoreChirho::new();
                let vals_chirho: Vec<i64> = (0..n as i64).collect();
                let list1_chirho = store_chirho.list_ints_chirho(&vals_chirho);
                let list2_chirho = store_chirho.list_ints_chirho(&vals_chirho);
                b.iter(|| {
                    let mut subst_chirho = SubstChirho::new();
                    black_box(unify_chirho(list1_chirho, list2_chirho, &mut subst_chirho, &store_chirho))
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark goal execution
fn bench_goals_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Goals");

    for branches_chirho in [3, 10, 30].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("conde", branches_chirho),
            branches_chirho,
            |b, &n| {
                let mut store_chirho = TermStoreChirho::new();
                let (_, x_chirho) = store_chirho.fresh_var_chirho();
                let vals_chirho: Vec<_> = (0..n as i64).map(|i| store_chirho.int_chirho(i)).collect();
                let clauses_chirho: Vec<Vec<_>> = vals_chirho.iter().map(|&v| vec![eq_chirho(x_chirho, v)]).collect();
                let goal_chirho = conde_chirho(clauses_chirho);
                b.iter(|| {
                    black_box(run_chirho(n as usize * 2, x_chirho, goal_chirho.clone(), &store_chirho))
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark constraint propagation (AC-3)
fn bench_constraint_prop_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ConstraintProp");

    for vars_chirho in [4, 8, 16].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("AC3_chain", vars_chirho),
            vars_chirho,
            |b, &n| {
                b.iter(|| {
                    let mut cstore_chirho = ConstraintStoreChirho::new();
                    for i_chirho in 0..n {
                        let domain_chirho = if i_chirho == 0 {
                            0b1111_1111
                        } else {
                            0b0011_1111 >> (i_chirho % 4)
                        };
                        cstore_chirho.add_var_chirho(i_chirho as u32, DomainChirho(domain_chirho));
                    }
                    for i_chirho in 0..(n - 1) {
                        cstore_chirho.add_constraint_chirho(
                            BinaryConstraintChirho::equality_chirho(i_chirho as u32, (i_chirho + 1) as u32, 8)
                        );
                    }
                    black_box(cstore_chirho.propagate_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark hardware-oriented state operations
fn bench_hw_state_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("HardwareState");

    group_chirho.bench_function("SearchStateHw8_unify", |b| {
        b.iter(|| {
            let mut state_chirho = SearchStateHwChirho::<8>::new_chirho();
            state_chirho.set_domain_chirho(0, BitVec64Chirho(0xFF00));
            state_chirho.set_domain_chirho(1, BitVec64Chirho(0x0FF0));
            state_chirho.unify_chirho(0, 1);
            black_box(state_chirho.domains_chirho[0])
        })
    });

    group_chirho.bench_function("SearchState256Hw8_unify", |b| {
        b.iter(|| {
            let mut state_chirho = SearchState256HwChirho::<8>::new_chirho();
            state_chirho.set_domain_chirho(0, BitVec256Chirho([0xFF00, 0, 0, 0]));
            state_chirho.set_domain_chirho(1, BitVec256Chirho([0x0FF0, 0, 0, 0]));
            state_chirho.unify_chirho(0, 1);
            black_box(state_chirho.domains_chirho[0])
        })
    });

    group_chirho.bench_function("CAM64_lookup", |b| {
        let mut cam_chirho = CamHwChirho::<64>::new_chirho();
        for i_chirho in 0..60u32 {
            cam_chirho.insert_chirho(i_chirho % 10, i_chirho);
        }
        b.iter(|| black_box(cam_chirho.lookup_chirho(5)))
    });

    group_chirho.finish();
}

/// Benchmark semiring matrix operations
fn bench_semiring_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Semiring");

    for size_chirho in [5, 10, 20].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("matmul_count", size_chirho),
            size_chirho,
            |b, &n| {
                let mut a_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(n, n);
                let mut b_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(n, n);
                for i in 0..n as u32 {
                    for j in 0..n as u32 {
                        if (i + j) % 3 == 0 {
                            a_chirho.set_chirho(i, j, CountSemiringChirho(1));
                        }
                        if (i * j) % 2 == 0 {
                            b_chirho.set_chirho(i, j, CountSemiringChirho(1));
                        }
                    }
                }
                b.iter(|| black_box(a_chirho.matmul_chirho(&b_chirho)))
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark neural/soft domain operations
fn bench_neural_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Neural");

    group_chirho.bench_function("SoftDomain_unify_10", |b| {
        let a_chirho = SoftDomainChirho::uniform_chirho(10);
        let b_soft_chirho = SoftDomainChirho::uniform_chirho(10);
        b.iter(|| black_box(a_chirho.unify_chirho(&b_soft_chirho)))
    });

    group_chirho.bench_function("beam_search_2x4", |b| {
        let initial_chirho = NeuralStateChirho::new_chirho(2, 4);
        let heuristic_chirho = NeuralHeuristicChirho::uniform_chirho(2, 4);
        b.iter(|| black_box(beam_search_chirho(initial_chirho.clone(), &heuristic_chirho, 4, 10)))
    });

    group_chirho.finish();
}

/// Benchmark Sudoku solver (practical application)
fn bench_sudoku_chirho(c_chirho: &mut Criterion) {
    use minikanren_1bit_chirho::sudoku_chirho::{SudokuSolverChirho, puzzles_chirho};

    let mut group_chirho = c_chirho.benchmark_group("Sudoku");

    // Fast solver (basic propagation only)
    group_chirho.bench_function("fast/easy", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::EASY_CHIRHO);
            black_box(solver_chirho.solve_fast_chirho())
        })
    });

    group_chirho.bench_function("fast/hard_17clue", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::HARD_CHIRHO);
            black_box(solver_chirho.solve_fast_chirho())
        })
    });

    // Full solver (with hidden singles + naked pairs)
    group_chirho.bench_function("full/easy", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::EASY_CHIRHO);
            black_box(solver_chirho.solve_chirho())
        })
    });

    group_chirho.bench_function("full/hard_17clue", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::HARD_CHIRHO);
            black_box(solver_chirho.solve_chirho())
        })
    });

    // Adaptive solver (best of both)
    group_chirho.bench_function("adaptive/easy", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::EASY_CHIRHO);
            black_box(solver_chirho.solve_adaptive_chirho())
        })
    });

    group_chirho.bench_function("adaptive/hard_17clue", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::HARD_CHIRHO);
            black_box(solver_chirho.solve_adaptive_chirho())
        })
    });

    group_chirho.bench_function("adaptive/escargot", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = SudokuSolverChirho::new_chirho();
            solver_chirho.load_puzzle_chirho(puzzles_chirho::ESCARGOT_CHIRHO);
            black_box(solver_chirho.solve_adaptive_chirho())
        })
    });

    group_chirho.finish();
}

/// Benchmark N-Queens solver
fn bench_nqueens_chirho(c_chirho: &mut Criterion) {
    use minikanren_1bit_chirho::nqueens_chirho::NQueensSolverChirho;

    let mut group_chirho = c_chirho.benchmark_group("NQueens");

    group_chirho.bench_function("8_count", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = NQueensSolverChirho::new_chirho(8);
            black_box(solver_chirho.count_solutions_chirho())
        })
    });

    group_chirho.bench_function("12_count", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = NQueensSolverChirho::new_chirho(12);
            black_box(solver_chirho.count_solutions_chirho())
        })
    });

    group_chirho.bench_function("14_count", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = NQueensSolverChirho::new_chirho(14);
            black_box(solver_chirho.count_solutions_chirho())
        })
    });

    group_chirho.bench_function("20_one", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = NQueensSolverChirho::new_chirho(20);
            black_box(solver_chirho.solve_one_chirho())
        })
    });

    group_chirho.bench_function("32_one", |b_chirho| {
        b_chirho.iter(|| {
            let mut solver_chirho = NQueensSolverChirho::new_chirho(32);
            black_box(solver_chirho.solve_one_chirho())
        })
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_bit_ops_chirho,
    bench_union_find_chirho,
    bench_unify_chirho,
    bench_goals_chirho,
    bench_constraint_prop_chirho,
    bench_hw_state_chirho,
    bench_semiring_chirho,
    bench_neural_chirho,
    bench_sudoku_chirho,
    bench_nqueens_chirho,
);

criterion_main!(benches_chirho);
