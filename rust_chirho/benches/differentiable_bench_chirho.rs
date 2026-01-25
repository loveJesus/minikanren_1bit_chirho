//! Differentiable Logic Benchmarks ☧
//!
//! Benchmarks for learnable relations and soft logic operations.
//!
//! Run with: cargo bench --bench differentiable_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::learn_chirho::{
    learnable_chirho::LearnableRelationExtChirho,
    anneal_chirho::{AnnealingScheduleChirho, LinearAnnealChirho, CosineAnnealChirho},
    gumbel_chirho::{GumbelSoftmaxSamplerChirho, StraightThroughEstimatorChirho, DifferentiableBranchChirho},
};
use minikanren_1bit_chirho::semiring_chirho::diff_semiring_chirho::DiffProbChirho;

// ============================================================================
// Learnable Relation benchmarks
// ============================================================================

fn bench_learnable_relation_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("LearnableRelation");

    for size_chirho in [10, 100, 1000].iter() {
        // Create relation with n tuples
        let tuples_chirho: Vec<(u32, u32)> = (0..*size_chirho)
            .map(|i_chirho| (i_chirho, i_chirho + 1))
            .collect();

        group_chirho.bench_with_input(
            BenchmarkId::new("soft_query", size_chirho),
            size_chirho,
            |bench_chirho, &_n_chirho| {
                let rel_chirho = LearnableRelationExtChirho::new_chirho(tuples_chirho.clone(), 0.1);
                bench_chirho.iter(|| {
                    // Query: does (5, 6) exist?
                    let result_chirho = rel_chirho.soft_query_chirho(|t_chirho| {
                        if *t_chirho == (5, 6) { 1.0 } else { 0.0 }
                    });
                    black_box(result_chirho.value_chirho)
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("train_step", size_chirho),
            size_chirho,
            |bench_chirho, &_n_chirho| {
                let mut rel_chirho = LearnableRelationExtChirho::new_chirho(tuples_chirho.clone(), 0.1);
                bench_chirho.iter(|| {
                    let loss_chirho = rel_chirho.train_step_chirho(
                        |t_chirho| if *t_chirho == (5, 6) { 1.0 } else { 0.0 },
                        1.0,
                    );
                    black_box(loss_chirho)
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("weights", size_chirho),
            size_chirho,
            |bench_chirho, &_n_chirho| {
                let rel_chirho = LearnableRelationExtChirho::new_chirho(tuples_chirho.clone(), 0.1);
                bench_chirho.iter(|| {
                    black_box(rel_chirho.weights_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Annealing benchmarks
// ============================================================================

fn bench_annealing_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Annealing");

    // Exponential annealing
    group_chirho.bench_function("exponential_temp", |bench_chirho| {
        let schedule_chirho = AnnealingScheduleChirho::new_chirho(1.0, 0.01, 1000);
        bench_chirho.iter(|| {
            black_box(schedule_chirho.temp_at_step_chirho(500))
        })
    });

    // Linear annealing
    group_chirho.bench_function("linear_temp", |bench_chirho| {
        let schedule_chirho = LinearAnnealChirho::new_chirho(1.0, 0.01, 1000);
        bench_chirho.iter(|| {
            black_box(schedule_chirho.temp_at_step_chirho(500))
        })
    });

    // Cosine annealing
    group_chirho.bench_function("cosine_temp", |bench_chirho| {
        let schedule_chirho = CosineAnnealChirho::new_chirho(1.0, 0.01, 1000);
        bench_chirho.iter(|| {
            black_box(schedule_chirho.temp_at_step_chirho(500))
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Gumbel-Softmax benchmarks
// ============================================================================

fn bench_gumbel_softmax_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("GumbelSoftmax");

    // Sample with different number of branches
    for n_branches_chirho in [2, 4, 8, 16].iter() {
        let logits_chirho: Vec<f64> = (0..*n_branches_chirho)
            .map(|i_chirho| i_chirho as f64 * 0.5)
            .collect();

        group_chirho.bench_with_input(
            BenchmarkId::new("sample", n_branches_chirho),
            &logits_chirho,
            |bench_chirho, logits_ref_chirho| {
                let mut sampler_chirho = GumbelSoftmaxSamplerChirho::new_chirho(1.0);
                bench_chirho.iter(|| {
                    black_box(sampler_chirho.sample_chirho(logits_ref_chirho))
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("sample_hard", n_branches_chirho),
            &logits_chirho,
            |bench_chirho, logits_ref_chirho| {
                let mut sampler_chirho = GumbelSoftmaxSamplerChirho::new_chirho(1.0);
                bench_chirho.iter(|| {
                    black_box(sampler_chirho.sample_hard_chirho(logits_ref_chirho))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Straight-through estimator benchmarks
// ============================================================================

fn bench_straight_through_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("StraightThrough");

    for n_chirho in [4, 8, 16].iter() {
        let probs_chirho: Vec<f64> = (0..*n_chirho)
            .map(|i_chirho| (i_chirho as f64 + 1.0) / (*n_chirho as f64 + 1.0))
            .collect();

        group_chirho.bench_with_input(
            BenchmarkId::new("forward", n_chirho),
            &probs_chirho,
            |bench_chirho, probs_ref_chirho| {
                bench_chirho.iter(|| {
                    black_box(StraightThroughEstimatorChirho::forward_chirho(probs_ref_chirho))
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("backward", n_chirho),
            &probs_chirho,
            |bench_chirho, probs_ref_chirho| {
                bench_chirho.iter(|| {
                    black_box(StraightThroughEstimatorChirho::backward_chirho(probs_ref_chirho, 1.0))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Differentiable branch selection benchmarks
// ============================================================================

fn bench_differentiable_branch_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("DifferentiableBranch");

    for n_branches_chirho in [2, 4, 8].iter() {
        let log_weights_chirho: Vec<f64> = vec![0.0; *n_branches_chirho];

        group_chirho.bench_with_input(
            BenchmarkId::new("branch_probs", n_branches_chirho),
            n_branches_chirho,
            |bench_chirho, &_n_chirho| {
                let branch_chirho = DifferentiableBranchChirho::new_chirho(log_weights_chirho.clone(), 1.0);
                bench_chirho.iter(|| {
                    black_box(branch_chirho.branch_probs_chirho())
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("soft_conde", n_branches_chirho),
            n_branches_chirho,
            |bench_chirho, &n_chirho| {
                let branch_chirho = DifferentiableBranchChirho::new_chirho(log_weights_chirho.clone(), 1.0);
                let _branches_chirho: Vec<Box<dyn Fn() -> f64>> = (0..n_chirho)
                    .map(|i_chirho| Box::new(move || (i_chirho + 1) as f64) as Box<dyn Fn() -> f64>)
                    .collect();
                bench_chirho.iter(|| {
                    // soft_conde uses closures, but for bench we just compute probs
                    let probs_chirho = branch_chirho.branch_probs_chirho();
                    let sum_chirho: f64 = probs_chirho.iter()
                        .enumerate()
                        .map(|(i_chirho, p_chirho)| p_chirho * (i_chirho + 1) as f64)
                        .sum();
                    black_box(sum_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// DiffProbChirho operations benchmarks
// ============================================================================

fn bench_diff_prob_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("DiffProb");

    group_chirho.bench_function("and", |bench_chirho| {
        let a_chirho = DiffProbChirho::new_chirho(0.7);
        let b_chirho = DiffProbChirho::new_chirho(0.8);
        bench_chirho.iter(|| {
            black_box(a_chirho.and_chirho(&b_chirho))
        })
    });

    group_chirho.bench_function("or", |bench_chirho| {
        let a_chirho = DiffProbChirho::new_chirho(0.7);
        let b_chirho = DiffProbChirho::new_chirho(0.8);
        bench_chirho.iter(|| {
            black_box(a_chirho.or_chirho(&b_chirho))
        })
    });

    group_chirho.bench_function("not", |bench_chirho| {
        let a_chirho = DiffProbChirho::new_chirho(0.7);
        bench_chirho.iter(|| {
            black_box(a_chirho.not_chirho())
        })
    });

    // Chain of operations
    group_chirho.bench_function("chain_5", |bench_chirho| {
        let probs_chirho: Vec<DiffProbChirho> = (0..5)
            .map(|i_chirho| DiffProbChirho::new_chirho(0.5 + i_chirho as f64 * 0.1))
            .collect();
        bench_chirho.iter(|| {
            let mut result_chirho = probs_chirho[0].clone();
            for p_chirho in &probs_chirho[1..] {
                result_chirho = result_chirho.and_chirho(p_chirho);
            }
            black_box(result_chirho)
        })
    });

    group_chirho.finish();
}

// ============================================================================
// End-to-end training simulation
// ============================================================================

fn bench_training_epoch_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Training");

    // Simulate a small training epoch
    group_chirho.bench_function("epoch_10_tuples_8_examples", |bench_chirho| {
        let tuples_chirho: Vec<(u32, u32)> = (0..10)
            .map(|i_chirho| (i_chirho, i_chirho + 1))
            .collect();

        let examples_chirho: Vec<((u32, u32), f64)> = vec![
            ((0, 1), 1.0),
            ((1, 2), 1.0),
            ((5, 6), 1.0),
            ((9, 10), 1.0),
            ((0, 5), 0.0),
            ((2, 8), 0.0),
            ((7, 3), 0.0),
            ((4, 9), 0.0),
        ];

        bench_chirho.iter(|| {
            let mut rel_chirho = LearnableRelationExtChirho::new_chirho(tuples_chirho.clone(), 0.1);
            let mut total_loss_chirho = 0.0;

            for ((a_chirho, b_chirho), target_chirho) in &examples_chirho {
                let loss_chirho = rel_chirho.train_step_chirho(
                    |t_chirho| if *t_chirho == (*a_chirho, *b_chirho) { 1.0 } else { 0.0 },
                    *target_chirho,
                );
                total_loss_chirho += loss_chirho;
            }

            black_box(total_loss_chirho)
        })
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_learnable_relation_chirho,
    bench_annealing_chirho,
    bench_gumbel_softmax_chirho,
    bench_straight_through_chirho,
    bench_differentiable_branch_chirho,
    bench_diff_prob_chirho,
    bench_training_epoch_chirho,
);

criterion_main!(benches_chirho);
