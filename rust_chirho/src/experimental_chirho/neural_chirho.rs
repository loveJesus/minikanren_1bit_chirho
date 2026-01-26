// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Neural-Guided Search ☧
//!
//! Soft/differentiable relaxation of Boolean tensor operations.
//! Instead of hard 0/1, use probabilities in [0,1].
//! This enables gradient-based optimization and learned heuristics.
//!
//! Key insight: Boolean AND/OR can be relaxed to:
//!   - AND(a,b) → a * b (product)
//!   - OR(a,b) → a + b - a*b (probabilistic or)
//!   - NOT(a) → 1 - a
//!
//! At the limit (a,b → {0,1}), these recover exact Boolean logic.

/// Soft domain: probabilities instead of bits
/// Each value 0..N has a probability of being the true value
#[derive(Debug, Clone)]
pub struct SoftDomainChirho {
    /// Probability distribution over values
    pub probs_chirho: Vec<f32>,
}

impl SoftDomainChirho {
    /// Create uniform distribution over n values
    pub fn uniform_chirho(n_chirho: usize) -> Self {
        let p_chirho = 1.0 / n_chirho as f32;
        Self {
            probs_chirho: vec![p_chirho; n_chirho],
        }
    }

    /// Create singleton (one value has prob 1.0)
    pub fn singleton_chirho(val_chirho: usize, n_chirho: usize) -> Self {
        let mut probs_chirho = vec![0.0; n_chirho];
        if val_chirho < n_chirho {
            probs_chirho[val_chirho] = 1.0;
        }
        Self { probs_chirho }
    }

    /// Soft unification: element-wise product + renormalization
    /// If domains are compatible, result is normalized intersection
    /// If incompatible, result approaches zero everywhere
    pub fn unify_chirho(&self, other_chirho: &Self) -> Self {
        let n_chirho = self.probs_chirho.len().min(other_chirho.probs_chirho.len());
        let mut probs_chirho = vec![0.0; n_chirho];
        let mut sum_chirho = 0.0f32;

        for i_chirho in 0..n_chirho {
            probs_chirho[i_chirho] = self.probs_chirho[i_chirho] * other_chirho.probs_chirho[i_chirho];
            sum_chirho += probs_chirho[i_chirho];
        }

        // Renormalize (avoid division by zero)
        if sum_chirho > 1e-10 {
            for p_chirho in &mut probs_chirho {
                *p_chirho /= sum_chirho;
            }
        }

        Self { probs_chirho }
    }

    /// Soft disjunction: probabilistic OR
    pub fn disjoin_chirho(&self, other_chirho: &Self) -> Self {
        let n_chirho = self.probs_chirho.len().max(other_chirho.probs_chirho.len());
        let mut probs_chirho = vec![0.0; n_chirho];

        for i_chirho in 0..n_chirho {
            let a_chirho = self.probs_chirho.get(i_chirho).copied().unwrap_or(0.0);
            let b_chirho = other_chirho.probs_chirho.get(i_chirho).copied().unwrap_or(0.0);
            // Probabilistic OR: P(A ∪ B) = P(A) + P(B) - P(A)P(B)
            probs_chirho[i_chirho] = a_chirho + b_chirho - a_chirho * b_chirho;
        }

        Self { probs_chirho }
    }

    /// Entropy of distribution (uncertainty measure)
    pub fn entropy_chirho(&self) -> f32 {
        let mut h_chirho = 0.0f32;
        for &p_chirho in &self.probs_chirho {
            if p_chirho > 1e-10 {
                h_chirho -= p_chirho * p_chirho.ln();
            }
        }
        h_chirho
    }

    /// Most likely value
    pub fn argmax_chirho(&self) -> usize {
        self.probs_chirho
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Apply temperature (sharpen/soften distribution)
    /// temp < 1 → sharper (more deterministic)
    /// temp > 1 → softer (more uniform)
    pub fn with_temperature_chirho(&self, temp_chirho: f32) -> Self {
        let mut probs_chirho: Vec<f32> = self
            .probs_chirho
            .iter()
            .map(|&p| p.powf(1.0 / temp_chirho))
            .collect();

        let sum_chirho: f32 = probs_chirho.iter().sum();
        if sum_chirho > 1e-10 {
            for p_chirho in &mut probs_chirho {
                *p_chirho /= sum_chirho;
            }
        }

        Self { probs_chirho }
    }

    /// Convert to hard domain (threshold at 0.5)
    pub fn to_hard_chirho(&self) -> u64 {
        let mut result_chirho = 0u64;
        for (i_chirho, &p_chirho) in self.probs_chirho.iter().enumerate() {
            if i_chirho < 64 && p_chirho > 0.5 {
                result_chirho |= 1u64 << i_chirho;
            }
        }
        result_chirho
    }
}

/// Neural search state with soft domains
#[derive(Debug, Clone)]
pub struct NeuralStateChirho {
    /// Soft domains for each variable
    pub domains_chirho: Vec<SoftDomainChirho>,
    /// Log probability of this state (for beam search)
    pub log_prob_chirho: f32,
}

impl NeuralStateChirho {
    /// Create initial state with uniform distributions
    pub fn new_chirho(num_vars_chirho: usize, domain_size_chirho: usize) -> Self {
        Self {
            domains_chirho: vec![SoftDomainChirho::uniform_chirho(domain_size_chirho); num_vars_chirho],
            log_prob_chirho: 0.0,
        }
    }

    /// Soft unify two variables
    pub fn unify_chirho(&mut self, var1_chirho: usize, var2_chirho: usize) {
        if var1_chirho < self.domains_chirho.len() && var2_chirho < self.domains_chirho.len() {
            let unified_chirho = self.domains_chirho[var1_chirho].unify_chirho(&self.domains_chirho[var2_chirho]);
            self.domains_chirho[var1_chirho] = unified_chirho.clone();
            self.domains_chirho[var2_chirho] = unified_chirho;
        }
    }

    /// Total entropy (uncertainty in state)
    pub fn total_entropy_chirho(&self) -> f32 {
        self.domains_chirho.iter().map(|d| d.entropy_chirho()).sum()
    }

    /// Apply constraint with learned weight
    /// Higher weight → more influence on distribution
    pub fn apply_weighted_constraint_chirho(
        &mut self,
        var_chirho: usize,
        target_probs_chirho: &[f32],
        weight_chirho: f32,
    ) {
        if var_chirho >= self.domains_chirho.len() {
            return;
        }

        let domain_chirho = &mut self.domains_chirho[var_chirho];
        let n_chirho = domain_chirho.probs_chirho.len().min(target_probs_chirho.len());

        // Weighted geometric mean of current and target
        let mut sum_chirho = 0.0f32;
        for i_chirho in 0..n_chirho {
            let curr_chirho = domain_chirho.probs_chirho[i_chirho];
            let target_chirho = target_probs_chirho[i_chirho];
            // Interpolate in log space
            domain_chirho.probs_chirho[i_chirho] =
                (curr_chirho.ln() * (1.0 - weight_chirho) + target_chirho.max(1e-10).ln() * weight_chirho).exp();
            sum_chirho += domain_chirho.probs_chirho[i_chirho];
        }

        // Renormalize
        if sum_chirho > 1e-10 {
            for p_chirho in &mut domain_chirho.probs_chirho {
                *p_chirho /= sum_chirho;
            }
        }
    }
}

/// Simple neural heuristic for variable/value ordering
/// In practice, this would be a trained neural network
#[derive(Debug, Clone)]
pub struct NeuralHeuristicChirho {
    /// Learned weights for variable selection (higher = select first)
    pub var_weights_chirho: Vec<f32>,
    /// Learned biases for value ordering per variable
    pub val_biases_chirho: Vec<Vec<f32>>,
}

impl NeuralHeuristicChirho {
    /// Create with uniform weights (no learned preference)
    pub fn uniform_chirho(num_vars_chirho: usize, domain_size_chirho: usize) -> Self {
        Self {
            var_weights_chirho: vec![1.0; num_vars_chirho],
            val_biases_chirho: vec![vec![0.0; domain_size_chirho]; num_vars_chirho],
        }
    }

    /// Score a variable for selection (MRV + learned weight)
    pub fn score_var_chirho(&self, var_chirho: usize, domain_chirho: &SoftDomainChirho) -> f32 {
        let entropy_chirho = domain_chirho.entropy_chirho();
        let weight_chirho = self.var_weights_chirho.get(var_chirho).copied().unwrap_or(1.0);

        // Lower entropy (more constrained) + higher weight = higher score
        weight_chirho - entropy_chirho
    }

    /// Select best variable to branch on
    pub fn select_var_chirho(&self, state_chirho: &NeuralStateChirho) -> Option<usize> {
        state_chirho
            .domains_chirho
            .iter()
            .enumerate()
            .filter(|(_, d)| d.probs_chirho.iter().filter(|&&p| p > 0.01).count() > 1)
            .max_by(|(i, d1), (j, d2)| {
                self.score_var_chirho(*i, d1)
                    .partial_cmp(&self.score_var_chirho(*j, d2))
                    .unwrap()
            })
            .map(|(i, _)| i)
    }

    /// Order values for a variable (apply learned biases to probabilities)
    pub fn order_values_chirho(&self, var_chirho: usize, domain_chirho: &SoftDomainChirho) -> Vec<(usize, f32)> {
        let biases_chirho = self.val_biases_chirho.get(var_chirho);

        let mut scored_chirho: Vec<(usize, f32)> = domain_chirho
            .probs_chirho
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                let bias_chirho = biases_chirho
                    .and_then(|b| b.get(i))
                    .copied()
                    .unwrap_or(0.0);
                (i, p + bias_chirho)
            })
            .filter(|(_, score)| *score > 0.01)
            .collect();

        scored_chirho.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        scored_chirho
    }
}

/// Beam search with neural guidance
pub fn beam_search_chirho(
    initial_chirho: NeuralStateChirho,
    heuristic_chirho: &NeuralHeuristicChirho,
    beam_width_chirho: usize,
    max_steps_chirho: usize,
) -> Vec<NeuralStateChirho> {
    let mut beam_chirho = vec![initial_chirho];
    let mut solutions_chirho = Vec::new();

    for _step_chirho in 0..max_steps_chirho {
        let mut next_beam_chirho = Vec::new();

        for state_chirho in &beam_chirho {
            // Check if solved (all singletons)
            if state_chirho.domains_chirho.iter().all(|d| {
                d.probs_chirho.iter().filter(|&&p| p > 0.99).count() == 1
            }) {
                solutions_chirho.push(state_chirho.clone());
                continue;
            }

            // Select variable to branch on
            if let Some(var_chirho) = heuristic_chirho.select_var_chirho(state_chirho) {
                // Get ordered values
                let values_chirho = heuristic_chirho.order_values_chirho(
                    var_chirho,
                    &state_chirho.domains_chirho[var_chirho],
                );

                // Create successor states
                for (val_chirho, score_chirho) in values_chirho.iter().take(3) {
                    let mut next_chirho = state_chirho.clone();
                    next_chirho.domains_chirho[var_chirho] =
                        SoftDomainChirho::singleton_chirho(*val_chirho, next_chirho.domains_chirho[var_chirho].probs_chirho.len());
                    next_chirho.log_prob_chirho += score_chirho.ln();
                    next_beam_chirho.push(next_chirho);
                }
            }
        }

        if next_beam_chirho.is_empty() {
            break;
        }

        // Keep top beam_width states by log probability
        next_beam_chirho.sort_by(|a, b| b.log_prob_chirho.partial_cmp(&a.log_prob_chirho).unwrap());
        next_beam_chirho.truncate(beam_width_chirho);
        beam_chirho = next_beam_chirho;
    }

    solutions_chirho
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_soft_domain_unify_chirho() {
        // x in {0, 1} with prob [0.7, 0.3]
        let x_chirho = SoftDomainChirho {
            probs_chirho: vec![0.7, 0.3],
        };
        // y in {0, 1} with prob [0.4, 0.6]
        let y_chirho = SoftDomainChirho {
            probs_chirho: vec![0.4, 0.6],
        };

        let unified_chirho = x_chirho.unify_chirho(&y_chirho);

        // Product: [0.28, 0.18], normalized: [0.609, 0.391]
        assert!((unified_chirho.probs_chirho[0] - 0.609).abs() < 0.01);
        assert!((unified_chirho.probs_chirho[1] - 0.391).abs() < 0.01);
    }

    #[test]
    fn test_soft_domain_entropy_chirho() {
        let uniform_chirho = SoftDomainChirho::uniform_chirho(4);
        let singleton_chirho = SoftDomainChirho::singleton_chirho(2, 4);

        // Uniform has higher entropy than singleton
        assert!(uniform_chirho.entropy_chirho() > singleton_chirho.entropy_chirho());
        // Singleton entropy should be ~0
        assert!(singleton_chirho.entropy_chirho().abs() < 0.01);
    }

    #[test]
    fn test_neural_state_unify_chirho() {
        let mut state_chirho = NeuralStateChirho::new_chirho(2, 4);

        // Make var 0 prefer value 1
        state_chirho.domains_chirho[0] = SoftDomainChirho {
            probs_chirho: vec![0.1, 0.7, 0.1, 0.1],
        };
        // Make var 1 prefer value 1 or 2
        state_chirho.domains_chirho[1] = SoftDomainChirho {
            probs_chirho: vec![0.1, 0.4, 0.4, 0.1],
        };

        state_chirho.unify_chirho(0, 1);

        // After unification, both should strongly prefer value 1
        assert!(state_chirho.domains_chirho[0].argmax_chirho() == 1);
        assert!(state_chirho.domains_chirho[1].argmax_chirho() == 1);
    }

    #[test]
    fn test_beam_search_chirho() {
        let initial_chirho = NeuralStateChirho::new_chirho(2, 4);
        let heuristic_chirho = NeuralHeuristicChirho::uniform_chirho(2, 4);

        let solutions_chirho = beam_search_chirho(initial_chirho, &heuristic_chirho, 4, 10);

        // Should find some solutions
        assert!(!solutions_chirho.is_empty());
    }
}
