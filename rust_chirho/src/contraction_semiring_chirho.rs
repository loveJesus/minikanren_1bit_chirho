//! Semiring-Generic Tensor Contraction ☧
//!
//! Thread SemiringChirho through contraction so one codebase handles:
//! - Boolean (logic): OR for +, AND for *
//! - Tropical (shortest path): min for +, plus for *
//! - Probabilistic (Scallop-style): + for +, * for *
//! - Counting: how many derivations

use crate::semiring_chirho::SemiringChirho;
use std::collections::{HashMap, HashSet};

/// A tensor with semiring-valued entries
#[derive(Debug, Clone)]
pub struct SemiringTensorChirho<S: SemiringChirho> {
    /// Indices this tensor ranges over
    pub indices_chirho: Vec<String>,
    /// Entries: (index_values -> semiring_value)
    /// Key is tuple of index values
    entries_chirho: HashMap<Vec<u32>, S>,
    /// Size of each index
    index_sizes_chirho: HashMap<String, u32>,
}

impl<S: SemiringChirho> SemiringTensorChirho<S> {
    pub fn new_chirho(indices_chirho: Vec<String>) -> Self {
        Self {
            indices_chirho,
            entries_chirho: HashMap::new(),
            index_sizes_chirho: HashMap::new(),
        }
    }

    pub fn set_chirho(&mut self, index_vals_chirho: Vec<u32>, val_chirho: S) {
        if !val_chirho.is_zero_chirho() {
            // Update index sizes
            for (i_chirho, &v_chirho) in index_vals_chirho.iter().enumerate() {
                let idx_name_chirho = &self.indices_chirho[i_chirho];
                let size_chirho = self.index_sizes_chirho.entry(idx_name_chirho.clone()).or_insert(0);
                *size_chirho = (*size_chirho).max(v_chirho + 1);
            }
            self.entries_chirho.insert(index_vals_chirho, val_chirho);
        }
    }

    pub fn get_chirho(&self, index_vals_chirho: &[u32]) -> S {
        self.entries_chirho
            .get(index_vals_chirho)
            .cloned()
            .unwrap_or_else(S::zero_chirho)
    }

    /// Get number of non-zero entries
    pub fn nnz_chirho(&self) -> usize {
        self.entries_chirho.len()
    }

    /// Contract two tensors along shared indices
    /// Result has indices from both tensors minus the shared ones
    pub fn contract_chirho(&self, other_chirho: &Self) -> Self {
        // Find shared indices
        let self_idx_set_chirho: HashSet<_> = self.indices_chirho.iter().cloned().collect();
        let other_idx_set_chirho: HashSet<_> = other_chirho.indices_chirho.iter().cloned().collect();
        let shared_chirho: HashSet<_> = self_idx_set_chirho.intersection(&other_idx_set_chirho).cloned().collect();

        // Result indices = union - shared
        let mut result_indices_chirho: Vec<String> = Vec::new();
        for idx_chirho in &self.indices_chirho {
            if !shared_chirho.contains(idx_chirho) {
                result_indices_chirho.push(idx_chirho.clone());
            }
        }
        for idx_chirho in &other_chirho.indices_chirho {
            if !shared_chirho.contains(idx_chirho) && !result_indices_chirho.contains(idx_chirho) {
                result_indices_chirho.push(idx_chirho.clone());
            }
        }

        let mut result_chirho = SemiringTensorChirho::new_chirho(result_indices_chirho.clone());

        // Build index position maps
        let self_idx_pos_chirho: HashMap<_, _> = self.indices_chirho.iter().enumerate()
            .map(|(i_chirho, s_chirho)| (s_chirho.clone(), i_chirho))
            .collect();
        let other_idx_pos_chirho: HashMap<_, _> = other_chirho.indices_chirho.iter().enumerate()
            .map(|(i_chirho, s_chirho)| (s_chirho.clone(), i_chirho))
            .collect();

        // For each pair of entries
        for (self_vals_chirho, self_weight_chirho) in &self.entries_chirho {
            for (other_vals_chirho, other_weight_chirho) in &other_chirho.entries_chirho {
                // Check if shared indices match
                let mut matches_chirho = true;
                for shared_idx_chirho in &shared_chirho {
                    let self_pos_chirho = self_idx_pos_chirho[shared_idx_chirho];
                    let other_pos_chirho = other_idx_pos_chirho[shared_idx_chirho];
                    if self_vals_chirho[self_pos_chirho] != other_vals_chirho[other_pos_chirho] {
                        matches_chirho = false;
                        break;
                    }
                }

                if matches_chirho {
                    // Build result key
                    let mut result_key_chirho = Vec::new();
                    for idx_chirho in &result_indices_chirho {
                        if let Some(&pos_chirho) = self_idx_pos_chirho.get(idx_chirho) {
                            result_key_chirho.push(self_vals_chirho[pos_chirho]);
                        } else if let Some(&pos_chirho) = other_idx_pos_chirho.get(idx_chirho) {
                            result_key_chirho.push(other_vals_chirho[pos_chirho]);
                        }
                    }

                    // Multiply weights and add to result
                    let product_chirho = self_weight_chirho.clone() * other_weight_chirho.clone();
                    let existing_chirho = result_chirho.get_chirho(&result_key_chirho);
                    result_chirho.set_chirho(result_key_chirho, existing_chirho + product_chirho);
                }
            }
        }

        result_chirho
    }

    /// Contract all tensors in a network
    pub fn contract_all_chirho(tensors_chirho: &[Self]) -> Option<Self> {
        if tensors_chirho.is_empty() {
            return None;
        }

        let mut result_chirho = tensors_chirho[0].clone();
        for tensor_chirho in &tensors_chirho[1..] {
            result_chirho = result_chirho.contract_chirho(tensor_chirho);
        }
        Some(result_chirho)
    }
}

/// Tensor network with semiring weights
#[derive(Debug, Clone)]
pub struct SemiringNetworkChirho<S: SemiringChirho> {
    pub tensors_chirho: Vec<SemiringTensorChirho<S>>,
    pub output_indices_chirho: HashSet<String>,
}

impl<S: SemiringChirho> SemiringNetworkChirho<S> {
    pub fn new_chirho() -> Self {
        Self {
            tensors_chirho: Vec::new(),
            output_indices_chirho: HashSet::new(),
        }
    }

    pub fn add_tensor_chirho(&mut self, tensor_chirho: SemiringTensorChirho<S>) {
        self.tensors_chirho.push(tensor_chirho);
    }

    pub fn set_output_chirho(&mut self, indices_chirho: &[&str]) {
        self.output_indices_chirho = indices_chirho.iter().map(|s_chirho| s_chirho.to_string()).collect();
    }

    /// Contract entire network to output tensor
    pub fn contract_chirho(&self) -> Option<SemiringTensorChirho<S>> {
        SemiringTensorChirho::contract_all_chirho(&self.tensors_chirho)
    }

    /// Greedy contraction order
    pub fn contract_greedy_chirho(&self) -> Option<SemiringTensorChirho<S>> {
        if self.tensors_chirho.is_empty() {
            return None;
        }
        if self.tensors_chirho.len() == 1 {
            return Some(self.tensors_chirho[0].clone());
        }

        let mut remaining_chirho = self.tensors_chirho.clone();

        while remaining_chirho.len() > 1 {
            // Find pair with most shared indices (greedy for locality)
            let mut best_pair_chirho: Option<(usize, usize)> = None;
            let mut best_shared_chirho = 0;

            for i_chirho in 0..remaining_chirho.len() {
                for j_chirho in (i_chirho + 1)..remaining_chirho.len() {
                    let t1_chirho = &remaining_chirho[i_chirho];
                    let t2_chirho = &remaining_chirho[j_chirho];

                    let s1_chirho: HashSet<_> = t1_chirho.indices_chirho.iter().collect();
                    let s2_chirho: HashSet<_> = t2_chirho.indices_chirho.iter().collect();
                    let shared_count_chirho = s1_chirho.intersection(&s2_chirho).count();

                    if shared_count_chirho > best_shared_chirho {
                        best_shared_chirho = shared_count_chirho;
                        best_pair_chirho = Some((i_chirho, j_chirho));
                    }
                }
            }

            let (i_chirho, j_chirho) = best_pair_chirho.unwrap_or((0, 1));
            let t2_chirho = remaining_chirho.remove(j_chirho);
            let t1_chirho = remaining_chirho.remove(i_chirho);
            let contracted_chirho = t1_chirho.contract_chirho(&t2_chirho);
            remaining_chirho.push(contracted_chirho);
        }

        remaining_chirho.pop()
    }
}

// === Convenience type aliases ===

use crate::semiring_chirho::{BoolSemiringChirho, ProbSemiringChirho, TropicalSemiringChirho, CountSemiringChirho};

/// Boolean tensor (standard logic programming)
pub type BoolTensorChirho = SemiringTensorChirho<BoolSemiringChirho>;

/// Probabilistic tensor (Scallop-style)
pub type ProbTensorChirho = SemiringTensorChirho<ProbSemiringChirho>;

/// Tropical tensor (shortest path / Viterbi)
pub type TropicalTensorChirho = SemiringTensorChirho<TropicalSemiringChirho>;

/// Counting tensor (number of derivations)
pub type CountTensorChirho = SemiringTensorChirho<CountSemiringChirho>;

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bool_tensor_contract_chirho() {
        // T1[X, Y] and T2[Y, Z] -> R[X, Z]
        let mut t1_chirho = BoolTensorChirho::new_chirho(vec!["X".into(), "Y".into()]);
        let mut t2_chirho = BoolTensorChirho::new_chirho(vec!["Y".into(), "Z".into()]);

        t1_chirho.set_chirho(vec![0, 0], BoolSemiringChirho(true));
        t1_chirho.set_chirho(vec![0, 1], BoolSemiringChirho(true));

        t2_chirho.set_chirho(vec![0, 0], BoolSemiringChirho(true));
        t2_chirho.set_chirho(vec![1, 1], BoolSemiringChirho(true));

        let result_chirho = t1_chirho.contract_chirho(&t2_chirho);

        // R[0,0] = T1[0,0] AND T2[0,0] OR T1[0,1] AND T2[1,0]
        //        = true AND true OR true AND false = true
        assert_eq!(result_chirho.get_chirho(&[0, 0]).0, true);

        // R[0,1] = T1[0,0] AND T2[0,1] OR T1[0,1] AND T2[1,1]
        //        = true AND false OR true AND true = true
        assert_eq!(result_chirho.get_chirho(&[0, 1]).0, true);
    }

    #[test]
    fn test_count_tensor_contract_chirho() {
        // Count number of paths
        let mut t1_chirho = CountTensorChirho::new_chirho(vec!["X".into(), "Y".into()]);
        let mut t2_chirho = CountTensorChirho::new_chirho(vec!["Y".into(), "Z".into()]);

        // Two paths from X=0: via Y=0 and Y=1
        t1_chirho.set_chirho(vec![0, 0], CountSemiringChirho(1));
        t1_chirho.set_chirho(vec![0, 1], CountSemiringChirho(1));

        // Both Y values lead to Z=0
        t2_chirho.set_chirho(vec![0, 0], CountSemiringChirho(1));
        t2_chirho.set_chirho(vec![1, 0], CountSemiringChirho(1));

        let result_chirho = t1_chirho.contract_chirho(&t2_chirho);

        // R[0,0] = 1*1 + 1*1 = 2 paths
        assert_eq!(result_chirho.get_chirho(&[0, 0]).0, 2);
    }

    #[test]
    fn test_tropical_tensor_shortest_path_chirho() {
        // Shortest path: A -> B -> C
        let mut ab_chirho = TropicalTensorChirho::new_chirho(vec!["A".into(), "B".into()]);
        let mut bc_chirho = TropicalTensorChirho::new_chirho(vec!["B".into(), "C".into()]);

        // Costs from A=0 to various B
        ab_chirho.set_chirho(vec![0, 0], TropicalSemiringChirho(3.0)); // A0->B0: 3
        ab_chirho.set_chirho(vec![0, 1], TropicalSemiringChirho(1.0)); // A0->B1: 1

        // Costs from B to C=0
        bc_chirho.set_chirho(vec![0, 0], TropicalSemiringChirho(2.0)); // B0->C0: 2
        bc_chirho.set_chirho(vec![1, 0], TropicalSemiringChirho(4.0)); // B1->C0: 4

        let result_chirho = ab_chirho.contract_chirho(&bc_chirho);

        // AC[0,0] = min(3+2, 1+4) = min(5, 5) = 5
        assert_eq!(result_chirho.get_chirho(&[0, 0]).0, 5.0);
    }

    #[test]
    fn test_prob_tensor_marginal_chirho() {
        // Joint P(X,Y) -> marginal P(X)
        let mut joint_chirho = ProbTensorChirho::new_chirho(vec!["X".into(), "Y".into()]);

        joint_chirho.set_chirho(vec![0, 0], ProbSemiringChirho(0.3));
        joint_chirho.set_chirho(vec![0, 1], ProbSemiringChirho(0.2));
        joint_chirho.set_chirho(vec![1, 0], ProbSemiringChirho(0.1));
        joint_chirho.set_chirho(vec![1, 1], ProbSemiringChirho(0.4));

        // Create "summing" tensor over Y
        let mut sum_y_chirho = ProbTensorChirho::new_chirho(vec!["Y".into()]);
        sum_y_chirho.set_chirho(vec![0], ProbSemiringChirho(1.0));
        sum_y_chirho.set_chirho(vec![1], ProbSemiringChirho(1.0));

        let marginal_chirho = joint_chirho.contract_chirho(&sum_y_chirho);

        // P(X=0) = 0.3 + 0.2 = 0.5
        assert!((marginal_chirho.get_chirho(&[0]).0 - 0.5).abs() < 1e-10);
        // P(X=1) = 0.1 + 0.4 = 0.5
        assert!((marginal_chirho.get_chirho(&[1]).0 - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_network_greedy_chirho() {
        let mut net_chirho = SemiringNetworkChirho::<BoolSemiringChirho>::new_chirho();

        let mut t1_chirho = BoolTensorChirho::new_chirho(vec!["A".into(), "X".into()]);
        let mut t2_chirho = BoolTensorChirho::new_chirho(vec!["X".into(), "Y".into()]);
        let mut t3_chirho = BoolTensorChirho::new_chirho(vec!["Y".into(), "B".into()]);

        t1_chirho.set_chirho(vec![0, 0], BoolSemiringChirho(true));
        t2_chirho.set_chirho(vec![0, 0], BoolSemiringChirho(true));
        t3_chirho.set_chirho(vec![0, 0], BoolSemiringChirho(true));

        net_chirho.add_tensor_chirho(t1_chirho);
        net_chirho.add_tensor_chirho(t2_chirho);
        net_chirho.add_tensor_chirho(t3_chirho);

        let result_chirho = net_chirho.contract_greedy_chirho().unwrap();
        assert_eq!(result_chirho.get_chirho(&[0, 0]).0, true);
    }
}
