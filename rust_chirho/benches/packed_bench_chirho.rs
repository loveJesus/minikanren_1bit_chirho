//! Benchmarks for packed bit matrices ☧
//!
//! Compare sparse HashSet vs packed u64 vs SIMD-aligned implementations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::{
    BitMatrixChirho,
    BitMatrix64Chirho,
    Word64Chirho,
};

fn bench_word64_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Word64 Operations");

    let a_chirho = Word64Chirho::new_chirho(0xAAAA_AAAA_AAAA_AAAAu64);
    let b_chirho = Word64Chirho::new_chirho(0x5555_5555_5555_5555u64);

    group_chirho.bench_function("and", |bench_chirho| {
        bench_chirho.iter(|| black_box(a_chirho) & black_box(b_chirho))
    });

    group_chirho.bench_function("or", |bench_chirho| {
        bench_chirho.iter(|| black_box(a_chirho) | black_box(b_chirho))
    });

    group_chirho.bench_function("popcount", |bench_chirho| {
        bench_chirho.iter(|| black_box(a_chirho).popcount_chirho())
    });

    group_chirho.bench_function("iter_ones", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut count_chirho = 0u32;
            for _bit_chirho in black_box(a_chirho).iter_ones_chirho() {
                count_chirho += 1;
            }
            count_chirho
        })
    });

    group_chirho.finish();
}

fn bench_matrix64_and_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Matrix64 AND");

    for size_chirho in [8u8, 16, 32, 64] {
        // Dense matrices (50% fill)
        let mut a_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);

        for i_chirho in 0..size_chirho {
            for j_chirho in 0..size_chirho {
                if (i_chirho + j_chirho) % 2 == 0 {
                    a_chirho.set_chirho(i_chirho, j_chirho);
                }
                if (i_chirho + j_chirho) % 2 == 1 {
                    b_chirho.set_chirho(i_chirho, j_chirho);
                }
            }
        }

        group_chirho.bench_with_input(
            BenchmarkId::new("packed", size_chirho),
            &(a_chirho.clone(), b_chirho.clone()),
            |bench_chirho, (a_ref_chirho, b_ref_chirho)| {
                bench_chirho.iter(|| black_box(a_ref_chirho).and_chirho(black_box(b_ref_chirho)))
            },
        );
    }

    group_chirho.finish();
}

fn bench_matrix64_matmul_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Matrix64 Matmul");

    for size_chirho in [8u8, 16, 32] {
        // Sparse matrices (~10% fill)
        let mut a_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);

        for i_chirho in 0..size_chirho {
            // Chain pattern
            if i_chirho + 1 < size_chirho {
                a_chirho.set_chirho(i_chirho, i_chirho + 1);
                b_chirho.set_chirho(i_chirho, i_chirho + 1);
            }
        }

        group_chirho.bench_with_input(
            BenchmarkId::new("packed", size_chirho),
            &(a_chirho.clone(), b_chirho.clone()),
            |bench_chirho, (a_ref_chirho, b_ref_chirho)| {
                bench_chirho.iter(|| black_box(a_ref_chirho).matmul_chirho(black_box(b_ref_chirho)))
            },
        );
    }

    group_chirho.finish();
}

fn bench_transitive_closure_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Transitive Closure");

    for size_chirho in [8u8, 16, 32] {
        // Chain graph: 0->1->2->...->n-1
        let mut adj_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);
        for i_chirho in 0..(size_chirho - 1) {
            adj_chirho.set_chirho(i_chirho, i_chirho + 1);
        }

        group_chirho.bench_with_input(
            BenchmarkId::new("chain", size_chirho),
            &adj_chirho,
            |bench_chirho, adj_ref_chirho| {
                bench_chirho.iter(|| black_box(adj_ref_chirho).transitive_closure_chirho())
            },
        );
    }

    group_chirho.finish();
}

fn bench_sparse_vs_packed_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Sparse vs Packed");

    // Compare HashSet-based sparse matrix vs packed u64 matrix
    let size_chirho = 32;

    // Build equivalent matrices
    let mut sparse_chirho = BitMatrixChirho::new(size_chirho as u32, size_chirho as u32);
    let mut packed_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);

    // 10% density
    for i_chirho in 0..size_chirho {
        for j_chirho in 0..size_chirho {
            if (i_chirho * 7 + j_chirho * 13) % 10 == 0 {
                sparse_chirho.set_chirho(i_chirho as u32, j_chirho as u32);
                packed_chirho.set_chirho(i_chirho, j_chirho);
            }
        }
    }

    let mut sparse2_chirho = BitMatrixChirho::new(size_chirho as u32, size_chirho as u32);
    let mut packed2_chirho = BitMatrix64Chirho::new_chirho(size_chirho, size_chirho);
    for i_chirho in 0..size_chirho {
        for j_chirho in 0..size_chirho {
            if (i_chirho * 11 + j_chirho * 17) % 10 == 0 {
                sparse2_chirho.set_chirho(i_chirho as u32, j_chirho as u32);
                packed2_chirho.set_chirho(i_chirho, j_chirho);
            }
        }
    }

    group_chirho.bench_function("sparse_and", |bench_chirho| {
        bench_chirho.iter(|| black_box(&sparse_chirho).and_chirho(black_box(&sparse2_chirho)))
    });

    group_chirho.bench_function("packed_and", |bench_chirho| {
        bench_chirho.iter(|| black_box(&packed_chirho).and_chirho(black_box(&packed2_chirho)))
    });

    group_chirho.bench_function("sparse_matmul", |bench_chirho| {
        bench_chirho.iter(|| black_box(&sparse_chirho).matmul_chirho(black_box(&sparse2_chirho)))
    });

    group_chirho.bench_function("packed_matmul", |bench_chirho| {
        bench_chirho.iter(|| black_box(&packed_chirho).matmul_chirho(black_box(&packed2_chirho)))
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_word64_ops_chirho,
    bench_matrix64_and_chirho,
    bench_matrix64_matmul_chirho,
    bench_transitive_closure_chirho,
    bench_sparse_vs_packed_chirho,
);

criterion_main!(benches_chirho);
