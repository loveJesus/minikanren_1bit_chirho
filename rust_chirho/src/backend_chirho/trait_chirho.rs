// For God so loved the world that He gave His only begotten Son ☧
//! Solver Backend Trait
//!
//! Defines the interface that both CPU and FPGA backends implement.

use std::fmt::Debug;

/// Information about a solver backend
#[derive(Debug, Clone)]
pub struct BackendInfoChirho {
    /// Backend name: "cpu" or "fpga"
    pub name_chirho: &'static str,
    /// Version string (e.g., "0.2.0" or "0xF0051D0F")
    pub version_chirho: String,
    /// Estimated throughput (queries/sec)
    pub throughput_estimate_chirho: u64,
    /// Whether batch operations are hardware-accelerated
    pub batch_accelerated_chirho: bool,
}

/// A constraint between variables
#[derive(Debug, Clone)]
pub enum ConstraintChirho {
    /// Variables must have equal values
    EqualChirho {
        var_a_chirho: usize,
        var_b_chirho: usize,
    },
    /// Variables must have different values
    NotEqualChirho {
        var_a_chirho: usize,
        var_b_chirho: usize,
    },
    /// Custom binary constraint (indices into domain)
    BinaryChirho {
        var_a_chirho: usize,
        var_b_chirho: usize,
        /// Allowed pairs as bitmask: bit i*64+j set means (i,j) allowed
        allowed_pairs_chirho: Vec<u64>,
    },
    /// Variable must have specific value
    FixedChirho {
        var_chirho: usize,
        value_chirho: u64,
    },
}

/// A solution: assignment of values to variables
#[derive(Debug, Clone, PartialEq)]
pub struct SolutionChirho {
    /// Value assigned to each variable (index = variable, value = domain element)
    pub assignments_chirho: Vec<u64>,
}

/// Domain vector: one 64-bit domain per variable
pub type DomainVecChirho = Vec<u64>;

/// Backend-agnostic solving interface
///
/// Implementations:
/// - [`CpuBackendChirho`]: Rust with SIMD (default, always available)
/// - [`FpgaBackendChirho`]: AWS F2 via PCIe (feature-gated)
///
/// # Design Notes
///
/// - All operations are synchronous for simplicity
/// - Batch operations return results in same order as input
/// - Backends are Send + Sync for use in async contexts
pub trait SolverBackendChirho: Send + Sync + Debug {
    // ========================================================================
    // Core Operations
    // ========================================================================

    /// Intersect two 64-bit domains (AND)
    ///
    /// This is the fundamental operation: `a & b`
    fn intersect_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64;

    /// Union two 64-bit domains (OR)
    fn union_64_chirho(&self, a_chirho: u64, b_chirho: u64) -> u64;

    /// Complement a 64-bit domain (NOT)
    fn complement_64_chirho(&self, a_chirho: u64) -> u64;

    /// Count set bits (population count)
    fn popcount_64_chirho(&self, a_chirho: u64) -> u32;

    // ========================================================================
    // Batch Operations (where FPGA shines)
    // ========================================================================

    /// Batch intersect pairs of domains
    ///
    /// Default implementation loops; FPGA uses DMA + HBM parallelism.
    fn intersect_batch_chirho(&self, pairs_chirho: &[(u64, u64)]) -> Vec<u64> {
        pairs_chirho
            .iter()
            .map(|(a, b)| self.intersect_64_chirho(*a, *b))
            .collect()
    }

    /// Batch popcount
    fn popcount_batch_chirho(&self, domains_chirho: &[u64]) -> Vec<u32> {
        domains_chirho
            .iter()
            .map(|d| self.popcount_64_chirho(*d))
            .collect()
    }

    // ========================================================================
    // Constraint Propagation
    // ========================================================================

    /// Propagate constraints until fixpoint
    ///
    /// Returns updated domains after arc consistency.
    fn propagate_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
    ) -> DomainVecChirho;

    // ========================================================================
    // Solving
    // ========================================================================

    /// Solve CSP, returning all solutions
    ///
    /// # Arguments
    /// - `domains_chirho`: Initial domain for each variable
    /// - `constraints_chirho`: Constraints between variables
    /// - `max_solutions_chirho`: Maximum solutions to return (0 = unlimited)
    ///
    /// # Returns
    /// Vector of solutions, each containing an assignment for every variable.
    fn solve_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
        max_solutions_chirho: usize,
    ) -> Vec<SolutionChirho>;

    /// Check if a CSP has any solution (faster than solve if you just need yes/no)
    fn is_satisfiable_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
    ) -> bool {
        !self.solve_chirho(domains_chirho, constraints_chirho, 1).is_empty()
    }

    /// Count solutions (may be faster than enumerating all)
    fn count_solutions_chirho(
        &self,
        domains_chirho: &DomainVecChirho,
        constraints_chirho: &[ConstraintChirho],
    ) -> u64 {
        self.solve_chirho(domains_chirho, constraints_chirho, 0).len() as u64
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get backend information
    fn info_chirho(&self) -> BackendInfoChirho;

    /// Check if backend is healthy (FPGA may disconnect)
    fn is_healthy_chirho(&self) -> bool {
        true
    }
}
