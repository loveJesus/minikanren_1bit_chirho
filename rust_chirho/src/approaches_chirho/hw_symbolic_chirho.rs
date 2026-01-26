// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Hardware-Accelerated Symbolic Domains ☧
//!
//! Key insight: Some symbolic operations have efficient hardware implementations.
//!
//! ```text
//! Operation           | Software        | Hardware
//! --------------------|-----------------|---------------------------
//! Range [a,b]         | loop + compare  | Comparator circuit
//! Mod(r, m)           | division        | Lookup table / counter
//! CRT intersection    | ExtGCD          | Systolic array
//! Materialize range   | loop            | Parallel bit-gen
//! ```
//!
//! ## FPGA-Friendly Symbolic Ops
//!
//! 1. **Range bounds**: Two comparators (lo ≤ x ≤ hi)
//! 2. **Modular**: Counter mod m, compare to r
//! 3. **Enumeration**: Parallel generate all bits in range
//! 4. **CRT**: Lookup table for small moduli, systolic for large

use crate::hardware_chirho::BitVec64Chirho;

/// Symbolic operation that can be hardware-accelerated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolicOpChirho {
    /// Value in range [lo, hi]
    /// Hardware: 2 comparators, 1 AND gate
    RangeChirho { lo_chirho: u32, hi_chirho: u32 },

    /// Value ≡ r (mod m)
    /// Hardware: Counter + comparator, or lookup table
    ModularChirho { remainder_chirho: u32, modulus_chirho: u32 },

    /// Value ≠ x
    /// Hardware: XOR + OR reduction (all bits differ)
    NotEqualChirho { value_chirho: u32 },

    /// Value ≤ x
    /// Hardware: Single comparator
    LessEqualChirho { bound_chirho: u32 },

    /// Value ≥ x
    /// Hardware: Single comparator
    GreaterEqualChirho { bound_chirho: u32 },
}

/// Hardware-accelerated symbolic domain
#[derive(Debug, Clone)]
pub struct HwSymbolicDomainChirho {
    /// Chain of constraints (all must hold)
    pub constraints_chirho: Vec<SymbolicOpChirho>,

    /// Cached materialization (if computed)
    cached_bits_chirho: Option<BitVec64Chirho>,

    /// Universe size (for enumeration)
    universe_chirho: u32,
}

impl HwSymbolicDomainChirho {
    /// Create domain over [0, universe)
    pub fn new_chirho(universe_chirho: u32) -> Self {
        Self {
            constraints_chirho: Vec::new(),
            cached_bits_chirho: None,
            universe_chirho,
        }
    }

    /// Add range constraint
    pub fn with_range_chirho(mut self, lo_chirho: u32, hi_chirho: u32) -> Self {
        self.constraints_chirho.push(SymbolicOpChirho::RangeChirho { lo_chirho, hi_chirho });
        self.cached_bits_chirho = None;
        self
    }

    /// Add modular constraint
    pub fn with_modular_chirho(mut self, remainder_chirho: u32, modulus_chirho: u32) -> Self {
        self.constraints_chirho.push(SymbolicOpChirho::ModularChirho {
            remainder_chirho: remainder_chirho % modulus_chirho,
            modulus_chirho,
        });
        self.cached_bits_chirho = None;
        self
    }

    /// Add disequality constraint
    pub fn with_not_equal_chirho(mut self, value_chirho: u32) -> Self {
        self.constraints_chirho.push(SymbolicOpChirho::NotEqualChirho { value_chirho });
        self.cached_bits_chirho = None;
        self
    }

    /// Check if value satisfies all constraints
    /// This simulates what hardware would compute in parallel
    pub fn contains_hw_chirho(&self, value_chirho: u32) -> bool {
        if value_chirho >= self.universe_chirho {
            return false;
        }

        for constraint_chirho in &self.constraints_chirho {
            let satisfied_chirho = match constraint_chirho {
                SymbolicOpChirho::RangeChirho { lo_chirho, hi_chirho } => {
                    value_chirho >= *lo_chirho && value_chirho <= *hi_chirho
                }
                SymbolicOpChirho::ModularChirho { remainder_chirho, modulus_chirho } => {
                    value_chirho % modulus_chirho == *remainder_chirho
                }
                SymbolicOpChirho::NotEqualChirho { value_chirho: v } => {
                    value_chirho != *v
                }
                SymbolicOpChirho::LessEqualChirho { bound_chirho } => {
                    value_chirho <= *bound_chirho
                }
                SymbolicOpChirho::GreaterEqualChirho { bound_chirho } => {
                    value_chirho >= *bound_chirho
                }
            };

            if !satisfied_chirho {
                return false;
            }
        }

        true
    }

    /// Materialize to BitVec64 using hardware-style parallel evaluation
    ///
    /// In real hardware, this would be:
    /// - 64 parallel constraint checkers
    /// - Each produces 1 bit
    /// - Combined into BitVec64 in single cycle
    pub fn materialize_hw_chirho(&mut self) -> BitVec64Chirho {
        if let Some(cached) = self.cached_bits_chirho {
            return cached;
        }

        let max_val_chirho = self.universe_chirho.min(64);
        let mut bits_chirho = 0u64;

        // In hardware: 64 parallel units, each checking constraints
        // Here: simulate with loop (but structure maps to hardware)
        for value_chirho in 0..max_val_chirho {
            if self.contains_hw_chirho(value_chirho) {
                bits_chirho |= 1u64 << value_chirho;
            }
        }

        let result_chirho = BitVec64Chirho(bits_chirho);
        self.cached_bits_chirho = Some(result_chirho);
        result_chirho
    }

    /// Intersect two symbolic domains
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self {
            constraints_chirho: self.constraints_chirho.clone(),
            cached_bits_chirho: None,
            universe_chirho: self.universe_chirho.min(other_chirho.universe_chirho),
        };

        // Add other's constraints (hardware: just wire both constraint sets)
        result_chirho.constraints_chirho.extend(other_chirho.constraints_chirho.iter().cloned());

        // Simplify: merge overlapping ranges
        result_chirho.simplify_chirho();

        result_chirho
    }

    /// Simplify constraint set (hardware: optimized at synthesis time)
    fn simplify_chirho(&mut self) {
        // Find tightest range bounds
        let mut lo_chirho = 0u32;
        let mut hi_chirho = self.universe_chirho.saturating_sub(1);

        let mut remaining_chirho = Vec::new();

        for constraint_chirho in &self.constraints_chirho {
            match constraint_chirho {
                SymbolicOpChirho::RangeChirho { lo_chirho: l, hi_chirho: h } => {
                    lo_chirho = lo_chirho.max(*l);
                    hi_chirho = hi_chirho.min(*h);
                }
                SymbolicOpChirho::LessEqualChirho { bound_chirho } => {
                    hi_chirho = hi_chirho.min(*bound_chirho);
                }
                SymbolicOpChirho::GreaterEqualChirho { bound_chirho } => {
                    lo_chirho = lo_chirho.max(*bound_chirho);
                }
                other => {
                    remaining_chirho.push(*other);
                }
            }
        }

        self.constraints_chirho = vec![SymbolicOpChirho::RangeChirho { lo_chirho, hi_chirho }];
        self.constraints_chirho.extend(remaining_chirho);
    }

    /// Estimate hardware cost (LUTs) for this domain
    pub fn hw_cost_chirho(&self) -> u32 {
        let mut cost_chirho = 0u32;

        for constraint_chirho in &self.constraints_chirho {
            cost_chirho += match constraint_chirho {
                SymbolicOpChirho::RangeChirho { .. } => 12, // 2 comparators + AND
                SymbolicOpChirho::ModularChirho { modulus_chirho, .. } => {
                    // Small moduli use LUT, large use divider
                    if *modulus_chirho <= 8 { 4 } else { 32 }
                }
                SymbolicOpChirho::NotEqualChirho { .. } => 8, // XOR + OR tree
                SymbolicOpChirho::LessEqualChirho { .. } => 6, // 1 comparator
                SymbolicOpChirho::GreaterEqualChirho { .. } => 6, // 1 comparator
            };
        }

        // Parallel 64-bit output generation
        cost_chirho += 64 * (self.constraints_chirho.len() as u32);

        cost_chirho
    }
}

/// Hardware lookup table for modular arithmetic
/// Pre-computed: which values in [0, 64) satisfy x ≡ r (mod m)
pub fn mod_lut_chirho(remainder_chirho: u32, modulus_chirho: u32) -> BitVec64Chirho {
    let mut bits_chirho = 0u64;
    let mut val_chirho = remainder_chirho;

    while val_chirho < 64 {
        bits_chirho |= 1u64 << val_chirho;
        val_chirho += modulus_chirho;
    }

    BitVec64Chirho(bits_chirho)
}

/// Hardware range materializer
/// Generate all bits in [lo, hi] ∩ [0, 64)
pub fn range_bits_chirho(lo_chirho: u32, hi_chirho: u32) -> BitVec64Chirho {
    if lo_chirho > hi_chirho || lo_chirho >= 64 {
        return BitVec64Chirho::ZERO_CHIRHO;
    }

    let effective_lo_chirho = lo_chirho;
    let effective_hi_chirho = hi_chirho.min(63);

    let count_chirho = effective_hi_chirho - effective_lo_chirho + 1;
    let mask_chirho = if count_chirho >= 64 {
        u64::MAX
    } else {
        ((1u64 << count_chirho) - 1) << effective_lo_chirho
    };

    BitVec64Chirho(mask_chirho)
}

/// Combined hardware operation: Range AND Modular
/// Efficient: generate mod bits, then mask by range
pub fn range_mod_hw_chirho(lo_chirho: u32, hi_chirho: u32, r_chirho: u32, m_chirho: u32) -> BitVec64Chirho {
    let range_bits_chirho_val = range_bits_chirho(lo_chirho, hi_chirho);
    let mod_bits_chirho = mod_lut_chirho(r_chirho, m_chirho);
    range_bits_chirho_val.and_chirho(mod_bits_chirho)
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_range_materialize_chirho() {
        let domain_chirho = HwSymbolicDomainChirho::new_chirho(64)
            .with_range_chirho(10, 20);

        let mut domain_mut_chirho = domain_chirho;
        let bits_chirho = domain_mut_chirho.materialize_hw_chirho();

        assert!(!bits_chirho.test_bit_chirho(9));
        assert!(bits_chirho.test_bit_chirho(10));
        assert!(bits_chirho.test_bit_chirho(15));
        assert!(bits_chirho.test_bit_chirho(20));
        assert!(!bits_chirho.test_bit_chirho(21));
    }

    #[test]
    fn test_mod_lut_chirho() {
        // Even numbers: x ≡ 0 (mod 2)
        let evens_chirho = mod_lut_chirho(0, 2);
        assert!(evens_chirho.test_bit_chirho(0));
        assert!(!evens_chirho.test_bit_chirho(1));
        assert!(evens_chirho.test_bit_chirho(2));
        assert!(evens_chirho.test_bit_chirho(62));
        assert!(!evens_chirho.test_bit_chirho(63));
    }

    #[test]
    fn test_range_mod_combined_chirho() {
        // [0, 20] AND x ≡ 0 (mod 5) = {0, 5, 10, 15, 20}
        let bits_chirho = range_mod_hw_chirho(0, 20, 0, 5);

        assert!(bits_chirho.test_bit_chirho(0));
        assert!(!bits_chirho.test_bit_chirho(1));
        assert!(bits_chirho.test_bit_chirho(5));
        assert!(bits_chirho.test_bit_chirho(10));
        assert!(bits_chirho.test_bit_chirho(15));
        assert!(bits_chirho.test_bit_chirho(20));
        assert!(!bits_chirho.test_bit_chirho(25)); // Out of range
    }

    #[test]
    fn test_symbolic_intersect_chirho() {
        let a_chirho = HwSymbolicDomainChirho::new_chirho(64)
            .with_range_chirho(0, 30);

        let b_chirho = HwSymbolicDomainChirho::new_chirho(64)
            .with_modular_chirho(0, 7); // Multiples of 7

        let mut c_chirho = a_chirho.intersect_chirho(&b_chirho);
        let bits_chirho = c_chirho.materialize_hw_chirho();

        // Should be {0, 7, 14, 21, 28}
        assert!(bits_chirho.test_bit_chirho(0));
        assert!(bits_chirho.test_bit_chirho(7));
        assert!(bits_chirho.test_bit_chirho(14));
        assert!(bits_chirho.test_bit_chirho(21));
        assert!(bits_chirho.test_bit_chirho(28));
        assert!(!bits_chirho.test_bit_chirho(35)); // Out of range
        assert_eq!(bits_chirho.popcount_chirho(), 5);
    }

    #[test]
    fn test_hw_cost_chirho() {
        let simple_chirho = HwSymbolicDomainChirho::new_chirho(64)
            .with_range_chirho(0, 10);

        let complex_chirho = HwSymbolicDomainChirho::new_chirho(64)
            .with_range_chirho(0, 50)
            .with_modular_chirho(0, 3)
            .with_not_equal_chirho(15);

        assert!(complex_chirho.hw_cost_chirho() > simple_chirho.hw_cost_chirho());
    }
}
