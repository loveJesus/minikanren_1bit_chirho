//! Symbolic Domain Constraints ☧
//!
//! Represent infinite domains symbolically using algebraic constraints.
//! Intersection computed via constraint solving, not enumeration.
//!
//! ```text
//! Range[10, 100] ∩ Range[50, 200] = Range[50, 100]
//! Mod(2, 5) ∩ Mod(3, 7) = Mod(17, 35)  // Chinese Remainder Theorem
//! Range[0, 100] ∩ Mod(0, 10) → {0, 10, 20, ..., 100}  // Materialize
//! ```

use crate::hardware_chirho::BitVec64Chirho;

/// Symbolic constraint on a domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicConstraintChirho {
    /// All values (unconstrained)
    AnyChirho,

    /// No values (contradiction)
    NoneChirho,

    /// Single value
    ExactChirho(i64),

    /// Range: lo <= x <= hi
    RangeChirho { lo_chirho: i64, hi_chirho: i64 },

    /// Modular: x ≡ remainder (mod modulus)
    /// Represents infinite set {..., r-m, r, r+m, r+2m, ...}
    ModularChirho { remainder_chirho: i64, modulus_chirho: i64 },

    /// Not equal to specific value
    NotEqualChirho(i64),

    /// Multiple constraints (all must hold)
    AndChirho(Vec<SymbolicConstraintChirho>),

    /// Type tag constraint
    TypeChirho(TypeTagChirho),
}

/// Type tags for term classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeTagChirho {
    IntChirho,
    BoolChirho,
    SymbolChirho,
    ListChirho,
    PairChirho,
    NilChirho,
}

/// Symbolic domain with lazy intersection
#[derive(Debug, Clone)]
pub struct SymbolicDomainChirho {
    pub constraint_chirho: SymbolicConstraintChirho,
}

impl SymbolicDomainChirho {
    /// Unconstrained domain (any value)
    pub fn any_chirho() -> Self {
        Self { constraint_chirho: SymbolicConstraintChirho::AnyChirho }
    }

    /// Empty domain
    pub fn none_chirho() -> Self {
        Self { constraint_chirho: SymbolicConstraintChirho::NoneChirho }
    }

    /// Single value
    pub fn exact_chirho(value_chirho: i64) -> Self {
        Self { constraint_chirho: SymbolicConstraintChirho::ExactChirho(value_chirho) }
    }

    /// Range constraint
    pub fn range_chirho(lo_chirho: i64, hi_chirho: i64) -> Self {
        if lo_chirho > hi_chirho {
            Self::none_chirho()
        } else if lo_chirho == hi_chirho {
            Self::exact_chirho(lo_chirho)
        } else {
            Self {
                constraint_chirho: SymbolicConstraintChirho::RangeChirho { lo_chirho, hi_chirho },
            }
        }
    }

    /// Modular constraint: x ≡ r (mod m)
    pub fn modular_chirho(remainder_chirho: i64, modulus_chirho: i64) -> Self {
        assert!(modulus_chirho > 0, "Modulus must be positive");
        let r_chirho = remainder_chirho.rem_euclid(modulus_chirho);
        Self {
            constraint_chirho: SymbolicConstraintChirho::ModularChirho {
                remainder_chirho: r_chirho,
                modulus_chirho,
            },
        }
    }

    /// Intersect two symbolic domains
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        use SymbolicConstraintChirho::*;

        let result_chirho = match (&self.constraint_chirho, &other_chirho.constraint_chirho) {
            // Identity cases
            (AnyChirho, x) | (x, AnyChirho) => x.clone(),
            (NoneChirho, _) | (_, NoneChirho) => NoneChirho,

            // Exact value
            (ExactChirho(a), ExactChirho(b)) => {
                if a == b { ExactChirho(*a) } else { NoneChirho }
            }
            (ExactChirho(v), other) | (other, ExactChirho(v)) => {
                if (Self { constraint_chirho: other.clone() }).contains_chirho(*v) {
                    ExactChirho(*v)
                } else {
                    NoneChirho
                }
            }

            // Range intersection
            (RangeChirho { lo_chirho: lo1, hi_chirho: hi1 },
             RangeChirho { lo_chirho: lo2, hi_chirho: hi2 }) => {
                let new_lo_chirho = (*lo1).max(*lo2);
                let new_hi_chirho = (*hi1).min(*hi2);
                if new_lo_chirho > new_hi_chirho {
                    NoneChirho
                } else if new_lo_chirho == new_hi_chirho {
                    ExactChirho(new_lo_chirho)
                } else {
                    RangeChirho { lo_chirho: new_lo_chirho, hi_chirho: new_hi_chirho }
                }
            }

            // Modular intersection (Chinese Remainder Theorem)
            (ModularChirho { remainder_chirho: r1, modulus_chirho: m1 },
             ModularChirho { remainder_chirho: r2, modulus_chirho: m2 }) => {
                match chinese_remainder_chirho(*r1, *m1, *r2, *m2) {
                    Some((r, m)) => ModularChirho { remainder_chirho: r, modulus_chirho: m },
                    None => NoneChirho, // No solution
                }
            }

            // Range ∩ Modular: may need to enumerate or keep symbolic
            (RangeChirho { lo_chirho, hi_chirho },
             ModularChirho { remainder_chirho, modulus_chirho }) |
            (ModularChirho { remainder_chirho, modulus_chirho },
             RangeChirho { lo_chirho, hi_chirho }) => {
                // Keep as conjunction for now
                AndChirho(vec![
                    RangeChirho { lo_chirho: *lo_chirho, hi_chirho: *hi_chirho },
                    ModularChirho { remainder_chirho: *remainder_chirho, modulus_chirho: *modulus_chirho },
                ])
            }

            // NotEqual intersection
            (NotEqualChirho(v), other) | (other, NotEqualChirho(v)) => {
                AndChirho(vec![other.clone(), NotEqualChirho(*v)])
            }

            // And with anything: flatten and merge
            (AndChirho(constraints), other) | (other, AndChirho(constraints)) => {
                let mut new_constraints_chirho = constraints.clone();
                new_constraints_chirho.push(other.clone());
                // TODO: Simplify by pairwise intersection
                AndChirho(new_constraints_chirho)
            }

            // Type constraints
            (TypeChirho(t1), TypeChirho(t2)) => {
                if t1 == t2 { TypeChirho(*t1) } else { NoneChirho }
            }
            (TypeChirho(t), other) | (other, TypeChirho(t)) => {
                AndChirho(vec![TypeChirho(*t), other.clone()])
            }
        };

        Self { constraint_chirho: result_chirho }
    }

    /// Check if a specific value satisfies the constraint
    pub fn contains_chirho(&self, value_chirho: i64) -> bool {
        use SymbolicConstraintChirho::*;

        match &self.constraint_chirho {
            AnyChirho => true,
            NoneChirho => false,
            ExactChirho(v) => *v == value_chirho,
            RangeChirho { lo_chirho, hi_chirho } => {
                value_chirho >= *lo_chirho && value_chirho <= *hi_chirho
            }
            ModularChirho { remainder_chirho, modulus_chirho } => {
                value_chirho.rem_euclid(*modulus_chirho) == *remainder_chirho
            }
            NotEqualChirho(v) => *v != value_chirho,
            AndChirho(constraints) => {
                constraints.iter().all(|c| {
                    Self { constraint_chirho: c.clone() }.contains_chirho(value_chirho)
                })
            }
            TypeChirho(_) => true, // Type constraint doesn't restrict integer values
        }
    }

    /// Try to enumerate domain into BitVec64 (if finite and small enough)
    pub fn try_enumerate_chirho(&self, max_values_chirho: usize) -> Option<BitVec64Chirho> {
        use SymbolicConstraintChirho::*;

        match &self.constraint_chirho {
            NoneChirho => Some(BitVec64Chirho::ZERO_CHIRHO),
            ExactChirho(v) if *v >= 0 && *v < 64 => {
                Some(BitVec64Chirho(1u64 << *v))
            }
            RangeChirho { lo_chirho, hi_chirho }
                if *lo_chirho >= 0 && *hi_chirho < 64 =>
            {
                let count_chirho = (hi_chirho - lo_chirho + 1) as usize;
                if count_chirho <= max_values_chirho {
                    let mask_chirho = ((1u64 << count_chirho) - 1) << *lo_chirho;
                    Some(BitVec64Chirho(mask_chirho))
                } else {
                    None
                }
            }
            AndChirho(constraints) => {
                // Try to enumerate if there's a small range
                if let Some(range) = constraints.iter().find_map(|c| {
                    if let RangeChirho { lo_chirho, hi_chirho } = c {
                        if *lo_chirho >= 0 && *hi_chirho < 64 {
                            return Some((*lo_chirho, *hi_chirho));
                        }
                    }
                    None
                }) {
                    let (lo, hi) = range;
                    let mut bits_chirho = 0u64;
                    for v in lo..=hi {
                        if self.contains_chirho(v) {
                            bits_chirho |= 1u64 << v;
                        }
                    }
                    Some(BitVec64Chirho(bits_chirho))
                } else {
                    None
                }
            }
            _ => None, // Can't enumerate infinite domains
        }
    }

    /// Check if domain is definitely empty
    pub fn is_empty_chirho(&self) -> bool {
        matches!(self.constraint_chirho, SymbolicConstraintChirho::NoneChirho)
    }

    /// Check if domain is definitely a singleton
    pub fn is_singleton_chirho(&self) -> Option<i64> {
        if let SymbolicConstraintChirho::ExactChirho(v) = self.constraint_chirho {
            Some(v)
        } else {
            None
        }
    }
}

/// Chinese Remainder Theorem: solve x ≡ r1 (mod m1) AND x ≡ r2 (mod m2)
/// Returns (remainder, modulus) of combined constraint, or None if no solution
fn chinese_remainder_chirho(r1: i64, m1: i64, r2: i64, m2: i64) -> Option<(i64, i64)> {
    let (g, x, _) = extended_gcd_chirho(m1, m2);

    // Check if solution exists
    if (r2 - r1) % g != 0 {
        return None;
    }

    let lcm_chirho = (m1 / g) * m2;
    let solution_chirho = (r1 + m1 * ((r2 - r1) / g) * x).rem_euclid(lcm_chirho);

    Some((solution_chirho, lcm_chirho))
}

/// Extended Euclidean algorithm: returns (gcd, x, y) where gcd = a*x + b*y
fn extended_gcd_chirho(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = extended_gcd_chirho(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_range_intersect_chirho() {
        let a_chirho = SymbolicDomainChirho::range_chirho(10, 100);
        let b_chirho = SymbolicDomainChirho::range_chirho(50, 200);
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert!(c_chirho.contains_chirho(50));
        assert!(c_chirho.contains_chirho(100));
        assert!(!c_chirho.contains_chirho(9));
        assert!(!c_chirho.contains_chirho(101));
    }

    #[test]
    fn test_modular_chirho() {
        // x ≡ 2 (mod 5) means {..., -3, 2, 7, 12, 17, ...}
        let domain_chirho = SymbolicDomainChirho::modular_chirho(2, 5);

        assert!(domain_chirho.contains_chirho(2));
        assert!(domain_chirho.contains_chirho(7));
        assert!(domain_chirho.contains_chirho(12));
        assert!(domain_chirho.contains_chirho(-3));
        assert!(!domain_chirho.contains_chirho(3));
    }

    #[test]
    fn test_chinese_remainder_chirho() {
        // x ≡ 2 (mod 5) AND x ≡ 3 (mod 7)
        // Solution: x ≡ 17 (mod 35)
        let a_chirho = SymbolicDomainChirho::modular_chirho(2, 5);
        let b_chirho = SymbolicDomainChirho::modular_chirho(3, 7);
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert!(c_chirho.contains_chirho(17));
        assert!(c_chirho.contains_chirho(52));  // 17 + 35
        assert!(!c_chirho.contains_chirho(2));
        assert!(!c_chirho.contains_chirho(3));
    }

    #[test]
    fn test_enumerate_range_chirho() {
        let domain_chirho = SymbolicDomainChirho::range_chirho(5, 10);
        let bits_chirho = domain_chirho.try_enumerate_chirho(64).unwrap();

        assert!(bits_chirho.test_bit_chirho(5));
        assert!(bits_chirho.test_bit_chirho(10));
        assert!(!bits_chirho.test_bit_chirho(4));
        assert!(!bits_chirho.test_bit_chirho(11));
    }
}
