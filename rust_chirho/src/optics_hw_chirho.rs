//! Hardware Optics ☧
//!
//! Optics over 1-bit domains using BitVec64/256.
//! FPGA-ready: no heap, fixed size, pure bit operations.
//!
//! Key insight: A "lens" into a domain is a bitmask projection.

use crate::hardware_chirho::BitVec64Chirho;

// ============================================================================
// Domain as Bitmask (max 64 known term IDs)
// ============================================================================

/// Domain over hash-consed term IDs (≤64 terms)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DomainHwChirho {
    /// Bitmask: bit i = 1 means term ID i is possible
    pub bits_chirho: BitVec64Chirho,
}

impl DomainHwChirho {
    pub fn full_chirho() -> Self {
        Self { bits_chirho: BitVec64Chirho::ONES_CHIRHO }
    }

    pub fn empty_chirho() -> Self {
        Self { bits_chirho: BitVec64Chirho::ZERO_CHIRHO }
    }

    pub fn singleton_chirho(term_id_chirho: u32) -> Self {
        Self {
            bits_chirho: BitVec64Chirho::ZERO_CHIRHO.set_bit_chirho(term_id_chirho)
        }
    }

    /// Intersection (AND) - unification
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        Self { bits_chirho: self.bits_chirho.and_chirho(other_chirho.bits_chirho) }
    }

    /// Union (OR) - disjunction
    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        Self { bits_chirho: self.bits_chirho.or_chirho(other_chirho.bits_chirho) }
    }

    /// Check if empty (failure)
    pub fn is_empty_chirho(&self) -> bool {
        self.bits_chirho.is_zero_chirho()
    }

    /// Check if singleton (determined)
    pub fn is_singleton_chirho(&self) -> bool {
        self.bits_chirho.is_singleton_chirho()
    }

    /// Get singleton value (if determined)
    pub fn get_singleton_chirho(&self) -> Option<u32> {
        if self.is_singleton_chirho() {
            // ctz gives index of lowest (only) set bit
            Some(self.bits_chirho.ctz_chirho())
        } else {
            None
        }
    }

    /// Population count
    pub fn popcount_chirho(&self) -> u32 {
        self.bits_chirho.popcount_chirho()
    }
}

// ============================================================================
// Prism: Focus on subset of domain
// ============================================================================

/// Hardware Prism: bitmask that selects a subset
#[derive(Debug, Clone, Copy)]
pub struct PrismHwChirho {
    /// Mask of term IDs this prism focuses on
    pub mask_chirho: BitVec64Chirho,
}

impl PrismHwChirho {
    /// Create prism that focuses on terms matching a predicate
    /// (predicate encoded as bitmask)
    pub fn new_chirho(mask_chirho: BitVec64Chirho) -> Self {
        Self { mask_chirho }
    }

    /// Preview: extract focused subset (if any match)
    pub fn preview_chirho(&self, domain_chirho: &DomainHwChirho) -> Option<DomainHwChirho> {
        let focused_chirho = domain_chirho.bits_chirho.and_chirho(self.mask_chirho);
        if focused_chirho.is_zero_chirho() {
            None
        } else {
            Some(DomainHwChirho { bits_chirho: focused_chirho })
        }
    }

    /// Review: embed focused domain back (just the mask intersection)
    pub fn review_chirho(&self, focused_chirho: &DomainHwChirho) -> DomainHwChirho {
        DomainHwChirho {
            bits_chirho: focused_chirho.bits_chirho.and_chirho(self.mask_chirho)
        }
    }
}

// ============================================================================
// Lens: Pair of domains (for cons cells)
// ============================================================================

/// Hardware state: array of domain bitmasks (one per variable)
/// Fixed size for FPGA (no heap allocation)
#[derive(Debug, Clone, Copy)]
pub struct StateHwChirho<const N: usize> {
    /// Domain for each variable
    pub domains_chirho: [DomainHwChirho; N],
}

impl<const N: usize> StateHwChirho<N> {
    pub fn new_all_full_chirho() -> Self {
        Self { domains_chirho: [DomainHwChirho::full_chirho(); N] }
    }

    pub fn new_all_empty_chirho() -> Self {
        Self { domains_chirho: [DomainHwChirho::empty_chirho(); N] }
    }
}

/// Hardware Lens: index into state's domain array
#[derive(Debug, Clone, Copy)]
pub struct LensHwChirho {
    /// Which variable this lens focuses on
    pub var_idx_chirho: usize,
}

impl LensHwChirho {
    pub fn new_chirho(var_idx_chirho: usize) -> Self {
        Self { var_idx_chirho }
    }

    /// Get domain at this variable
    pub fn get_chirho<const N: usize>(&self, state_chirho: &StateHwChirho<N>) -> DomainHwChirho {
        state_chirho.domains_chirho[self.var_idx_chirho]
    }

    /// Set domain at this variable
    pub fn set_chirho<const N: usize>(
        &self,
        state_chirho: StateHwChirho<N>,
        domain_chirho: DomainHwChirho,
    ) -> StateHwChirho<N> {
        let mut new_state_chirho = state_chirho;
        new_state_chirho.domains_chirho[self.var_idx_chirho] = domain_chirho;
        new_state_chirho
    }

    /// Modify domain at this variable
    pub fn over_chirho<const N: usize, F>(
        &self,
        state_chirho: StateHwChirho<N>,
        f_chirho: F,
    ) -> StateHwChirho<N>
    where
        F: FnOnce(DomainHwChirho) -> DomainHwChirho,
    {
        let old_chirho = self.get_chirho(&state_chirho);
        let new_chirho = f_chirho(old_chirho);
        self.set_chirho(state_chirho, new_chirho)
    }
}

// ============================================================================
// Traversal: Over all variables
// ============================================================================

/// Apply function to all domains in state
pub fn traverse_all_chirho<const N: usize, F>(
    state_chirho: StateHwChirho<N>,
    f_chirho: F,
) -> StateHwChirho<N>
where
    F: Fn(DomainHwChirho) -> DomainHwChirho,
{
    let mut new_state_chirho = state_chirho;
    for i in 0..N {
        new_state_chirho.domains_chirho[i] = f_chirho(state_chirho.domains_chirho[i]);
    }
    new_state_chirho
}

/// Collect all non-empty domains
pub fn collect_nonempty_chirho<const N: usize>(
    state_chirho: &StateHwChirho<N>,
) -> [bool; N] {
    let mut result_chirho = [false; N];
    for i in 0..N {
        result_chirho[i] = !state_chirho.domains_chirho[i].is_empty_chirho();
    }
    result_chirho
}

// ============================================================================
// Partitioned Domain (for >64 terms)
// ============================================================================

/// Domain partitioned by bucket (4 buckets × 64 bits = 256 terms)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionedDomainChirho {
    pub buckets_chirho: [BitVec64Chirho; 4],
}

impl PartitionedDomainChirho {
    pub fn empty_chirho() -> Self {
        Self { buckets_chirho: [BitVec64Chirho::ZERO_CHIRHO; 4] }
    }

    pub fn full_chirho() -> Self {
        Self { buckets_chirho: [BitVec64Chirho::ONES_CHIRHO; 4] }
    }

    /// Get bucket index for term ID
    fn bucket_idx_chirho(term_id_chirho: u32) -> usize {
        (term_id_chirho as usize) / 64
    }

    /// Get bit index within bucket
    fn bit_idx_chirho(term_id_chirho: u32) -> u32 {
        term_id_chirho % 64
    }

    pub fn singleton_chirho(term_id_chirho: u32) -> Self {
        let mut result_chirho = Self::empty_chirho();
        let bucket_chirho = Self::bucket_idx_chirho(term_id_chirho);
        let bit_chirho = Self::bit_idx_chirho(term_id_chirho);
        if bucket_chirho < 4 {
            result_chirho.buckets_chirho[bucket_chirho] =
                BitVec64Chirho::ZERO_CHIRHO.set_bit_chirho(bit_chirho);
        }
        result_chirho
    }

    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        Self {
            buckets_chirho: [
                self.buckets_chirho[0].and_chirho(other_chirho.buckets_chirho[0]),
                self.buckets_chirho[1].and_chirho(other_chirho.buckets_chirho[1]),
                self.buckets_chirho[2].and_chirho(other_chirho.buckets_chirho[2]),
                self.buckets_chirho[3].and_chirho(other_chirho.buckets_chirho[3]),
            ],
        }
    }

    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        Self {
            buckets_chirho: [
                self.buckets_chirho[0].or_chirho(other_chirho.buckets_chirho[0]),
                self.buckets_chirho[1].or_chirho(other_chirho.buckets_chirho[1]),
                self.buckets_chirho[2].or_chirho(other_chirho.buckets_chirho[2]),
                self.buckets_chirho[3].or_chirho(other_chirho.buckets_chirho[3]),
            ],
        }
    }

    pub fn is_empty_chirho(&self) -> bool {
        self.buckets_chirho.iter().all(|b| b.is_zero_chirho())
    }

    pub fn popcount_chirho(&self) -> u32 {
        self.buckets_chirho.iter().map(|b| b.popcount_chirho()).sum()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_domain_hw_chirho() {
        let d1_chirho = DomainHwChirho::singleton_chirho(5);
        let d2_chirho = DomainHwChirho::singleton_chirho(5);
        let d3_chirho = DomainHwChirho::singleton_chirho(7);

        // Same singleton intersects
        let i1_chirho = d1_chirho.intersect_chirho(&d2_chirho);
        assert!(!i1_chirho.is_empty_chirho());
        assert_eq!(i1_chirho.get_singleton_chirho(), Some(5));

        // Different singletons → empty
        let i2_chirho = d1_chirho.intersect_chirho(&d3_chirho);
        assert!(i2_chirho.is_empty_chirho());
    }

    #[test]
    fn test_prism_hw_chirho() {
        // Prism focusing on even term IDs (bits 0, 2, 4, ...)
        let even_mask_chirho = BitVec64Chirho(0x5555555555555555);
        let prism_chirho = PrismHwChirho::new_chirho(even_mask_chirho);

        // Domain with bits 0, 1, 2, 3 set
        let domain_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0b1111) };

        // Preview should give 0, 2 (even ones)
        let focused_chirho = prism_chirho.preview_chirho(&domain_chirho).unwrap();
        assert_eq!(focused_chirho.bits_chirho.0, 0b0101);
    }

    #[test]
    fn test_lens_hw_chirho() {
        let mut state_chirho = StateHwChirho::<4>::new_all_full_chirho();
        let lens_chirho = LensHwChirho::new_chirho(2);

        // Set variable 2 to singleton
        state_chirho = lens_chirho.set_chirho(state_chirho, DomainHwChirho::singleton_chirho(42));

        let got_chirho = lens_chirho.get_chirho(&state_chirho);
        assert_eq!(got_chirho.get_singleton_chirho(), Some(42));
    }

    #[test]
    fn test_traverse_all_chirho() {
        let state_chirho = StateHwChirho::<3>::new_all_full_chirho();

        // Intersect all domains with mask for values 0-7
        let mask_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xFF) };
        let new_state_chirho = traverse_all_chirho(state_chirho, |d| d.intersect_chirho(&mask_chirho));

        for i in 0..3 {
            assert_eq!(new_state_chirho.domains_chirho[i].bits_chirho.0, 0xFF);
        }
    }

    #[test]
    fn test_partitioned_domain_chirho() {
        // Term ID 100 is in bucket 1 (100/64=1), bit 36 (100%64=36)
        let d1_chirho = PartitionedDomainChirho::singleton_chirho(100);
        assert_eq!(d1_chirho.popcount_chirho(), 1);

        // Term ID 200 is in bucket 3 (200/64=3), bit 8 (200%64=8)
        let d2_chirho = PartitionedDomainChirho::singleton_chirho(200);

        // Union should have 2 bits
        let union_chirho = d1_chirho.union_chirho(&d2_chirho);
        assert_eq!(union_chirho.popcount_chirho(), 2);

        // Intersection should be empty
        let inter_chirho = d1_chirho.intersect_chirho(&d2_chirho);
        assert!(inter_chirho.is_empty_chirho());
    }
}
