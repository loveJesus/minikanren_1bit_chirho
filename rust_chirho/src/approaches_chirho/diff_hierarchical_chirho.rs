// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Differentiable Hierarchical Domains ☧
//!
//! Soft relaxation of Hierarchical4kChirho where each bit position
//! has a probability in [0, 1] instead of hard 0/1.
//!
//! Key insight: The 2-level hierarchy maps directly to differentiable ops:
//! - Root probability: P(any value in this leaf block)
//! - Leaf probabilities: P(specific value | block is active)
//!
//! Gradients flow through soft AND/OR, enabling learning over domains.

use crate::approaches_chirho::Hierarchical4kChirho;

/// Differentiable 4096-value hierarchical domain
///
/// Structure mirrors Hierarchical4kChirho but with soft probabilities:
/// - `root_probs_chirho[i]` = P(leaf block i has any active values)
/// - `leaf_probs_chirho[i][j]` = P(value i*64+j is in domain)
#[derive(Clone, Debug)]
pub struct DiffHierarchical4kChirho {
    /// Root probabilities: which leaf blocks are active
    pub root_probs_chirho: [f64; 64],
    /// Leaf probabilities: which values are in domain
    pub leaf_probs_chirho: Box<[[f64; 64]; 64]>,
    /// Accumulated gradients for root
    pub root_grads_chirho: [f64; 64],
    /// Accumulated gradients for leaves
    pub leaf_grads_chirho: Box<[[f64; 64]; 64]>,
}

impl Default for DiffHierarchical4kChirho {
    fn default() -> Self {
        Self::empty_chirho()
    }
}

impl DiffHierarchical4kChirho {
    /// Empty domain (all probabilities 0)
    pub fn empty_chirho() -> Self {
        Self {
            root_probs_chirho: [0.0; 64],
            leaf_probs_chirho: Box::new([[0.0; 64]; 64]),
            root_grads_chirho: [0.0; 64],
            leaf_grads_chirho: Box::new([[0.0; 64]; 64]),
        }
    }

    /// Full domain (all probabilities 1)
    pub fn full_chirho() -> Self {
        Self {
            root_probs_chirho: [1.0; 64],
            leaf_probs_chirho: Box::new([[1.0; 64]; 64]),
            root_grads_chirho: [0.0; 64],
            leaf_grads_chirho: Box::new([[0.0; 64]; 64]),
        }
    }

    /// Create from hard domain (convert 0/1 to 0.0/1.0)
    pub fn from_hard_chirho(hard_chirho: &Hierarchical4kChirho) -> Self {
        let mut result_chirho = Self::empty_chirho();

        for i_chirho in 0..64 {
            // Root bit
            let root_bit_chirho = (hard_chirho.root_chirho.0 >> i_chirho) & 1;
            result_chirho.root_probs_chirho[i_chirho] = root_bit_chirho as f64;

            // Leaf bits
            for j_chirho in 0..64 {
                let leaf_bit_chirho = (hard_chirho.leaves_chirho[i_chirho].0 >> j_chirho) & 1;
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] = leaf_bit_chirho as f64;
            }
        }

        result_chirho
    }

    /// Convert to hard domain via thresholding
    pub fn to_hard_chirho(&self, threshold_chirho: f64) -> Hierarchical4kChirho {
        let mut result_chirho = Hierarchical4kChirho::empty_chirho();

        for i_chirho in 0..64 {
            let mut any_leaf_set_chirho = false;

            for j_chirho in 0..64 {
                if self.leaf_probs_chirho[i_chirho][j_chirho] >= threshold_chirho {
                    result_chirho.leaves_chirho[i_chirho].0 |= 1u64 << j_chirho;
                    any_leaf_set_chirho = true;
                }
            }

            // Set root bit if any leaf value is set
            if any_leaf_set_chirho {
                result_chirho.root_chirho.0 |= 1u64 << i_chirho;
            }
        }

        result_chirho
    }

    /// Soft probability that a value is in the domain
    pub fn prob_contains_chirho(&self, value_chirho: u32) -> f64 {
        if value_chirho >= 4096 {
            return 0.0;
        }

        let leaf_idx_chirho = (value_chirho / 64) as usize;
        let bit_idx_chirho = (value_chirho % 64) as usize;

        // P(value in domain) = P(leaf active) * P(value | leaf active)
        // For proper probability, we use the leaf prob directly
        // (root is just for sparse acceleration in hard case)
        self.leaf_probs_chirho[leaf_idx_chirho][bit_idx_chirho]
    }

    /// Soft intersection: P(x in A ∩ B) = P(x in A) * P(x in B)
    pub fn soft_intersect_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self::empty_chirho();

        for i_chirho in 0..64 {
            // Root: P(leaf has values in both) ≈ product of root probs
            // (This is an approximation; exact would need joint distribution)
            result_chirho.root_probs_chirho[i_chirho] =
                self.root_probs_chirho[i_chirho] * other_chirho.root_probs_chirho[i_chirho];

            for j_chirho in 0..64 {
                // Leaf: exact soft AND
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    self.leaf_probs_chirho[i_chirho][j_chirho]
                        * other_chirho.leaf_probs_chirho[i_chirho][j_chirho];
            }
        }

        result_chirho
    }

    /// Soft intersection with gradient computation
    ///
    /// Returns (result, grad_self, grad_other) where grads are accumulated
    pub fn soft_intersect_with_grad_chirho(
        &self,
        other_chirho: &Self,
        grad_out_chirho: &Self,
    ) -> (Self, Self, Self) {
        let mut result_chirho = Self::empty_chirho();
        let mut grad_self_chirho = Self::empty_chirho();
        let mut grad_other_chirho = Self::empty_chirho();

        for i_chirho in 0..64 {
            // Root gradients
            let out_grad_chirho = grad_out_chirho.root_probs_chirho[i_chirho];
            grad_self_chirho.root_probs_chirho[i_chirho] =
                out_grad_chirho * other_chirho.root_probs_chirho[i_chirho];
            grad_other_chirho.root_probs_chirho[i_chirho] =
                out_grad_chirho * self.root_probs_chirho[i_chirho];
            result_chirho.root_probs_chirho[i_chirho] =
                self.root_probs_chirho[i_chirho] * other_chirho.root_probs_chirho[i_chirho];

            for j_chirho in 0..64 {
                // Leaf gradients: d(a*b)/da = b, d(a*b)/db = a
                let leaf_grad_out_chirho = grad_out_chirho.leaf_probs_chirho[i_chirho][j_chirho];
                grad_self_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    leaf_grad_out_chirho * other_chirho.leaf_probs_chirho[i_chirho][j_chirho];
                grad_other_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    leaf_grad_out_chirho * self.leaf_probs_chirho[i_chirho][j_chirho];
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    self.leaf_probs_chirho[i_chirho][j_chirho]
                        * other_chirho.leaf_probs_chirho[i_chirho][j_chirho];
            }
        }

        (result_chirho, grad_self_chirho, grad_other_chirho)
    }

    /// Soft union: P(x in A ∪ B) = P(x in A) + P(x in B) - P(x in A)*P(x in B)
    pub fn soft_union_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self::empty_chirho();

        for i_chirho in 0..64 {
            let a_chirho = self.root_probs_chirho[i_chirho];
            let b_chirho = other_chirho.root_probs_chirho[i_chirho];
            result_chirho.root_probs_chirho[i_chirho] = a_chirho + b_chirho - a_chirho * b_chirho;

            for j_chirho in 0..64 {
                let a_leaf_chirho = self.leaf_probs_chirho[i_chirho][j_chirho];
                let b_leaf_chirho = other_chirho.leaf_probs_chirho[i_chirho][j_chirho];
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    a_leaf_chirho + b_leaf_chirho - a_leaf_chirho * b_leaf_chirho;
            }
        }

        result_chirho
    }

    /// Soft union with gradient computation
    pub fn soft_union_with_grad_chirho(
        &self,
        other_chirho: &Self,
        grad_out_chirho: &Self,
    ) -> (Self, Self, Self) {
        let mut result_chirho = Self::empty_chirho();
        let mut grad_self_chirho = Self::empty_chirho();
        let mut grad_other_chirho = Self::empty_chirho();

        for i_chirho in 0..64 {
            let a_chirho = self.root_probs_chirho[i_chirho];
            let b_chirho = other_chirho.root_probs_chirho[i_chirho];
            let out_grad_chirho = grad_out_chirho.root_probs_chirho[i_chirho];

            // d(a + b - a*b)/da = 1 - b
            // d(a + b - a*b)/db = 1 - a
            grad_self_chirho.root_probs_chirho[i_chirho] = out_grad_chirho * (1.0 - b_chirho);
            grad_other_chirho.root_probs_chirho[i_chirho] = out_grad_chirho * (1.0 - a_chirho);
            result_chirho.root_probs_chirho[i_chirho] = a_chirho + b_chirho - a_chirho * b_chirho;

            for j_chirho in 0..64 {
                let a_leaf_chirho = self.leaf_probs_chirho[i_chirho][j_chirho];
                let b_leaf_chirho = other_chirho.leaf_probs_chirho[i_chirho][j_chirho];
                let leaf_grad_out_chirho = grad_out_chirho.leaf_probs_chirho[i_chirho][j_chirho];

                grad_self_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    leaf_grad_out_chirho * (1.0 - b_leaf_chirho);
                grad_other_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    leaf_grad_out_chirho * (1.0 - a_leaf_chirho);
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    a_leaf_chirho + b_leaf_chirho - a_leaf_chirho * b_leaf_chirho;
            }
        }

        (result_chirho, grad_self_chirho, grad_other_chirho)
    }

    /// Expected count of values in domain (sum of all probabilities)
    pub fn expected_count_chirho(&self) -> f64 {
        let mut count_chirho = 0.0;
        for i_chirho in 0..64 {
            for j_chirho in 0..64 {
                count_chirho += self.leaf_probs_chirho[i_chirho][j_chirho];
            }
        }
        count_chirho
    }

    /// Entropy of the domain distribution
    pub fn entropy_chirho(&self) -> f64 {
        let mut entropy_chirho = 0.0;
        for i_chirho in 0..64 {
            for j_chirho in 0..64 {
                let p_chirho = self.leaf_probs_chirho[i_chirho][j_chirho];
                if p_chirho > 1e-10 && p_chirho < 1.0 - 1e-10 {
                    entropy_chirho -= p_chirho * p_chirho.ln()
                        + (1.0 - p_chirho) * (1.0 - p_chirho).ln();
                }
            }
        }
        entropy_chirho
    }

    /// Apply temperature to sharpen/soften probabilities
    ///
    /// temp < 1: sharpen (more extreme)
    /// temp > 1: soften (more uniform)
    pub fn with_temperature_chirho(&self, temp_chirho: f64) -> Self {
        let mut result_chirho = Self::empty_chirho();
        let inv_temp_chirho = 1.0 / temp_chirho.max(1e-10);

        for i_chirho in 0..64 {
            // Apply sigmoid with temperature: σ(logit/temp)
            let root_logit_chirho = logit_chirho(self.root_probs_chirho[i_chirho]);
            result_chirho.root_probs_chirho[i_chirho] =
                sigmoid_chirho(root_logit_chirho * inv_temp_chirho);

            for j_chirho in 0..64 {
                let leaf_logit_chirho = logit_chirho(self.leaf_probs_chirho[i_chirho][j_chirho]);
                result_chirho.leaf_probs_chirho[i_chirho][j_chirho] =
                    sigmoid_chirho(leaf_logit_chirho * inv_temp_chirho);
            }
        }

        result_chirho
    }

    /// Zero all gradients
    pub fn zero_grad_chirho(&mut self) {
        self.root_grads_chirho = [0.0; 64];
        for i_chirho in 0..64 {
            self.leaf_grads_chirho[i_chirho] = [0.0; 64];
        }
    }

    /// Accumulate gradients from another domain
    pub fn accumulate_grad_chirho(&mut self, grad_chirho: &Self) {
        for i_chirho in 0..64 {
            self.root_grads_chirho[i_chirho] += grad_chirho.root_probs_chirho[i_chirho];
            for j_chirho in 0..64 {
                self.leaf_grads_chirho[i_chirho][j_chirho] +=
                    grad_chirho.leaf_probs_chirho[i_chirho][j_chirho];
            }
        }
    }

    /// Apply gradient descent step
    pub fn apply_gradients_chirho(&mut self, lr_chirho: f64) {
        for i_chirho in 0..64 {
            self.root_probs_chirho[i_chirho] =
                (self.root_probs_chirho[i_chirho] - lr_chirho * self.root_grads_chirho[i_chirho])
                    .clamp(0.0, 1.0);

            for j_chirho in 0..64 {
                self.leaf_probs_chirho[i_chirho][j_chirho] = (self.leaf_probs_chirho[i_chirho]
                    [j_chirho]
                    - lr_chirho * self.leaf_grads_chirho[i_chirho][j_chirho])
                    .clamp(0.0, 1.0);
            }
        }
        self.zero_grad_chirho();
    }

    /// Sample a hard domain from the soft probabilities
    pub fn sample_hard_chirho(&self, seed_chirho: u64) -> Hierarchical4kChirho {
        let mut result_chirho = Hierarchical4kChirho::empty_chirho();
        let mut rng_chirho = seed_chirho;

        for i_chirho in 0..64 {
            for j_chirho in 0..64 {
                // LCG random
                rng_chirho = rng_chirho.wrapping_mul(6364136223846793005).wrapping_add(1);
                let u_chirho = (rng_chirho as f64) / (u64::MAX as f64);

                if u_chirho < self.leaf_probs_chirho[i_chirho][j_chirho] {
                    result_chirho.leaves_chirho[i_chirho].0 |= 1u64 << j_chirho;
                    result_chirho.root_chirho.0 |= 1u64 << i_chirho;
                }
            }
        }

        result_chirho
    }
}

/// Sigmoid function
fn sigmoid_chirho(x_chirho: f64) -> f64 {
    1.0 / (1.0 + (-x_chirho).exp())
}

/// Logit function (inverse sigmoid)
fn logit_chirho(p_chirho: f64) -> f64 {
    let p_clamped_chirho = p_chirho.clamp(1e-10, 1.0 - 1e-10);
    (p_clamped_chirho / (1.0 - p_clamped_chirho)).ln()
}

/// Soft unification state with differentiable domains
#[derive(Clone, Debug)]
pub struct DiffUnifyStateChirho {
    /// Variable domains (soft)
    pub domains_chirho: Vec<DiffHierarchical4kChirho>,
    /// Temperature for annealing
    pub temp_chirho: f64,
}

impl DiffUnifyStateChirho {
    /// Create new state with n variables, all with full domains
    pub fn new_chirho(n_vars_chirho: usize, temp_chirho: f64) -> Self {
        Self {
            domains_chirho: vec![DiffHierarchical4kChirho::full_chirho(); n_vars_chirho],
            temp_chirho,
        }
    }

    /// Soft unification: constrain var to equal a value
    ///
    /// Returns probability of success (non-empty intersection)
    pub fn soft_unify_value_chirho(&mut self, var_chirho: usize, value_chirho: u32) -> f64 {
        if var_chirho >= self.domains_chirho.len() || value_chirho >= 4096 {
            return 0.0;
        }

        // Current probability that value is possible
        let prob_chirho = self.domains_chirho[var_chirho].prob_contains_chirho(value_chirho);

        // Update domain to singleton (or soft singleton based on temp)
        let leaf_idx_chirho = (value_chirho / 64) as usize;
        let bit_idx_chirho = (value_chirho % 64) as usize;

        // At low temp, this becomes hard singleton
        // At high temp, other values retain some probability
        let sharpness_chirho = 1.0 / self.temp_chirho.max(1e-10);

        for i_chirho in 0..64 {
            for j_chirho in 0..64 {
                if i_chirho == leaf_idx_chirho && j_chirho == bit_idx_chirho {
                    // Target value: boost probability
                    self.domains_chirho[var_chirho].leaf_probs_chirho[i_chirho][j_chirho] =
                        sigmoid_chirho(sharpness_chirho);
                } else {
                    // Other values: reduce probability
                    let current_chirho =
                        self.domains_chirho[var_chirho].leaf_probs_chirho[i_chirho][j_chirho];
                    self.domains_chirho[var_chirho].leaf_probs_chirho[i_chirho][j_chirho] =
                        current_chirho * sigmoid_chirho(-sharpness_chirho);
                }
            }
        }

        prob_chirho
    }

    /// Soft unification: constrain two variables to be equal
    ///
    /// Returns probability of success
    pub fn soft_unify_vars_chirho(&mut self, var1_chirho: usize, var2_chirho: usize) -> f64 {
        if var1_chirho >= self.domains_chirho.len() || var2_chirho >= self.domains_chirho.len() {
            return 0.0;
        }

        // Intersection of domains
        let intersection_chirho = self.domains_chirho[var1_chirho]
            .soft_intersect_chirho(&self.domains_chirho[var2_chirho]);

        // Probability of success = expected values in intersection
        let success_prob_chirho =
            (intersection_chirho.expected_count_chirho() / 4096.0).min(1.0);

        // Update both domains to intersection
        self.domains_chirho[var1_chirho] = intersection_chirho.clone();
        self.domains_chirho[var2_chirho] = intersection_chirho;

        success_prob_chirho
    }

    /// Anneal temperature
    pub fn anneal_chirho(&mut self, new_temp_chirho: f64) {
        self.temp_chirho = new_temp_chirho;

        // Apply temperature to all domains
        for domain_chirho in &mut self.domains_chirho {
            *domain_chirho = domain_chirho.with_temperature_chirho(new_temp_chirho);
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_from_hard_chirho() {
        let hard_chirho = Hierarchical4kChirho::range_chirho(100);
        let soft_chirho = DiffHierarchical4kChirho::from_hard_chirho(&hard_chirho);

        // First 100 values should have prob 1.0
        assert!((soft_chirho.prob_contains_chirho(0) - 1.0).abs() < 1e-10);
        assert!((soft_chirho.prob_contains_chirho(99) - 1.0).abs() < 1e-10);

        // Value 100 should have prob 0.0
        assert!((soft_chirho.prob_contains_chirho(100) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_to_hard_chirho() {
        let mut soft_chirho = DiffHierarchical4kChirho::empty_chirho();
        soft_chirho.leaf_probs_chirho[0][0] = 0.9;
        soft_chirho.leaf_probs_chirho[0][1] = 0.3;

        let hard_chirho = soft_chirho.to_hard_chirho(0.5);
        assert!(hard_chirho.contains_chirho(0)); // 0.9 >= 0.5
        assert!(!hard_chirho.contains_chirho(1)); // 0.3 < 0.5
    }

    #[test]
    fn test_soft_intersect_chirho() {
        let mut a_chirho = DiffHierarchical4kChirho::empty_chirho();
        let mut b_chirho = DiffHierarchical4kChirho::empty_chirho();

        a_chirho.leaf_probs_chirho[0][0] = 0.8;
        b_chirho.leaf_probs_chirho[0][0] = 0.6;

        let result_chirho = a_chirho.soft_intersect_chirho(&b_chirho);

        // P(in both) = 0.8 * 0.6 = 0.48
        assert!((result_chirho.leaf_probs_chirho[0][0] - 0.48).abs() < 1e-10);
    }

    #[test]
    fn test_soft_union_chirho() {
        let mut a_chirho = DiffHierarchical4kChirho::empty_chirho();
        let mut b_chirho = DiffHierarchical4kChirho::empty_chirho();

        a_chirho.leaf_probs_chirho[0][0] = 0.3;
        b_chirho.leaf_probs_chirho[0][0] = 0.4;

        let result_chirho = a_chirho.soft_union_chirho(&b_chirho);

        // P(in either) = 0.3 + 0.4 - 0.3*0.4 = 0.58
        assert!((result_chirho.leaf_probs_chirho[0][0] - 0.58).abs() < 1e-10);
    }

    #[test]
    fn test_temperature_chirho() {
        let mut soft_chirho = DiffHierarchical4kChirho::empty_chirho();
        soft_chirho.leaf_probs_chirho[0][0] = 0.6;

        // Low temp should sharpen
        let sharp_chirho = soft_chirho.with_temperature_chirho(0.1);
        assert!(sharp_chirho.leaf_probs_chirho[0][0] > 0.6);

        // High temp should soften toward 0.5
        let soft2_chirho = soft_chirho.with_temperature_chirho(10.0);
        assert!((soft2_chirho.leaf_probs_chirho[0][0] - 0.5).abs() < (0.6_f64 - 0.5).abs());
    }

    #[test]
    fn test_gradient_flow_chirho() {
        let mut a_chirho = DiffHierarchical4kChirho::empty_chirho();
        let mut b_chirho = DiffHierarchical4kChirho::empty_chirho();

        a_chirho.leaf_probs_chirho[0][0] = 0.7;
        b_chirho.leaf_probs_chirho[0][0] = 0.5;

        // Output gradient = 1.0 for position [0][0]
        let mut grad_out_chirho = DiffHierarchical4kChirho::empty_chirho();
        grad_out_chirho.leaf_probs_chirho[0][0] = 1.0;

        let (result_chirho, grad_a_chirho, grad_b_chirho) =
            a_chirho.soft_intersect_with_grad_chirho(&b_chirho, &grad_out_chirho);

        // Result = 0.7 * 0.5 = 0.35
        assert!((result_chirho.leaf_probs_chirho[0][0] - 0.35).abs() < 1e-10);

        // Gradient of a = output_grad * b = 1.0 * 0.5 = 0.5
        assert!((grad_a_chirho.leaf_probs_chirho[0][0] - 0.5).abs() < 1e-10);

        // Gradient of b = output_grad * a = 1.0 * 0.7 = 0.7
        assert!((grad_b_chirho.leaf_probs_chirho[0][0] - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_soft_unify_state_chirho() {
        let mut state_chirho = DiffUnifyStateChirho::new_chirho(2, 1.0);

        // Unify var 0 with value 42
        let prob_chirho = state_chirho.soft_unify_value_chirho(0, 42);
        assert!(prob_chirho > 0.0);

        // Value 42 should have high probability now
        assert!(state_chirho.domains_chirho[0].prob_contains_chirho(42) > 0.5);
    }
}
