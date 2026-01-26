//! Benchmark: Finite-Difference vs Analytic Gradients ☧
//!
//! Compares training step performance between:
//! - Finite-difference gradients (perturb and measure)
//! - Analytic gradients (backprop through tensor contraction)
//!
//! Run: cargo bench --bench symbolic_addition_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ============================================================================
// Shared components
// ============================================================================

fn softmax_chirho(logits_chirho: &[f64; 10], temp_chirho: f64) -> [f64; 10] {
    let max_chirho = logits_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut exp_chirho = [0.0; 10];
    let mut sum_chirho = 0.0;
    for (i_chirho, &l_chirho) in logits_chirho.iter().enumerate() {
        exp_chirho[i_chirho] = ((l_chirho - max_chirho) / temp_chirho).exp();
        sum_chirho += exp_chirho[i_chirho];
    }
    for e_chirho in &mut exp_chirho {
        *e_chirho /= sum_chirho;
    }
    exp_chirho
}

fn soft_addition_constraint_chirho(
    probs_a_chirho: &[f64; 10],
    probs_b_chirho: &[f64; 10],
    target_sum_chirho: usize,
) -> f64 {
    let mut prob_chirho = 0.0;
    for d1_chirho in 0..10 {
        for d2_chirho in 0..10 {
            if d1_chirho + d2_chirho == target_sum_chirho {
                prob_chirho += probs_a_chirho[d1_chirho] * probs_b_chirho[d2_chirho];
            }
        }
    }
    prob_chirho
}

fn forward_pass_chirho(
    weights_chirho: &[[f64; 10]; 10],
    pattern_a_chirho: &[f64; 10],
    pattern_b_chirho: &[f64; 10],
    target_sum_chirho: usize,
    temp_chirho: f64,
) -> (f64, [f64; 10], [f64; 10]) {
    // Compute logits
    let mut logits_a_chirho = [0.0; 10];
    let mut logits_b_chirho = [0.0; 10];
    for d_chirho in 0..10 {
        for i_chirho in 0..10 {
            logits_a_chirho[d_chirho] += weights_chirho[d_chirho][i_chirho] * pattern_a_chirho[i_chirho];
            logits_b_chirho[d_chirho] += weights_chirho[d_chirho][i_chirho] * pattern_b_chirho[i_chirho];
        }
    }

    let probs_a_chirho = softmax_chirho(&logits_a_chirho, temp_chirho);
    let probs_b_chirho = softmax_chirho(&logits_b_chirho, temp_chirho);
    let prob_sum_chirho = soft_addition_constraint_chirho(&probs_a_chirho, &probs_b_chirho, target_sum_chirho);
    let loss_chirho = -prob_sum_chirho.max(1e-10).ln();

    (loss_chirho, probs_a_chirho, probs_b_chirho)
}

// ============================================================================
// Finite-difference gradient
// ============================================================================

fn finite_diff_gradient_chirho(
    weights_chirho: &mut [[f64; 10]; 10],
    pattern_a_chirho: &[f64; 10],
    pattern_b_chirho: &[f64; 10],
    target_sum_chirho: usize,
    temp_chirho: f64,
    lr_chirho: f64,
) {
    let (loss_chirho, _, _) = forward_pass_chirho(
        weights_chirho, pattern_a_chirho, pattern_b_chirho, target_sum_chirho, temp_chirho
    );

    let eps_chirho = 1e-4;
    let mut grad_chirho = [[0.0; 10]; 10];

    for d_chirho in 0..10 {
        for i_chirho in 0..10 {
            weights_chirho[d_chirho][i_chirho] += eps_chirho;
            let (loss_plus_chirho, _, _) = forward_pass_chirho(
                weights_chirho, pattern_a_chirho, pattern_b_chirho, target_sum_chirho, temp_chirho
            );
            grad_chirho[d_chirho][i_chirho] = (loss_plus_chirho - loss_chirho) / eps_chirho;
            weights_chirho[d_chirho][i_chirho] -= eps_chirho;
        }
    }

    // Apply gradient
    for d_chirho in 0..10 {
        for i_chirho in 0..10 {
            weights_chirho[d_chirho][i_chirho] -= lr_chirho * grad_chirho[d_chirho][i_chirho];
        }
    }
}

// ============================================================================
// Analytic gradient (backprop through tensor contraction)
// ============================================================================

fn analytic_gradient_chirho(
    weights_chirho: &mut [[f64; 10]; 10],
    pattern_a_chirho: &[f64; 10],
    pattern_b_chirho: &[f64; 10],
    target_sum_chirho: usize,
    temp_chirho: f64,
    lr_chirho: f64,
) {
    // Forward pass
    let mut logits_a_chirho = [0.0; 10];
    let mut logits_b_chirho = [0.0; 10];
    for d_chirho in 0..10 {
        for i_chirho in 0..10 {
            logits_a_chirho[d_chirho] += weights_chirho[d_chirho][i_chirho] * pattern_a_chirho[i_chirho];
            logits_b_chirho[d_chirho] += weights_chirho[d_chirho][i_chirho] * pattern_b_chirho[i_chirho];
        }
    }

    let probs_a_chirho = softmax_chirho(&logits_a_chirho, temp_chirho);
    let probs_b_chirho = softmax_chirho(&logits_b_chirho, temp_chirho);
    let prob_sum_chirho = soft_addition_constraint_chirho(&probs_a_chirho, &probs_b_chirho, target_sum_chirho);

    // Backward pass
    // ∂L/∂prob_sum = -1/prob_sum
    let grad_prob_sum_chirho = -1.0 / prob_sum_chirho.max(1e-10);

    // Tensor contraction adjoint
    let mut grad_probs_a_chirho = [0.0; 10];
    let mut grad_probs_b_chirho = [0.0; 10];
    for d1_chirho in 0..10 {
        for d2_chirho in 0..10 {
            if d1_chirho + d2_chirho == target_sum_chirho {
                grad_probs_a_chirho[d1_chirho] += probs_b_chirho[d2_chirho] * grad_prob_sum_chirho;
                grad_probs_b_chirho[d2_chirho] += probs_a_chirho[d1_chirho] * grad_prob_sum_chirho;
            }
        }
    }

    // Softmax backward
    let dot_a_chirho: f64 = probs_a_chirho.iter().zip(grad_probs_a_chirho.iter()).map(|(p, g)| p * g).sum();
    let dot_b_chirho: f64 = probs_b_chirho.iter().zip(grad_probs_b_chirho.iter()).map(|(p, g)| p * g).sum();

    let mut grad_logits_a_chirho = [0.0; 10];
    let mut grad_logits_b_chirho = [0.0; 10];
    for i_chirho in 0..10 {
        grad_logits_a_chirho[i_chirho] = probs_a_chirho[i_chirho] * (grad_probs_a_chirho[i_chirho] - dot_a_chirho) / temp_chirho;
        grad_logits_b_chirho[i_chirho] = probs_b_chirho[i_chirho] * (grad_probs_b_chirho[i_chirho] - dot_b_chirho) / temp_chirho;
    }

    // Linear layer backward and update
    for d_chirho in 0..10 {
        for i_chirho in 0..10 {
            let grad_chirho = grad_logits_a_chirho[d_chirho] * pattern_a_chirho[i_chirho]
                            + grad_logits_b_chirho[d_chirho] * pattern_b_chirho[i_chirho];
            weights_chirho[d_chirho][i_chirho] -= lr_chirho * grad_chirho;
        }
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_gradient_methods_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SymbolicAddition");

    // Test pattern (digit 3)
    let mut pattern_a_chirho = [0.01; 10];
    pattern_a_chirho[3] = 0.91;
    let mut pattern_b_chirho = [0.01; 10];
    pattern_b_chirho[5] = 0.91;
    let target_sum_chirho = 8; // 3 + 5

    let temp_chirho = 1.0;
    let lr_chirho = 0.01;

    // Finite-difference: 100 forward passes per training step (10x10 weights)
    group_chirho.bench_function("finite_diff_step", |b_chirho| {
        let mut weights_chirho = [[0.1; 10]; 10];
        for i in 0..10 { weights_chirho[i][i] = 1.0; }

        b_chirho.iter(|| {
            finite_diff_gradient_chirho(
                black_box(&mut weights_chirho),
                black_box(&pattern_a_chirho),
                black_box(&pattern_b_chirho),
                black_box(target_sum_chirho),
                black_box(temp_chirho),
                black_box(lr_chirho),
            );
        });
    });

    // Analytic: 1 forward + 1 backward pass per training step
    group_chirho.bench_function("analytic_step", |b_chirho| {
        let mut weights_chirho = [[0.1; 10]; 10];
        for i in 0..10 { weights_chirho[i][i] = 1.0; }

        b_chirho.iter(|| {
            analytic_gradient_chirho(
                black_box(&mut weights_chirho),
                black_box(&pattern_a_chirho),
                black_box(&pattern_b_chirho),
                black_box(target_sum_chirho),
                black_box(temp_chirho),
                black_box(lr_chirho),
            );
        });
    });

    group_chirho.finish();
}

criterion_group!(benches_chirho, bench_gradient_methods_chirho);
criterion_main!(benches_chirho);
