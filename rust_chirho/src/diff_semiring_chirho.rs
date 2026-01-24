//! Differentiable Semiring ☧
//!
//! Extends the semiring abstraction with gradient computation,
//! enabling gradient-based learning through logic programs.
//!
//! Key insight: logic operations become differentiable when we
//! relax Boolean AND/OR to probability multiplication/sum.

/// Differentiable probability value with gradient tracking
#[derive(Clone, Debug, Default)]
pub struct DiffProbChirho {
    /// Forward value (probability in [0, 1])
    pub value_chirho: f64,
    /// Accumulated gradient ∂L/∂value
    pub grad_chirho: f64,
}

impl DiffProbChirho {
    /// Create a new differentiable probability
    pub fn new_chirho(value_chirho: f64) -> Self {
        Self {
            value_chirho: value_chirho.clamp(0.0, 1.0),
            grad_chirho: 0.0,
        }
    }

    /// Create from a probability, requiring gradient tracking
    pub fn with_grad_chirho(value_chirho: f64, grad_chirho: f64) -> Self {
        Self {
            value_chirho: value_chirho.clamp(0.0, 1.0),
            grad_chirho,
        }
    }

    /// Soft AND: a ∧ b ≈ a * b
    ///
    /// Forward: value = a * b
    /// Backward: ∂L/∂a = ∂L/∂out * b, ∂L/∂b = ∂L/∂out * a
    pub fn and_chirho(&self, other_chirho: &Self) -> Self {
        Self {
            value_chirho: self.value_chirho * other_chirho.value_chirho,
            grad_chirho: 0.0, // Accumulated during backward pass
        }
    }

    /// Soft AND with gradient propagation
    pub fn and_with_grad_chirho(
        &self,
        other_chirho: &Self,
        grad_out_chirho: f64,
    ) -> (Self, f64, f64) {
        let out_chirho = Self::new_chirho(self.value_chirho * other_chirho.value_chirho);
        let grad_a_chirho = grad_out_chirho * other_chirho.value_chirho;
        let grad_b_chirho = grad_out_chirho * self.value_chirho;
        (out_chirho, grad_a_chirho, grad_b_chirho)
    }

    /// Soft OR: a ∨ b ≈ a + b - a*b (inclusion-exclusion)
    ///
    /// Forward: value = a + b - a*b
    /// Backward: ∂L/∂a = ∂L/∂out * (1 - b), ∂L/∂b = ∂L/∂out * (1 - a)
    pub fn or_chirho(&self, other_chirho: &Self) -> Self {
        let a_chirho = self.value_chirho;
        let b_chirho = other_chirho.value_chirho;
        Self {
            value_chirho: a_chirho + b_chirho - a_chirho * b_chirho,
            grad_chirho: 0.0,
        }
    }

    /// Soft OR with gradient propagation
    pub fn or_with_grad_chirho(
        &self,
        other_chirho: &Self,
        grad_out_chirho: f64,
    ) -> (Self, f64, f64) {
        let a_chirho = self.value_chirho;
        let b_chirho = other_chirho.value_chirho;
        let out_chirho = Self::new_chirho(a_chirho + b_chirho - a_chirho * b_chirho);
        let grad_a_chirho = grad_out_chirho * (1.0 - b_chirho);
        let grad_b_chirho = grad_out_chirho * (1.0 - a_chirho);
        (out_chirho, grad_a_chirho, grad_b_chirho)
    }

    /// Soft NOT: ¬a ≈ 1 - a
    pub fn not_chirho(&self) -> Self {
        Self {
            value_chirho: 1.0 - self.value_chirho,
            grad_chirho: 0.0,
        }
    }

    /// Soft NOT with gradient propagation
    pub fn not_with_grad_chirho(&self, grad_out_chirho: f64) -> (Self, f64) {
        let out_chirho = Self::new_chirho(1.0 - self.value_chirho);
        let grad_chirho = -grad_out_chirho; // ∂(1-a)/∂a = -1
        (out_chirho, grad_chirho)
    }
}

/// Temperature-controlled soft equality
///
/// temp → 0: approaches hard equality (1 if equal, 0 otherwise)
/// temp → ∞: everything is equally likely
pub fn soft_eq_chirho(a_chirho: f64, b_chirho: f64, temp_chirho: f64) -> f64 {
    let diff_chirho = (a_chirho - b_chirho).abs();
    (-diff_chirho / temp_chirho.max(1e-10)).exp()
}

/// Temperature-controlled soft equality with gradient
///
/// Returns (probability, ∂prob/∂a, ∂prob/∂b)
pub fn soft_eq_with_grad_chirho(a_chirho: f64, b_chirho: f64, temp_chirho: f64) -> (f64, f64, f64) {
    let diff_chirho = a_chirho - b_chirho;
    let abs_diff_chirho = diff_chirho.abs();
    let t_chirho = temp_chirho.max(1e-10);
    let prob_chirho = (-abs_diff_chirho / t_chirho).exp();

    // ∂prob/∂a = prob * (-sign(a-b) / temp)
    let sign_chirho = if diff_chirho >= 0.0 { 1.0 } else { -1.0 };
    let grad_a_chirho = prob_chirho * (-sign_chirho / t_chirho);
    let grad_b_chirho = -grad_a_chirho;

    (prob_chirho, grad_a_chirho, grad_b_chirho)
}

/// Temperature annealing schedule
///
/// Starts with high temperature (soft), ends with low temperature (hard).
pub fn annealed_temp_chirho(
    initial_temp_chirho: f64,
    step_chirho: usize,
    total_steps_chirho: usize,
    min_temp_chirho: f64,
) -> f64 {
    // Exponential decay: T(t) = T0 * (T_min/T0)^(t/T)
    let progress_chirho = (step_chirho as f64) / (total_steps_chirho as f64).max(1.0);
    let ratio_chirho = (min_temp_chirho / initial_temp_chirho).max(1e-10);
    initial_temp_chirho * ratio_chirho.powf(progress_chirho)
}

/// Gumbel-Softmax: differentiable approximation to argmax
///
/// Enables gradient flow through discrete branch selection.
/// At low temperature, approaches one-hot; at high temperature, approaches uniform.
#[derive(Clone, Debug)]
pub struct GumbelSoftmaxChirho {
    /// Temperature parameter
    pub temp_chirho: f64,
}

impl GumbelSoftmaxChirho {
    pub fn new_chirho(temp_chirho: f64) -> Self {
        Self {
            temp_chirho: temp_chirho.max(1e-10),
        }
    }

    /// Sample from Gumbel-Softmax distribution
    ///
    /// Given log-probabilities (logits), returns soft one-hot vector.
    pub fn sample_chirho(&self, logits_chirho: &[f64], rng_seed_chirho: u64) -> Vec<f64> {
        // Generate Gumbel noise: -log(-log(U)) where U ~ Uniform(0,1)
        let mut gumbels_chirho = Vec::with_capacity(logits_chirho.len());
        let mut seed_chirho = rng_seed_chirho;
        for _ in 0..logits_chirho.len() {
            // Simple LCG for reproducible random numbers
            seed_chirho = seed_chirho.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u_chirho = (seed_chirho as f64) / (u64::MAX as f64);
            let u_clamped_chirho = u_chirho.clamp(1e-10, 1.0 - 1e-10);
            let gumbel_chirho = -((-u_clamped_chirho.ln()).ln());
            gumbels_chirho.push(gumbel_chirho);
        }

        // y_i = softmax((logit_i + gumbel_i) / temperature)
        let scaled_chirho: Vec<f64> = logits_chirho
            .iter()
            .zip(gumbels_chirho.iter())
            .map(|(&l, &g)| (l + g) / self.temp_chirho)
            .collect();

        softmax_chirho(&scaled_chirho)
    }

    /// Compute gradients for Gumbel-Softmax
    ///
    /// Returns gradient of loss w.r.t. each logit.
    pub fn backward_chirho(&self, probs_chirho: &[f64], grad_out_chirho: &[f64]) -> Vec<f64> {
        // Softmax gradient: ∂softmax_i/∂logit_j = softmax_i * (δ_ij - softmax_j) / temp
        let n_chirho = probs_chirho.len();
        let mut grads_chirho = vec![0.0; n_chirho];

        for i in 0..n_chirho {
            for j in 0..n_chirho {
                let jacobian_chirho = if i == j {
                    probs_chirho[i] * (1.0 - probs_chirho[j])
                } else {
                    -probs_chirho[i] * probs_chirho[j]
                };
                grads_chirho[j] += grad_out_chirho[i] * jacobian_chirho / self.temp_chirho;
            }
        }

        grads_chirho
    }
}

/// Straight-Through Estimator for hard constraints
///
/// Forward: uses hard decision (argmax)
/// Backward: uses soft gradient (softmax)
#[derive(Clone, Debug)]
pub struct StraightThroughChirho;

impl StraightThroughChirho {
    /// Forward pass: return argmax (hard decision)
    pub fn forward_chirho(probs_chirho: &[f64]) -> usize {
        probs_chirho
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Backward pass: distribute gradient via softmax
    pub fn backward_chirho(probs_chirho: &[f64], grad_out_chirho: f64) -> Vec<f64> {
        probs_chirho.iter().map(|p| p * grad_out_chirho).collect()
    }

    /// Combined forward-backward for one-hot selection
    ///
    /// Returns (selected_index, one_hot_soft) where one_hot_soft can be used
    /// for gradient computation while selected_index gives the hard decision.
    pub fn forward_backward_chirho(probs_chirho: &[f64]) -> (usize, Vec<f64>) {
        let idx_chirho = Self::forward_chirho(probs_chirho);
        let soft_chirho = probs_chirho.to_vec();
        (idx_chirho, soft_chirho)
    }
}

/// Softmax function
fn softmax_chirho(logits_chirho: &[f64]) -> Vec<f64> {
    let max_chirho = logits_chirho
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let exp_chirho: Vec<f64> = logits_chirho.iter().map(|&l| (l - max_chirho).exp()).collect();
    let sum_chirho: f64 = exp_chirho.iter().sum();
    exp_chirho.iter().map(|&e| e / sum_chirho).collect()
}

/// Log-sum-exp (numerically stable)
pub fn log_sum_exp_chirho(values_chirho: &[f64]) -> f64 {
    if values_chirho.is_empty() {
        return f64::NEG_INFINITY;
    }
    let max_chirho = values_chirho
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    if max_chirho.is_infinite() {
        return max_chirho;
    }
    let sum_exp_chirho: f64 = values_chirho.iter().map(|&v| (v - max_chirho).exp()).sum();
    max_chirho + sum_exp_chirho.ln()
}

/// Differentiable relation tuple with learnable weight
#[derive(Clone, Debug)]
pub struct WeightedTupleChirho {
    /// Log-weight (for numerical stability)
    pub log_weight_chirho: f64,
    /// Gradient of loss w.r.t. log_weight
    pub grad_log_weight_chirho: f64,
    /// Tuple data (indices into term store)
    pub data_chirho: Vec<u32>,
}

impl WeightedTupleChirho {
    pub fn new_chirho(data_chirho: Vec<u32>) -> Self {
        Self {
            log_weight_chirho: 0.0, // weight = 1.0
            grad_log_weight_chirho: 0.0,
            data_chirho,
        }
    }

    /// Get normalized weight
    pub fn weight_chirho(&self) -> f64 {
        self.log_weight_chirho.exp()
    }

    /// Apply gradient descent step
    pub fn update_chirho(&mut self, lr_chirho: f64) {
        self.log_weight_chirho -= lr_chirho * self.grad_log_weight_chirho;
        self.grad_log_weight_chirho = 0.0;
    }
}

/// Learnable relation with differentiable tuple weights
#[derive(Clone, Debug)]
pub struct LearnableRelationChirho {
    pub name_chirho: String,
    pub tuples_chirho: Vec<WeightedTupleChirho>,
    pub arity_chirho: usize,
}

impl LearnableRelationChirho {
    pub fn new_chirho(name_chirho: &str, arity_chirho: usize) -> Self {
        Self {
            name_chirho: name_chirho.to_string(),
            tuples_chirho: Vec::new(),
            arity_chirho,
        }
    }

    /// Add a tuple with initial weight 1.0
    pub fn add_tuple_chirho(&mut self, data_chirho: Vec<u32>) {
        assert_eq!(data_chirho.len(), self.arity_chirho);
        self.tuples_chirho.push(WeightedTupleChirho::new_chirho(data_chirho));
    }

    /// Get normalized weights (softmax over log-weights)
    pub fn normalized_weights_chirho(&self) -> Vec<f64> {
        let log_weights_chirho: Vec<f64> = self.tuples_chirho.iter().map(|t| t.log_weight_chirho).collect();
        softmax_chirho(&log_weights_chirho)
    }

    /// Apply gradient descent to all tuples
    pub fn update_all_chirho(&mut self, lr_chirho: f64) {
        for tuple_chirho in &mut self.tuples_chirho {
            tuple_chirho.update_chirho(lr_chirho);
        }
    }

    /// Zero all gradients
    pub fn zero_grad_chirho(&mut self) {
        for tuple_chirho in &mut self.tuples_chirho {
            tuple_chirho.grad_log_weight_chirho = 0.0;
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_diff_prob_and_chirho() {
        let a_chirho = DiffProbChirho::new_chirho(0.8);
        let b_chirho = DiffProbChirho::new_chirho(0.9);
        let result_chirho = a_chirho.and_chirho(&b_chirho);
        assert!((result_chirho.value_chirho - 0.72).abs() < 1e-10);
    }

    #[test]
    fn test_diff_prob_or_chirho() {
        let a_chirho = DiffProbChirho::new_chirho(0.3);
        let b_chirho = DiffProbChirho::new_chirho(0.4);
        let result_chirho = a_chirho.or_chirho(&b_chirho);
        // 0.3 + 0.4 - 0.3*0.4 = 0.58
        assert!((result_chirho.value_chirho - 0.58).abs() < 1e-10);
    }

    #[test]
    fn test_soft_eq_chirho() {
        // Same value → probability 1
        let eq_prob_chirho = soft_eq_chirho(0.5, 0.5, 0.1);
        assert!((eq_prob_chirho - 1.0).abs() < 1e-10);

        // Different values → lower probability
        let neq_prob_chirho = soft_eq_chirho(0.0, 1.0, 1.0);
        assert!(neq_prob_chirho < 0.5);
    }

    #[test]
    fn test_temperature_annealing_chirho() {
        let temp_start_chirho = annealed_temp_chirho(1.0, 0, 100, 0.01);
        let temp_end_chirho = annealed_temp_chirho(1.0, 100, 100, 0.01);

        assert!((temp_start_chirho - 1.0).abs() < 1e-10);
        assert!((temp_end_chirho - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_gumbel_softmax_chirho() {
        let gs_chirho = GumbelSoftmaxChirho::new_chirho(0.1);
        let logits_chirho = vec![0.0, 1.0, -1.0];
        let probs_chirho = gs_chirho.sample_chirho(&logits_chirho, 42);

        // Probs should sum to 1
        let sum_chirho: f64 = probs_chirho.iter().sum();
        assert!((sum_chirho - 1.0).abs() < 1e-10);

        // With low temp, should be close to one-hot
        let max_prob_chirho = probs_chirho.iter().cloned().fold(0.0, f64::max);
        assert!(max_prob_chirho > 0.9);
    }

    #[test]
    fn test_straight_through_chirho() {
        let probs_chirho = vec![0.1, 0.6, 0.3];
        let idx_chirho = StraightThroughChirho::forward_chirho(&probs_chirho);
        assert_eq!(idx_chirho, 1); // Max is at index 1

        let grads_chirho = StraightThroughChirho::backward_chirho(&probs_chirho, 1.0);
        assert_eq!(grads_chirho.len(), 3);
        assert!((grads_chirho[1] - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_learnable_relation_chirho() {
        let mut rel_chirho = LearnableRelationChirho::new_chirho("appendo", 3);
        rel_chirho.add_tuple_chirho(vec![0, 0, 0]);
        rel_chirho.add_tuple_chirho(vec![1, 1, 2]);

        let weights_chirho = rel_chirho.normalized_weights_chirho();
        assert_eq!(weights_chirho.len(), 2);
        assert!((weights_chirho[0] - 0.5).abs() < 1e-10);
        assert!((weights_chirho[1] - 0.5).abs() < 1e-10);
    }
}
