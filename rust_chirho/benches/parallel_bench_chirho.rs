//! Parallel Scaling Benchmark ☧
//!
//! Demonstrates multi-core speedup for batch domain operations.
//! Addresses Gemini critique P2-8.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use minikanren_1bit_chirho::parallel_chirho::{ParallelBatchChirho, ParallelStateChirho};
use minikanren_1bit_chirho::DomainHwChirho;

fn create_batch_chirho(num_states_chirho: usize, num_vars_chirho: usize) -> ParallelBatchChirho {
    let states_chirho: Vec<_> = (0..num_states_chirho)
        .map(|_| ParallelStateChirho::new_chirho(num_vars_chirho))
        .collect();
    ParallelBatchChirho::new_chirho(states_chirho)
}

fn bench_sequential_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Sequential");

    for num_states_chirho in [1000, 10000, 100000] {
        group_chirho.throughput(Throughput::Elements(num_states_chirho as u64));

        group_chirho.bench_with_input(
            BenchmarkId::new("apply_constraint", num_states_chirho),
            &num_states_chirho,
            |b_chirho, &n_chirho| {
                let mut batch_chirho = create_batch_chirho(n_chirho, 8);
                let constraint_chirho = DomainHwChirho::singleton_chirho(32);

                b_chirho.iter(|| {
                    batch_chirho.seq_apply_constraint_chirho(&constraint_chirho);
                });
            },
        );
    }

    group_chirho.finish();
}

fn bench_parallel_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Parallel");

    for num_states_chirho in [1000, 10000, 100000] {
        group_chirho.throughput(Throughput::Elements(num_states_chirho as u64));

        group_chirho.bench_with_input(
            BenchmarkId::new("apply_constraint", num_states_chirho),
            &num_states_chirho,
            |b_chirho, &n_chirho| {
                let mut batch_chirho = create_batch_chirho(n_chirho, 8);
                let constraint_chirho = DomainHwChirho::singleton_chirho(32);

                b_chirho.iter(|| {
                    batch_chirho.par_apply_constraint_chirho(&constraint_chirho);
                });
            },
        );
    }

    group_chirho.finish();
}

fn bench_comparison_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SeqVsPar");

    let num_states_chirho = 50000;
    let num_vars_chirho = 16;
    let constraint_chirho = DomainHwChirho::singleton_chirho(32);

    group_chirho.throughput(Throughput::Elements((num_states_chirho * num_vars_chirho) as u64));

    group_chirho.bench_function("sequential", |b_chirho| {
        let mut batch_chirho = create_batch_chirho(num_states_chirho, num_vars_chirho);
        b_chirho.iter(|| {
            batch_chirho.seq_apply_constraint_chirho(&constraint_chirho);
        });
    });

    group_chirho.bench_function("parallel", |b_chirho| {
        let mut batch_chirho = create_batch_chirho(num_states_chirho, num_vars_chirho);
        b_chirho.iter(|| {
            batch_chirho.par_apply_constraint_chirho(&constraint_chirho);
        });
    });

    group_chirho.finish();
}

fn bench_prune_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Prune");

    let num_states_chirho = 100000;
    let num_vars_chirho = 8;

    // Create batch with 50% failed states
    let mut states_chirho = Vec::with_capacity(num_states_chirho);
    for i_chirho in 0..num_states_chirho {
        let mut state_chirho = ParallelStateChirho::new_chirho(num_vars_chirho);
        if i_chirho % 2 == 0 {
            state_chirho.domains_chirho[0] = DomainHwChirho::empty_chirho();
        }
        states_chirho.push(state_chirho);
    }

    group_chirho.throughput(Throughput::Elements(num_states_chirho as u64));

    group_chirho.bench_function("sequential", |b_chirho| {
        let mut batch_chirho = ParallelBatchChirho::new_chirho(states_chirho.clone());
        b_chirho.iter(|| {
            batch_chirho.seq_prune_failed_chirho();
        });
    });

    group_chirho.bench_function("parallel", |b_chirho| {
        let mut batch_chirho = ParallelBatchChirho::new_chirho(states_chirho.clone());
        b_chirho.iter(|| {
            batch_chirho.par_prune_failed_chirho();
        });
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_sequential_chirho,
    bench_parallel_chirho,
    bench_comparison_chirho,
    bench_prune_chirho
);
criterion_main!(benches_chirho);
