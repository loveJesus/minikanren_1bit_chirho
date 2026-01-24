//! Hardware-Oriented Primitives ☧
//!
//! Data structures designed for FPGA/ASIC synthesis.
//! These use fixed-size arrays and bit operations that map directly to hardware.
//!
//! Target: Clash (Haskell → Verilog) or Calyx (hardware IR)

/// Fixed-width bit vector for domains (hardware register)
/// 64 bits = 64 possible values per variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct BitVec64Chirho(pub u64);

impl BitVec64Chirho {
    pub const ZERO_CHIRHO: Self = BitVec64Chirho(0);
    pub const ONES_CHIRHO: Self = BitVec64Chirho(u64::MAX);

    /// AND - unification/constraint
    #[inline]
    pub const fn and_chirho(self, other_chirho: Self) -> Self {
        BitVec64Chirho(self.0 & other_chirho.0)
    }

    /// OR - disjunction/conde
    #[inline]
    pub const fn or_chirho(self, other_chirho: Self) -> Self {
        BitVec64Chirho(self.0 | other_chirho.0)
    }

    /// NOT - complement
    #[inline]
    pub const fn not_chirho(self) -> Self {
        BitVec64Chirho(!self.0)
    }

    /// XOR
    #[inline]
    pub const fn xor_chirho(self, other_chirho: Self) -> Self {
        BitVec64Chirho(self.0 ^ other_chirho.0)
    }

    /// Population count (number of 1s)
    #[inline]
    pub const fn popcount_chirho(self) -> u32 {
        self.0.count_ones()
    }

    /// Leading zeros
    #[inline]
    pub const fn clz_chirho(self) -> u32 {
        self.0.leading_zeros()
    }

    /// Trailing zeros (find first set bit)
    #[inline]
    pub const fn ctz_chirho(self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Is zero (empty domain = failure)
    #[inline]
    pub const fn is_zero_chirho(self) -> bool {
        self.0 == 0
    }

    /// Is singleton (exactly one bit set)
    #[inline]
    pub const fn is_singleton_chirho(self) -> bool {
        self.0 != 0 && (self.0 & (self.0 - 1)) == 0
    }

    /// Set bit at index
    #[inline]
    pub const fn set_bit_chirho(self, idx_chirho: u32) -> Self {
        BitVec64Chirho(self.0 | (1u64 << idx_chirho))
    }

    /// Clear bit at index
    #[inline]
    pub const fn clear_bit_chirho(self, idx_chirho: u32) -> Self {
        BitVec64Chirho(self.0 & !(1u64 << idx_chirho))
    }

    /// Test bit at index
    #[inline]
    pub const fn test_bit_chirho(self, idx_chirho: u32) -> bool {
        (self.0 & (1u64 << idx_chirho)) != 0
    }

    /// Extract lowest set bit (isolate rightmost 1)
    #[inline]
    pub const fn lowest_bit_chirho(self) -> Self {
        BitVec64Chirho(self.0 & self.0.wrapping_neg())
    }

    /// Clear lowest set bit
    #[inline]
    pub const fn clear_lowest_chirho(self) -> Self {
        BitVec64Chirho(self.0 & (self.0 - 1))
    }
}

/// Hardware state machine for search
/// Each state holds N variable domains
#[derive(Debug, Clone, Copy)]
pub struct SearchStateHwChirho<const N: usize> {
    /// Variable domains (each is a 64-bit mask)
    pub domains_chirho: [BitVec64Chirho; N],
    /// Valid flag (false = failed state)
    pub valid_chirho: bool,
}

impl<const N: usize> SearchStateHwChirho<N> {
    /// Create initial state with full domains
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: [BitVec64Chirho::ONES_CHIRHO; N],
            valid_chirho: true,
        }
    }

    /// Unify two variables (AND their domains)
    pub fn unify_chirho(&mut self, var1_chirho: usize, var2_chirho: usize) {
        if var1_chirho < N && var2_chirho < N {
            let intersection_chirho = self.domains_chirho[var1_chirho]
                .and_chirho(self.domains_chirho[var2_chirho]);

            self.domains_chirho[var1_chirho] = intersection_chirho;
            self.domains_chirho[var2_chirho] = intersection_chirho;

            if intersection_chirho.is_zero_chirho() {
                self.valid_chirho = false;
            }
        }
    }

    /// Constrain variable to specific value
    pub fn constrain_chirho(&mut self, var_chirho: usize, val_chirho: u32) {
        if var_chirho < N && val_chirho < 64 {
            let mask_chirho = BitVec64Chirho(1u64 << val_chirho);
            self.domains_chirho[var_chirho] =
                self.domains_chirho[var_chirho].and_chirho(mask_chirho);

            if self.domains_chirho[var_chirho].is_zero_chirho() {
                self.valid_chirho = false;
            }
        }
    }

    /// Set domain directly
    pub fn set_domain_chirho(&mut self, var_chirho: usize, domain_chirho: BitVec64Chirho) {
        if var_chirho < N {
            self.domains_chirho[var_chirho] = domain_chirho;
            if domain_chirho.is_zero_chirho() {
                self.valid_chirho = false;
            }
        }
    }

    /// Check if all variables are bound (singleton domains)
    pub fn is_solved_chirho(&self) -> bool {
        self.valid_chirho
            && self
                .domains_chirho
                .iter()
                .all(|d| d.is_singleton_chirho())
    }

    /// Get bound value for variable (if singleton)
    pub fn get_value_chirho(&self, var_chirho: usize) -> Option<u32> {
        if var_chirho < N && self.domains_chirho[var_chirho].is_singleton_chirho() {
            Some(self.domains_chirho[var_chirho].ctz_chirho())
        } else {
            None
        }
    }

    /// Fork: create two states, one with bit set, one with bit clear
    /// Returns (state_with_bit, state_without_bit)
    pub fn fork_on_bit_chirho(
        &self,
        var_chirho: usize,
        bit_chirho: u32,
    ) -> (Self, Self) {
        let mut with_chirho = *self;
        let mut without_chirho = *self;

        if var_chirho < N && bit_chirho < 64 {
            let bit_mask_chirho = BitVec64Chirho(1u64 << bit_chirho);

            // State where variable has this value
            with_chirho.domains_chirho[var_chirho] = self.domains_chirho[var_chirho]
                .and_chirho(bit_mask_chirho);

            // State where variable doesn't have this value
            without_chirho.domains_chirho[var_chirho] = self.domains_chirho[var_chirho]
                .and_chirho(bit_mask_chirho.not_chirho());

            with_chirho.valid_chirho = !with_chirho.domains_chirho[var_chirho].is_zero_chirho();
            without_chirho.valid_chirho = !without_chirho.domains_chirho[var_chirho].is_zero_chirho();
        }

        (with_chirho, without_chirho)
    }
}

/// Content-Addressable Memory entry for relations
/// Stores (key, value) pairs for parallel lookup
#[derive(Debug, Clone, Copy, Default)]
pub struct CamEntryChirho {
    pub key_chirho: u32,
    pub value_chirho: u32,
    pub valid_chirho: bool,
}

/// Hardware CAM (Content-Addressable Memory)
/// All entries searched in parallel (single cycle lookup)
#[derive(Debug, Clone)]
pub struct CamHwChirho<const SIZE: usize> {
    entries_chirho: [CamEntryChirho; SIZE],
    count_chirho: usize,
}

impl<const SIZE: usize> CamHwChirho<SIZE> {
    pub fn new_chirho() -> Self {
        Self {
            entries_chirho: [CamEntryChirho::default(); SIZE],
            count_chirho: 0,
        }
    }

    /// Insert key-value pair
    pub fn insert_chirho(&mut self, key_chirho: u32, value_chirho: u32) -> bool {
        if self.count_chirho < SIZE {
            self.entries_chirho[self.count_chirho] = CamEntryChirho {
                key_chirho,
                value_chirho,
                valid_chirho: true,
            };
            self.count_chirho += 1;
            true
        } else {
            false
        }
    }

    /// Parallel lookup: find all values matching key
    /// In hardware, this is a single-cycle operation
    pub fn lookup_chirho(&self, key_chirho: u32) -> BitVec64Chirho {
        let mut result_chirho = BitVec64Chirho::ZERO_CHIRHO;

        for entry_chirho in &self.entries_chirho {
            if entry_chirho.valid_chirho && entry_chirho.key_chirho == key_chirho {
                if entry_chirho.value_chirho < 64 {
                    result_chirho = result_chirho.set_bit_chirho(entry_chirho.value_chirho);
                }
            }
        }

        result_chirho
    }

    /// Parallel lookup with mask: find values where key matches any bit in mask
    pub fn lookup_masked_chirho(&self, key_mask_chirho: BitVec64Chirho) -> BitVec64Chirho {
        let mut result_chirho = BitVec64Chirho::ZERO_CHIRHO;

        for entry_chirho in &self.entries_chirho {
            if entry_chirho.valid_chirho && entry_chirho.key_chirho < 64 {
                if key_mask_chirho.test_bit_chirho(entry_chirho.key_chirho) {
                    if entry_chirho.value_chirho < 64 {
                        result_chirho = result_chirho.set_bit_chirho(entry_chirho.value_chirho);
                    }
                }
            }
        }

        result_chirho
    }
}

/// Parallel unification unit
/// Processes multiple variable pairs simultaneously
#[derive(Debug, Clone, Copy)]
pub struct UnifyUnitHwChirho<const NVARS: usize> {
    pub domains_chirho: [BitVec64Chirho; NVARS],
}

impl<const NVARS: usize> UnifyUnitHwChirho<NVARS> {
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: [BitVec64Chirho::ONES_CHIRHO; NVARS],
        }
    }

    /// Apply equality constraint between two variables
    /// In hardware: parallel AND of two registers
    #[inline]
    pub fn apply_eq_chirho(&mut self, v1_chirho: usize, v2_chirho: usize) -> bool {
        if v1_chirho < NVARS && v2_chirho < NVARS {
            let unified_chirho = self.domains_chirho[v1_chirho]
                .and_chirho(self.domains_chirho[v2_chirho]);
            self.domains_chirho[v1_chirho] = unified_chirho;
            self.domains_chirho[v2_chirho] = unified_chirho;
            !unified_chirho.is_zero_chirho()
        } else {
            false
        }
    }

    /// Apply multiple equality constraints in parallel
    /// Each constraint is (var1, var2)
    pub fn apply_constraints_chirho(&mut self, constraints_chirho: &[(usize, usize)]) -> bool {
        // In real hardware, these would all happen in one clock cycle
        for &(v1_chirho, v2_chirho) in constraints_chirho {
            if !self.apply_eq_chirho(v1_chirho, v2_chirho) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bitvec_basic_chirho() {
        let a_chirho = BitVec64Chirho(0b1100);
        let b_chirho = BitVec64Chirho(0b1010);

        assert_eq!(a_chirho.and_chirho(b_chirho).0, 0b1000);
        assert_eq!(a_chirho.or_chirho(b_chirho).0, 0b1110);
        assert_eq!(a_chirho.popcount_chirho(), 2);
    }

    #[test]
    fn test_bitvec_singleton_chirho() {
        let single_chirho = BitVec64Chirho(0b1000);
        assert!(single_chirho.is_singleton_chirho());
        assert_eq!(single_chirho.ctz_chirho(), 3);

        let multi_chirho = BitVec64Chirho(0b1100);
        assert!(!multi_chirho.is_singleton_chirho());
    }

    #[test]
    fn test_search_state_chirho() {
        let mut state_chirho = SearchStateHwChirho::<4>::new_chirho();

        // x in {0,1,2,3}, y in {2,3,4,5}
        state_chirho.set_domain_chirho(0, BitVec64Chirho(0b1111));   // {0,1,2,3}
        state_chirho.set_domain_chirho(1, BitVec64Chirho(0b111100)); // {2,3,4,5}

        // Unify x == y
        state_chirho.unify_chirho(0, 1);

        // Should narrow to {2,3}
        assert_eq!(state_chirho.domains_chirho[0].0, 0b1100);
        assert_eq!(state_chirho.domains_chirho[1].0, 0b1100);
        assert!(state_chirho.valid_chirho);
    }

    #[test]
    fn test_search_failure_chirho() {
        let mut state_chirho = SearchStateHwChirho::<2>::new_chirho();

        state_chirho.set_domain_chirho(0, BitVec64Chirho(0b0001)); // {0}
        state_chirho.set_domain_chirho(1, BitVec64Chirho(0b0010)); // {1}

        // x == y should fail
        state_chirho.unify_chirho(0, 1);
        assert!(!state_chirho.valid_chirho);
    }

    #[test]
    fn test_cam_lookup_chirho() {
        let mut cam_chirho = CamHwChirho::<16>::new_chirho();

        // appendo facts: key=l, value=out for l++[1]=out
        cam_chirho.insert_chirho(0, 1);  // [] ++ [1] = [1]
        cam_chirho.insert_chirho(2, 3);  // [0] ++ [1] = [0,1]

        let results_chirho = cam_chirho.lookup_chirho(0);
        assert!(results_chirho.test_bit_chirho(1));
        assert!(!results_chirho.test_bit_chirho(3));
    }

    #[test]
    fn test_fork_chirho() {
        let state_chirho = SearchStateHwChirho::<2>::new_chirho();

        // x in {0,1,2,3}
        let mut state2_chirho = state_chirho;
        state2_chirho.set_domain_chirho(0, BitVec64Chirho(0b1111));

        // Fork on bit 1: x=1 vs x≠1
        let (with_chirho, without_chirho) = state2_chirho.fork_on_bit_chirho(0, 1);

        assert_eq!(with_chirho.domains_chirho[0].0, 0b0010);     // {1}
        assert_eq!(without_chirho.domains_chirho[0].0, 0b1101);  // {0,2,3}
    }
}
