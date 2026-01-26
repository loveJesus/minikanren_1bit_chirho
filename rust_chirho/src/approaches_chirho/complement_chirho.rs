// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Complement (Cofinite) Domains ☧
//!
//! Represent "all values except these" efficiently.
//! Useful for disequality constraints (x ≠ 5).
//!
//! ```text
//! Finite({1,2,3})     = exactly these values
//! Cofinite({5})       = all integers EXCEPT 5
//! All                 = everything
//! Empty               = nothing
//! ```

use std::collections::HashSet;

/// Domain that can be finite or cofinite (complement of finite)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplementDomainChirho {
    /// All values
    AllChirho,
    /// No values
    EmptyChirho,
    /// Exactly these values
    FiniteChirho(HashSet<i64>),
    /// All values EXCEPT these
    CofiniteChirho(HashSet<i64>),
}

impl ComplementDomainChirho {
    /// All values
    pub fn all_chirho() -> Self {
        ComplementDomainChirho::AllChirho
    }

    /// No values
    pub fn empty_chirho() -> Self {
        ComplementDomainChirho::EmptyChirho
    }

    /// Singleton
    pub fn singleton_chirho(value_chirho: i64) -> Self {
        let mut set_chirho = HashSet::new();
        set_chirho.insert(value_chirho);
        ComplementDomainChirho::FiniteChirho(set_chirho)
    }

    /// All except one value (disequality)
    pub fn all_except_chirho(value_chirho: i64) -> Self {
        let mut set_chirho = HashSet::new();
        set_chirho.insert(value_chirho);
        ComplementDomainChirho::CofiniteChirho(set_chirho)
    }

    /// From finite set
    pub fn from_set_chirho(values_chirho: HashSet<i64>) -> Self {
        if values_chirho.is_empty() {
            ComplementDomainChirho::EmptyChirho
        } else {
            ComplementDomainChirho::FiniteChirho(values_chirho)
        }
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: i64) -> bool {
        match self {
            ComplementDomainChirho::AllChirho => true,
            ComplementDomainChirho::EmptyChirho => false,
            ComplementDomainChirho::FiniteChirho(set) => set.contains(&value_chirho),
            ComplementDomainChirho::CofiniteChirho(excluded) => !excluded.contains(&value_chirho),
        }
    }

    /// Intersect two domains
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        use ComplementDomainChirho::*;

        match (self, other_chirho) {
            // Identity cases
            (AllChirho, x) | (x, AllChirho) => x.clone(),
            (EmptyChirho, _) | (_, EmptyChirho) => EmptyChirho,

            // Finite ∩ Finite = intersection of sets
            (FiniteChirho(a), FiniteChirho(b)) => {
                let intersection_chirho: HashSet<_> = a.intersection(b).copied().collect();
                if intersection_chirho.is_empty() {
                    EmptyChirho
                } else {
                    FiniteChirho(intersection_chirho)
                }
            }

            // Finite ∩ Cofinite = Finite - excluded
            (FiniteChirho(finite), CofiniteChirho(excluded)) |
            (CofiniteChirho(excluded), FiniteChirho(finite)) => {
                let result_chirho: HashSet<_> = finite.difference(excluded).copied().collect();
                if result_chirho.is_empty() {
                    EmptyChirho
                } else {
                    FiniteChirho(result_chirho)
                }
            }

            // Cofinite ∩ Cofinite = Cofinite with union of exclusions
            (CofiniteChirho(a), CofiniteChirho(b)) => {
                let union_chirho: HashSet<_> = a.union(b).copied().collect();
                CofiniteChirho(union_chirho)
            }
        }
    }

    /// Union of two domains
    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        use ComplementDomainChirho::*;

        match (self, other_chirho) {
            // Identity cases
            (EmptyChirho, x) | (x, EmptyChirho) => x.clone(),
            (AllChirho, _) | (_, AllChirho) => AllChirho,

            // Finite ∪ Finite = union of sets
            (FiniteChirho(a), FiniteChirho(b)) => {
                let union_chirho: HashSet<_> = a.union(b).copied().collect();
                FiniteChirho(union_chirho)
            }

            // Finite ∪ Cofinite = Cofinite - finite (more excluded become included)
            (FiniteChirho(finite), CofiniteChirho(excluded)) |
            (CofiniteChirho(excluded), FiniteChirho(finite)) => {
                let new_excluded_chirho: HashSet<_> = excluded.difference(finite).copied().collect();
                if new_excluded_chirho.is_empty() {
                    AllChirho
                } else {
                    CofiniteChirho(new_excluded_chirho)
                }
            }

            // Cofinite ∪ Cofinite = Cofinite with intersection of exclusions
            (CofiniteChirho(a), CofiniteChirho(b)) => {
                let intersection_chirho: HashSet<_> = a.intersection(b).copied().collect();
                if intersection_chirho.is_empty() {
                    AllChirho
                } else {
                    CofiniteChirho(intersection_chirho)
                }
            }
        }
    }

    /// Complement (negate) the domain
    pub fn complement_chirho(&self) -> Self {
        match self {
            ComplementDomainChirho::AllChirho => ComplementDomainChirho::EmptyChirho,
            ComplementDomainChirho::EmptyChirho => ComplementDomainChirho::AllChirho,
            ComplementDomainChirho::FiniteChirho(set) => {
                ComplementDomainChirho::CofiniteChirho(set.clone())
            }
            ComplementDomainChirho::CofiniteChirho(set) => {
                ComplementDomainChirho::FiniteChirho(set.clone())
            }
        }
    }

    /// Check if definitely empty
    pub fn is_empty_chirho(&self) -> bool {
        matches!(self, ComplementDomainChirho::EmptyChirho)
    }

    /// Check if infinite
    pub fn is_infinite_chirho(&self) -> bool {
        matches!(self, ComplementDomainChirho::AllChirho | ComplementDomainChirho::CofiniteChirho(_))
    }

    /// Check if finite
    pub fn is_finite_chirho(&self) -> bool {
        matches!(self, ComplementDomainChirho::EmptyChirho | ComplementDomainChirho::FiniteChirho(_))
    }

    /// Get size if finite
    pub fn size_chirho(&self) -> Option<usize> {
        match self {
            ComplementDomainChirho::EmptyChirho => Some(0),
            ComplementDomainChirho::FiniteChirho(set) => Some(set.len()),
            _ => None, // Infinite
        }
    }

    /// Iterate values if finite (returns boxed iterator for type uniformity)
    pub fn iter_chirho(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        match self {
            ComplementDomainChirho::FiniteChirho(set) => {
                Some(Box::new(set.iter().copied()))
            }
            ComplementDomainChirho::EmptyChirho => {
                Some(Box::new(std::iter::empty()))
            }
            _ => None, // Can't iterate infinite
        }
    }
}

impl Default for ComplementDomainChirho {
    fn default() -> Self {
        ComplementDomainChirho::AllChirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_disequality_chirho() {
        // x ≠ 5
        let domain_chirho = ComplementDomainChirho::all_except_chirho(5);

        assert!(domain_chirho.contains_chirho(0));
        assert!(domain_chirho.contains_chirho(4));
        assert!(!domain_chirho.contains_chirho(5));
        assert!(domain_chirho.contains_chirho(6));
        assert!(domain_chirho.contains_chirho(1000));
    }

    #[test]
    fn test_finite_intersect_cofinite_chirho() {
        // {1,2,3,4,5} ∩ (all except 3) = {1,2,4,5}
        let finite_chirho = ComplementDomainChirho::from_set_chirho(
            [1, 2, 3, 4, 5].into_iter().collect()
        );
        let cofinite_chirho = ComplementDomainChirho::all_except_chirho(3);

        let result_chirho = finite_chirho.intersect_chirho(&cofinite_chirho);

        assert!(result_chirho.contains_chirho(1));
        assert!(result_chirho.contains_chirho(2));
        assert!(!result_chirho.contains_chirho(3));
        assert!(result_chirho.contains_chirho(4));
        assert!(result_chirho.contains_chirho(5));
        assert!(!result_chirho.contains_chirho(6)); // Not in original finite set
    }

    #[test]
    fn test_multiple_disequalities_chirho() {
        // (all except 1) ∩ (all except 2) = all except {1, 2}
        let a_chirho = ComplementDomainChirho::all_except_chirho(1);
        let b_chirho = ComplementDomainChirho::all_except_chirho(2);
        let result_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert!(!result_chirho.contains_chirho(1));
        assert!(!result_chirho.contains_chirho(2));
        assert!(result_chirho.contains_chirho(0));
        assert!(result_chirho.contains_chirho(3));
    }

    #[test]
    fn test_complement_chirho() {
        let finite_chirho = ComplementDomainChirho::from_set_chirho([1, 2, 3].into_iter().collect());
        let complement_chirho = finite_chirho.complement_chirho();

        // Complement of {1,2,3} is "all except {1,2,3}"
        assert!(!complement_chirho.contains_chirho(1));
        assert!(!complement_chirho.contains_chirho(2));
        assert!(!complement_chirho.contains_chirho(3));
        assert!(complement_chirho.contains_chirho(0));
        assert!(complement_chirho.contains_chirho(100));
    }
}
