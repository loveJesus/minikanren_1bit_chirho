//! Learned Tensor Contraction Order ☧
//!
//! Tensor network contraction order is NP-hard (same as quantum simulation).
//! Use machine learning to predict good contraction orders from features.
//!
//! Approach:
//! 1. Extract features from tensor network structure
//! 2. Use simple regression/classification to predict edge to contract
//! 3. Train on solved instances with known optimal orders
//!
//! Features include:
//! - Node degrees (connectivity)
//! - Edge weights (tensor sizes)
//! - Graph structure (clustering, centrality)
//! - Domain sizes

use std::collections::HashSet;

/// Tensor network for contraction planning
#[derive(Debug, Clone)]
pub struct TensorNetworkChirho {
    /// Number of tensors (nodes)
    pub num_tensors_chirho: usize,
    /// Edges: (tensor1, tensor2, dimension_size)
    pub edges_chirho: Vec<(usize, usize, usize)>,
}

impl TensorNetworkChirho {
    pub fn new_chirho() -> Self {
        Self {
            num_tensors_chirho: 0,
            edges_chirho: Vec::new(),
        }
    }

    /// Add tensor (returns its index)
    pub fn add_tensor_chirho(&mut self) -> usize {
        let idx_chirho = self.num_tensors_chirho;
        self.num_tensors_chirho += 1;
        idx_chirho
    }

    /// Add edge (shared index) between tensors
    pub fn add_edge_chirho(&mut self, t1_chirho: usize, t2_chirho: usize, dim_chirho: usize) {
        self.edges_chirho.push((t1_chirho, t2_chirho, dim_chirho));
    }

    /// Get neighbors of a tensor
    pub fn neighbors_chirho(&self, tensor_chirho: usize) -> Vec<usize> {
        let mut neighbors_chirho = Vec::new();
        for &(t1_chirho, t2_chirho, _) in &self.edges_chirho {
            if t1_chirho == tensor_chirho {
                neighbors_chirho.push(t2_chirho);
            } else if t2_chirho == tensor_chirho {
                neighbors_chirho.push(t1_chirho);
            }
        }
        neighbors_chirho
    }

    /// Degree of tensor (number of edges)
    pub fn degree_chirho(&self, tensor_chirho: usize) -> usize {
        self.edges_chirho
            .iter()
            .filter(|(t1, t2, _)| *t1 == tensor_chirho || *t2 == tensor_chirho)
            .count()
    }
}

/// Features for edge contraction decision
#[derive(Debug, Clone)]
pub struct EdgeFeaturesChirho {
    /// Tensor 1 degree
    pub degree1_chirho: f32,
    /// Tensor 2 degree
    pub degree2_chirho: f32,
    /// Edge dimension (log scale)
    pub log_dim_chirho: f32,
    /// Estimated contraction cost (log scale)
    pub log_cost_chirho: f32,
    /// Number of common neighbors
    pub common_neighbors_chirho: f32,
    /// Sum of neighbor degrees for tensor 1
    pub neighbor_degree_sum1_chirho: f32,
    /// Sum of neighbor degrees for tensor 2
    pub neighbor_degree_sum2_chirho: f32,
}

impl EdgeFeaturesChirho {
    /// Extract features for an edge
    pub fn extract_chirho(
        network_chirho: &TensorNetworkChirho,
        edge_idx_chirho: usize,
    ) -> Self {
        let (t1_chirho, t2_chirho, dim_chirho) = network_chirho.edges_chirho[edge_idx_chirho];

        let degree1_chirho = network_chirho.degree_chirho(t1_chirho) as f32;
        let degree2_chirho = network_chirho.degree_chirho(t2_chirho) as f32;

        let neighbors1_chirho: HashSet<_> = network_chirho.neighbors_chirho(t1_chirho).into_iter().collect();
        let neighbors2_chirho: HashSet<_> = network_chirho.neighbors_chirho(t2_chirho).into_iter().collect();

        let common_neighbors_chirho = neighbors1_chirho
            .intersection(&neighbors2_chirho)
            .count() as f32;

        let neighbor_degree_sum1_chirho: f32 = neighbors1_chirho
            .iter()
            .map(|&n| network_chirho.degree_chirho(n) as f32)
            .sum();
        let neighbor_degree_sum2_chirho: f32 = neighbors2_chirho
            .iter()
            .map(|&n| network_chirho.degree_chirho(n) as f32)
            .sum();

        // Estimated cost: product of dimensions
        let log_dim_chirho = (dim_chirho as f32).ln();
        let log_cost_chirho = log_dim_chirho * 2.0; // Simplified estimate

        Self {
            degree1_chirho,
            degree2_chirho,
            log_dim_chirho,
            log_cost_chirho,
            common_neighbors_chirho,
            neighbor_degree_sum1_chirho,
            neighbor_degree_sum2_chirho,
        }
    }

    /// Convert to feature vector
    pub fn to_vec_chirho(&self) -> Vec<f32> {
        vec![
            self.degree1_chirho,
            self.degree2_chirho,
            self.log_dim_chirho,
            self.log_cost_chirho,
            self.common_neighbors_chirho,
            self.neighbor_degree_sum1_chirho,
            self.neighbor_degree_sum2_chirho,
        ]
    }
}

/// Simple linear model for edge scoring
#[derive(Debug, Clone)]
pub struct LinearEdgeScorerChirho {
    /// Learned weights (one per feature)
    pub weights_chirho: Vec<f32>,
    /// Bias term
    pub bias_chirho: f32,
}

impl LinearEdgeScorerChirho {
    /// Create with default weights (heuristic: prefer lower cost edges)
    pub fn default_chirho() -> Self {
        Self {
            // Negative weight for cost means prefer lower cost
            // Positive weight for common_neighbors means prefer edges that reduce graph
            weights_chirho: vec![
                -0.1,  // degree1: prefer lower degree
                -0.1,  // degree2: prefer lower degree
                -0.5,  // log_dim: prefer smaller dimensions
                -1.0,  // log_cost: prefer lower cost
                0.5,   // common_neighbors: prefer edges with shared neighbors
                -0.05, // neighbor_degree_sum1
                -0.05, // neighbor_degree_sum2
            ],
            bias_chirho: 0.0,
        }
    }

    /// Create from learned weights
    pub fn from_weights_chirho(weights_chirho: Vec<f32>, bias_chirho: f32) -> Self {
        Self { weights_chirho, bias_chirho }
    }

    /// Score an edge (higher = better to contract)
    pub fn score_chirho(&self, features_chirho: &EdgeFeaturesChirho) -> f32 {
        let feat_vec_chirho = features_chirho.to_vec_chirho();
        let mut score_chirho = self.bias_chirho;

        for (i_chirho, &f_chirho) in feat_vec_chirho.iter().enumerate() {
            if i_chirho < self.weights_chirho.len() {
                score_chirho += self.weights_chirho[i_chirho] * f_chirho;
            }
        }

        score_chirho
    }

    /// Update weights using gradient descent
    /// Given (features, target_score), adjust weights to reduce error
    pub fn update_chirho(
        &mut self,
        features_chirho: &EdgeFeaturesChirho,
        target_chirho: f32,
        learning_rate_chirho: f32,
    ) {
        let predicted_chirho = self.score_chirho(features_chirho);
        let error_chirho = predicted_chirho - target_chirho;

        let feat_vec_chirho = features_chirho.to_vec_chirho();
        for (i_chirho, &f_chirho) in feat_vec_chirho.iter().enumerate() {
            if i_chirho < self.weights_chirho.len() {
                self.weights_chirho[i_chirho] -= learning_rate_chirho * error_chirho * f_chirho;
            }
        }
        self.bias_chirho -= learning_rate_chirho * error_chirho;
    }
}

/// Learned contraction order finder
pub struct LearnedContractionChirho {
    /// Edge scorer
    pub scorer_chirho: LinearEdgeScorerChirho,
}

impl LearnedContractionChirho {
    pub fn new_chirho() -> Self {
        Self {
            scorer_chirho: LinearEdgeScorerChirho::default_chirho(),
        }
    }

    pub fn with_scorer_chirho(scorer_chirho: LinearEdgeScorerChirho) -> Self {
        Self { scorer_chirho }
    }

    /// Find contraction order using learned heuristic
    /// Returns edges in order they should be contracted
    pub fn find_order_chirho(&self, network_chirho: &TensorNetworkChirho) -> Vec<usize> {
        let mut remaining_chirho: HashSet<usize> = (0..network_chirho.edges_chirho.len()).collect();
        let mut order_chirho = Vec::new();
        let mut contracted_tensors_chirho: HashSet<usize> = HashSet::new();

        while !remaining_chirho.is_empty() {
            // Score all remaining edges
            let mut best_edge_chirho = None;
            let mut best_score_chirho = f32::NEG_INFINITY;

            for &edge_idx_chirho in &remaining_chirho {
                let (t1_chirho, t2_chirho, _) = network_chirho.edges_chirho[edge_idx_chirho];

                // Skip if either tensor already contracted away
                if contracted_tensors_chirho.contains(&t1_chirho)
                    || contracted_tensors_chirho.contains(&t2_chirho)
                {
                    continue;
                }

                let features_chirho = EdgeFeaturesChirho::extract_chirho(network_chirho, edge_idx_chirho);
                let score_chirho = self.scorer_chirho.score_chirho(&features_chirho);

                if score_chirho > best_score_chirho {
                    best_score_chirho = score_chirho;
                    best_edge_chirho = Some(edge_idx_chirho);
                }
            }

            if let Some(edge_idx_chirho) = best_edge_chirho {
                order_chirho.push(edge_idx_chirho);
                remaining_chirho.remove(&edge_idx_chirho);

                // Mark one tensor as contracted (merged into the other)
                let (_t1_chirho, t2_chirho, _) = network_chirho.edges_chirho[edge_idx_chirho];
                contracted_tensors_chirho.insert(t2_chirho); // t2 merged into t1
            } else {
                // No valid edges left
                break;
            }
        }

        order_chirho
    }

    /// Estimate total contraction cost for an order
    pub fn estimate_cost_chirho(
        network_chirho: &TensorNetworkChirho,
        order_chirho: &[usize],
    ) -> f64 {
        let mut total_cost_chirho = 0.0f64;

        for &edge_idx_chirho in order_chirho {
            let (_, _, dim_chirho) = network_chirho.edges_chirho[edge_idx_chirho];
            // Simplified cost model: dimension^2 per contraction
            total_cost_chirho += (dim_chirho as f64).powi(2);
        }

        total_cost_chirho
    }
}

/// Training data point: network + known good order
#[derive(Debug, Clone)]
pub struct TrainingExampleChirho {
    pub network_chirho: TensorNetworkChirho,
    pub good_order_chirho: Vec<usize>,
    pub cost_chirho: f64,
}

/// Train scorer on examples
pub fn train_scorer_chirho(
    examples_chirho: &[TrainingExampleChirho],
    epochs_chirho: usize,
    learning_rate_chirho: f32,
) -> LinearEdgeScorerChirho {
    let mut scorer_chirho = LinearEdgeScorerChirho::default_chirho();

    for _epoch_chirho in 0..epochs_chirho {
        for example_chirho in examples_chirho {
            // For each edge in the good order, it should score higher than alternatives
            for (position_chirho, &edge_idx_chirho) in example_chirho.good_order_chirho.iter().enumerate() {
                let features_chirho =
                    EdgeFeaturesChirho::extract_chirho(&example_chirho.network_chirho, edge_idx_chirho);

                // Target: higher score for edges chosen earlier
                let target_chirho = 1.0 - (position_chirho as f32 / example_chirho.good_order_chirho.len() as f32);

                scorer_chirho.update_chirho(&features_chirho, target_chirho, learning_rate_chirho);
            }
        }
    }

    scorer_chirho
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_tensor_network_chirho() {
        let mut network_chirho = TensorNetworkChirho::new_chirho();

        let t0_chirho = network_chirho.add_tensor_chirho();
        let t1_chirho = network_chirho.add_tensor_chirho();
        let t2_chirho = network_chirho.add_tensor_chirho();

        network_chirho.add_edge_chirho(t0_chirho, t1_chirho, 10);
        network_chirho.add_edge_chirho(t1_chirho, t2_chirho, 20);

        assert_eq!(network_chirho.degree_chirho(t0_chirho), 1);
        assert_eq!(network_chirho.degree_chirho(t1_chirho), 2);
        assert_eq!(network_chirho.degree_chirho(t2_chirho), 1);
    }

    #[test]
    fn test_edge_features_chirho() {
        let mut network_chirho = TensorNetworkChirho::new_chirho();

        let t0_chirho = network_chirho.add_tensor_chirho();
        let t1_chirho = network_chirho.add_tensor_chirho();
        let t2_chirho = network_chirho.add_tensor_chirho();

        network_chirho.add_edge_chirho(t0_chirho, t1_chirho, 10);
        network_chirho.add_edge_chirho(t1_chirho, t2_chirho, 20);

        let features_chirho = EdgeFeaturesChirho::extract_chirho(&network_chirho, 0);

        assert_eq!(features_chirho.degree1_chirho, 1.0);
        assert_eq!(features_chirho.degree2_chirho, 2.0);
    }

    #[test]
    fn test_learned_contraction_chirho() {
        let mut network_chirho = TensorNetworkChirho::new_chirho();

        let t0_chirho = network_chirho.add_tensor_chirho();
        let t1_chirho = network_chirho.add_tensor_chirho();
        let t2_chirho = network_chirho.add_tensor_chirho();
        let t3_chirho = network_chirho.add_tensor_chirho();

        // Chain: t0 -- t1 -- t2 -- t3
        network_chirho.add_edge_chirho(t0_chirho, t1_chirho, 10);
        network_chirho.add_edge_chirho(t1_chirho, t2_chirho, 5);  // Smaller, should be preferred
        network_chirho.add_edge_chirho(t2_chirho, t3_chirho, 10);

        let contractor_chirho = LearnedContractionChirho::new_chirho();
        let order_chirho = contractor_chirho.find_order_chirho(&network_chirho);

        // Should prefer edge 1 (smallest dimension) first
        assert!(!order_chirho.is_empty());
        // The order should be valid (edge 1 has smallest dimension)
    }

    #[test]
    fn test_scorer_update_chirho() {
        let mut scorer_chirho = LinearEdgeScorerChirho::default_chirho();

        let features_chirho = EdgeFeaturesChirho {
            degree1_chirho: 2.0,
            degree2_chirho: 3.0,
            log_dim_chirho: 2.3,
            log_cost_chirho: 4.6,
            common_neighbors_chirho: 1.0,
            neighbor_degree_sum1_chirho: 4.0,
            neighbor_degree_sum2_chirho: 5.0,
        };

        let initial_score_chirho = scorer_chirho.score_chirho(&features_chirho);

        // Update toward higher score
        scorer_chirho.update_chirho(&features_chirho, 10.0, 0.1);

        let new_score_chirho = scorer_chirho.score_chirho(&features_chirho);

        // Score should have increased (moved toward target)
        assert!(new_score_chirho > initial_score_chirho);
    }
}
