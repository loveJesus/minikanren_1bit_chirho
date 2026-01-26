//! Contraction Heuristics Benchmark ☧
//!
//! Compares greedy, min-degree, and min-fill heuristics for tensor contraction ordering.
//!
//! For God so loved the world that he gave his only begotten Son,
//! that whoever believes in him should not perish but have eternal life.
//! John 3:16
//!
//! Run: cargo bench --bench contraction_heuristics_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::{HashMap, HashSet};

/// Tensor network node (simplified representation)
#[derive(Clone, Debug)]
struct TensorNodeChirho {
    id_chirho: usize,
    indices_chirho: HashSet<usize>,
    size_chirho: usize, // Product of dimension sizes
}

/// Tensor network for contraction ordering
#[derive(Clone, Debug)]
struct TensorNetworkChirho {
    nodes_chirho: Vec<TensorNodeChirho>,
    index_dims_chirho: HashMap<usize, usize>,
}

impl TensorNetworkChirho {
    fn new_chirho() -> Self {
        Self {
            nodes_chirho: Vec::new(),
            index_dims_chirho: HashMap::new(),
        }
    }

    fn add_node_chirho(&mut self, indices_chirho: Vec<usize>, dims_chirho: Vec<usize>) {
        let id_chirho = self.nodes_chirho.len();
        let indices_set_chirho: HashSet<usize> = indices_chirho.iter().copied().collect();
        let size_chirho: usize = dims_chirho.iter().product();

        for (idx_chirho, dim_chirho) in indices_chirho.iter().zip(dims_chirho.iter()) {
            self.index_dims_chirho.insert(*idx_chirho, *dim_chirho);
        }

        self.nodes_chirho.push(TensorNodeChirho {
            id_chirho,
            indices_chirho: indices_set_chirho,
            size_chirho,
        });
    }

    /// Greedy contraction: always contract the pair producing smallest intermediate
    fn greedy_order_chirho(&self) -> Vec<(usize, usize)> {
        let mut nodes_chirho = self.nodes_chirho.clone();
        let mut order_chirho = Vec::new();
        let mut next_id_chirho = nodes_chirho.len();

        while nodes_chirho.len() > 1 {
            let mut best_cost_chirho = usize::MAX;
            let mut best_pair_chirho = (0, 1);

            for i_chirho in 0..nodes_chirho.len() {
                for j_chirho in (i_chirho + 1)..nodes_chirho.len() {
                    let cost_chirho = self.contraction_cost_chirho(
                        &nodes_chirho[i_chirho],
                        &nodes_chirho[j_chirho],
                    );
                    if cost_chirho < best_cost_chirho {
                        best_cost_chirho = cost_chirho;
                        best_pair_chirho = (i_chirho, j_chirho);
                    }
                }
            }

            let (i_chirho, j_chirho) = best_pair_chirho;
            order_chirho.push((
                nodes_chirho[i_chirho].id_chirho,
                nodes_chirho[j_chirho].id_chirho,
            ));

            // Merge nodes
            let new_indices_chirho: HashSet<usize> = nodes_chirho[i_chirho]
                .indices_chirho
                .symmetric_difference(&nodes_chirho[j_chirho].indices_chirho)
                .copied()
                .collect();

            let new_size_chirho = new_indices_chirho
                .iter()
                .map(|idx| self.index_dims_chirho.get(idx).unwrap_or(&1))
                .product();

            let new_node_chirho = TensorNodeChirho {
                id_chirho: next_id_chirho,
                indices_chirho: new_indices_chirho,
                size_chirho: new_size_chirho,
            };
            next_id_chirho += 1;

            // Remove old, add new
            if j_chirho > i_chirho {
                nodes_chirho.remove(j_chirho);
                nodes_chirho.remove(i_chirho);
            } else {
                nodes_chirho.remove(i_chirho);
                nodes_chirho.remove(j_chirho);
            }
            nodes_chirho.push(new_node_chirho);
        }

        order_chirho
    }

    /// Min-degree: eliminate variable with fewest neighbors
    fn min_degree_order_chirho(&self) -> Vec<usize> {
        let mut index_nodes_chirho: HashMap<usize, HashSet<usize>> = HashMap::new();

        for node_chirho in &self.nodes_chirho {
            for idx_chirho in &node_chirho.indices_chirho {
                index_nodes_chirho
                    .entry(*idx_chirho)
                    .or_default()
                    .insert(node_chirho.id_chirho);
            }
        }

        let mut order_chirho = Vec::new();
        let mut remaining_chirho: HashSet<usize> = index_nodes_chirho.keys().copied().collect();

        while !remaining_chirho.is_empty() {
            // Find index with minimum degree (fewest nodes containing it)
            let min_idx_chirho = remaining_chirho
                .iter()
                .min_by_key(|idx| {
                    index_nodes_chirho
                        .get(*idx)
                        .map(|s| s.len())
                        .unwrap_or(0)
                })
                .copied()
                .unwrap();

            order_chirho.push(min_idx_chirho);
            remaining_chirho.remove(&min_idx_chirho);
        }

        order_chirho
    }

    /// Min-fill: eliminate variable adding fewest new edges
    fn min_fill_order_chirho(&self) -> Vec<usize> {
        let mut index_nodes_chirho: HashMap<usize, HashSet<usize>> = HashMap::new();

        for node_chirho in &self.nodes_chirho {
            for idx_chirho in &node_chirho.indices_chirho {
                index_nodes_chirho
                    .entry(*idx_chirho)
                    .or_default()
                    .insert(node_chirho.id_chirho);
            }
        }

        let mut order_chirho = Vec::new();
        let mut remaining_chirho: HashSet<usize> = index_nodes_chirho.keys().copied().collect();

        // Build adjacency graph (index -> neighboring indices)
        let mut adj_chirho: HashMap<usize, HashSet<usize>> = HashMap::new();
        for node_chirho in &self.nodes_chirho {
            let indices_vec_chirho: Vec<usize> = node_chirho.indices_chirho.iter().copied().collect();
            for i_chirho in 0..indices_vec_chirho.len() {
                for j_chirho in (i_chirho + 1)..indices_vec_chirho.len() {
                    adj_chirho
                        .entry(indices_vec_chirho[i_chirho])
                        .or_default()
                        .insert(indices_vec_chirho[j_chirho]);
                    adj_chirho
                        .entry(indices_vec_chirho[j_chirho])
                        .or_default()
                        .insert(indices_vec_chirho[i_chirho]);
                }
            }
        }

        while !remaining_chirho.is_empty() {
            // Find index adding minimum fill edges
            let min_idx_chirho = remaining_chirho
                .iter()
                .min_by_key(|idx| {
                    let neighbors_chirho = adj_chirho.get(*idx).cloned().unwrap_or_default();
                    let remaining_neighbors_chirho: Vec<usize> = neighbors_chirho
                        .iter()
                        .filter(|n| remaining_chirho.contains(n))
                        .copied()
                        .collect();

                    // Count fill edges needed
                    let mut fill_chirho = 0;
                    for i_chirho in 0..remaining_neighbors_chirho.len() {
                        for j_chirho in (i_chirho + 1)..remaining_neighbors_chirho.len() {
                            let a_chirho = remaining_neighbors_chirho[i_chirho];
                            let b_chirho = remaining_neighbors_chirho[j_chirho];
                            if !adj_chirho
                                .get(&a_chirho)
                                .map(|s| s.contains(&b_chirho))
                                .unwrap_or(false)
                            {
                                fill_chirho += 1;
                            }
                        }
                    }
                    fill_chirho
                })
                .copied()
                .unwrap();

            // Add fill edges
            let neighbors_chirho = adj_chirho.get(&min_idx_chirho).cloned().unwrap_or_default();
            let remaining_neighbors_chirho: Vec<usize> = neighbors_chirho
                .iter()
                .filter(|n| remaining_chirho.contains(n))
                .copied()
                .collect();

            for i_chirho in 0..remaining_neighbors_chirho.len() {
                for j_chirho in (i_chirho + 1)..remaining_neighbors_chirho.len() {
                    let a_chirho = remaining_neighbors_chirho[i_chirho];
                    let b_chirho = remaining_neighbors_chirho[j_chirho];
                    adj_chirho.entry(a_chirho).or_default().insert(b_chirho);
                    adj_chirho.entry(b_chirho).or_default().insert(a_chirho);
                }
            }

            order_chirho.push(min_idx_chirho);
            remaining_chirho.remove(&min_idx_chirho);
        }

        order_chirho
    }

    fn contraction_cost_chirho(&self, a_chirho: &TensorNodeChirho, b_chirho: &TensorNodeChirho) -> usize {
        // Cost = size of resulting tensor
        let contracted_chirho: HashSet<usize> = a_chirho
            .indices_chirho
            .intersection(&b_chirho.indices_chirho)
            .copied()
            .collect();

        let result_indices_chirho: HashSet<usize> = a_chirho
            .indices_chirho
            .symmetric_difference(&b_chirho.indices_chirho)
            .copied()
            .collect();

        let contract_size_chirho: usize = contracted_chirho
            .iter()
            .map(|idx| self.index_dims_chirho.get(idx).unwrap_or(&1))
            .product();

        let result_size_chirho: usize = result_indices_chirho
            .iter()
            .map(|idx| self.index_dims_chirho.get(idx).unwrap_or(&1))
            .product();

        // FLOPS ~ input sizes * contraction size
        a_chirho.size_chirho * b_chirho.size_chirho / contract_size_chirho.max(1) + result_size_chirho
    }
}

/// Create a chain network: T0[i,j] * T1[j,k] * T2[k,l] * ...
fn create_chain_network_chirho(n_chirho: usize, dim_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new_chirho();
    for i_chirho in 0..n_chirho {
        net_chirho.add_node_chirho(vec![i_chirho, i_chirho + 1], vec![dim_chirho, dim_chirho]);
    }
    net_chirho
}

/// Create a star network: T0[i,c] * T1[j,c] * T2[k,c] * ... (all share center)
fn create_star_network_chirho(n_chirho: usize, dim_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new_chirho();
    let center_chirho = 0;
    for i_chirho in 0..n_chirho {
        net_chirho.add_node_chirho(vec![i_chirho + 1, center_chirho], vec![dim_chirho, dim_chirho]);
    }
    net_chirho
}

/// Create a grid network (2D lattice)
fn create_grid_network_chirho(rows_chirho: usize, cols_chirho: usize, dim_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new_chirho();
    let mut idx_chirho = 0;

    for r_chirho in 0..rows_chirho {
        for c_chirho in 0..cols_chirho {
            let mut indices_chirho = Vec::new();
            let mut dims_chirho = Vec::new();

            // Connect to right neighbor
            if c_chirho < cols_chirho - 1 {
                indices_chirho.push(idx_chirho);
                dims_chirho.push(dim_chirho);
                idx_chirho += 1;
            }

            // Connect to bottom neighbor
            if r_chirho < rows_chirho - 1 {
                indices_chirho.push(idx_chirho);
                dims_chirho.push(dim_chirho);
                idx_chirho += 1;
            }

            if !indices_chirho.is_empty() {
                net_chirho.add_node_chirho(indices_chirho, dims_chirho);
            }
        }
    }
    net_chirho
}

fn bench_contraction_heuristics_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ContractionHeuristics");

    // Chain networks
    for n_chirho in [5, 10, 20] {
        let net_chirho = create_chain_network_chirho(n_chirho, 64);

        group_chirho.bench_with_input(
            BenchmarkId::new("chain_greedy", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.greedy_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("chain_min_degree", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_degree_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("chain_min_fill", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_fill_order_chirho()));
            },
        );
    }

    // Star networks
    for n_chirho in [5, 10, 20] {
        let net_chirho = create_star_network_chirho(n_chirho, 64);

        group_chirho.bench_with_input(
            BenchmarkId::new("star_greedy", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.greedy_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("star_min_degree", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_degree_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("star_min_fill", n_chirho),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_fill_order_chirho()));
            },
        );
    }

    // Grid networks
    for size_chirho in [3, 4, 5] {
        let net_chirho = create_grid_network_chirho(size_chirho, size_chirho, 64);

        group_chirho.bench_with_input(
            BenchmarkId::new("grid_greedy", format!("{}x{}", size_chirho, size_chirho)),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.greedy_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("grid_min_degree", format!("{}x{}", size_chirho, size_chirho)),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_degree_order_chirho()));
            },
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("grid_min_fill", format!("{}x{}", size_chirho, size_chirho)),
            &net_chirho,
            |b_chirho, net_chirho| {
                b_chirho.iter(|| black_box(net_chirho.min_fill_order_chirho()));
            },
        );
    }

    group_chirho.finish();
}

criterion_group!(benches_chirho, bench_contraction_heuristics_chirho);
criterion_main!(benches_chirho);
