//! Contraction Order Benchmarks ☧
//!
//! Measures heuristic time vs execution time to address ablation study requirement.
//! Run with: cargo bench --bench contraction_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::experimental_chirho::contraction_chirho::{
    greedy_order_chirho, min_degree_order_chirho, total_cost_chirho, TensorNetworkChirho,
};
use std::time::Instant;

/// Create a chain network: A-B-C-D-E (linear)
fn chain_network_chirho(n_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new();

    for i_chirho in 0..n_chirho {
        let name_chirho = format!("T{}", i_chirho);
        if i_chirho == 0 {
            net_chirho.add_tensor_chirho(&name_chirho, &["i0"], 64);
        } else if i_chirho == n_chirho - 1 {
            net_chirho.add_tensor_chirho(&name_chirho, &[&format!("i{}", i_chirho - 1)], 64);
        } else {
            net_chirho.add_tensor_chirho(
                &name_chirho,
                &[&format!("i{}", i_chirho - 1), &format!("i{}", i_chirho)],
                64,
            );
        }
    }

    for i_chirho in 0..n_chirho {
        net_chirho.set_index_size_chirho(&format!("i{}", i_chirho), 64);
    }

    net_chirho
}

/// Create a star network: central hub connected to N leaves
fn star_network_chirho(n_leaves_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new();

    // Hub tensor with N indices
    let hub_indices_chirho: Vec<String> = (0..n_leaves_chirho)
        .map(|i| format!("i{}", i))
        .collect();
    let hub_idx_refs_chirho: Vec<&str> = hub_indices_chirho.iter().map(|s| s.as_str()).collect();
    net_chirho.add_tensor_chirho("Hub", &hub_idx_refs_chirho, 64);

    // Leaf tensors
    for i_chirho in 0..n_leaves_chirho {
        net_chirho.add_tensor_chirho(
            &format!("L{}", i_chirho),
            &[&format!("i{}", i_chirho)],
            64,
        );
        net_chirho.set_index_size_chirho(&format!("i{}", i_chirho), 64);
    }

    net_chirho
}

/// Create a grid network: N x N lattice
fn grid_network_chirho(n_chirho: usize) -> TensorNetworkChirho {
    let mut net_chirho = TensorNetworkChirho::new();

    for i_chirho in 0..n_chirho {
        for j_chirho in 0..n_chirho {
            let name_chirho = format!("T{}_{}", i_chirho, j_chirho);
            let mut indices_chirho = Vec::new();

            // Horizontal edge to right neighbor
            if j_chirho < n_chirho - 1 {
                indices_chirho.push(format!("h{}_{}",  i_chirho, j_chirho));
            }
            // Horizontal edge from left neighbor
            if j_chirho > 0 {
                indices_chirho.push(format!("h{}_{}", i_chirho, j_chirho - 1));
            }
            // Vertical edge to bottom neighbor
            if i_chirho < n_chirho - 1 {
                indices_chirho.push(format!("v{}_{}", i_chirho, j_chirho));
            }
            // Vertical edge from top neighbor
            if i_chirho > 0 {
                indices_chirho.push(format!("v{}_{}", i_chirho - 1, j_chirho));
            }

            let idx_refs_chirho: Vec<&str> = indices_chirho.iter().map(|s| s.as_str()).collect();
            net_chirho.add_tensor_chirho(&name_chirho, &idx_refs_chirho, 64);
        }
    }

    // Set all index sizes
    for i_chirho in 0..n_chirho {
        for j_chirho in 0..n_chirho {
            if j_chirho < n_chirho - 1 {
                net_chirho.set_index_size_chirho(&format!("h{}_{}", i_chirho, j_chirho), 64);
            }
            if i_chirho < n_chirho - 1 {
                net_chirho.set_index_size_chirho(&format!("v{}_{}", i_chirho, j_chirho), 64);
            }
        }
    }

    net_chirho
}

/// Benchmark heuristic planning time (not execution)
fn bench_heuristic_time_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ContractionHeuristic");

    // Chain networks
    for n_chirho in [5, 10, 20].iter() {
        let net_chirho = chain_network_chirho(*n_chirho);

        group_chirho.bench_with_input(
            BenchmarkId::new("greedy_chain", n_chirho),
            &net_chirho,
            |b, net| b.iter(|| black_box(greedy_order_chirho(net))),
        );

        group_chirho.bench_with_input(
            BenchmarkId::new("min_degree_chain", n_chirho),
            &net_chirho,
            |b, net| b.iter(|| black_box(min_degree_order_chirho(net))),
        );
    }

    // Star networks
    for n_chirho in [4, 8, 16].iter() {
        let net_chirho = star_network_chirho(*n_chirho);

        group_chirho.bench_with_input(
            BenchmarkId::new("greedy_star", n_chirho),
            &net_chirho,
            |b, net| b.iter(|| black_box(greedy_order_chirho(net))),
        );
    }

    // Grid networks (treewidth challenge)
    for n_chirho in [3, 4, 5].iter() {
        let net_chirho = grid_network_chirho(*n_chirho);

        group_chirho.bench_with_input(
            BenchmarkId::new("greedy_grid", n_chirho),
            &net_chirho,
            |b, net| b.iter(|| black_box(greedy_order_chirho(net))),
        );
    }

    group_chirho.finish();
}

/// Benchmark showing breakdown: heuristic_time + "execution cost"
/// Note: We measure cost estimate, not actual tensor contraction
fn bench_ablation_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ContractionAblation");

    // Measure both heuristic time and resulting cost for various networks
    // Use smaller networks to avoid cost overflow (star networks explode)
    let test_cases_chirho = vec![
        ("chain_10", chain_network_chirho(10)),
        ("chain_20", chain_network_chirho(20)),
        ("star_4", star_network_chirho(4)),
        ("grid_3x3", grid_network_chirho(3)),
        ("grid_4x4", grid_network_chirho(4)),
    ];

    for (name_chirho, net_chirho) in &test_cases_chirho {
        // Time the heuristic
        group_chirho.bench_function(
            &format!("{}_heuristic", name_chirho),
            |b| {
                b.iter(|| {
                    let order_chirho = greedy_order_chirho(net_chirho);
                    black_box(order_chirho)
                })
            },
        );

        // Compute and report the cost estimate
        let order_chirho = greedy_order_chirho(net_chirho);
        let cost_chirho = total_cost_chirho(&order_chirho);
        println!("Network {}: estimated contraction cost = {}", name_chirho, cost_chirho);
    }

    group_chirho.finish();
}

/// Print detailed ablation report (run separately)
pub fn print_ablation_report_chirho() {
    println!("\n=== Contraction Order Ablation Study ===\n");
    println!("{:<15} {:>12} {:>12} {:>12}", "Network", "Heuristic", "Cost Est.", "Overhead%");
    println!("{}", "-".repeat(55));

    // Use smaller networks to avoid cost overflow (star networks explode)
    let test_cases_chirho = vec![
        ("chain_10", chain_network_chirho(10)),
        ("chain_20", chain_network_chirho(20)),
        ("star_4", star_network_chirho(4)),
        ("grid_3x3", grid_network_chirho(3)),
        ("grid_4x4", grid_network_chirho(4)),
    ];

    for (name_chirho, net_chirho) in test_cases_chirho {
        // Time the heuristic (average over 100 runs)
        let start_chirho = Instant::now();
        for _ in 0..100 {
            let _ = black_box(greedy_order_chirho(&net_chirho));
        }
        let heuristic_ns_chirho = start_chirho.elapsed().as_nanos() / 100;

        // Get cost estimate
        let order_chirho = greedy_order_chirho(&net_chirho);
        let cost_chirho = total_cost_chirho(&order_chirho);

        // Assuming 1 op = 1 ns for cost (very rough)
        let overhead_pct_chirho = if cost_chirho > 0 {
            (heuristic_ns_chirho as f64 / cost_chirho as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "{:<15} {:>10} ns {:>10} ops {:>10.1}%",
            name_chirho, heuristic_ns_chirho, cost_chirho, overhead_pct_chirho
        );
    }

    println!("\nNote: Overhead% shows heuristic time relative to estimated execution cost.");
    println!("For real applications, actual execution is much slower than cost estimate.\n");
}

criterion_group!(
    benches_chirho,
    bench_heuristic_time_chirho,
    bench_ablation_chirho,
);
criterion_main!(benches_chirho);

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_chain_network_chirho() {
        let net_chirho = chain_network_chirho(5);
        assert_eq!(net_chirho.tensors_chirho.len(), 5);
    }

    #[test]
    fn test_star_network_chirho() {
        let net_chirho = star_network_chirho(4);
        assert_eq!(net_chirho.tensors_chirho.len(), 5); // hub + 4 leaves
    }

    #[test]
    fn test_grid_network_chirho() {
        let net_chirho = grid_network_chirho(3);
        assert_eq!(net_chirho.tensors_chirho.len(), 9); // 3x3
    }

    #[test]
    fn test_ablation_report_chirho() {
        // Just ensure it doesn't crash
        print_ablation_report_chirho();
    }
}
