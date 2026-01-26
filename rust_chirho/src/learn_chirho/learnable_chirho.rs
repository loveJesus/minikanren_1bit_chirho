// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Learnable Relations ☧
//!
//! Relations with learnable tuple weights for gradient-based training.

use crate::semiring_chirho::diff_semiring_chirho::DiffProbChirho;

/// A relation with learnable weights on each tuple
#[derive(Debug, Clone)]
pub struct LearnableRelationExtChirho<T: Clone + Eq + std::hash::Hash> {
    /// Tuples in the relation
    pub tuples_chirho: Vec<T>,
    /// Log-weights for numerical stability (softmax over these)
    pub log_weights_chirho: Vec<f64>,
    /// Learning rate
    pub lr_chirho: f64,
    /// Accumulated gradients (for batch updates)
    accumulated_grads_chirho: Vec<f64>,
}

impl<T: Clone + Eq + std::hash::Hash> LearnableRelationExtChirho<T> {
    /// Create a new learnable relation with uniform weights
    pub fn new_chirho(tuples_chirho: Vec<T>, lr_chirho: f64) -> Self {
        let n_chirho = tuples_chirho.len();
        Self {
            tuples_chirho,
            log_weights_chirho: vec![0.0; n_chirho],
            lr_chirho,
            accumulated_grads_chirho: vec![0.0; n_chirho],
        }
    }

    /// Get normalized weights via softmax
    pub fn weights_chirho(&self) -> Vec<f64> {
        if self.log_weights_chirho.is_empty() {
            return vec![];
        }

        let max_chirho = self.log_weights_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_chirho: Vec<f64> = self.log_weights_chirho.iter()
            .map(|w_chirho| (w_chirho - max_chirho).exp())
            .collect();
        let sum_chirho: f64 = exp_chirho.iter().sum();

        exp_chirho.iter().map(|e_chirho| e_chirho / sum_chirho).collect()
    }

    /// Get weight for a specific tuple index
    pub fn weight_at_chirho(&self, idx_chirho: usize) -> f64 {
        let weights_chirho = self.weights_chirho();
        weights_chirho.get(idx_chirho).copied().unwrap_or(0.0)
    }

    /// Query the relation with a soft match function
    /// Returns probability that the pattern matches any tuple
    pub fn soft_query_chirho<F>(&self, match_fn_chirho: F) -> DiffProbChirho
    where
        F: Fn(&T) -> f64,
    {
        let weights_chirho = self.weights_chirho();
        let mut total_prob_chirho = 0.0;

        for (tuple_chirho, weight_chirho) in self.tuples_chirho.iter().zip(weights_chirho.iter()) {
            let match_prob_chirho = match_fn_chirho(tuple_chirho);
            total_prob_chirho += match_prob_chirho * weight_chirho;
        }

        DiffProbChirho::new_chirho(total_prob_chirho)
    }

    /// Accumulate gradient for a tuple
    pub fn accumulate_grad_chirho(&mut self, idx_chirho: usize, grad_chirho: f64) {
        if idx_chirho < self.accumulated_grads_chirho.len() {
            self.accumulated_grads_chirho[idx_chirho] += grad_chirho;
        }
    }

    /// Apply accumulated gradients (SGD step)
    pub fn apply_gradients_chirho(&mut self) {
        for (w_chirho, g_chirho) in self.log_weights_chirho.iter_mut()
            .zip(self.accumulated_grads_chirho.iter())
        {
            *w_chirho -= self.lr_chirho * g_chirho;
        }
        // Reset accumulated gradients
        self.accumulated_grads_chirho.fill(0.0);
    }

    /// Train on a single example
    /// Returns the loss
    pub fn train_step_chirho<F>(&mut self, match_fn_chirho: F, target_chirho: f64) -> f64
    where
        F: Fn(&T) -> f64,
    {
        let weights_chirho = self.weights_chirho();
        let mut pred_chirho = 0.0;
        let mut match_probs_chirho = Vec::with_capacity(self.tuples_chirho.len());

        // Forward pass
        for (tuple_chirho, weight_chirho) in self.tuples_chirho.iter().zip(weights_chirho.iter()) {
            let match_prob_chirho = match_fn_chirho(tuple_chirho);
            match_probs_chirho.push(match_prob_chirho);
            pred_chirho += match_prob_chirho * weight_chirho;
        }

        // Loss = (pred - target)^2
        let loss_chirho = (pred_chirho - target_chirho).powi(2);
        let grad_pred_chirho = 2.0 * (pred_chirho - target_chirho);

        // Backward pass: gradient w.r.t. log_weights
        for (i_chirho, (weight_chirho, match_prob_chirho)) in weights_chirho.iter()
            .zip(match_probs_chirho.iter())
            .enumerate()
        {
            // Gradient of softmax: dL/d(log_w_i) = dL/d(pred) * d(pred)/d(w_i) * d(w_i)/d(log_w_i)
            // d(pred)/d(w_i) = match_prob_i
            // d(w_i)/d(log_w_i) = w_i * (1 - w_i) for the specific term
            // Simplified: gradient flows through softmax
            let grad_chirho = grad_pred_chirho * match_prob_chirho * weight_chirho * (1.0 - weight_chirho);
            self.accumulated_grads_chirho[i_chirho] += grad_chirho;
        }

        // Apply gradients immediately (online learning)
        self.apply_gradients_chirho();

        loss_chirho
    }

    /// Get index of tuple with highest weight
    pub fn most_likely_chirho(&self) -> Option<usize> {
        let weights_chirho = self.weights_chirho();
        weights_chirho.iter()
            .enumerate()
            .max_by(|(_, a_chirho), (_, b_chirho)| a_chirho.partial_cmp(b_chirho).unwrap())
            .map(|(i_chirho, _)| i_chirho)
    }
}

/// Binary learnable relation (two-argument)
pub type BinaryRelationChirho<A, B> = LearnableRelationExtChirho<(A, B)>;

/// Ternary learnable relation (three-argument)
pub type TernaryRelationChirho<A, B, C> = LearnableRelationExtChirho<(A, B, C)>;

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_learnable_relation_chirho() {
        let tuples_chirho = vec![
            ("alice", "bob"),
            ("bob", "charlie"),
            ("eve", "dave"),
        ];
        let mut rel_chirho = BinaryRelationChirho::new_chirho(tuples_chirho, 0.1);

        // Initial weights should be uniform
        let weights_chirho = rel_chirho.weights_chirho();
        assert!((weights_chirho[0] - weights_chirho[1]).abs() < 1e-6);

        // Train to prefer ("alice", "bob")
        for _ in 0..100 {
            rel_chirho.train_step_chirho(
                |t_chirho| if *t_chirho == ("alice", "bob") { 1.0 } else { 0.0 },
                1.0,
            );
        }

        // Weight for ("alice", "bob") should be highest
        let weights_chirho = rel_chirho.weights_chirho();
        assert!(weights_chirho[0] > weights_chirho[1]);
        assert!(weights_chirho[0] > weights_chirho[2]);
    }

    #[test]
    fn test_soft_query_chirho() {
        let tuples_chirho = vec![(1, 2), (2, 3), (3, 4)];
        let rel_chirho = BinaryRelationChirho::new_chirho(tuples_chirho, 0.1);

        // Query: does (1, 2) exist?
        let result_chirho = rel_chirho.soft_query_chirho(|t_chirho| {
            if *t_chirho == (1, 2) { 1.0 } else { 0.0 }
        });

        // Should have ~1/3 probability (uniform weights)
        assert!(result_chirho.value_chirho > 0.3 && result_chirho.value_chirho < 0.4);
    }
}
