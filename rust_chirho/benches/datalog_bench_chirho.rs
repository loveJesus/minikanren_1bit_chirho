//! Datalog Benchmark: Transitive Closure ☧
//!
//! Compares our 1-bit tensor approach against Soufflé for graph reachability.
//! Addresses Gemini critique P2-1.
//!
//! Run with: cargo bench --bench datalog_bench_chirho
//!
//! "The LORD is my shepherd; I shall not want." — Psalm 23:1

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use minikanren_1bit_chirho::hardware_chirho::bitmatrix_chirho::BitMatrixChirho;
use std::collections::HashSet;

/// Generate random directed graph
fn generate_graph_chirho(n_nodes_chirho: usize, n_edges_chirho: usize, seed_chirho: u64) -> Vec<(usize, usize)> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut edges_chirho = HashSet::new();
    let mut counter_chirho = 0u64;

    while edges_chirho.len() < n_edges_chirho {
        let mut hasher_chirho = DefaultHasher::new();
        (seed_chirho, counter_chirho).hash(&mut hasher_chirho);
        let h_chirho = hasher_chirho.finish();

        let src_chirho = (h_chirho as usize) % n_nodes_chirho;
        let dst_chirho = ((h_chirho >> 32) as usize) % n_nodes_chirho;

        if src_chirho != dst_chirho {
            edges_chirho.insert((src_chirho, dst_chirho));
        }
        counter_chirho += 1;
    }

    edges_chirho.into_iter().collect()
}

/// Transitive closure using our BitMatrix (1-bit tensor approach)
fn transitive_closure_bitmatrix_chirho(edges_chirho: &[(usize, usize)], n_nodes_chirho: usize) -> BitMatrixChirho {
    let mut mat_chirho = BitMatrixChirho::new(n_nodes_chirho as u32, n_nodes_chirho as u32);

    // Initialize with edges
    for &(src_chirho, dst_chirho) in edges_chirho {
        mat_chirho.set_chirho(src_chirho as u32, dst_chirho as u32);
    }

    // Warshall's algorithm: path[i,j] = path[i,j] OR (path[i,k] AND path[k,j])
    loop {
        let old_count_chirho = mat_chirho.nnz_chirho();

        // One step of transitive closure
        let squared_chirho = mat_chirho.matmul_chirho(&mat_chirho);
        mat_chirho = mat_chirho.or_chirho(&squared_chirho);

        let new_count_chirho = mat_chirho.nnz_chirho();
        if new_count_chirho == old_count_chirho {
            break; // Fixed point reached
        }
    }

    mat_chirho
}

/// Transitive closure using HashSet (baseline)
fn transitive_closure_hashset_chirho(
    edges_chirho: &[(usize, usize)],
    _n_nodes_chirho: usize,
) -> HashSet<(usize, usize)> {
    let mut paths_chirho: HashSet<(usize, usize)> = edges_chirho.iter().copied().collect();

    loop {
        let old_count_chirho = paths_chirho.len();

        // Compute new paths: if (a,b) in paths and (b,c) in paths, add (a,c)
        let new_paths_chirho: Vec<(usize, usize)> = {
            let mut result_chirho = Vec::new();
            for &(a_chirho, b_chirho) in &paths_chirho {
                for &(b2_chirho, c_chirho) in &paths_chirho {
                    if b_chirho == b2_chirho && !paths_chirho.contains(&(a_chirho, c_chirho)) {
                        result_chirho.push((a_chirho, c_chirho));
                    }
                }
            }
            result_chirho
        };

        for path_chirho in new_paths_chirho {
            paths_chirho.insert(path_chirho);
        }

        if paths_chirho.len() == old_count_chirho {
            break;
        }
    }

    paths_chirho
}

fn bench_transitive_closure_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("TransitiveClosureChirho");

    // Test configurations: (nodes, edges, label)
    let configs_chirho = [
        (50, 200, "200"),
        (100, 1000, "1k"),
        (200, 5000, "5k"),
    ];

    for (n_nodes_chirho, n_edges_chirho, label_chirho) in configs_chirho {
        let edges_chirho = generate_graph_chirho(n_nodes_chirho, n_edges_chirho, 42);

        group_chirho.bench_with_input(
            BenchmarkId::new("BitMatrix", label_chirho),
            &edges_chirho,
            |b_chirho, edges_chirho| {
                b_chirho.iter(|| {
                    black_box(transitive_closure_bitmatrix_chirho(edges_chirho, n_nodes_chirho))
                })
            },
        );

        // Only benchmark HashSet on smaller sizes (it's very slow)
        if n_edges_chirho <= 1000 {
            group_chirho.bench_with_input(
                BenchmarkId::new("HashSet", label_chirho),
                &edges_chirho,
                |b_chirho, edges_chirho| {
                    b_chirho.iter(|| {
                        black_box(transitive_closure_hashset_chirho(edges_chirho, n_nodes_chirho))
                    })
                },
            );
        }
    }

    group_chirho.finish();
}

fn bench_bool_matmul_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("BoolMatMulChirho");

    for size_chirho in [32, 64, 128, 256] {
        let edges_chirho = generate_graph_chirho(size_chirho, size_chirho * 4, 42);
        let mut mat_chirho = BitMatrixChirho::new(size_chirho as u32, size_chirho as u32);
        for (s_chirho, d_chirho) in edges_chirho {
            mat_chirho.set_chirho(s_chirho as u32, d_chirho as u32);
        }

        group_chirho.bench_with_input(
            BenchmarkId::from_parameter(size_chirho),
            &mat_chirho,
            |b_chirho, mat_chirho: &BitMatrixChirho| {
                b_chirho.iter(|| black_box(mat_chirho.matmul_chirho(mat_chirho)))
            },
        );
    }

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_transitive_closure_chirho,
    bench_bool_matmul_chirho
);
criterion_main!(benches_chirho);
