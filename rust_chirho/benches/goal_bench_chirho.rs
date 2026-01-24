//! Benchmarks for goal representations ☧
//!
//! Compare AST-based goals vs closure-based goals.
//!
//! Requires feature: goal_ast_chirho
//! Run with: cargo bench --bench goal_bench_chirho --features goal_ast_chirho

#![cfg(feature = "goal_ast_chirho")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::{
    TermStoreChirho,
    eq_chirho as eq_closure_chirho,
    conj_chirho as conj_closure_chirho,
    disj_chirho as disj_closure_chirho,
    run_chirho as run_closure_chirho,
};
use minikanren_1bit_chirho::goal_ast_chirho::{
    eq_ast_chirho,
    conj_ast_chirho,
    disj_ast_chirho,
    conde_ast_chirho,
    run_ast_chirho,
    goal_size_chirho,
    GoalAstChirho,
};
use std::rc::Rc;

/// Benchmark simple unification: x == 42
fn bench_simple_unify_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Simple Unify");

    // Closure-based
    group_chirho.bench_function("closure", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut store_chirho = TermStoreChirho::new();
            let (_, x_chirho) = store_chirho.fresh_var_chirho();
            let forty_two_chirho = store_chirho.int_chirho(42);
            let goal_chirho = eq_closure_chirho(x_chirho, forty_two_chirho);
            let results_chirho = run_closure_chirho(10, x_chirho, goal_chirho, &store_chirho);
            black_box(results_chirho)
        })
    });

    // AST-based
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

/// Benchmark conjunction chain: x == 1, y == 2, z == 3
fn bench_conj_chain_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Conjunction Chain");

    for chain_len_chirho in [2, 4, 8, 16] {
        // Closure-based
        group_chirho.bench_with_input(
            BenchmarkId::new("closure", chain_len_chirho),
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

                    let mut goal_chirho = eq_closure_chirho(vars_chirho[0], vals_chirho[0]);
                    for i_chirho in 1..n_chirho {
                        goal_chirho = conj_closure_chirho(goal_chirho, eq_closure_chirho(vars_chirho[i_chirho], vals_chirho[i_chirho]));
                    }

                    let results_chirho = run_closure_chirho(10, vars_chirho[0], goal_chirho, &store_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // AST-based
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
                        goal_chirho = conj_ast_chirho(goal_chirho, eq_ast_chirho(vars_chirho[i_chirho], vals_chirho[i_chirho]));
                    }

                    let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark disjunction (conde): x == 1 | x == 2 | x == 3 | ...
fn bench_conde_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Conde (Disjunction)");

    for num_branches_chirho in [2, 4, 8, 16] {
        // Closure-based
        group_chirho.bench_with_input(
            BenchmarkId::new("closure", num_branches_chirho),
            &num_branches_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let (_, x_chirho) = store_chirho.fresh_var_chirho();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let mut goal_chirho = eq_closure_chirho(x_chirho, vals_chirho[0]);
                    for i_chirho in 1..n_chirho {
                        goal_chirho = disj_closure_chirho(goal_chirho, eq_closure_chirho(x_chirho, vals_chirho[i_chirho]));
                    }

                    let results_chirho = run_closure_chirho(n_chirho + 5, x_chirho, goal_chirho, &store_chirho);
                    black_box(results_chirho)
                })
            },
        );

        // AST-based with conde
        group_chirho.bench_with_input(
            BenchmarkId::new("ast_conde", num_branches_chirho),
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

        // AST-based with disj chain
        group_chirho.bench_with_input(
            BenchmarkId::new("ast_disj", num_branches_chirho),
            &num_branches_chirho,
            |bench_chirho, &n_chirho| {
                bench_chirho.iter(|| {
                    let mut store_chirho = TermStoreChirho::new();
                    let (_, x_chirho) = store_chirho.fresh_var_chirho();
                    let vals_chirho: Vec<_> = (0..n_chirho)
                        .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
                        .collect();

                    let mut goal_chirho = eq_ast_chirho(x_chirho, vals_chirho[0]);
                    for i_chirho in 1..n_chirho {
                        goal_chirho = disj_ast_chirho(goal_chirho, eq_ast_chirho(x_chirho, vals_chirho[i_chirho]));
                    }

                    let results_chirho = run_ast_chirho(n_chirho + 5, store_chirho, goal_chirho);
                    black_box(results_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

/// Benchmark goal introspection (AST-only capability)
fn bench_goal_analysis_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Goal Analysis (AST only)");

    for size_chirho in [10, 50, 100] {
        // Build a goal tree
        let mut store_chirho = TermStoreChirho::new();
        let vars_chirho: Vec<_> = (0..size_chirho)
            .map(|_| store_chirho.fresh_var_chirho().1)
            .collect();
        let vals_chirho: Vec<_> = (0..size_chirho)
            .map(|i_chirho| store_chirho.int_chirho(i_chirho as i64))
            .collect();

        // Build deep tree: (eq & eq) | (eq & eq) | ...
        let pairs_chirho: Vec<GoalAstChirho> = (0..size_chirho / 2)
            .map(|i_chirho| {
                GoalAstChirho::ConjChirho(
                    Rc::new(eq_ast_chirho(vars_chirho[2 * i_chirho], vals_chirho[2 * i_chirho])),
                    Rc::new(eq_ast_chirho(vars_chirho[2 * i_chirho + 1], vals_chirho[2 * i_chirho + 1])),
                )
            })
            .collect();

        let mut goal_chirho = pairs_chirho[0].clone();
        for pair_chirho in pairs_chirho.iter().skip(1) {
            goal_chirho = GoalAstChirho::DisjChirho(Rc::new(goal_chirho), Rc::new(pair_chirho.clone()));
        }

        group_chirho.bench_with_input(
            BenchmarkId::new("goal_size", size_chirho),
            &goal_chirho,
            |bench_chirho, g_chirho| {
                bench_chirho.iter(|| black_box(goal_size_chirho(black_box(g_chirho))))
            },
        );
    }

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_simple_unify_chirho,
    bench_conj_chain_chirho,
    bench_conde_chirho,
    bench_goal_analysis_chirho,
);

criterion_main!(benches_chirho);
