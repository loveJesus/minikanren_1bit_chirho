//! GPU vs CPU Benchmarks ☧
//!
//! Compares GPU acceleration with CPU implementations.
//! Requires `gpu_chirho` feature: cargo bench --bench gpu_bench_chirho --features gpu_chirho
//!
//! Run with: cargo bench --bench gpu_bench_chirho --features gpu_chirho

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// GPU benchmarks only compile with gpu_chirho feature
#[cfg(feature = "gpu_chirho")]
use minikanren_1bit_chirho::experimental_chirho::gpu_chirho::GpuContextChirho;

// Always available CPU implementations
use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;

// ============================================================================
// CPU baseline benchmarks (always available)
// ============================================================================

fn bench_cpu_bulk_and_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BulkAnd/CPU");

    for size_chirho in [1000, 10000, 100000].iter() {
        let a_chirho: Vec<u64> = (0..*size_chirho).map(|i_chirho| i_chirho as u64 | 0xFF).collect();
        let b_chirho: Vec<u64> = (0..*size_chirho).map(|i_chirho| i_chirho as u64 ^ 0xAA).collect();

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(size_chirho),
            size_chirho,
            |bench_chirho, &_n_chirho| {
                bench_chirho.iter(|| {
                    let result_chirho: Vec<u64> = a_chirho.iter()
                        .zip(b_chirho.iter())
                        .map(|(a_val_chirho, b_val_chirho)| a_val_chirho & b_val_chirho)
                        .collect();
                    black_box(result_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

fn bench_cpu_matmul_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BoolMatMul/CPU");

    for n_chirho in [32, 64, 128].iter() {
        let words_per_row_chirho = (*n_chirho + 31) / 32;
        let total_chirho = *n_chirho * words_per_row_chirho;

        let a_chirho: Vec<u32> = (0..total_chirho).map(|i_chirho| (i_chirho * 17 + 1) as u32).collect();
        let b_chirho: Vec<u32> = (0..total_chirho).map(|i_chirho| (i_chirho * 23 + 3) as u32).collect();

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(n_chirho),
            n_chirho,
            |bench_chirho, &n_val_chirho| {
                bench_chirho.iter(|| {
                    let wpr_chirho = (n_val_chirho + 31) / 32;
                    let mut result_chirho = vec![0u32; (n_val_chirho * wpr_chirho) as usize];

                    // CPU Boolean matrix multiplication
                    for i_chirho in 0..n_val_chirho {
                        for j_chirho in 0..n_val_chirho {
                            let mut dot_chirho = false;
                            for k_chirho in 0..n_val_chirho {
                                let a_bit_chirho = (a_chirho[(i_chirho * wpr_chirho + k_chirho / 32) as usize]
                                    >> (k_chirho % 32)) & 1;
                                let b_bit_chirho = (b_chirho[(k_chirho * wpr_chirho + j_chirho / 32) as usize]
                                    >> (j_chirho % 32)) & 1;
                                if a_bit_chirho == 1 && b_bit_chirho == 1 {
                                    dot_chirho = true;
                                    break;
                                }
                            }
                            if dot_chirho {
                                let word_idx_chirho = (i_chirho * wpr_chirho + j_chirho / 32) as usize;
                                result_chirho[word_idx_chirho] |= 1 << (j_chirho % 32);
                            }
                        }
                    }
                    black_box(result_chirho)
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// GPU benchmarks (only with gpu_chirho feature)
// ============================================================================

#[cfg(feature = "gpu_chirho")]
fn bench_gpu_bulk_and_chirho(c_chirho: &mut Criterion) {
    let ctx_chirho = match GpuContextChirho::new_chirho() {
        Some(c_chirho) => c_chirho,
        None => {
            eprintln!("No GPU available, skipping GPU benchmarks");
            return;
        }
    };

    let mut group_chirho = c_chirho.benchmark_group("BulkAnd/GPU");

    for size_chirho in [1000, 10000, 100000].iter() {
        let a_chirho: Vec<u32> = (0..*size_chirho).map(|i_chirho| (i_chirho as u32) | 0xFF).collect();
        let b_chirho: Vec<u32> = (0..*size_chirho).map(|i_chirho| (i_chirho as u32) ^ 0xAA).collect();

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(size_chirho),
            size_chirho,
            |bench_chirho, &_n_chirho| {
                bench_chirho.iter(|| {
                    black_box(ctx_chirho.bulk_and_chirho(&a_chirho, &b_chirho))
                })
            },
        );
    }

    group_chirho.finish();
}

#[cfg(feature = "gpu_chirho")]
fn bench_gpu_matmul_chirho(c_chirho: &mut Criterion) {
    let ctx_chirho = match GpuContextChirho::new_chirho() {
        Some(c_chirho) => c_chirho,
        None => {
            eprintln!("No GPU available, skipping GPU benchmarks");
            return;
        }
    };

    let mut group_chirho = c_chirho.benchmark_group("BoolMatMul/GPU");

    for n_chirho in [32, 64, 128].iter() {
        let words_per_row_chirho = (*n_chirho + 31) / 32;
        let total_chirho = (*n_chirho * words_per_row_chirho) as usize;

        let a_chirho: Vec<u32> = (0..total_chirho).map(|i_chirho| (i_chirho * 17 + 1) as u32).collect();
        let b_chirho: Vec<u32> = (0..total_chirho).map(|i_chirho| (i_chirho * 23 + 3) as u32).collect();

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(n_chirho),
            n_chirho,
            |bench_chirho, &n_val_chirho| {
                bench_chirho.iter(|| {
                    black_box(ctx_chirho.matmul_chirho(&a_chirho, &b_chirho, n_val_chirho as u32))
                })
            },
        );
    }

    group_chirho.finish();
}

#[cfg(feature = "gpu_chirho")]
fn bench_gpu_transitive_closure_chirho(c_chirho: &mut Criterion) {
    let ctx_chirho = match GpuContextChirho::new_chirho() {
        Some(c_chirho) => c_chirho,
        None => {
            eprintln!("No GPU available, skipping GPU benchmarks");
            return;
        }
    };

    let mut group_chirho = c_chirho.benchmark_group("TransitiveClosure/GPU");

    for n_chirho in [32, 64].iter() {
        let words_per_row_chirho = (*n_chirho + 31) / 32;
        let total_chirho = (*n_chirho * words_per_row_chirho) as usize;

        // Create a chain matrix: i -> i+1
        let mut matrix_chirho = vec![0u32; total_chirho];
        for i_chirho in 0..(*n_chirho - 1) {
            let j_chirho = i_chirho + 1;
            let word_idx_chirho = (i_chirho * words_per_row_chirho + j_chirho / 32) as usize;
            matrix_chirho[word_idx_chirho] |= 1 << (j_chirho % 32);
        }

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(n_chirho),
            n_chirho,
            |bench_chirho, &n_val_chirho| {
                bench_chirho.iter(|| {
                    black_box(ctx_chirho.transitive_closure_chirho(&matrix_chirho, n_val_chirho as u32))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// BitVec64 micro-benchmarks
// ============================================================================

fn bench_bitvec64_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BitVec64");

    group_chirho.bench_function("and", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0xFFFF_0000_FFFF_0000);
        let b_chirho = BitVec64Chirho(0x0000_FFFF_FFFF_0000);
        bench_chirho.iter(|| {
            black_box(a_chirho.and_chirho(b_chirho))
        })
    });

    group_chirho.bench_function("or", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0xFFFF_0000_FFFF_0000);
        let b_chirho = BitVec64Chirho(0x0000_FFFF_FFFF_0000);
        bench_chirho.iter(|| {
            black_box(a_chirho.or_chirho(b_chirho))
        })
    });

    group_chirho.bench_function("popcount", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0xAAAA_BBBB_CCCC_DDDD);
        bench_chirho.iter(|| {
            black_box(a_chirho.popcount_chirho())
        })
    });

    group_chirho.bench_function("is_zero", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0);
        bench_chirho.iter(|| {
            black_box(a_chirho.is_zero_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Criterion groups
// ============================================================================

#[cfg(feature = "gpu_chirho")]
criterion_group!(
    benches_chirho,
    bench_cpu_bulk_and_chirho,
    bench_cpu_matmul_chirho,
    bench_gpu_bulk_and_chirho,
    bench_gpu_matmul_chirho,
    bench_gpu_transitive_closure_chirho,
    bench_bitvec64_ops_chirho,
);

#[cfg(not(feature = "gpu_chirho"))]
criterion_group!(
    benches_chirho,
    bench_cpu_bulk_and_chirho,
    bench_cpu_matmul_chirho,
    bench_bitvec64_ops_chirho,
);

criterion_main!(benches_chirho);
