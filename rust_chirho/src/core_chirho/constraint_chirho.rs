//! Constraint Propagation ☧
//!
//! Arc consistency for domain pruning.
//! Each variable has a bitmask domain; constraints prune impossible values.

use std::collections::{HashMap, HashSet, VecDeque};

/// Domain as a bitmask (up to 64 values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainChirho(pub u64);

impl DomainChirho {
    /// Full domain (all bits set)
    pub fn full_chirho() -> Self {
        DomainChirho(u64::MAX)
    }

    /// Empty domain (no values possible)
    pub fn empty_chirho() -> Self {
        DomainChirho(0)
    }

    /// Singleton domain
    pub fn singleton_chirho(val_chirho: u32) -> Self {
        DomainChirho(1u64 << val_chirho)
    }

    /// Domain from range [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        if n_chirho >= 64 {
            DomainChirho(u64::MAX)
        } else {
            DomainChirho((1u64 << n_chirho) - 1)
        }
    }

    /// Intersection (AND)
    pub fn intersect_chirho(self, other_chirho: DomainChirho) -> DomainChirho {
        DomainChirho(self.0 & other_chirho.0)
    }

    /// Union (OR)
    pub fn union_chirho(self, other_chirho: DomainChirho) -> DomainChirho {
        DomainChirho(self.0 | other_chirho.0)
    }

    /// Check if empty
    pub fn is_empty_chirho(self) -> bool {
        self.0 == 0
    }

    /// Check if singleton
    pub fn is_singleton_chirho(self) -> bool {
        self.0 != 0 && (self.0 & (self.0 - 1)) == 0
    }

    /// Get singleton value (if singleton)
    pub fn get_singleton_chirho(self) -> Option<u32> {
        if self.is_singleton_chirho() {
            Some(self.0.trailing_zeros())
        } else {
            None
        }
    }

    /// Count possible values
    pub fn count_chirho(self) -> u32 {
        self.0.count_ones()
    }

    /// Check if value is in domain
    pub fn contains_chirho(self, val_chirho: u32) -> bool {
        if val_chirho >= 64 {
            false
        } else {
            (self.0 & (1u64 << val_chirho)) != 0
        }
    }

    /// Remove value from domain
    pub fn remove_chirho(self, val_chirho: u32) -> DomainChirho {
        if val_chirho >= 64 {
            self
        } else {
            DomainChirho(self.0 & !(1u64 << val_chirho))
        }
    }

    /// Iterate over values in domain
    pub fn iter_chirho(self) -> impl Iterator<Item = u32> {
        (0..64).filter(move |&i| self.contains_chirho(i))
    }
}

/// Variable ID
pub type VarIdChirho = u32;

/// Binary constraint: restricts pairs of values
#[derive(Debug, Clone)]
pub struct BinaryConstraintChirho {
    pub var1_chirho: VarIdChirho,
    pub var2_chirho: VarIdChirho,
    /// Allowed pairs as sparse set of (v1, v2)
    pub allowed_chirho: HashSet<(u32, u32)>,
}

impl BinaryConstraintChirho {
    pub fn new(var1_chirho: VarIdChirho, var2_chirho: VarIdChirho) -> Self {
        Self {
            var1_chirho,
            var2_chirho,
            allowed_chirho: HashSet::new(),
        }
    }

    /// Add allowed pair
    pub fn allow_chirho(&mut self, v1_chirho: u32, v2_chirho: u32) {
        self.allowed_chirho.insert((v1_chirho, v2_chirho));
    }

    /// Equality constraint
    pub fn equality_chirho(var1_chirho: VarIdChirho, var2_chirho: VarIdChirho, max_val_chirho: u32) -> Self {
        let mut c_chirho = Self::new(var1_chirho, var2_chirho);
        for v_chirho in 0..max_val_chirho {
            c_chirho.allow_chirho(v_chirho, v_chirho);
        }
        c_chirho
    }

    /// Not-equal constraint
    pub fn not_equal_chirho(var1_chirho: VarIdChirho, var2_chirho: VarIdChirho, max_val_chirho: u32) -> Self {
        let mut c_chirho = Self::new(var1_chirho, var2_chirho);
        for v1_chirho in 0..max_val_chirho {
            for v2_chirho in 0..max_val_chirho {
                if v1_chirho != v2_chirho {
                    c_chirho.allow_chirho(v1_chirho, v2_chirho);
                }
            }
        }
        c_chirho
    }
}

/// Constraint store with domains
#[derive(Debug, Clone)]
pub struct ConstraintStoreChirho {
    /// Variable domains
    pub domains_chirho: HashMap<VarIdChirho, DomainChirho>,
    /// Binary constraints indexed by variable
    pub constraints_chirho: Vec<BinaryConstraintChirho>,
    /// Constraint index: var -> constraint indices
    constraint_index_chirho: HashMap<VarIdChirho, Vec<usize>>,
}

impl ConstraintStoreChirho {
    pub fn new() -> Self {
        Self {
            domains_chirho: HashMap::new(),
            constraints_chirho: Vec::new(),
            constraint_index_chirho: HashMap::new(),
        }
    }

    /// Add variable with domain
    pub fn add_var_chirho(&mut self, var_chirho: VarIdChirho, domain_chirho: DomainChirho) {
        self.domains_chirho.insert(var_chirho, domain_chirho);
    }

    /// Add constraint
    pub fn add_constraint_chirho(&mut self, constraint_chirho: BinaryConstraintChirho) {
        let idx_chirho = self.constraints_chirho.len();
        self.constraint_index_chirho
            .entry(constraint_chirho.var1_chirho)
            .or_default()
            .push(idx_chirho);
        self.constraint_index_chirho
            .entry(constraint_chirho.var2_chirho)
            .or_default()
            .push(idx_chirho);
        self.constraints_chirho.push(constraint_chirho);
    }

    /// Get domain
    pub fn domain_chirho(&self, var_chirho: VarIdChirho) -> DomainChirho {
        self.domains_chirho.get(&var_chirho).copied().unwrap_or(DomainChirho::full_chirho())
    }

    /// Set domain (returns false if became empty)
    pub fn set_domain_chirho(&mut self, var_chirho: VarIdChirho, domain_chirho: DomainChirho) -> bool {
        self.domains_chirho.insert(var_chirho, domain_chirho);
        !domain_chirho.is_empty_chirho()
    }

    /// Arc consistency (AC-3 algorithm)
    /// Returns false if a domain becomes empty (inconsistent)
    pub fn propagate_chirho(&mut self) -> bool {
        // Queue of (constraint_idx, direction) to process
        // direction: true = revise var1, false = revise var2
        let mut queue_chirho: VecDeque<(usize, bool)> = VecDeque::new();

        // Initialize queue with all arcs
        for (idx_chirho, _) in self.constraints_chirho.iter().enumerate() {
            queue_chirho.push_back((idx_chirho, true));
            queue_chirho.push_back((idx_chirho, false));
        }

        while let Some((idx_chirho, revise_first_chirho)) = queue_chirho.pop_front() {
            let constraint_chirho = &self.constraints_chirho[idx_chirho];
            let (var_chirho, other_var_chirho) = if revise_first_chirho {
                (constraint_chirho.var1_chirho, constraint_chirho.var2_chirho)
            } else {
                (constraint_chirho.var2_chirho, constraint_chirho.var1_chirho)
            };

            let domain_chirho = self.domain_chirho(var_chirho);
            let other_domain_chirho = self.domain_chirho(other_var_chirho);

            // Compute supported values
            let mut new_domain_chirho = DomainChirho::empty_chirho();
            for val_chirho in domain_chirho.iter_chirho() {
                // Check if any value in other domain supports this
                let has_support_chirho = other_domain_chirho.iter_chirho().any(|other_val_chirho| {
                    let pair_chirho = if revise_first_chirho {
                        (val_chirho, other_val_chirho)
                    } else {
                        (other_val_chirho, val_chirho)
                    };
                    constraint_chirho.allowed_chirho.contains(&pair_chirho)
                });

                if has_support_chirho {
                    new_domain_chirho = new_domain_chirho.union_chirho(DomainChirho::singleton_chirho(val_chirho));
                }
            }

            // If domain changed, update and add affected constraints to queue
            if new_domain_chirho.0 != domain_chirho.0 {
                if !self.set_domain_chirho(var_chirho, new_domain_chirho) {
                    return false; // Empty domain = failure
                }

                // Add all constraints involving this variable
                if let Some(indices_chirho) = self.constraint_index_chirho.get(&var_chirho) {
                    for &other_idx_chirho in indices_chirho {
                        if other_idx_chirho != idx_chirho {
                            let c_chirho = &self.constraints_chirho[other_idx_chirho];
                            if c_chirho.var1_chirho == var_chirho {
                                queue_chirho.push_back((other_idx_chirho, false));
                            } else {
                                queue_chirho.push_back((other_idx_chirho, true));
                            }
                        }
                    }
                }
            }
        }

        true
    }

    /// Check if all variables are bound (singleton domains)
    pub fn is_solved_chirho(&self) -> bool {
        self.domains_chirho.values().all(|d| d.is_singleton_chirho())
    }

    /// Get solution if solved
    pub fn get_solution_chirho(&self) -> Option<HashMap<VarIdChirho, u32>> {
        if !self.is_solved_chirho() {
            return None;
        }
        Some(
            self.domains_chirho
                .iter()
                .map(|(&var_chirho, &dom_chirho)| (var_chirho, dom_chirho.get_singleton_chirho().unwrap()))
                .collect(),
        )
    }
}

impl Default for ConstraintStoreChirho {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_domain_basic_chirho() {
        let d_chirho = DomainChirho::range_chirho(10);
        assert_eq!(d_chirho.count_chirho(), 10);
        assert!(d_chirho.contains_chirho(5));
        assert!(!d_chirho.contains_chirho(15));

        let d2_chirho = d_chirho.remove_chirho(5);
        assert!(!d2_chirho.contains_chirho(5));
        assert_eq!(d2_chirho.count_chirho(), 9);
    }

    #[test]
    fn test_domain_singleton_chirho() {
        let d_chirho = DomainChirho::singleton_chirho(7);
        assert!(d_chirho.is_singleton_chirho());
        assert_eq!(d_chirho.get_singleton_chirho(), Some(7));
    }

    #[test]
    fn test_propagate_equality_chirho() {
        let mut store_chirho = ConstraintStoreChirho::new();

        // x in {0,1,2}, y in {1,2,3}, x == y
        store_chirho.add_var_chirho(0, DomainChirho(0b0111)); // {0,1,2}
        store_chirho.add_var_chirho(1, DomainChirho(0b1110)); // {1,2,3}

        let eq_chirho = BinaryConstraintChirho::equality_chirho(0, 1, 10);
        store_chirho.add_constraint_chirho(eq_chirho);

        assert!(store_chirho.propagate_chirho());

        // Should narrow to {1,2}
        assert_eq!(store_chirho.domain_chirho(0), DomainChirho(0b0110));
        assert_eq!(store_chirho.domain_chirho(1), DomainChirho(0b0110));
    }

    #[test]
    fn test_propagate_failure_chirho() {
        let mut store_chirho = ConstraintStoreChirho::new();

        // x in {0}, y in {1}, x == y -> failure
        store_chirho.add_var_chirho(0, DomainChirho::singleton_chirho(0));
        store_chirho.add_var_chirho(1, DomainChirho::singleton_chirho(1));

        let eq_chirho = BinaryConstraintChirho::equality_chirho(0, 1, 10);
        store_chirho.add_constraint_chirho(eq_chirho);

        assert!(!store_chirho.propagate_chirho());
    }
}
