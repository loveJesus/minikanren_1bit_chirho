//! Parallel Search State Processing ☧
//!
//! Uses Rayon for multi-core parallel domain operations.
//! Addresses Gemini critique P2-8: Prove data-level parallelism at scale.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use minikanren_1bit_chirho::parallel_chirho::ParallelStateChirho;
//!
//! let mut state = ParallelStateChirho::new_chirho(8);  // 8 variables
//! state.par_intersect_all_chirho(&constraint);  // Parallel intersection
//! ```
//!
//! ## Feature Gate
//!
//! Requires `parallel_chirho` feature:
//! ```toml
//! [dependencies]
//! minikanren_1bit_chirho = { version = "0.2", features = ["parallel_chirho"] }
//! ```

#[cfg(feature = "parallel_chirho")]
use rayon::prelude::*;

use crate::DomainHwChirho;

/// Parallel search state with multi-core domain operations
#[derive(Debug, Clone)]
pub struct ParallelStateChirho {
    /// Domains for each variable
    pub domains_chirho: Vec<DomainHwChirho>,
}

impl ParallelStateChirho {
    /// Create state with all full domains
    pub fn new_chirho(num_vars_chirho: usize) -> Self {
        Self {
            domains_chirho: vec![DomainHwChirho::full_chirho(); num_vars_chirho],
        }
    }

    /// Create state with specific domains
    pub fn with_domains_chirho(domains_chirho: Vec<DomainHwChirho>) -> Self {
        Self { domains_chirho }
    }

    /// Number of variables
    pub fn num_vars_chirho(&self) -> usize {
        self.domains_chirho.len()
    }

    /// Check if state is failed (any empty domain)
    pub fn is_failed_chirho(&self) -> bool {
        self.domains_chirho.iter().any(|d| d.is_empty_chirho())
    }

    /// Sequential intersection (baseline)
    pub fn seq_intersect_all_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        for domain_chirho in &mut self.domains_chirho {
            *domain_chirho = domain_chirho.intersect_chirho(constraint_chirho);
        }
    }

    /// Parallel intersection using Rayon
    #[cfg(feature = "parallel_chirho")]
    pub fn par_intersect_all_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        self.domains_chirho
            .par_iter_mut()
            .for_each(|domain_chirho| {
                *domain_chirho = domain_chirho.intersect_chirho(constraint_chirho);
            });
    }

    /// Fallback for when rayon is not enabled
    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_intersect_all_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        self.seq_intersect_all_chirho(constraint_chirho);
    }

    /// Sequential pairwise intersection
    pub fn seq_pairwise_intersect_chirho(&mut self, other_chirho: &Self) {
        for (d1_chirho, d2_chirho) in self.domains_chirho.iter_mut().zip(&other_chirho.domains_chirho) {
            *d1_chirho = d1_chirho.intersect_chirho(d2_chirho);
        }
    }

    /// Parallel pairwise intersection
    #[cfg(feature = "parallel_chirho")]
    pub fn par_pairwise_intersect_chirho(&mut self, other_chirho: &Self) {
        self.domains_chirho
            .par_iter_mut()
            .zip(&other_chirho.domains_chirho)
            .for_each(|(d1_chirho, d2_chirho)| {
                *d1_chirho = d1_chirho.intersect_chirho(d2_chirho);
            });
    }

    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_pairwise_intersect_chirho(&mut self, other_chirho: &Self) {
        self.seq_pairwise_intersect_chirho(other_chirho);
    }

    /// Count total possible values (sequential)
    pub fn seq_count_possible_chirho(&self) -> u64 {
        self.domains_chirho
            .iter()
            .map(|d| d.bits_chirho.popcount_chirho() as u64)
            .sum()
    }

    /// Count total possible values (parallel)
    #[cfg(feature = "parallel_chirho")]
    pub fn par_count_possible_chirho(&self) -> u64 {
        self.domains_chirho
            .par_iter()
            .map(|d| d.bits_chirho.popcount_chirho() as u64)
            .sum()
    }

    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_count_possible_chirho(&self) -> u64 {
        self.seq_count_possible_chirho()
    }
}

/// Batch of search states for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelBatchChirho {
    pub states_chirho: Vec<ParallelStateChirho>,
}

impl ParallelBatchChirho {
    /// Create batch from states
    pub fn new_chirho(states_chirho: Vec<ParallelStateChirho>) -> Self {
        Self { states_chirho }
    }

    /// Number of states in batch
    pub fn len_chirho(&self) -> usize {
        self.states_chirho.len()
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        self.states_chirho.is_empty()
    }

    /// Apply constraint to all states sequentially
    pub fn seq_apply_constraint_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        for state_chirho in &mut self.states_chirho {
            state_chirho.seq_intersect_all_chirho(constraint_chirho);
        }
    }

    /// Apply constraint to all states in parallel
    #[cfg(feature = "parallel_chirho")]
    pub fn par_apply_constraint_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        self.states_chirho
            .par_iter_mut()
            .for_each(|state_chirho| {
                state_chirho.par_intersect_all_chirho(constraint_chirho);
            });
    }

    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_apply_constraint_chirho(&mut self, constraint_chirho: &DomainHwChirho) {
        self.seq_apply_constraint_chirho(constraint_chirho);
    }

    /// Filter out failed states (sequential)
    pub fn seq_prune_failed_chirho(&mut self) {
        self.states_chirho.retain(|s| !s.is_failed_chirho());
    }

    /// Filter out failed states (parallel collect then replace)
    #[cfg(feature = "parallel_chirho")]
    pub fn par_prune_failed_chirho(&mut self) {
        let filtered_chirho: Vec<_> = self.states_chirho
            .par_iter()
            .filter(|s| !s.is_failed_chirho())
            .cloned()
            .collect();
        self.states_chirho = filtered_chirho;
    }

    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_prune_failed_chirho(&mut self) {
        self.seq_prune_failed_chirho();
    }

    /// Count non-failed states (sequential)
    pub fn seq_count_live_chirho(&self) -> usize {
        self.states_chirho.iter().filter(|s| !s.is_failed_chirho()).count()
    }

    /// Count non-failed states (parallel)
    #[cfg(feature = "parallel_chirho")]
    pub fn par_count_live_chirho(&self) -> usize {
        self.states_chirho
            .par_iter()
            .filter(|s| !s.is_failed_chirho())
            .count()
    }

    #[cfg(not(feature = "parallel_chirho"))]
    pub fn par_count_live_chirho(&self) -> usize {
        self.seq_count_live_chirho()
    }
}

/// Result of scaling benchmark
#[derive(Debug, Clone)]
pub struct ScalingResultChirho {
    pub num_cores_chirho: usize,
    pub seq_time_ns_chirho: u64,
    pub par_time_ns_chirho: u64,
    pub speedup_chirho: f64,
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_parallel_state_basic_chirho() {
        let mut state_chirho = ParallelStateChirho::new_chirho(8);
        assert_eq!(state_chirho.num_vars_chirho(), 8);
        assert!(!state_chirho.is_failed_chirho());

        let constraint_chirho = DomainHwChirho::singleton_chirho(5);
        state_chirho.par_intersect_all_chirho(&constraint_chirho);

        // All domains should now be singleton {5}
        for domain_chirho in &state_chirho.domains_chirho {
            assert!(domain_chirho.is_singleton_chirho());
            assert_eq!(domain_chirho.get_singleton_chirho(), Some(5));
        }
    }

    #[test]
    fn test_parallel_batch_chirho() {
        let states_chirho: Vec<_> = (0..100)
            .map(|_| ParallelStateChirho::new_chirho(8))
            .collect();

        let mut batch_chirho = ParallelBatchChirho::new_chirho(states_chirho);
        assert_eq!(batch_chirho.len_chirho(), 100);

        let constraint_chirho = DomainHwChirho::singleton_chirho(10);
        batch_chirho.par_apply_constraint_chirho(&constraint_chirho);

        // All states should be valid with singleton domains
        assert_eq!(batch_chirho.par_count_live_chirho(), 100);
    }

    #[test]
    fn test_pairwise_intersect_chirho() {
        let mut state1_chirho = ParallelStateChirho::new_chirho(4);
        let state2_chirho = ParallelStateChirho::with_domains_chirho(vec![
            DomainHwChirho::singleton_chirho(1),
            DomainHwChirho::singleton_chirho(2),
            DomainHwChirho::singleton_chirho(3),
            DomainHwChirho::singleton_chirho(4),
        ]);

        state1_chirho.par_pairwise_intersect_chirho(&state2_chirho);

        // Should be singletons from state2
        for (i_chirho, domain_chirho) in state1_chirho.domains_chirho.iter().enumerate() {
            assert_eq!(domain_chirho.get_singleton_chirho(), Some((i_chirho + 1) as u32));
        }
    }

    #[test]
    fn test_prune_failed_chirho() {
        let mut states_chirho: Vec<ParallelStateChirho> = Vec::new();

        // Some valid states
        for _ in 0..50 {
            states_chirho.push(ParallelStateChirho::new_chirho(4));
        }

        // Some failed states (empty domain)
        for _ in 0..50 {
            let mut failed_chirho = ParallelStateChirho::new_chirho(4);
            failed_chirho.domains_chirho[0] = DomainHwChirho::empty_chirho();
            states_chirho.push(failed_chirho);
        }

        let mut batch_chirho = ParallelBatchChirho::new_chirho(states_chirho);
        assert_eq!(batch_chirho.len_chirho(), 100);
        assert_eq!(batch_chirho.par_count_live_chirho(), 50);

        batch_chirho.par_prune_failed_chirho();
        assert_eq!(batch_chirho.len_chirho(), 50);
    }
}
