//! Benchmarks for miniKanren goal implementations ☧
//!
//! Compares three implementations:
//! - **hardware_chirho**: Bit-parallel goals (THE 1-BIT MINIKANREN)
//! - **reference_chirho**: Traditional stream-based goals
//! - **goal_ast_chirho**: AST-based goals (feature-gated)
//!
//! Run with: cargo bench --bench goal_bench_chirho
//! For AST comparison: cargo bench --bench goal_bench_chirho --features goal_ast_chirho

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Hardware (1-bit) implementation
use minikanren_1bit_chirho::hardware_chirho::{
    eq_hw_chirho, conde_hw_chirho, conj_hw_chirho, run_hw_chirho, BitVec64Chirho,
};

// Reference (stream) implementation
use minikanren_1bit_chirho::reference_chirho::{
    goals_chirho::{eq_chirho, conj_chirho, conde_chirho, run_chirho},
    terms_chirho::TermStoreChirho,
};

// AST-based (optional)
#[cfg(feature = "goal_ast_chirho")]
use minikanren_1bit_chirho::experimental_chirho::goal_ast_chirho::{
    conde_ast_chirho, conj_ast_chirho, disj_ast_chirho, eq_ast_chirho, run_ast_chirho,
    GoalAstChirho,
};

/// Benchmark simple unification: x == 42
fn bench_simple_unify_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SimpleUnifyChirho");

    // Hardware (1-bit): x ∈ {42}
    group_chirho.bench_function("hardware_1bit", |bench_chirho| {
        bench_chirho.iter(|| {
            let goal_chirho = eq_hw_chirho::<4>(0, BitVec64Chirho(1u64 << 42));
            let results_chirho = run_hw_chirho(10, 0, goal_chirho);
            black_box(results_chirho)
        })
    });

    // Reference (streams)
    group_chirho.bench_function("reference_streams", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut store_chirho = TermStoreChirho::new();
            let (_, x_chirho) = store_chirho.fresh_var_chirho();
            let forty_two_chirho = store_chirho.int_chirho(42);
            let goal_chirho = eq_chirho(x_chirho, forty_two_chirho);
            let results_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);
            black_box(results_chirho)
        })
    });

    // AST-based (if feature enabled)
    #[cfg(feature = "goal_ast_chirho")]
    group_chirho.bench_function("ast", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut store_chirho = TermStoreChirho::new();
            let (_, x_chirho) = store_chirho.fresh_var_chirho();
            let forty_two_chirho = store_chirho.int_chirho(42);
            let goal_chirho = eq_ast_chirho(x_chirho, forty_two_chirho);
            let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);
            black_box(results_chirho)
        })
    });

    group_chirho.finish();
}

/// Benchmark conjunction chain: x == 1, y == 2, z == 3, ...
fn bench_conj_chain_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ConjChainChirho");

    for chain_len_chirho in [2, 4, 8, 16] {
        // Hardware (1-bit): chain of domain constraints
        group_chirho.bench_with_input(
            BenchmarkId::new("hardware_1bit", chain_len_chirho),
            &chain_len_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    // Build chain: var0 == 0, var1 == 1, var2 == 2, ...
                    let mut goal_chirho = eq_hw_chirho::<16>(0, BitVec64Chirho(1u64 << 0));
                    for i_chirho in 1..n_chirho {
                        goal_chirho = conj_hw_chirho(
                            goal_chirho,
                            eq_hw_chirho(i_chirho, BitVec64Chirho(1u64 << i_chirho)),
                        );
                    }
                    let results_chirho = run_hw_chirho(10, 0, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // Reference (streams)
        group_chirho.bench_with_input(
            BenchmarkId::new("reference_streams", chain_len_chirho),
            &chain_len_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let vars_chirho: Vec<_> = (0..n_chirho)
                        .map(|_| store_chirho.fresh_var_chirho().1)
                        .collect();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let mut goal_chirho = eq_chirho(vars_chirho[0], vals_chirho[0]);
                    for i_chirho in 1..n_chirho {
                        goal_chirho = conj_chirho(
                            goal_chirho,
                            eq_chirho(vars_chirho[i_chirho], vals_chirho[i_chirho]),
                        );
                    }

                    let results_chirho = run_chirho(10, vars_chirho[0], goal_chirho, &store_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // AST-based (if feature enabled)
        #[cfg(feature = "goal_ast_chirho")]
        group_chirho.bench_with_input(
            BenchmarkId::new("ast", chain_len_chirho),
            &chain_len_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let vars_chirho: Vec<_> = (0..n_chirho)
                        .map(|_| store_chirho.fresh_var_chirho().1)
                        .collect();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let mut goal_chirho = eq_ast_chirho(vars_chirho[0], vals_chirho[0]);
                    for i_chirho in 1..n_chirho {
                        goal_chirho = conj_ast_chirho(
                            goal_chirho,
                            eq_ast_chirho(vars_chirho[i_chirho], vals_chirho[i_chirho]),
                        );
                    }

                    let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark disjunction (conde): x == 0 | x == 1 | x == 2 | ...
fn bench_conde_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("CondeChirho");

    for num_branches_chirho in [2, 4, 8, 16] {
        // Hardware (1-bit): each branch sets domain to single value
        group_chirho.bench_with_input(
            BenchmarkId::new("hardware_1bit", num_branches_chirho),
            &num_branches_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let branches_chirho: Vec<Vec<_>> = (0..n_chirho)
                        .map(|i_chirho| vec![eq_hw_chirho::<4>(0, BitVec64Chirho(1u64 << i_chirho))])
                        .collect();
                    let goal_chirho = conde_hw_chirho(branches_chirho);
                    let results_chirho = run_hw_chirho(n_chirho + 5, 0, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // Reference (streams)
        group_chirho.bench_with_input(
            BenchmarkId::new("reference_streams", num_branches_chirho),
            &num_branches_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let (_, x_chirho) = store_chirho.fresh_var_chirho();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let branches_chirho: Vec<Vec<_>> = vals_chirho
                        .iter()
                        .map(|&v_chirho| vec![eq_chirho(x_chirho, v_chirho)])
                        .collect();
                    let goal_chirho = conde_chirho(branches_chirho);

                    let results_chirho = run_chirho(n_chirho + 5, x_chirho, goal_chirho, &store_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // AST-based with conde (if feature enabled)
        #[cfg(feature = "goal_ast_chirho")]
        group_chirho.bench_with_input(
            BenchmarkId::new("ast", num_branches_chirho),
            &num_branches_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let (_, x_chirho) = store_chirho.fresh_var_chirho();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let clauses_chirho: Vec<Vec<GoalAstChirho>> = vals_chirho
                        .iter()
                        .map(|&v_chirho| vec![eq_ast_chirho(x_chirho, v_chirho)])
                        .collect();
                    let goal_chirho = conde_ast_chirho(clauses_chirho);

                    let results_chirho = run_ast_chirho(n_chirho + 5, store_chirho, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark domain intersection (the core 1-bit operation)
fn bench_domain_intersection_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("DomainIntersectionChirho");

    // Hardware: multiple domain constraints on same variable
    // x ∈ {0..31} AND x ∈ {16..47} → x ∈ {16..31}
    group_chirho.bench_function("hardware_1bit", |bench_chirho| {
        bench_chirho.iter(|| {
            let goal_chirho = conj_hw_chirho::<4>(
                eq_hw_chirho(0, BitVec64Chirho(0xFFFF_FFFF)), // bits 0-31
                eq_hw_chirho(0, BitVec64Chirho(0xFFFF_FFFF_0000)), // bits 16-47
            );
            let results_chirho = run_hw_chirho(64, 0, goal_chirho);
            black_box(results_chirho)
        })
    });

    // Reference: no direct equivalent (would need constraint store)
    // The hardware version is THE point - direct bit intersection

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_simple_unify_chirho,
    bench_conj_chain_chirho,
    bench_conde_chirho,
    bench_domain_intersection_chirho,
);

criterion_main!(benches_chirho);
