//! Gumbel-Softmax for Differentiable Discrete Choices ☧
//!
//! Enables gradient flow through discrete branch selections (conde).

/// Gumbel-Softmax sampler for differentiable discrete choices
#[derive(Debug, Clone)]
pub struct GumbelSoftmaxSamplerChirho {
    /// Temperature for softmax
    pub temp_chirho: f64,
    /// Random seed for reproducibility
    seed_chirho: u64,
}

impl GumbelSoftmaxSamplerChirho {
    /// Create new sampler with given temperature
    pub fn new_chirho(temp_chirho: f64) -> Self {
        Self {
            temp_chirho,
            seed_chirho: 42,
        }
    }

    /// Create sampler with custom seed
    pub fn with_seed_chirho(temp_chirho: f64, seed_chirho: u64) -> Self {
        Self { temp_chirho, seed_chirho }
    }

    /// Simple LCG random number generator
    fn next_random_chirho(&mut self) -> f64 {
        self.seed_chirho = self.seed_chirho.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Convert to (0, 1)
        (self.seed_chirho as f64) / (u64::MAX as f64)
    }

    /// Sample Gumbel(0, 1) noise
    fn sample_gumbel_chirho(&mut self) -> f64 {
        let u_chirho = self.next_random_chirho().max(1e-20);
        -(-u_chirho.ln()).ln()
    }

    /// Apply Gumbel-Softmax to logits
    /// Returns soft one-hot vector (sums to ~1)
    pub fn sample_chirho(&mut self, logits_chirho: &[f64]) -> Vec<f64> {
        if logits_chirho.is_empty() {
            return vec![];
        }

        // Add Gumbel noise
        let gumbels_chirho: Vec<f64> = (0..logits_chirho.len())
            .map(|_| self.sample_gumbel_chirho())
            .collect();

        // Scale by temperature
        let scaled_chirho: Vec<f64> = logits_chirho.iter()
            .zip(gumbels_chirho.iter())
            .map(|(l_chirho, g_chirho)| (l_chirho + g_chirho) / self.temp_chirho.max(1e-10))
            .collect();

        // Softmax
        let max_chirho = scaled_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_chirho: Vec<f64> = scaled_chirho.iter()
            .map(|s_chirho| (s_chirho - max_chirho).exp())
            .collect();
        let sum_chirho: f64 = exp_chirho.iter().sum();

        exp_chirho.iter().map(|e_chirho| e_chirho / sum_chirho).collect()
    }

    /// Hard sample with straight-through estimator
    /// Returns one-hot vector (for forward) but uses soft (for backward)
    pub fn sample_hard_chirho(&mut self, logits_chirho: &[f64]) -> Vec<f64> {
        let soft_chirho = self.sample_chirho(logits_chirho);

        // Find argmax
        let max_idx_chirho = soft_chirho.iter()
            .enumerate()
            .max_by(|(_, a_chirho), (_, b_chirho)| a_chirho.partial_cmp(b_chirho).unwrap())
            .map(|(i_chirho, _)| i_chirho)
            .unwrap_or(0);

        // Create hard one-hot, but keep soft for gradient
        // In actual autodiff, this would be: hard - soft.detach() + soft
        let mut hard_chirho = vec![0.0; logits_chirho.len()];
        hard_chirho[max_idx_chirho] = 1.0;
        hard_chirho
    }

    /// Get the selected index (argmax of soft sample)
    pub fn sample_index_chirho(&mut self, logits_chirho: &[f64]) -> usize {
        let soft_chirho = self.sample_chirho(logits_chirho);
        soft_chirho.iter()
            .enumerate()
            .max_by(|(_, a_chirho), (_, b_chirho)| a_chirho.partial_cmp(b_chirho).unwrap())
            .map(|(i_chirho, _)| i_chirho)
            .unwrap_or(0)
    }
}

/// Straight-through estimator for hard discrete choices
pub struct StraightThroughEstimatorChirho;

impl StraightThroughEstimatorChirho {
    /// Forward: argmax (hard decision)
    pub fn forward_chirho(probs_chirho: &[f64]) -> usize {
        probs_chirho.iter()
            .enumerate()
            .max_by(|(_, a_chirho), (_, b_chirho)| a_chirho.partial_cmp(b_chirho).unwrap())
            .map(|(i_chirho, _)| i_chirho)
            .unwrap_or(0)
    }

    /// Backward: distribute gradient via softmax-like weights
    pub fn backward_chirho(probs_chirho: &[f64], grad_out_chirho: f64) -> Vec<f64> {
        probs_chirho.iter()
            .map(|p_chirho| p_chirho * grad_out_chirho)
            .collect()
    }
}

/// Differentiable branch selection for conde-like operations
#[derive(Debug, Clone)]
pub struct DifferentiableBranchChirho {
    /// Log-weights for each branch
    pub log_weights_chirho: Vec<f64>,
    /// Temperature
    pub temp_chirho: f64,
}

impl DifferentiableBranchChirho {
    /// Create with uniform weights
    pub fn uniform_chirho(num_branches_chirho: usize, temp_chirho: f64) -> Self {
        Self {
            log_weights_chirho: vec![0.0; num_branches_chirho],
            temp_chirho,
        }
    }

    /// Create with initial weights
    pub fn new_chirho(log_weights_chirho: Vec<f64>, temp_chirho: f64) -> Self {
        Self { log_weights_chirho, temp_chirho }
    }

    /// Get branch probabilities (softmax)
    pub fn branch_probs_chirho(&self) -> Vec<f64> {
        if self.log_weights_chirho.is_empty() {
            return vec![];
        }

        let max_chirho = self.log_weights_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_chirho: Vec<f64> = self.log_weights_chirho.iter()
            .map(|w_chirho| ((w_chirho - max_chirho) / self.temp_chirho.max(1e-10)).exp())
            .collect();
        let sum_chirho: f64 = exp_chirho.iter().sum();

        exp_chirho.iter().map(|e_chirho| e_chirho / sum_chirho).collect()
    }

    /// Soft conde: weighted combination of branch results
    pub fn soft_conde_chirho<F, R>(&self, branches_chirho: &[F]) -> f64
    where
        F: Fn() -> f64,
    {
        let probs_chirho = self.branch_probs_chirho();

        probs_chirho.iter()
            .zip(branches_chirho.iter())
            .map(|(p_chirho, branch_chirho)| p_chirho * branch_chirho())
            .sum()
    }

    /// Update weights based on gradient
    pub fn update_chirho(&mut self, gradients_chirho: &[f64], lr_chirho: f64) {
        for (w_chirho, g_chirho) in self.log_weights_chirho.iter_mut().zip(gradients_chirho.iter()) {
            *w_chirho -= lr_chirho * g_chirho;
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_gumbel_softmax_chirho() {
        let mut sampler_chirho = GumbelSoftmaxSamplerChirho::new_chirho(1.0);
        let logits_chirho = vec![0.0, 0.0, 0.0];

        let soft_chirho = sampler_chirho.sample_chirho(&logits_chirho);

        // Should sum to ~1
        let sum_chirho: f64 = soft_chirho.iter().sum();
        assert!((sum_chirho - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gumbel_temperature_chirho() {
        let logits_chirho = vec![2.0, 0.0, 0.0];

        // High temp: more uniform
        let mut high_temp_chirho = GumbelSoftmaxSamplerChirho::with_seed_chirho(10.0, 123);
        let soft_high_chirho = high_temp_chirho.sample_chirho(&logits_chirho);

        // Low temp: more peaked
        let mut low_temp_chirho = GumbelSoftmaxSamplerChirho::with_seed_chirho(0.1, 123);
        let soft_low_chirho = low_temp_chirho.sample_chirho(&logits_chirho);

        // Low temp should concentrate more on the highest logit
        assert!(soft_low_chirho[0] > soft_high_chirho[0] || soft_low_chirho[0] > 0.9);
    }

    #[test]
    fn test_differentiable_branch_chirho() {
        let branch_chirho = DifferentiableBranchChirho::uniform_chirho(3, 1.0);
        let probs_chirho = branch_chirho.branch_probs_chirho();

        // Uniform weights should give uniform probs
        for p_chirho in &probs_chirho {
            assert!((*p_chirho - 1.0/3.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_straight_through_chirho() {
        let probs_chirho = vec![0.1, 0.7, 0.2];
        let idx_chirho = StraightThroughEstimatorChirho::forward_chirho(&probs_chirho);
        assert_eq!(idx_chirho, 1);

        let grads_chirho = StraightThroughEstimatorChirho::backward_chirho(&probs_chirho, 1.0);
        assert!((grads_chirho[1] - 0.7).abs() < 1e-6);
    }
}
