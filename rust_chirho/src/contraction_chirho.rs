//! Tensor Network Contraction ☧
//!
//! Heuristics for finding good contraction orders.

use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Reverse;

/// A tensor in the network
#[derive(Debug, Clone)]
pub struct TensorNodeChirho {
    pub name_chirho: String,
    pub indices_chirho: HashSet<String>,
    pub size_chirho: usize,
}

/// Tensor network
#[derive(Debug, Default)]
pub struct TensorNetworkChirho {
    pub tensors_chirho: Vec<TensorNodeChirho>,
    pub index_sizes_chirho: HashMap<String, usize>,
}

impl TensorNetworkChirho {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tensor_chirho(&mut self, name_chirho: &str, indices_chirho: &[&str], size_chirho: usize) {
        self.tensors_chirho.push(TensorNodeChirho {
            name_chirho: name_chirho.to_string(),
            indices_chirho: indices_chirho.iter().map(|s| s.to_string()).collect(),
            size_chirho,
        });
    }

    pub fn set_index_size_chirho(&mut self, index_chirho: &str, size_chirho: usize) {
        self.index_sizes_chirho.insert(index_chirho.to_string(), size_chirho);
    }
}

/// Contraction step
#[derive(Debug, Clone)]
pub struct ContractionStepChirho {
    pub tensor1_chirho: String,
    pub tensor2_chirho: String,
    pub result_chirho: String,
    pub cost_chirho: usize,
}

/// Greedy contraction: always pick pair with smallest result size
pub fn greedy_order_chirho(network_chirho: &TensorNetworkChirho) -> Vec<ContractionStepChirho> {
    let mut steps_chirho = Vec::new();
    let mut remaining_chirho: Vec<TensorNodeChirho> = network_chirho.tensors_chirho.clone();
    let mut step_id_chirho = 0;

    while remaining_chirho.len() > 1 {
        let mut best_cost_chirho = usize::MAX;
        let mut best_pair_chirho: Option<(usize, usize)> = None;
        let mut best_result_chirho: Option<TensorNodeChirho> = None;

        for i_chirho in 0..remaining_chirho.len() {
            for j_chirho in (i_chirho + 1)..remaining_chirho.len() {
                let t1_chirho = &remaining_chirho[i_chirho];
                let t2_chirho = &remaining_chirho[j_chirho];

                // Only contract if they share indices
                let shared_chirho: HashSet<_> = t1_chirho.indices_chirho
                    .intersection(&t2_chirho.indices_chirho)
                    .cloned()
                    .collect();

                if shared_chirho.is_empty() {
                    continue;
                }

                // Result indices = union - shared
                let result_indices_chirho: HashSet<_> = t1_chirho.indices_chirho
                    .union(&t2_chirho.indices_chirho)
                    .filter(|x| !shared_chirho.contains(*x))
                    .cloned()
                    .collect();

                // Estimate cost
                let cost_chirho: usize = result_indices_chirho
                    .iter()
                    .map(|idx| network_chirho.index_sizes_chirho.get(idx).copied().unwrap_or(10))
                    .product();

                if cost_chirho < best_cost_chirho {
                    best_cost_chirho = cost_chirho;
                    best_pair_chirho = Some((i_chirho, j_chirho));
                    best_result_chirho = Some(TensorNodeChirho {
                        name_chirho: format!("R{}", step_id_chirho),
                        indices_chirho: result_indices_chirho,
                        size_chirho: cost_chirho,
                    });
                }
            }
        }

        if let (Some((i_chirho, j_chirho)), Some(result_chirho)) = (best_pair_chirho, best_result_chirho) {
            steps_chirho.push(ContractionStepChirho {
                tensor1_chirho: remaining_chirho[i_chirho].name_chirho.clone(),
                tensor2_chirho: remaining_chirho[j_chirho].name_chirho.clone(),
                result_chirho: result_chirho.name_chirho.clone(),
                cost_chirho: best_cost_chirho,
            });

            // Remove contracted tensors, add result
            let t2_chirho = remaining_chirho.remove(j_chirho);
            let t1_chirho = remaining_chirho.remove(i_chirho);
            remaining_chirho.push(result_chirho);
            step_id_chirho += 1;
        } else {
            // No connected pairs, just combine first two
            if remaining_chirho.len() >= 2 {
                let t1_chirho = remaining_chirho.remove(0);
                let t2_chirho = remaining_chirho.remove(0);
                let result_chirho = TensorNodeChirho {
                    name_chirho: format!("R{}", step_id_chirho),
                    indices_chirho: t1_chirho.indices_chirho.union(&t2_chirho.indices_chirho).cloned().collect(),
                    size_chirho: t1_chirho.size_chirho * t2_chirho.size_chirho,
                };
                steps_chirho.push(ContractionStepChirho {
                    tensor1_chirho: t1_chirho.name_chirho,
                    tensor2_chirho: t2_chirho.name_chirho,
                    result_chirho: result_chirho.name_chirho.clone(),
                    cost_chirho: result_chirho.size_chirho,
                });
                remaining_chirho.push(result_chirho);
                step_id_chirho += 1;
            }
        }
    }

    steps_chirho
}

/// Min-degree variable elimination order
pub fn min_degree_order_chirho(network_chirho: &TensorNetworkChirho) -> Vec<String> {
    // Build interaction graph
    let mut graph_chirho: HashMap<String, HashSet<String>> = HashMap::new();

    for tensor_chirho in &network_chirho.tensors_chirho {
        let indices_chirho: Vec<_> = tensor_chirho.indices_chirho.iter().cloned().collect();
        for i_chirho in 0..indices_chirho.len() {
            for j_chirho in (i_chirho + 1)..indices_chirho.len() {
                graph_chirho.entry(indices_chirho[i_chirho].clone())
                    .or_default()
                    .insert(indices_chirho[j_chirho].clone());
                graph_chirho.entry(indices_chirho[j_chirho].clone())
                    .or_default()
                    .insert(indices_chirho[i_chirho].clone());
            }
        }
    }

    let mut order_chirho = Vec::new();
    let mut eliminated_chirho: HashSet<String> = HashSet::new();

    // Find output indices (appear in only one tensor)
    let mut index_count_chirho: HashMap<String, usize> = HashMap::new();
    for tensor_chirho in &network_chirho.tensors_chirho {
        for idx_chirho in &tensor_chirho.indices_chirho {
            *index_count_chirho.entry(idx_chirho.clone()).or_default() += 1;
        }
    }
    let output_indices_chirho: HashSet<_> = index_count_chirho
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(k, _)| k.clone())
        .collect();

    // Heap of (degree, index)
    let mut heap_chirho: BinaryHeap<Reverse<(usize, String)>> = graph_chirho
        .iter()
        .filter(|(k, _)| !output_indices_chirho.contains(*k))
        .map(|(k, v)| Reverse((v.len(), k.clone())))
        .collect();

    while let Some(Reverse((_, idx_chirho))) = heap_chirho.pop() {
        if eliminated_chirho.contains(&idx_chirho) {
            continue;
        }

        order_chirho.push(idx_chirho.clone());
        eliminated_chirho.insert(idx_chirho.clone());

        // Add fill edges and update degrees
        if let Some(neighbors_chirho) = graph_chirho.get(&idx_chirho).cloned() {
            let active_neighbors_chirho: Vec<_> = neighbors_chirho
                .iter()
                .filter(|n| !eliminated_chirho.contains(*n))
                .cloned()
                .collect();

            // Connect all neighbors (fill-in)
            for i_chirho in 0..active_neighbors_chirho.len() {
                for j_chirho in (i_chirho + 1)..active_neighbors_chirho.len() {
                    let n1_chirho = &active_neighbors_chirho[i_chirho];
                    let n2_chirho = &active_neighbors_chirho[j_chirho];
                    graph_chirho.entry(n1_chirho.clone()).or_default().insert(n2_chirho.clone());
                    graph_chirho.entry(n2_chirho.clone()).or_default().insert(n1_chirho.clone());
                }
            }

            // Re-add neighbors with updated degrees
            for n_chirho in active_neighbors_chirho {
                if !eliminated_chirho.contains(&n_chirho) && !output_indices_chirho.contains(&n_chirho) {
                    let degree_chirho = graph_chirho.get(&n_chirho)
                        .map(|s| s.iter().filter(|x| !eliminated_chirho.contains(*x)).count())
                        .unwrap_or(0);
                    heap_chirho.push(Reverse((degree_chirho, n_chirho)));
                }
            }
        }
    }

    order_chirho
}

/// Calculate total cost of a contraction plan
pub fn total_cost_chirho(steps_chirho: &[ContractionStepChirho]) -> usize {
    steps_chirho.iter().map(|s| s.cost_chirho).sum()
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_greedy_chirho() {
        let mut net_chirho = TensorNetworkChirho::new();

        net_chirho.add_tensor_chirho("T1", &["A", "B", "X"], 10);
        net_chirho.add_tensor_chirho("T2", &["X", "C", "Y"], 10);
        net_chirho.add_tensor_chirho("T3", &["Y", "D", "Out"], 10);

        for idx_chirho in &["A", "B", "C", "D", "X", "Y", "Out"] {
            net_chirho.set_index_size_chirho(idx_chirho, 5);
        }

        let steps_chirho = greedy_order_chirho(&net_chirho);
        assert_eq!(steps_chirho.len(), 2); // 3 tensors → 2 contractions
    }

    #[test]
    fn test_min_degree_chirho() {
        let mut net_chirho = TensorNetworkChirho::new();

        net_chirho.add_tensor_chirho("T1", &["A", "B", "X"], 10);
        net_chirho.add_tensor_chirho("T2", &["X", "C", "Y"], 10);

        let order_chirho = min_degree_order_chirho(&net_chirho);
        // X and Y should be eliminated (not A, B, C which are outputs)
        assert!(order_chirho.contains(&"X".to_string()));
    }
}
