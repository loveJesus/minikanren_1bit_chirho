// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Boolean relations as 1-bit matrices ☧
//!
//! Two implementations for comparison:
//! 1. **Sparse (HashSet)**: For large/sparse relations, unlimited domain size
//! 2. **Dense (BitVec64)**: For domains ≤64, uses our hardware primitives — THE THESIS!
//!
//! The dense version demonstrates the core insight: relations become bit arrays,
//! composition becomes bitwise AND/OR (single CPU cycles).

use crate::hardware_chirho::BitVec64Chirho;
use std::collections::HashSet;

// =============================================================================
// Sparse Implementation (HashSet) - for comparison / large domains
// =============================================================================

/// A binary relation as sparse (row, col) pairs.
/// Good for large or very sparse relations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseRelationChirho {
    pub name_chirho: String,
    pub pairs_chirho: HashSet<(u32, u32)>,
}

impl SparseRelationChirho {
    pub fn new_chirho(name_chirho: &str) -> Self {
        Self {
            name_chirho: name_chirho.to_string(),
            pairs_chirho: HashSet::new(),
        }
    }

    #[inline]
    pub fn add_chirho(&mut self, a_chirho: u32, b_chirho: u32) {
        self.pairs_chirho.insert((a_chirho, b_chirho));
    }

    pub fn query_first_chirho(&self, a_chirho: u32) -> Vec<u32> {
        self.pairs_chirho
            .iter()
            .filter(|(x_chirho, _)| *x_chirho == a_chirho)
            .map(|(_, y_chirho)| *y_chirho)
            .collect()
    }

    pub fn query_second_chirho(&self, b_chirho: u32) -> Vec<u32> {
        self.pairs_chirho
            .iter()
            .filter(|(_, y_chirho)| *y_chirho == b_chirho)
            .map(|(x_chirho, _)| *x_chirho)
            .collect()
    }

    /// Compose: (R1 ∘ R2)(a, c) = ∃b: R1(a, b) ∧ R2(b, c)
    pub fn compose_chirho(&self, other_chirho: &SparseRelationChirho) -> SparseRelationChirho {
        let mut result_chirho = SparseRelationChirho::new_chirho(&format!(
            "({} ∘ {})",
            self.name_chirho, other_chirho.name_chirho
        ));

        for (a_chirho, b_chirho) in &self.pairs_chirho {
            for (b2_chirho, c_chirho) in &other_chirho.pairs_chirho {
                if b_chirho == b2_chirho {
                    result_chirho.add_chirho(*a_chirho, *c_chirho);
                }
            }
        }
        result_chirho
    }

    pub fn union_chirho(&self, other_chirho: &SparseRelationChirho) -> SparseRelationChirho {
        let mut result_chirho = SparseRelationChirho::new_chirho(&format!(
            "({} ∪ {})",
            self.name_chirho, other_chirho.name_chirho
        ));
        result_chirho.pairs_chirho = self
            .pairs_chirho
            .union(&other_chirho.pairs_chirho)
            .cloned()
            .collect();
        result_chirho
    }

    pub fn transitive_closure_chirho(&self) -> SparseRelationChirho {
        let mut result_chirho = self.clone();
        result_chirho.name_chirho = format!("{}*", self.name_chirho);

        loop {
            let extended_chirho = result_chirho.union_chirho(&result_chirho.compose_chirho(self));
            if extended_chirho.pairs_chirho.len() == result_chirho.pairs_chirho.len() {
                break;
            }
            result_chirho = extended_chirho;
        }
        result_chirho
    }

    pub fn len_chirho(&self) -> usize {
        self.pairs_chirho.len()
    }

    pub fn is_empty_chirho(&self) -> bool {
        self.pairs_chirho.is_empty()
    }
}

// =============================================================================
// Dense Implementation (BitVec64) - THE THESIS!
// =============================================================================

/// Dense bit-matrix relation using BitVec64Chirho from our hardware primitives.
/// Each row is a 64-bit mask — operations are single CPU instructions!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseRelationChirho {
    pub name_chirho: String,
    /// rows_chirho[i] = bitmask of all j where relation(i, j) holds
    pub rows_chirho: Vec<BitVec64Chirho>,
    pub size_chirho: usize,
}

impl DenseRelationChirho {
    /// Create empty relation for domain of given size (max 64)
    pub fn new_chirho(name_chirho: &str, size_chirho: usize) -> Self {
        assert!(size_chirho <= 64, "DenseRelation supports max 64 elements");
        Self {
            name_chirho: name_chirho.to_string(),
            rows_chirho: vec![BitVec64Chirho::ZERO_CHIRHO; size_chirho],
            size_chirho,
        }
    }

    /// Add fact: relation(a, b) — single bit set
    #[inline]
    pub fn add_chirho(&mut self, a_chirho: usize, b_chirho: usize) {
        self.rows_chirho[a_chirho] = self.rows_chirho[a_chirho].set_bit_chirho(b_chirho as u32);
    }

    /// Check if relation(a, b) holds — single bit test
    #[inline]
    pub fn contains_chirho(&self, a_chirho: usize, b_chirho: usize) -> bool {
        self.rows_chirho[a_chirho].test_bit_chirho(b_chirho as u32)
    }

    /// Query: relation(a, ?) — returns bitmask (SINGLE ROW READ!)
    #[inline]
    pub fn query_first_mask_chirho(&self, a_chirho: usize) -> BitVec64Chirho {
        self.rows_chirho[a_chirho]
    }

    /// Query: relation(a, ?) — returns Vec
    pub fn query_first_chirho(&self, a_chirho: usize) -> Vec<usize> {
        let mask_chirho = self.rows_chirho[a_chirho];
        (0..self.size_chirho)
            .filter(|b_chirho| mask_chirho.test_bit_chirho(*b_chirho as u32))
            .collect()
    }

    /// Query: relation(?, b) — returns bitmask (column scan)
    pub fn query_second_mask_chirho(&self, b_chirho: usize) -> BitVec64Chirho {
        let mut result_chirho = BitVec64Chirho::ZERO_CHIRHO;
        for (a_chirho, row_chirho) in self.rows_chirho.iter().enumerate() {
            if row_chirho.test_bit_chirho(b_chirho as u32) {
                result_chirho = result_chirho.set_bit_chirho(a_chirho as u32);
            }
        }
        result_chirho
    }

    /// Query: relation(?, b) — returns Vec
    pub fn query_second_chirho(&self, b_chirho: usize) -> Vec<usize> {
        let mask_chirho = self.query_second_mask_chirho(b_chirho);
        (0..self.size_chirho)
            .filter(|a_chirho| mask_chirho.test_bit_chirho(*a_chirho as u32))
            .collect()
    }

    /// Compose: (R1 ∘ R2)(a, c) = ∃b: R1(a, b) ∧ R2(b, c)
    /// Boolean matrix multiply via OR of rows!
    pub fn compose_chirho(&self, other_chirho: &DenseRelationChirho) -> DenseRelationChirho {
        let mut result_chirho = DenseRelationChirho::new_chirho(
            &format!("({} ∘ {})", self.name_chirho, other_chirho.name_chirho),
            self.size_chirho,
        );

        for a_chirho in 0..self.size_chirho {
            let row_a_chirho = self.rows_chirho[a_chirho];
            let mut result_row_chirho = BitVec64Chirho::ZERO_CHIRHO;

            // For each b where self(a, b) holds, OR in other's row b
            let mut remaining_chirho = row_a_chirho.0;
            while remaining_chirho != 0 {
                let b_chirho = remaining_chirho.trailing_zeros() as usize;
                result_row_chirho = result_row_chirho.or_chirho(other_chirho.rows_chirho[b_chirho]);
                remaining_chirho &= remaining_chirho - 1;
            }

            result_chirho.rows_chirho[a_chirho] = result_row_chirho;
        }
        result_chirho
    }

    /// Union: just OR the rows!
    pub fn union_chirho(&self, other_chirho: &DenseRelationChirho) -> DenseRelationChirho {
        let mut result_chirho = DenseRelationChirho::new_chirho(
            &format!("({} ∪ {})", self.name_chirho, other_chirho.name_chirho),
            self.size_chirho,
        );

        for i_chirho in 0..self.size_chirho {
            result_chirho.rows_chirho[i_chirho] =
                self.rows_chirho[i_chirho].or_chirho(other_chirho.rows_chirho[i_chirho]);
        }
        result_chirho
    }

    /// Transitive closure via fixed-point iteration
    pub fn transitive_closure_chirho(&self) -> DenseRelationChirho {
        let mut result_chirho = self.clone();
        result_chirho.name_chirho = format!("{}*", self.name_chirho);

        loop {
            let extended_chirho = result_chirho.union_chirho(&result_chirho.compose_chirho(self));
            if extended_chirho.rows_chirho == result_chirho.rows_chirho {
                break;
            }
            result_chirho = extended_chirho;
        }
        result_chirho
    }

    /// Count facts (sum of popcounts)
    pub fn len_chirho(&self) -> usize {
        self.rows_chirho
            .iter()
            .map(|r_chirho| r_chirho.popcount_chirho() as usize)
            .sum()
    }

    pub fn is_empty_chirho(&self) -> bool {
        self.rows_chirho.iter().all(|r_chirho| r_chirho.is_zero_chirho())
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_sparse_basic_chirho() {
        let mut rel_chirho = SparseRelationChirho::new_chirho("test");
        rel_chirho.add_chirho(0, 1);
        rel_chirho.add_chirho(0, 2);

        assert_eq!(rel_chirho.query_first_chirho(0).len(), 2);
        assert_eq!(rel_chirho.query_second_chirho(1), vec![0]);
    }

    #[test]
    fn test_dense_basic_chirho() {
        let mut rel_chirho = DenseRelationChirho::new_chirho("test", 8);
        rel_chirho.add_chirho(0, 1);
        rel_chirho.add_chirho(0, 2);

        assert!(rel_chirho.contains_chirho(0, 1));
        assert!(rel_chirho.contains_chirho(0, 2));
        assert!(!rel_chirho.contains_chirho(0, 3));

        assert_eq!(rel_chirho.query_first_mask_chirho(0).0, 0b110);
    }

    #[test]
    fn test_dense_compose_chirho() {
        // parent: 0->2, 1->2, 2->4, 3->4
        let mut parent_chirho = DenseRelationChirho::new_chirho("parent", 8);
        parent_chirho.add_chirho(0, 2);
        parent_chirho.add_chirho(1, 2);
        parent_chirho.add_chirho(2, 4);
        parent_chirho.add_chirho(3, 4);

        let grandparent_chirho = parent_chirho.compose_chirho(&parent_chirho);

        assert!(grandparent_chirho.contains_chirho(0, 4));
        assert!(grandparent_chirho.contains_chirho(1, 4));
        assert!(!grandparent_chirho.contains_chirho(2, 4));
    }

    #[test]
    fn test_dense_closure_chirho() {
        // Chain: 0 -> 1 -> 2 -> 3
        let mut parent_chirho = DenseRelationChirho::new_chirho("parent", 8);
        parent_chirho.add_chirho(0, 1);
        parent_chirho.add_chirho(1, 2);
        parent_chirho.add_chirho(2, 3);

        let ancestor_chirho = parent_chirho.transitive_closure_chirho();

        // 0 is ancestor of 1, 2, 3
        assert_eq!(ancestor_chirho.query_first_mask_chirho(0).0, 0b1110);

        // 3's ancestors
        let ancestors_chirho = ancestor_chirho.query_second_chirho(3);
        assert_eq!(ancestors_chirho, vec![0, 1, 2]);
    }

    #[test]
    fn test_grandparents_bidirectional_chirho() {
        // tom(0), jane(1) -> james(4)
        // ed(2), fay(3) -> alice(5)
        // james(4), alice(5) -> bob(6)
        let mut parent_chirho = DenseRelationChirho::new_chirho("parent", 8);

        parent_chirho.add_chirho(0, 4);
        parent_chirho.add_chirho(1, 4);
        parent_chirho.add_chirho(2, 5);
        parent_chirho.add_chirho(3, 5);
        parent_chirho.add_chirho(4, 6);
        parent_chirho.add_chirho(5, 6);

        let grandparent_chirho = parent_chirho.compose_chirho(&parent_chirho);

        // Tom's grandchildren (DOWN)
        assert_eq!(grandparent_chirho.query_first_chirho(0), vec![6]);

        // Bob's grandparents (UP)
        let bobs_gp_chirho = grandparent_chirho.query_second_chirho(6);
        assert_eq!(bobs_gp_chirho.len(), 4);
    }
}
