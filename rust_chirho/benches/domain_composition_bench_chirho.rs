//! Domain Composition Benchmarks ☧
//!
//! Profiles different domain types and their compositions:
//!
//! | Domain Type | Size | Use Case |
//! |-------------|------|----------|
//! | BitVec64Chirho | 64 | Small enums, flags |
//! | Hierarchical4kChirho | 4096 | ASCII, small integers |
//! | Hierarchical256kChirho | 262k | Unicode BMP |
//! | DiffHierarchical4kChirho | 4096 soft | Learning/gradient flow |
//! | GPU Vec<u32> | Unlimited | Massive parallel search |
//!
//! Key questions:
//! 1. When does hierarchical beat flat?
//! 2. When does differentiable overhead matter?
//! 3. When is GPU worth the transfer cost?

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;
use minikanren_1bit_chirho::approaches_chirho::{
    Hierarchical4kChirho, Hierarchical256kChirho,
    DiffHierarchical4kChirho, DiffUnifyStateChirho,
};

// Helper: create BitVec64 with first n bits set (range [0, n))
fn bitvec64_range_chirho(n_chirho: u32) -> BitVec64Chirho {
    if n_chirho >= 64 {
        BitVec64Chirho::ONES_CHIRHO
    } else if n_chirho == 0 {
        BitVec64Chirho::ZERO_CHIRHO
    } else {
        BitVec64Chirho((1u64 << n_chirho) - 1)
    }
}

// =============================================================================
// Part 1: Single Domain Operations
// =============================================================================

fn bench_single_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SingleDomainOps");

    // --- BitVec64 (64 values) ---
    let bv_full_chirho = BitVec64Chirho::ONES_CHIRHO;
    let bv_half_chirho = bitvec64_range_chirho(32);

    group_chirho.bench_function("bitvec64_intersect", |b| {
        b.iter(|| black_box(bv_full_chirho.and_chirho(bv_half_chirho)))
    });

    group_chirho.bench_function("bitvec64_union", |b| {
        b.iter(|| black_box(bv_full_chirho.or_chirho(bv_half_chirho)))
    });

    group_chirho.bench_function("bitvec64_count", |b| {
        b.iter(|| black_box(bv_full_chirho.popcount_chirho()))
    });

    // --- Hierarchical4k (4096 values) ---
    let h4k_full_chirho = Hierarchical4kChirho::full_chirho();
    let h4k_half_chirho = Hierarchical4kChirho::range_chirho(2048);

    group_chirho.bench_function("hierarchical4k_intersect", |b| {
        b.iter(|| black_box(h4k_full_chirho.intersect_chirho(&h4k_half_chirho)))
    });

    group_chirho.bench_function("hierarchical4k_union", |b| {
        b.iter(|| black_box(h4k_full_chirho.union_chirho(&h4k_half_chirho)))
    });

    group_chirho.bench_function("hierarchical4k_count", |b| {
        b.iter(|| black_box(h4k_full_chirho.count_chirho()))
    });

    // --- Hierarchical256k (262144 values) ---
    // Note: range_chirho(262144) is very slow to construct, use smaller range for bench
    let h256k_full_chirho = Hierarchical256kChirho::range_chirho(4096);
    let h256k_half_chirho = Hierarchical256kChirho::range_chirho(2048);

    group_chirho.bench_function("hierarchical256k_intersect", |b| {
        b.iter(|| black_box(h256k_full_chirho.intersect_chirho(&h256k_half_chirho)))
    });

    group_chirho.bench_function("hierarchical256k_is_empty", |b| {
        b.iter(|| black_box(h256k_full_chirho.is_empty_chirho()))
    });

    // --- DiffHierarchical4k (4096 soft values) ---
    let diff_full_chirho = DiffHierarchical4kChirho::full_chirho();
    let diff_half_chirho = DiffHierarchical4kChirho::from_hard_chirho(&h4k_half_chirho);

    group_chirho.bench_function("diff_hierarchical_soft_intersect", |b| {
        b.iter(|| black_box(diff_full_chirho.soft_intersect_chirho(&diff_half_chirho)))
    });

    group_chirho.bench_function("diff_hierarchical_soft_union", |b| {
        b.iter(|| black_box(diff_full_chirho.soft_union_chirho(&diff_half_chirho)))
    });

    group_chirho.bench_function("diff_hierarchical_expected_count", |b| {
        b.iter(|| black_box(diff_full_chirho.expected_count_chirho()))
    });

    group_chirho.finish();
}

// =============================================================================
// Part 2: Gradient Operations (Differentiable Only)
// =============================================================================

fn bench_gradient_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("GradientOps");

    let a_chirho = DiffHierarchical4kChirho::full_chirho();
    let b_chirho = DiffHierarchical4kChirho::from_hard_chirho(&Hierarchical4kChirho::range_chirho(2048));
    let grad_out_chirho = DiffHierarchical4kChirho::full_chirho();

    group_chirho.bench_function("soft_intersect_with_grad", |b| {
        b.iter(|| {
            black_box(a_chirho.soft_intersect_with_grad_chirho(&b_chirho, &grad_out_chirho))
        })
    });

    group_chirho.bench_function("soft_union_with_grad", |b| {
        b.iter(|| {
            black_box(a_chirho.soft_union_with_grad_chirho(&b_chirho, &grad_out_chirho))
        })
    });

    // Temperature application
    group_chirho.bench_function("with_temperature_cold", |b| {
        b.iter(|| black_box(a_chirho.with_temperature_chirho(0.1)))
    });

    group_chirho.bench_function("with_temperature_hot", |b| {
        b.iter(|| black_box(a_chirho.with_temperature_chirho(10.0)))
    });

    // Entropy calculation
    group_chirho.bench_function("entropy", |b| {
        b.iter(|| black_box(a_chirho.entropy_chirho()))
    });

    // Sampling
    group_chirho.bench_function("sample_hard", |b| {
        b.iter(|| black_box(a_chirho.sample_hard_chirho(12345)))
    });

    group_chirho.finish();
}

// =============================================================================
// Part 3: Unification State Operations
// =============================================================================

fn bench_unify_state_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("UnifyState");

    // Soft unification state
    for n_vars_chirho in [4, 16, 64, 256] {
        group_chirho.throughput(Throughput::Elements(n_vars_chirho as u64));

        group_chirho.bench_with_input(
            BenchmarkId::new("create_state", n_vars_chirho),
            &n_vars_chirho,
            |b, &n| {
                b.iter(|| black_box(DiffUnifyStateChirho::new_chirho(n, 1.0)))
            },
        );

        let state_chirho = DiffUnifyStateChirho::new_chirho(n_vars_chirho, 1.0);

        group_chirho.bench_with_input(
            BenchmarkId::new("soft_unify_value", n_vars_chirho),
            &n_vars_chirho,
            |b, &n| {
                let mut s = state_chirho.clone();
                b.iter(|| {
                    for i_chirho in 0..n.min(10) {
                        black_box(s.soft_unify_value_chirho(i_chirho, (i_chirho * 100) as u32));
                    }
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("soft_unify_vars", n_vars_chirho),
            &n_vars_chirho,
            |b, &n| {
                let mut s = state_chirho.clone();
                b.iter(|| {
                    for i_chirho in 0..(n - 1).min(10) {
                        black_box(s.soft_unify_vars_chirho(i_chirho, i_chirho + 1));
                    }
                })
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("anneal", n_vars_chirho),
            &n_vars_chirho,
            |b, &_n| {
                let mut s = state_chirho.clone();
                b.iter(|| {
                    black_box(s.anneal_chirho(0.5));
                })
            },
        );
    }

    group_chirho.finish();
}

// =============================================================================
// Part 4: Hard vs Soft Comparison
// =============================================================================

fn bench_hard_vs_soft_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("HardVsSoft");

    // Compare hard hierarchical vs soft hierarchical
    let hard_a_chirho = Hierarchical4kChirho::range_chirho(2000);
    let hard_b_chirho = Hierarchical4kChirho::range_chirho(3000);

    let soft_a_chirho = DiffHierarchical4kChirho::from_hard_chirho(&hard_a_chirho);
    let soft_b_chirho = DiffHierarchical4kChirho::from_hard_chirho(&hard_b_chirho);

    group_chirho.bench_function("hard_intersect", |b| {
        b.iter(|| black_box(hard_a_chirho.intersect_chirho(&hard_b_chirho)))
    });

    group_chirho.bench_function("soft_intersect", |b| {
        b.iter(|| black_box(soft_a_chirho.soft_intersect_chirho(&soft_b_chirho)))
    });

    // Ratio should show soft overhead
    group_chirho.bench_function("hard_union", |b| {
        b.iter(|| black_box(hard_a_chirho.union_chirho(&hard_b_chirho)))
    });

    group_chirho.bench_function("soft_union", |b| {
        b.iter(|| black_box(soft_a_chirho.soft_union_chirho(&soft_b_chirho)))
    });

    // Conversion overhead
    group_chirho.bench_function("hard_to_soft", |b| {
        b.iter(|| black_box(DiffHierarchical4kChirho::from_hard_chirho(&hard_a_chirho)))
    });

    group_chirho.bench_function("soft_to_hard_threshold", |b| {
        b.iter(|| black_box(soft_a_chirho.to_hard_chirho(0.5)))
    });

    group_chirho.finish();
}

// =============================================================================
// Part 5: Bulk Operations at Different Scales
// =============================================================================

fn bench_bulk_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BulkOps");

    // Multiple intersections in sequence (simulates constraint propagation)
    for n_ops_chirho in [10, 100, 1000] {
        // BitVec64 bulk
        let bv_domains_chirho: Vec<_> = (0..n_ops_chirho)
            .map(|i| bitvec64_range_chirho((i % 64) as u32 + 1))
            .collect();

        group_chirho.throughput(Throughput::Elements(n_ops_chirho as u64));

        group_chirho.bench_with_input(
            BenchmarkId::new("bitvec64_bulk_intersect", n_ops_chirho),
            &bv_domains_chirho,
            |b, domains| {
                b.iter(|| {
                    let mut result_chirho = BitVec64Chirho::ONES_CHIRHO;
                    for d in domains {
                        result_chirho = result_chirho.and_chirho(*d);
                    }
                    black_box(result_chirho)
                })
            },
        );

        // Hierarchical4k bulk
        let h4k_domains_chirho: Vec<_> = (0..n_ops_chirho)
            .map(|i| Hierarchical4kChirho::range_chirho((i % 4096) as u32 + 1))
            .collect();

        group_chirho.bench_with_input(
            BenchmarkId::new("hierarchical4k_bulk_intersect", n_ops_chirho),
            &h4k_domains_chirho,
            |b, domains| {
                b.iter(|| {
                    let mut result_chirho = Hierarchical4kChirho::full_chirho();
                    for d in domains {
                        result_chirho = result_chirho.intersect_chirho(d);
                    }
                    black_box(result_chirho)
                })
            },
        );

        // DiffHierarchical4k bulk
        let diff_domains_chirho: Vec<_> = h4k_domains_chirho
            .iter()
            .map(|h| DiffHierarchical4kChirho::from_hard_chirho(h))
            .collect();

        group_chirho.bench_with_input(
            BenchmarkId::new("diff_hierarchical_bulk_soft_intersect", n_ops_chirho),
            &diff_domains_chirho,
            |b, domains| {
                b.iter(|| {
                    let mut result_chirho = DiffHierarchical4kChirho::full_chirho();
                    for d in domains {
                        result_chirho = result_chirho.soft_intersect_chirho(d);
                    }
                    black_box(result_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

// =============================================================================
// Part 6: Memory Footprint Comparison
// =============================================================================

fn bench_memory_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Memory");

    // Allocation benchmarks
    group_chirho.bench_function("alloc_bitvec64", |b| {
        b.iter(|| black_box(BitVec64Chirho::ONES_CHIRHO))
    });

    group_chirho.bench_function("alloc_hierarchical4k", |b| {
        b.iter(|| black_box(Hierarchical4kChirho::full_chirho()))
    });

    group_chirho.bench_function("alloc_hierarchical256k", |b| {
        b.iter(|| black_box(Hierarchical256kChirho::range_chirho(4096)))
    });

    group_chirho.bench_function("alloc_diff_hierarchical4k", |b| {
        b.iter(|| black_box(DiffHierarchical4kChirho::full_chirho()))
    });

    // Clone benchmarks (important for search state copying)
    let bv_chirho = BitVec64Chirho::ONES_CHIRHO;
    let h4k_chirho = Hierarchical4kChirho::full_chirho();
    let h256k_chirho = Hierarchical256kChirho::range_chirho(4096);
    let diff_chirho = DiffHierarchical4kChirho::full_chirho();

    group_chirho.bench_function("clone_bitvec64", |b| {
        b.iter(|| black_box(bv_chirho.clone()))
    });

    group_chirho.bench_function("clone_hierarchical4k", |b| {
        b.iter(|| black_box(h4k_chirho.clone()))
    });

    group_chirho.bench_function("clone_hierarchical256k", |b| {
        b.iter(|| black_box(h256k_chirho.clone()))
    });

    group_chirho.bench_function("clone_diff_hierarchical4k", |b| {
        b.iter(|| black_box(diff_chirho.clone()))
    });

    group_chirho.finish();
}

// =============================================================================
// Part 7: Sparsity Impact
// =============================================================================

fn bench_sparsity_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Sparsity");

    // Test performance with different sparsity levels
    for density_chirho in [1, 10, 50, 100] {
        // density% of values set

        // Hierarchical4k with different sparsities
        let h4k_sparse_chirho = {
            let mut d = Hierarchical4kChirho::empty_chirho();
            for i in (0..4096).step_by(100 / density_chirho.max(1)) {
                d = d.union_chirho(&Hierarchical4kChirho::singleton_chirho(i as u32));
            }
            d
        };

        group_chirho.bench_with_input(
            BenchmarkId::new("hierarchical4k_intersect_sparse", density_chirho),
            &h4k_sparse_chirho,
            |b, sparse| {
                let full = Hierarchical4kChirho::full_chirho();
                b.iter(|| black_box(full.intersect_chirho(sparse)))
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("hierarchical4k_count_sparse", density_chirho),
            &h4k_sparse_chirho,
            |b, sparse| {
                b.iter(|| black_box(sparse.count_chirho()))
            },
        );
    }

    group_chirho.finish();
}

// =============================================================================
// Part 8: Training Loop Simulation
// =============================================================================

fn bench_training_loop_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("TrainingLoop");

    // Simulate one training step
    group_chirho.bench_function("training_step_4vars", |b| {
        let hard_target_chirho = Hierarchical4kChirho::range_chirho(100);
        let target_chirho = DiffHierarchical4kChirho::from_hard_chirho(&hard_target_chirho);

        b.iter(|| {
            let mut state_chirho = DiffUnifyStateChirho::new_chirho(4, 1.0);

            // Forward pass: unify vars
            let p1_chirho = state_chirho.soft_unify_value_chirho(0, 50);
            let p2_chirho = state_chirho.soft_unify_vars_chirho(0, 1);
            let p3_chirho = state_chirho.soft_unify_vars_chirho(1, 2);

            // Loss: expected count difference
            let pred_count_chirho = state_chirho.domains_chirho[0].expected_count_chirho();
            let target_count_chirho = target_chirho.expected_count_chirho();
            let loss_chirho = (pred_count_chirho - target_count_chirho).powi(2);

            black_box((p1_chirho, p2_chirho, p3_chirho, loss_chirho))
        })
    });

    // Simulate training epoch (multiple steps)
    group_chirho.bench_function("training_epoch_100steps", |b| {
        b.iter(|| {
            let mut total_loss_chirho = 0.0;

            for step_chirho in 0..100 {
                let mut state_chirho = DiffUnifyStateChirho::new_chirho(4, 1.0);

                // Temperature annealing
                let temp_chirho = 1.0 - (step_chirho as f64 / 100.0) * 0.9;
                state_chirho.anneal_chirho(temp_chirho);

                // Forward pass
                state_chirho.soft_unify_value_chirho(0, (step_chirho % 100) as u32);
                state_chirho.soft_unify_vars_chirho(0, 1);

                total_loss_chirho += state_chirho.domains_chirho[0].expected_count_chirho();
            }

            black_box(total_loss_chirho)
        })
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_single_ops_chirho,
    bench_gradient_ops_chirho,
    bench_unify_state_chirho,
    bench_hard_vs_soft_chirho,
    bench_bulk_ops_chirho,
    bench_memory_chirho,
    bench_sparsity_chirho,
    bench_training_loop_chirho,
);

criterion_main!(benches_chirho);
