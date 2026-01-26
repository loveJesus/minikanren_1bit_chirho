// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Hybrid Domain System ☧
//!
//! Combines finite (hardware-accelerated) and infinite (symbolic) domains.
//! Uses mode analysis to determine which variables can use hardware path.
//!
//! ```text
//! Query: appendo(X, Y, [1,2,3,4,5])
//!
//! Analysis:
//! - Output [1,2,3,4,5] is ground → finite structure
//! - X can only be prefixes: [], [1], [1,2], ... → 6 values (finite!)
//! - Y determined by X → also finite
//!
//! Hardware path: intern all 6 X values, use BitVec64
//! ```

use std::collections::HashMap;
use crate::hardware_chirho::{BitVec64Chirho, SearchStateHwChirho};

/// Iterate over set bits in a u64
fn iter_bits_chirho(mut bits_chirho: u64) -> impl Iterator<Item = u32> {
    std::iter::from_fn(move || {
        if bits_chirho == 0 {
            None
        } else {
            let bit_chirho = bits_chirho.trailing_zeros();
            bits_chirho &= bits_chirho - 1;
            Some(bit_chirho)
        }
    })
}
use crate::reference_chirho::terms_chirho::{TermIdChirho, TermStoreChirho};
use super::symbolic_chirho::SymbolicDomainChirho;
use super::paged_chirho::PagedDomainChirho;

/// Hybrid domain: finite or symbolic
#[derive(Debug, Clone)]
pub enum HybridDomainChirho {
    /// Fits in 64 bits - use hardware
    Small(BitVec64Chirho),

    /// Larger finite - use paged
    Large(PagedDomainChirho),

    /// Symbolic constraint - use solver
    Symbolic(SymbolicDomainChirho),

    /// Truly infinite - must enumerate lazily
    Infinite,
}

impl HybridDomainChirho {
    /// Try to materialize to BitVec64 for hardware acceleration
    pub fn try_to_small_chirho(&self) -> Option<BitVec64Chirho> {
        match self {
            HybridDomainChirho::Small(bits) => Some(*bits),
            HybridDomainChirho::Large(paged) => paged.to_bitvec64_chirho(),
            HybridDomainChirho::Symbolic(sym) => sym.try_enumerate_chirho(64),
            HybridDomainChirho::Infinite => None,
        }
    }

    /// Check if domain is finite
    pub fn is_finite_chirho(&self) -> bool {
        !matches!(self, HybridDomainChirho::Infinite)
    }

    /// Intersect two hybrid domains
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        use HybridDomainChirho::*;

        match (self, other_chirho) {
            // Both small: hardware AND
            (Small(a), Small(b)) => {
                let result_chirho = a.and_chirho(*b);
                if result_chirho.is_zero_chirho() {
                    Small(BitVec64Chirho::ZERO_CHIRHO)
                } else {
                    Small(result_chirho)
                }
            }

            // Both large: paged intersection
            (Large(a), Large(b)) => {
                let result_chirho = a.intersect_chirho(b);
                // Try to downgrade to small
                if let Some(bits) = result_chirho.to_bitvec64_chirho() {
                    Small(bits)
                } else {
                    Large(result_chirho)
                }
            }

            // Small + Large: filter large by small
            (Small(bits), Large(paged)) | (Large(paged), Small(bits)) => {
                if let Some(paged_bits) = paged.to_bitvec64_chirho() {
                    Small(bits.and_chirho(paged_bits))
                } else {
                    // Can only keep values that are in both
                    let mut result_chirho = PagedDomainChirho::empty_chirho();
                    for val in iter_bits_chirho(bits.0) {
                        if paged.contains_chirho(val as u64) {
                            result_chirho.insert_chirho(val as u64);
                        }
                    }
                    if let Some(bits) = result_chirho.to_bitvec64_chirho() {
                        Small(bits)
                    } else {
                        Large(result_chirho)
                    }
                }
            }

            // Symbolic intersections
            (Symbolic(a), Symbolic(b)) => {
                let result_chirho = a.intersect_chirho(b);
                // Try to materialize
                if let Some(bits) = result_chirho.try_enumerate_chirho(64) {
                    Small(bits)
                } else {
                    Symbolic(result_chirho)
                }
            }

            // Finite + Symbolic: filter finite by constraint
            (Small(bits), Symbolic(sym)) | (Symbolic(sym), Small(bits)) => {
                let mut result_bits_chirho = 0u64;
                for val in iter_bits_chirho(bits.0) {
                    if sym.contains_chirho(val as i64) {
                        result_bits_chirho |= 1u64 << val;
                    }
                }
                Small(BitVec64Chirho(result_bits_chirho))
            }

            (Large(paged), Symbolic(sym)) | (Symbolic(sym), Large(paged)) => {
                let mut result_chirho = PagedDomainChirho::empty_chirho();
                for val in paged.iter_chirho() {
                    if sym.contains_chirho(val as i64) {
                        result_chirho.insert_chirho(val);
                    }
                }
                if let Some(bits) = result_chirho.to_bitvec64_chirho() {
                    Small(bits)
                } else {
                    Large(result_chirho)
                }
            }

            // Infinite with anything: stay infinite or narrow
            (Infinite, other) | (other, Infinite) => {
                if other.is_finite_chirho() {
                    other.clone()
                } else {
                    Infinite
                }
            }
        }
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        match self {
            HybridDomainChirho::Small(bits) => bits.is_zero_chirho(),
            HybridDomainChirho::Large(paged) => paged.is_empty_chirho(),
            HybridDomainChirho::Symbolic(sym) => sym.is_empty_chirho(),
            HybridDomainChirho::Infinite => false,
        }
    }
}

/// Variable mode: how ground is it?
#[derive(Debug, Clone)]
pub enum ModeChirho {
    /// Bound to specific term
    Ground(TermIdChirho),

    /// Constrained to finite set of term IDs
    Finite(BitVec64Chirho),

    /// Has symbolic constraint
    Constrained(SymbolicDomainChirho),

    /// Completely free
    Free,
}

/// Hybrid search state with mixed domains
#[derive(Debug, Clone)]
pub struct HybridStateChirho {
    /// Variable ID → domain
    pub domains_chirho: HashMap<u32, HybridDomainChirho>,

    /// Which variables are hardware-eligible (fit in 64 values)
    pub hw_eligible_chirho: Vec<u32>,

    /// Term store for creating/looking up terms
    pub store_chirho: TermStoreChirho,
}

impl HybridStateChirho {
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: HashMap::new(),
            hw_eligible_chirho: Vec::new(),
            store_chirho: TermStoreChirho::new(),
        }
    }

    /// Analyze which variables can use hardware path
    pub fn analyze_hw_eligible_chirho(&mut self) {
        self.hw_eligible_chirho.clear();

        for (&var_chirho, domain_chirho) in &self.domains_chirho {
            if domain_chirho.try_to_small_chirho().is_some() {
                self.hw_eligible_chirho.push(var_chirho);
            }
        }
    }

    /// Extract hardware state for eligible variables
    pub fn extract_hw_state_chirho<const N: usize>(&self) -> Option<SearchStateHwChirho<N>> {
        if self.hw_eligible_chirho.len() > N {
            return None;
        }

        let mut state_chirho = SearchStateHwChirho::<N>::new_chirho();

        for (idx_chirho, &var_chirho) in self.hw_eligible_chirho.iter().enumerate() {
            if idx_chirho >= N {
                break;
            }
            if let Some(domain_chirho) = self.domains_chirho.get(&var_chirho) {
                if let Some(bits_chirho) = domain_chirho.try_to_small_chirho() {
                    state_chirho.domains_chirho[idx_chirho] = bits_chirho;
                }
            }
        }

        Some(state_chirho)
    }

    /// Apply hardware results back to hybrid state
    pub fn apply_hw_results_chirho<const N: usize>(&mut self, hw_state_chirho: &SearchStateHwChirho<N>) {
        for (idx_chirho, &var_chirho) in self.hw_eligible_chirho.iter().enumerate() {
            if idx_chirho >= N {
                break;
            }
            let bits_chirho = hw_state_chirho.domains_chirho[idx_chirho];
            self.domains_chirho.insert(var_chirho, HybridDomainChirho::Small(bits_chirho));
        }
    }

    /// Constrain a variable's domain
    pub fn constrain_chirho(&mut self, var_chirho: u32, new_domain_chirho: HybridDomainChirho) {
        self.domains_chirho
            .entry(var_chirho)
            .and_modify(|existing| *existing = existing.intersect_chirho(&new_domain_chirho))
            .or_insert(new_domain_chirho);
    }

    /// Check if state is valid (no empty domains)
    pub fn is_valid_chirho(&self) -> bool {
        !self.domains_chirho.values().any(|d| d.is_empty_chirho())
    }
}

/// Partition variables into hardware and reference sets
pub fn partition_by_domain_chirho(
    domains_chirho: &HashMap<u32, HybridDomainChirho>
) -> (Vec<u32>, Vec<u32>) {
    let mut hw_vars_chirho = Vec::new();
    let mut ref_vars_chirho = Vec::new();

    for (&var_chirho, domain_chirho) in domains_chirho {
        if domain_chirho.try_to_small_chirho().is_some() {
            hw_vars_chirho.push(var_chirho);
        } else {
            ref_vars_chirho.push(var_chirho);
        }
    }

    (hw_vars_chirho, ref_vars_chirho)
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_hybrid_intersect_chirho() {
        let small_chirho = HybridDomainChirho::Small(BitVec64Chirho(0b11111)); // 0-4
        let sym_chirho = HybridDomainChirho::Symbolic(
            SymbolicDomainChirho::modular_chirho(0, 2) // Even numbers
        );

        let result_chirho = small_chirho.intersect_chirho(&sym_chirho);

        // Should be {0, 2, 4} = 0b10101
        if let HybridDomainChirho::Small(bits) = result_chirho {
            assert!(bits.test_bit_chirho(0));
            assert!(!bits.test_bit_chirho(1));
            assert!(bits.test_bit_chirho(2));
            assert!(!bits.test_bit_chirho(3));
            assert!(bits.test_bit_chirho(4));
        } else {
            panic!("Expected Small domain");
        }
    }

    #[test]
    fn test_partition_chirho() {
        let mut domains_chirho = HashMap::new();
        domains_chirho.insert(0, HybridDomainChirho::Small(BitVec64Chirho(0xFF)));
        domains_chirho.insert(1, HybridDomainChirho::Infinite);
        domains_chirho.insert(2, HybridDomainChirho::Small(BitVec64Chirho(0xF)));

        let (hw, reference) = partition_by_domain_chirho(&domains_chirho);

        assert_eq!(hw.len(), 2); // vars 0 and 2
        assert_eq!(reference.len(), 1); // var 1
    }
}
