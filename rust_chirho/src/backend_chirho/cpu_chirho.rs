// For God so loved the world that He gave His only begotten Son ☧
//! CPU Backend Implementation
//!
//! Pure Rust implementation with SIMD acceleration where available.

use super::trait_chirho::{
    BackendInfoChirho, ConstraintChirho, DomainVecChirho, SolutionChirho, SolverBackendChirho,
};

/// CPU-based solver backend
///
/// Uses:
/// - Native Rust bit operations
/// - SIMD (AVX2) for batch operations when available
/// - Rayon for parallel search (optional)
#[derive(Debug)]
pub struct CpuBackendChirho {
    /// Whether to use parallel search
    parallel_chirho: bool,
}

impl CpuBackendChirho {
    /// Create a new CPU backend with default settings
    pub fn new_chirho() -> Self {
        Self {
            parallel_chirho: true,
        }
    }

    /// Create with explicit parallelism setting
    pub fn with_parallel_chirho(parallel_chirho: bool) -> Self {
        Self { parallel_chirho }
    }

    /// Recursive search with backtracking
    fn search_chirho(
        &self,
        domains_chirho: &mut DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
        solutions_chirho: &mut Vec<SolutionChirho>,
        max_chirho: usize,
    ) {
        // Check if we've hit the limit
        if max_chirho > 0 && solutions_chirho.len() >= max_chirho {
            return;
        }

        // Propagate constraints
        let propagated_chirho = self.propagate_chirho(domains_chirho, constraints_chirho);

        // Check for failure (any domain empty)
        if propagated_chirho.iter().any(|d| *d == 0) {
            return;
        }

        // Check if all domains are singleton (solution found)
        if propagated_chirho.iter().all(|d| d.count_ones() == 1) {
            solutions_chirho.push(SolutionChirho {
                assignments_chirho: propagated_chirho
                    .iter()
                    .map(|d| d.trailing_zeros() as u64)
                    .collect(),
            });
            return;
        }

        // Find variable with smallest domain > 1 (MRV heuristic)
        let (var_chirho, domain_chirho) = propagated_chirho
            .iter()
            .enumerate()
            .filter(|(_, d)| d.count_ones() > 1)
            .min_by_key(|(_, d)| d.count_ones())
            .unwrap();

        // Branch on each value in the domain
        let mut val_chirho = *domain_chirho;
        while val_chirho != 0 {
            let bit_chirho = val_chirho & val_chirho.wrapping_neg(); // lowest set bit
            val_chirho &= val_chirho - 1; // clear lowest set bit

            // Create branch with this value fixed
            let mut branch_chirho = propagated_chirho.clone();
            branch_chirho[var_chirho] = bit_chirho;

            // Recurse
            self.search_chirho(
                &mut branch_chirho,
                constraints_chirho,
                solutions_chirho,
                max_chirho,
            );

            if max_chirho > 0 && solutions_chirho.len() >= max_chirho {
                return;
            }
        }
    }
}

impl SolverBackendChirho for CpuBackendChirho {
    fn intersect_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64 {
        a_chirho & b_chirho
    }

    fn union_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64 {
        a_chirho | b_chirho
    }

    fn complement_64_chirho(&self, a_chirho: u64) -> u64 {
        !a_chirho
    }

    fn popcount_64_chirho(&self, a_chirho: u64) -> u32 {
        a_chirho.count_ones()
    }

    fn intersect_batch_chirho(&self, pairs_chirho: &[(u64, u64)]) -> Vec<u64> {
        // TODO: Use SIMD from simd_chirho module for larger batches
        pairs_chirho
            .iter()
            .map(|(a, b)| a & b)
            .collect()
    }

    fn propagate_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
    ) -> DomainVecChirho {
        let mut result_chirho = domains_chirho.clone();
        let mut changed_chirho = true;

        // Iterate until fixpoint
        while changed_chirho {
            changed_chirho = false;

            for constraint_chirho in constraints_chirho {
                match constraint_chirho {
                    ConstraintChirho::EqualChirho { var_a_chirho, var_b_chirho } => {
                        let intersection_chirho =
                            result_chirho[*var_a_chirho] & result_chirho[*var_b_chirho];
                        if result_chirho[*var_a_chirho] != intersection_chirho {
                            result_chirho[*var_a_chirho] = intersection_chirho;
                            changed_chirho = true;
                        }
                        if result_chirho[*var_b_chirho] != intersection_chirho {
                            result_chirho[*var_b_chirho] = intersection_chirho;
                            changed_chirho = true;
                        }
                    }
                    ConstraintChirho::NotEqualChirho { var_a_chirho, var_b_chirho } => {
                        // If one is singleton, remove that value from the other
                        if result_chirho[*var_a_chirho].count_ones() == 1 {
                            let new_chirho =
                                result_chirho[*var_b_chirho] & !result_chirho[*var_a_chirho];
                            if result_chirho[*var_b_chirho] != new_chirho {
                                result_chirho[*var_b_chirho] = new_chirho;
                                changed_chirho = true;
                            }
                        }
                        if result_chirho[*var_b_chirho].count_ones() == 1 {
                            let new_chirho =
                                result_chirho[*var_a_chirho] & !result_chirho[*var_b_chirho];
                            if result_chirho[*var_a_chirho] != new_chirho {
                                result_chirho[*var_a_chirho] = new_chirho;
                                changed_chirho = true;
                            }
                        }
                    }
                    ConstraintChirho::FixedChirho { var_chirho, value_chirho } => {
                        let fixed_chirho = 1u64 << value_chirho;
                        let new_chirho = result_chirho[*var_chirho] & fixed_chirho;
                        if result_chirho[*var_chirho] != new_chirho {
                            result_chirho[*var_chirho] = new_chirho;
                            changed_chirho = true;
                        }
                    }
                    ConstraintChirho::BinaryChirho { var_a_chirho, var_b_chirho, allowed_pairs_chirho } => {
                        // Arc consistency for binary constraint
                        let mut new_a_chirho = 0u64;
                        let mut new_b_chirho = 0u64;

                        // For each value in var_a's domain
                        for i in 0..64 {
                            if result_chirho[*var_a_chirho] & (1 << i) != 0 {
                                // Check if any value in var_b is compatible
                                let compatible_chirho = if i < allowed_pairs_chirho.len() {
                                    allowed_pairs_chirho[i] & result_chirho[*var_b_chirho]
                                } else {
                                    0
                                };
                                if compatible_chirho != 0 {
                                    new_a_chirho |= 1 << i;
                                    new_b_chirho |= compatible_chirho;
                                }
                            }
                        }

                        if result_chirho[*var_a_chirho] != new_a_chirho {
                            result_chirho[*var_a_chirho] = new_a_chirho;
                            changed_chirho = true;
                        }
                        if result_chirho[*var_b_chirho] != new_b_chirho {
                            result_chirho[*var_b_chirho] = new_b_chirho;
                            changed_chirho = true;
                        }
                    }
                }
            }
        }

        result_chirho
    }

    fn solve_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
        max_solutions_chirho: usize,
    ) -> Vec<SolutionChirho> {
        let mut solutions_chirho = Vec::new();
        let mut domains_mut_chirho = domains_chirho.clone();

        self.search_chirho(
            &mut domains_mut_chirho,
            constraints_chirho,
            &mut solutions_chirho,
            max_solutions_chirho,
        );

        solutions_chirho
    }

    fn info_chirho(&self) -> BackendInfoChirho {
        BackendInfoChirho {
            name_chirho: "cpu",
            version_chirho: env!("CARGO_PKG_VERSION").to_string(),
            throughput_estimate_chirho: 500_000, // ~500K simple queries/sec
            batch_accelerated_chirho: cfg!(target_feature = "avx2"),
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_intersect_chirho() {
        let backend_chirho = CpuBackendChirho::new_chirho();
        assert_eq!(backend_chirho.intersect_64_chirho(0xFF00, 0x0FF0), 0x0F00);
    }

    #[test]
    fn test_simple_csp_chirho() {
        let backend_chirho = CpuBackendChirho::new_chirho();

        // Two variables, each can be 0 or 1, must be different
        let domains_chirho = vec![0b11, 0b11]; // {0, 1} for each
        let constraints_chirho = vec![ConstraintChirho::NotEqualChirho {
            var_a_chirho: 0,
            var_b_chirho: 1,
        }];

        let solutions_chirho = backend_chirho.solve_chirho(&domains_chirho, &constraints_chirho, 0);

        // Should have 2 solutions: (0,1) and (1,0)
        assert_eq!(solutions_chirho.len(), 2);
    }

    #[test]
    fn test_equal_constraint_chirho() {
        let backend_chirho = CpuBackendChirho::new_chirho();

        // Two variables must be equal
        let domains_chirho = vec![0b111, 0b110]; // {0,1,2} and {1,2}
        let constraints_chirho = vec![ConstraintChirho::EqualChirho {
            var_a_chirho: 0,
            var_b_chirho: 1,
        }];

        let propagated_chirho = backend_chirho.propagate_chirho(&domains_chirho, &constraints_chirho);

        // Should reduce to intersection: {1, 2}
        assert_eq!(propagated_chirho[0], 0b110);
        assert_eq!(propagated_chirho[1], 0b110);
    }
}
