//! Paged Bit Vector Domains ☧
//!
//! Extend beyond 64 values using sparse pages of BitVec64.
//! Each page covers 64 consecutive values.
//!
//! ```text
//! Page 0: values 0-63      (BitVec64)
//! Page 1: values 64-127    (BitVec64)
//! Page 2: values 128-191   (BitVec64)
//! ...
//! ```
//!
//! Only allocated pages consume memory. Intersection is O(min(pages)).

use std::collections::HashMap;
use crate::hardware_chirho::BitVec64Chirho;

/// Iterate over set bits in a u64
fn iter_bits_chirho(mut bits_chirho: u64) -> impl Iterator<Item = u32> {
    std::iter::from_fn(move || {
        if bits_chirho == 0 {
            None
        } else {
            let bit_chirho = bits_chirho.trailing_zeros();
            bits_chirho &= bits_chirho - 1; // Clear lowest bit
            Some(bit_chirho)
        }
    })
}

/// Sparse paged domain over arbitrary non-negative integers
#[derive(Debug, Clone, Default)]
pub struct PagedDomainChirho {
    /// Sparse pages: page_number → bits for that page
    /// Page N covers values [N*64, N*64+63]
    pages_chirho: HashMap<u64, BitVec64Chirho>,
}

impl PagedDomainChirho {
    /// Empty domain (no values possible)
    pub fn empty_chirho() -> Self {
        Self { pages_chirho: HashMap::new() }
    }

    /// Full domain for range [0, max_value]
    pub fn range_chirho(max_value_chirho: u64) -> Self {
        let mut pages_chirho = HashMap::new();
        let full_pages_chirho = max_value_chirho / 64;

        // Full pages
        for page_chirho in 0..full_pages_chirho {
            pages_chirho.insert(page_chirho, BitVec64Chirho::ONES_CHIRHO);
        }

        // Partial last page
        let remainder_chirho = max_value_chirho % 64;
        if remainder_chirho > 0 || full_pages_chirho == 0 {
            let last_page_chirho = full_pages_chirho;
            let mask_chirho = if remainder_chirho == 0 {
                1u64 // Just value 0
            } else {
                (1u64 << (remainder_chirho + 1)) - 1
            };
            pages_chirho.insert(last_page_chirho, BitVec64Chirho(mask_chirho));
        }

        Self { pages_chirho }
    }

    /// Singleton domain containing only one value
    pub fn singleton_chirho(value_chirho: u64) -> Self {
        let page_chirho = value_chirho / 64;
        let bit_chirho = value_chirho % 64;
        let mut pages_chirho = HashMap::new();
        pages_chirho.insert(page_chirho, BitVec64Chirho(1u64 << bit_chirho));
        Self { pages_chirho }
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: u64) -> bool {
        let page_chirho = value_chirho / 64;
        let bit_chirho = value_chirho % 64;
        self.pages_chirho
            .get(&page_chirho)
            .map_or(false, |p| p.test_bit_chirho(bit_chirho as u32))
    }

    /// Add a value to the domain
    pub fn insert_chirho(&mut self, value_chirho: u64) {
        let page_chirho = value_chirho / 64;
        let bit_chirho = value_chirho % 64;
        self.pages_chirho
            .entry(page_chirho)
            .or_insert(BitVec64Chirho::ZERO_CHIRHO)
            .0 |= 1u64 << bit_chirho;
    }

    /// Remove a value from the domain
    pub fn remove_chirho(&mut self, value_chirho: u64) {
        let page_chirho = value_chirho / 64;
        let bit_chirho = value_chirho % 64;
        if let Some(bits_chirho) = self.pages_chirho.get_mut(&page_chirho) {
            bits_chirho.0 &= !(1u64 << bit_chirho);
            if bits_chirho.is_zero_chirho() {
                self.pages_chirho.remove(&page_chirho);
            }
        }
    }

    /// Intersect two domains (hardware-accelerated per page)
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = HashMap::new();

        // Only process pages that exist in both domains
        for (&page_chirho, bits_chirho) in &self.pages_chirho {
            if let Some(other_bits_chirho) = other_chirho.pages_chirho.get(&page_chirho) {
                // Hardware-accelerated AND within page
                let intersection_chirho = bits_chirho.and_chirho(*other_bits_chirho);
                if !intersection_chirho.is_zero_chirho() {
                    result_chirho.insert(page_chirho, intersection_chirho);
                }
            }
        }

        Self { pages_chirho: result_chirho }
    }

    /// Union of two domains
    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = self.pages_chirho.clone();

        for (&page_chirho, bits_chirho) in &other_chirho.pages_chirho {
            result_chirho
                .entry(page_chirho)
                .and_modify(|existing| *existing = existing.or_chirho(*bits_chirho))
                .or_insert(*bits_chirho);
        }

        Self { pages_chirho: result_chirho }
    }

    /// Check if domain is empty
    pub fn is_empty_chirho(&self) -> bool {
        self.pages_chirho.is_empty()
    }

    /// Count total values in domain
    pub fn count_chirho(&self) -> u64 {
        self.pages_chirho
            .values()
            .map(|bits| bits.popcount_chirho() as u64)
            .sum()
    }

    /// Iterate over all values in domain
    pub fn iter_chirho(&self) -> impl Iterator<Item = u64> + '_ {
        let mut pages_chirho: Vec<_> = self.pages_chirho.iter().collect();
        pages_chirho.sort_by_key(|(page, _)| *page);

        pages_chirho.into_iter().flat_map(|(page_chirho, bits_chirho)| {
            let base_chirho = page_chirho * 64;
            iter_bits_chirho(bits_chirho.0).map(move |bit| base_chirho + bit as u64)
        })
    }

    /// Try to convert to BitVec64 if domain fits
    pub fn to_bitvec64_chirho(&self) -> Option<BitVec64Chirho> {
        if self.pages_chirho.len() == 0 {
            return Some(BitVec64Chirho::ZERO_CHIRHO);
        }
        if self.pages_chirho.len() == 1 {
            if let Some(bits_chirho) = self.pages_chirho.get(&0) {
                return Some(*bits_chirho);
            }
        }
        None // Doesn't fit in single page
    }

    /// Number of allocated pages
    pub fn page_count_chirho(&self) -> usize {
        self.pages_chirho.len()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_singleton_chirho() {
        let domain_chirho = PagedDomainChirho::singleton_chirho(100);
        assert!(domain_chirho.contains_chirho(100));
        assert!(!domain_chirho.contains_chirho(99));
        assert!(!domain_chirho.contains_chirho(101));
        assert_eq!(domain_chirho.count_chirho(), 1);
    }

    #[test]
    fn test_range_chirho() {
        let domain_chirho = PagedDomainChirho::range_chirho(100);
        assert!(domain_chirho.contains_chirho(0));
        assert!(domain_chirho.contains_chirho(50));
        assert!(domain_chirho.contains_chirho(100));
        assert!(!domain_chirho.contains_chirho(101));
    }

    #[test]
    fn test_intersect_chirho() {
        let a_chirho = PagedDomainChirho::range_chirho(100);  // 0-100
        let b_chirho = PagedDomainChirho::range_chirho(50);   // 0-50
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert!(c_chirho.contains_chirho(50));
        assert!(!c_chirho.contains_chirho(75));
    }

    #[test]
    fn test_large_values_chirho() {
        let mut domain_chirho = PagedDomainChirho::empty_chirho();
        domain_chirho.insert_chirho(1000);
        domain_chirho.insert_chirho(1000000);

        assert!(domain_chirho.contains_chirho(1000));
        assert!(domain_chirho.contains_chirho(1000000));
        assert!(!domain_chirho.contains_chirho(500));
        assert_eq!(domain_chirho.count_chirho(), 2);
    }
}
