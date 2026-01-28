// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Hierarchical BitVec64 Domain ☧
//!
//! Tree structure for domains larger than 64 values while preserving
//! hardware-accelerated operations.
//!
//! ```text
//! 2-level hierarchy (4096 values):
//!   root: BitVec64 where bit i → leaves[i] is non-empty
//!   leaves: [BitVec64; 64] → actual value membership
//!
//! 3-level hierarchy (262,144 values):
//!   root: BitVec64 → which level-1 nodes are non-empty
//!   index: [BitVec64; 64] → which leaves are non-empty
//!   leaves: [[BitVec64; 64]; 64] → actual values
//! ```
//!
//! Key insight: Intersection first ANDs the root masks to find
//! which subtrees need checking. Sparse domains skip most work.

use crate::hardware_chirho::BitVec64Chirho;

/// 2-level hierarchical domain: 64 × 64 = 4096 values
#[derive(Clone, Debug)]
pub struct Hierarchical4kChirho {
    /// Bit i set means leaves[i] is non-empty
    pub root_chirho: BitVec64Chirho,
    /// Actual value membership, grouped by 64s
    pub leaves_chirho: Box<[BitVec64Chirho; 64]>,
}

impl Hierarchical4kChirho {
    /// Empty domain
    pub fn empty_chirho() -> Self {
        Self {
            root_chirho: BitVec64Chirho::ZERO_CHIRHO,
            leaves_chirho: Box::new([BitVec64Chirho::ZERO_CHIRHO; 64]),
        }
    }

    /// Full domain (all 4096 values)
    pub fn full_chirho() -> Self {
        Self {
            root_chirho: BitVec64Chirho::ONES_CHIRHO,
            leaves_chirho: Box::new([BitVec64Chirho::ONES_CHIRHO; 64]),
        }
    }

    /// Range domain [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        let mut result_chirho = Self::empty_chirho();

        let full_leaves_chirho = n_chirho / 64;
        let remainder_chirho = n_chirho % 64;

        // Fill complete leaves
        for i in 0..full_leaves_chirho as usize {
            result_chirho.leaves_chirho[i] = BitVec64Chirho::ONES_CHIRHO;
            result_chirho.root_chirho = result_chirho.root_chirho
                .or_chirho(BitVec64Chirho(1 << i));
        }

        // Partial last leaf
        if remainder_chirho > 0 && (full_leaves_chirho as usize) < 64 {
            let mask_chirho = (1u64 << remainder_chirho) - 1;
            result_chirho.leaves_chirho[full_leaves_chirho as usize] = BitVec64Chirho(mask_chirho);
            result_chirho.root_chirho = result_chirho.root_chirho
                .or_chirho(BitVec64Chirho(1 << full_leaves_chirho));
        }

        result_chirho
    }

    /// Single value domain
    pub fn singleton_chirho(value_chirho: u32) -> Self {
        assert!(value_chirho < 4096, "Value must be < 4096");
        let mut result_chirho = Self::empty_chirho();

        let leaf_idx_chirho = (value_chirho / 64) as usize;
        let bit_idx_chirho = value_chirho % 64;

        result_chirho.leaves_chirho[leaf_idx_chirho] = BitVec64Chirho(1 << bit_idx_chirho);
        result_chirho.root_chirho = BitVec64Chirho(1 << leaf_idx_chirho);

        result_chirho
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: u32) -> bool {
        if value_chirho >= 4096 {
            return false;
        }

        let leaf_idx_chirho = (value_chirho / 64) as usize;
        let bit_idx_chirho = value_chirho % 64;

        // First check root (fast path for sparse domains)
        if !self.root_chirho.test_bit_chirho(leaf_idx_chirho as u32) {
            return false;
        }

        self.leaves_chirho[leaf_idx_chirho].test_bit_chirho(bit_idx_chirho)
    }

    /// Intersect two hierarchical domains
    ///
    /// Key optimization: Only check leaves where BOTH roots have bits set
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        // Step 1: AND roots to find candidate leaves
        let candidate_mask_chirho = self.root_chirho.and_chirho(other_chirho.root_chirho);

        // Fast path: no overlap at root level
        if candidate_mask_chirho.is_zero_chirho() {
            return Self::empty_chirho();
        }

        // Step 2: Intersect only candidate leaves
        let mut new_leaves_chirho = Box::new([BitVec64Chirho::ZERO_CHIRHO; 64]);
        let mut new_root_chirho = BitVec64Chirho::ZERO_CHIRHO;

        let mut bits_chirho = candidate_mask_chirho.0;
        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1; // Clear lowest bit

            let leaf_result_chirho = self.leaves_chirho[i_chirho]
                .and_chirho(other_chirho.leaves_chirho[i_chirho]);

            new_leaves_chirho[i_chirho] = leaf_result_chirho;

            if !leaf_result_chirho.is_zero_chirho() {
                new_root_chirho = new_root_chirho.or_chirho(BitVec64Chirho(1 << i_chirho));
            }
        }

        Self {
            root_chirho: new_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Union two hierarchical domains
    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        let new_root_chirho = self.root_chirho.or_chirho(other_chirho.root_chirho);
        let mut new_leaves_chirho = Box::new([BitVec64Chirho::ZERO_CHIRHO; 64]);

        let mut bits_chirho = new_root_chirho.0;
        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1;

            new_leaves_chirho[i_chirho] = self.leaves_chirho[i_chirho]
                .or_chirho(other_chirho.leaves_chirho[i_chirho]);
        }

        Self {
            root_chirho: new_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Check if domain is empty
    pub fn is_empty_chirho(&self) -> bool {
        self.root_chirho.is_zero_chirho()
    }

    /// Count values in domain
    pub fn count_chirho(&self) -> u32 {
        let mut count_chirho = 0u32;
        let mut bits_chirho = self.root_chirho.0;

        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1;
            count_chirho += self.leaves_chirho[i_chirho].0.count_ones();
        }

        count_chirho
    }

    /// Iterate over all values in domain
    pub fn iter_chirho(&self) -> impl Iterator<Item = u32> + '_ {
        let mut leaf_idx_chirho = 0usize;
        let mut root_bits_chirho = self.root_chirho.0;
        let mut leaf_bits_chirho = 0u64;

        std::iter::from_fn(move || {
            loop {
                // Try to get next bit from current leaf
                if leaf_bits_chirho != 0 {
                    let bit_chirho = leaf_bits_chirho.trailing_zeros();
                    leaf_bits_chirho &= leaf_bits_chirho - 1;
                    return Some((leaf_idx_chirho as u32) * 64 + bit_chirho);
                }

                // Move to next non-empty leaf
                if root_bits_chirho == 0 {
                    return None;
                }

                leaf_idx_chirho = root_bits_chirho.trailing_zeros() as usize;
                root_bits_chirho &= root_bits_chirho - 1;
                leaf_bits_chirho = self.leaves_chirho[leaf_idx_chirho].0;
            }
        })
    }
}

/// 3-level hierarchical domain: 64 × 64 × 64 = 262,144 values
#[derive(Clone)]
pub struct Hierarchical256kChirho {
    /// Root: bit i set means index[i] has non-empty leaves
    pub root_chirho: BitVec64Chirho,
    /// Index: bit j in index[i] means leaves[i][j] is non-empty
    pub index_chirho: Box<[BitVec64Chirho; 64]>,
    /// Leaves: actual value membership
    pub leaves_chirho: Box<[[BitVec64Chirho; 64]; 64]>,
}

impl Hierarchical256kChirho {
    /// Empty domain
    pub fn empty_chirho() -> Self {
        Self {
            root_chirho: BitVec64Chirho::ZERO_CHIRHO,
            index_chirho: Box::new([BitVec64Chirho::ZERO_CHIRHO; 64]),
            leaves_chirho: Box::new([[BitVec64Chirho::ZERO_CHIRHO; 64]; 64]),
        }
    }

    /// Range domain [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        assert!(n_chirho <= 262144, "Value must be <= 262144");
        let mut result_chirho = Self::empty_chirho();

        for v in 0..n_chirho {
            let idx1_chirho = (v / 4096) as usize;
            let idx2_chirho = ((v % 4096) / 64) as usize;
            let bit_chirho = v % 64;

            result_chirho.leaves_chirho[idx1_chirho][idx2_chirho].0 |= 1 << bit_chirho;
            result_chirho.index_chirho[idx1_chirho].0 |= 1 << idx2_chirho;
            result_chirho.root_chirho.0 |= 1 << idx1_chirho;
        }

        result_chirho
    }

    /// Intersect with 3-level hierarchy
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        // Level 1: AND roots
        let candidate_root_chirho = self.root_chirho.and_chirho(other_chirho.root_chirho);

        if candidate_root_chirho.is_zero_chirho() {
            return Self::empty_chirho();
        }

        let mut result_chirho = Self::empty_chirho();

        // Level 2: For each candidate in root, AND indices
        let mut root_bits_chirho = candidate_root_chirho.0;
        while root_bits_chirho != 0 {
            let i_chirho = root_bits_chirho.trailing_zeros() as usize;
            root_bits_chirho &= root_bits_chirho - 1;

            let candidate_index_chirho = self.index_chirho[i_chirho]
                .and_chirho(other_chirho.index_chirho[i_chirho]);

            if candidate_index_chirho.is_zero_chirho() {
                continue;
            }

            // Level 3: For each candidate in index, AND leaves
            let mut index_bits_chirho = candidate_index_chirho.0;
            while index_bits_chirho != 0 {
                let j_chirho = index_bits_chirho.trailing_zeros() as usize;
                index_bits_chirho &= index_bits_chirho - 1;

                let leaf_result_chirho = self.leaves_chirho[i_chirho][j_chirho]
                    .and_chirho(other_chirho.leaves_chirho[i_chirho][j_chirho]);

                if !leaf_result_chirho.is_zero_chirho() {
                    result_chirho.leaves_chirho[i_chirho][j_chirho] = leaf_result_chirho;
                    result_chirho.index_chirho[i_chirho].0 |= 1 << j_chirho;
                    result_chirho.root_chirho.0 |= 1 << i_chirho;
                }
            }
        }

        result_chirho
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        self.root_chirho.is_zero_chirho()
    }
}

// ============================================================================
// Hierarchical16kChirho: 64 × 256 = 16,384 values using BitVec256 leaves
// ============================================================================

use crate::hardware_chirho::BitVec256Chirho;

/// 2-level hierarchical domain with 256-bit leaves: 64 × 256 = 16,384 values
///
/// This provides 4× the capacity of Hierarchical4kChirho while maintaining
/// similar performance characteristics. Useful when 4K values isn't enough
/// but 256K is overkill.
///
/// ```text
/// root: BitVec64 (bit i → leaves[i] is non-empty)
/// leaves: [BitVec256; 64] (256 values per leaf)
/// ```
#[derive(Clone, Debug)]
pub struct Hierarchical16kChirho {
    /// Bit i set means leaves[i] is non-empty
    pub root_chirho: BitVec64Chirho,
    /// Actual value membership, 256 values per leaf
    pub leaves_chirho: Box<[BitVec256Chirho; 64]>,
}

impl Hierarchical16kChirho {
    /// Empty domain
    pub fn empty_chirho() -> Self {
        Self {
            root_chirho: BitVec64Chirho::ZERO_CHIRHO,
            leaves_chirho: Box::new([BitVec256Chirho::ZERO_CHIRHO; 64]),
        }
    }

    /// Full domain (all 16,384 values)
    pub fn full_chirho() -> Self {
        Self {
            root_chirho: BitVec64Chirho::ONES_CHIRHO,
            leaves_chirho: Box::new([BitVec256Chirho::ONES_CHIRHO; 64]),
        }
    }

    /// Range domain [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        assert!(n_chirho <= 16384, "Value must be <= 16384");
        let mut result_chirho = Self::empty_chirho();

        let full_leaves_chirho = n_chirho / 256;
        let remainder_chirho = n_chirho % 256;

        // Fill complete leaves
        for i_chirho in 0..full_leaves_chirho as usize {
            result_chirho.leaves_chirho[i_chirho] = BitVec256Chirho::ONES_CHIRHO;
            result_chirho.root_chirho = result_chirho.root_chirho
                .or_chirho(BitVec64Chirho(1 << i_chirho));
        }

        // Partial last leaf
        if remainder_chirho > 0 && (full_leaves_chirho as usize) < 64 {
            let leaf_idx_chirho = full_leaves_chirho as usize;
            // Set bits 0..remainder in the 256-bit leaf
            let mut words_chirho = [0u64; 4];
            let full_words_chirho = remainder_chirho / 64;
            let rem_bits_chirho = remainder_chirho % 64;

            for i_chirho in 0..full_words_chirho as usize {
                words_chirho[i_chirho] = u64::MAX;
            }
            if rem_bits_chirho > 0 && (full_words_chirho as usize) < 4 {
                words_chirho[full_words_chirho as usize] = (1u64 << rem_bits_chirho) - 1;
            }

            result_chirho.leaves_chirho[leaf_idx_chirho] = BitVec256Chirho(words_chirho);
            result_chirho.root_chirho = result_chirho.root_chirho
                .or_chirho(BitVec64Chirho(1 << leaf_idx_chirho));
        }

        result_chirho
    }

    /// Single value domain
    pub fn singleton_chirho(value_chirho: u32) -> Self {
        assert!(value_chirho < 16384, "Value must be < 16384");
        let mut result_chirho = Self::empty_chirho();

        let leaf_idx_chirho = (value_chirho / 256) as usize;
        let bit_idx_chirho = value_chirho % 256;

        // Set single bit in 256-bit leaf
        let word_idx_chirho = (bit_idx_chirho / 64) as usize;
        let bit_in_word_chirho = bit_idx_chirho % 64;
        let mut words_chirho = [0u64; 4];
        words_chirho[word_idx_chirho] = 1 << bit_in_word_chirho;

        result_chirho.leaves_chirho[leaf_idx_chirho] = BitVec256Chirho(words_chirho);
        result_chirho.root_chirho = BitVec64Chirho(1 << leaf_idx_chirho);

        result_chirho
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: u32) -> bool {
        if value_chirho >= 16384 {
            return false;
        }

        let leaf_idx_chirho = (value_chirho / 256) as usize;

        // First check root (fast path for sparse domains)
        if !self.root_chirho.test_bit_chirho(leaf_idx_chirho as u32) {
            return false;
        }

        let bit_idx_chirho = value_chirho % 256;
        self.leaves_chirho[leaf_idx_chirho].test_bit_chirho(bit_idx_chirho)
    }

    /// Intersect two domains
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        // First AND roots - if result is 0, no work needed
        let new_root_chirho = self.root_chirho.and_chirho(other_chirho.root_chirho);
        if new_root_chirho.is_zero_chirho() {
            return Self::empty_chirho();
        }

        let mut new_leaves_chirho = Box::new([BitVec256Chirho::ZERO_CHIRHO; 64]);
        let mut final_root_chirho = BitVec64Chirho::ZERO_CHIRHO;

        // Only check leaves where both roots have bits set
        let mut bits_chirho = new_root_chirho.0;
        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1;

            let leaf_result_chirho = self.leaves_chirho[i_chirho]
                .and_chirho(other_chirho.leaves_chirho[i_chirho]);

            if !leaf_result_chirho.is_zero_chirho() {
                new_leaves_chirho[i_chirho] = leaf_result_chirho;
                final_root_chirho = final_root_chirho.or_chirho(BitVec64Chirho(1 << i_chirho));
            }
        }

        Self {
            root_chirho: final_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Union two domains
    pub fn union_chirho(&self, other_chirho: &Self) -> Self {
        let new_root_chirho = self.root_chirho.or_chirho(other_chirho.root_chirho);
        let mut new_leaves_chirho = self.leaves_chirho.clone();

        let mut bits_chirho = other_chirho.root_chirho.0;
        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1;

            new_leaves_chirho[i_chirho] = new_leaves_chirho[i_chirho]
                .or_chirho(other_chirho.leaves_chirho[i_chirho]);
        }

        Self {
            root_chirho: new_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Check if domain is empty
    pub fn is_empty_chirho(&self) -> bool {
        self.root_chirho.is_zero_chirho()
    }

    /// Count values in domain
    pub fn count_chirho(&self) -> u32 {
        let mut count_chirho = 0u32;
        let mut bits_chirho = self.root_chirho.0;

        while bits_chirho != 0 {
            let i_chirho = bits_chirho.trailing_zeros() as usize;
            bits_chirho &= bits_chirho - 1;
            count_chirho += self.leaves_chirho[i_chirho].popcount_chirho();
        }

        count_chirho
    }
}

// ============================================================================
// Hierarchical65kChirho: 256 × 256 = 65,536 values (2-level with 256-bit words)
// ============================================================================

/// 2-level hierarchical domain with 256-bit words: 256 × 256 = 65,536 values
///
/// Trades depth for width: 2 levels instead of 3, but uses 256-bit SIMD.
/// May be faster than 64³ on CPUs with good AVX2 support.
///
/// ```text
/// root: BitVec256 (bit i → leaves[i] is non-empty)
/// leaves: [BitVec256; 256] (256 values per leaf)
/// ```
#[derive(Clone)]
pub struct Hierarchical65kChirho {
    /// Bit i set means leaves[i] is non-empty
    pub root_chirho: BitVec256Chirho,
    /// Actual value membership, 256 values per leaf
    pub leaves_chirho: Box<[BitVec256Chirho; 256]>,
}

impl Hierarchical65kChirho {
    /// Empty domain
    pub fn empty_chirho() -> Self {
        Self {
            root_chirho: BitVec256Chirho::ZERO_CHIRHO,
            leaves_chirho: Box::new([BitVec256Chirho::ZERO_CHIRHO; 256]),
        }
    }

    /// Full domain (all 65,536 values)
    pub fn full_chirho() -> Self {
        Self {
            root_chirho: BitVec256Chirho::ONES_CHIRHO,
            leaves_chirho: Box::new([BitVec256Chirho::ONES_CHIRHO; 256]),
        }
    }

    /// Range domain [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        assert!(n_chirho <= 65536, "Value must be <= 65536");
        let mut result_chirho = Self::empty_chirho();

        let full_leaves_chirho = n_chirho / 256;
        let remainder_chirho = n_chirho % 256;

        // Fill complete leaves
        for i_chirho in 0..full_leaves_chirho as usize {
            result_chirho.leaves_chirho[i_chirho] = BitVec256Chirho::ONES_CHIRHO;
            result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(i_chirho as u32);
        }

        // Partial last leaf
        if remainder_chirho > 0 && (full_leaves_chirho as usize) < 256 {
            let leaf_idx_chirho = full_leaves_chirho as usize;
            let mut words_chirho = [0u64; 4];
            let full_words_chirho = remainder_chirho / 64;
            let rem_bits_chirho = remainder_chirho % 64;

            for i_chirho in 0..full_words_chirho as usize {
                words_chirho[i_chirho] = u64::MAX;
            }
            if rem_bits_chirho > 0 && (full_words_chirho as usize) < 4 {
                words_chirho[full_words_chirho as usize] = (1u64 << rem_bits_chirho) - 1;
            }

            result_chirho.leaves_chirho[leaf_idx_chirho] = BitVec256Chirho(words_chirho);
            result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(leaf_idx_chirho as u32);
        }

        result_chirho
    }

    /// Single value domain
    pub fn singleton_chirho(value_chirho: u32) -> Self {
        assert!(value_chirho < 65536, "Value must be < 65536");
        let mut result_chirho = Self::empty_chirho();
        let leaf_idx_chirho = (value_chirho / 256) as usize;
        let bit_idx_chirho = value_chirho % 256;
        result_chirho.leaves_chirho[leaf_idx_chirho] = result_chirho.leaves_chirho[leaf_idx_chirho]
            .set_bit_chirho(bit_idx_chirho);
        result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(leaf_idx_chirho as u32);
        result_chirho
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: u32) -> bool {
        if value_chirho >= 65536 {
            return false;
        }
        let leaf_idx_chirho = (value_chirho / 256) as usize;
        if !self.root_chirho.test_bit_chirho(leaf_idx_chirho as u32) {
            return false;
        }
        let bit_idx_chirho = value_chirho % 256;
        self.leaves_chirho[leaf_idx_chirho].test_bit_chirho(bit_idx_chirho)
    }

    /// Intersect two domains (2-level: AND roots, then AND matching leaves)
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        let new_root_chirho = self.root_chirho.and_chirho(other_chirho.root_chirho);
        if new_root_chirho.is_zero_chirho() {
            return Self::empty_chirho();
        }

        let mut new_leaves_chirho = Box::new([BitVec256Chirho::ZERO_CHIRHO; 256]);
        let mut final_root_chirho = BitVec256Chirho::ZERO_CHIRHO;

        // Iterate through set bits in root
        for word_idx_chirho in 0..4 {
            let mut bits_chirho = new_root_chirho.0[word_idx_chirho];
            while bits_chirho != 0 {
                let bit_pos_chirho = bits_chirho.trailing_zeros();
                bits_chirho &= bits_chirho - 1;
                let i_chirho = (word_idx_chirho * 64 + bit_pos_chirho as usize) as usize;

                let leaf_result_chirho = self.leaves_chirho[i_chirho]
                    .and_chirho(other_chirho.leaves_chirho[i_chirho]);

                if !leaf_result_chirho.is_zero_chirho() {
                    new_leaves_chirho[i_chirho] = leaf_result_chirho;
                    final_root_chirho = final_root_chirho.set_bit_chirho(i_chirho as u32);
                }
            }
        }

        Self {
            root_chirho: final_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        self.root_chirho.is_zero_chirho()
    }

    /// Count values in domain
    pub fn count_chirho(&self) -> u32 {
        let mut count_chirho = 0u32;
        for word_idx_chirho in 0..4 {
            let mut bits_chirho = self.root_chirho.0[word_idx_chirho];
            while bits_chirho != 0 {
                let bit_pos_chirho = bits_chirho.trailing_zeros();
                bits_chirho &= bits_chirho - 1;
                let i_chirho = word_idx_chirho * 64 + bit_pos_chirho as usize;
                count_chirho += self.leaves_chirho[i_chirho].popcount_chirho();
            }
        }
        count_chirho
    }
}

// ============================================================================
// Hierarchical262kWideChirho: 512 × 512 = 262,144 values (2-level with 512-bit words)
// ============================================================================

use crate::hardware_chirho::BitVec512Chirho;

/// 2-level hierarchical domain with 512-bit words: 512 × 512 = 262,144 values
///
/// Same capacity as Hierarchical256kChirho (64³) but with only 2 levels.
/// Trades memory (larger leaves) for fewer indirections.
///
/// ```text
/// root: BitVec512 (bit i → leaves[i] is non-empty)
/// leaves: [BitVec512; 512] (512 values per leaf)
/// ```
#[derive(Clone)]
pub struct Hierarchical262kWideChirho {
    /// Bit i set means leaves[i] is non-empty
    pub root_chirho: BitVec512Chirho,
    /// Actual value membership, 512 values per leaf
    pub leaves_chirho: Box<[BitVec512Chirho; 512]>,
}

impl Hierarchical262kWideChirho {
    /// Empty domain
    pub fn empty_chirho() -> Self {
        Self {
            root_chirho: BitVec512Chirho::ZERO_CHIRHO,
            leaves_chirho: Box::new([BitVec512Chirho::ZERO_CHIRHO; 512]),
        }
    }

    /// Range domain [0, n)
    pub fn range_chirho(n_chirho: u32) -> Self {
        assert!(n_chirho <= 262144, "Value must be <= 262144");
        let mut result_chirho = Self::empty_chirho();

        let full_leaves_chirho = n_chirho / 512;
        let remainder_chirho = n_chirho % 512;

        // Fill complete leaves
        for i_chirho in 0..full_leaves_chirho as usize {
            result_chirho.leaves_chirho[i_chirho] = BitVec512Chirho::ONES_CHIRHO;
            result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(i_chirho as u32);
        }

        // Partial last leaf
        if remainder_chirho > 0 && (full_leaves_chirho as usize) < 512 {
            let leaf_idx_chirho = full_leaves_chirho as usize;
            let mut words_chirho = [0u64; 8];
            let full_words_chirho = remainder_chirho / 64;
            let rem_bits_chirho = remainder_chirho % 64;

            for i_chirho in 0..full_words_chirho as usize {
                words_chirho[i_chirho] = u64::MAX;
            }
            if rem_bits_chirho > 0 && (full_words_chirho as usize) < 8 {
                words_chirho[full_words_chirho as usize] = (1u64 << rem_bits_chirho) - 1;
            }

            result_chirho.leaves_chirho[leaf_idx_chirho] = BitVec512Chirho(words_chirho);
            result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(leaf_idx_chirho as u32);
        }

        result_chirho
    }

    /// Single value domain
    pub fn singleton_chirho(value_chirho: u32) -> Self {
        assert!(value_chirho < 262144, "Value must be < 262144");
        let mut result_chirho = Self::empty_chirho();
        let leaf_idx_chirho = (value_chirho / 512) as usize;
        let bit_idx_chirho = value_chirho % 512;
        result_chirho.leaves_chirho[leaf_idx_chirho] = result_chirho.leaves_chirho[leaf_idx_chirho]
            .set_bit_chirho(bit_idx_chirho);
        result_chirho.root_chirho = result_chirho.root_chirho.set_bit_chirho(leaf_idx_chirho as u32);
        result_chirho
    }

    /// Check if value is in domain
    pub fn contains_chirho(&self, value_chirho: u32) -> bool {
        if value_chirho >= 262144 {
            return false;
        }
        let leaf_idx_chirho = (value_chirho / 512) as usize;
        if !self.root_chirho.test_bit_chirho(leaf_idx_chirho as u32) {
            return false;
        }
        let bit_idx_chirho = value_chirho % 512;
        self.leaves_chirho[leaf_idx_chirho].test_bit_chirho(bit_idx_chirho)
    }

    /// Intersect two domains (2-level: AND roots, then AND matching leaves)
    pub fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        let new_root_chirho = self.root_chirho.and_chirho(other_chirho.root_chirho);
        if new_root_chirho.is_zero_chirho() {
            return Self::empty_chirho();
        }

        let mut new_leaves_chirho = Box::new([BitVec512Chirho::ZERO_CHIRHO; 512]);
        let mut final_root_chirho = BitVec512Chirho::ZERO_CHIRHO;

        // Iterate through set bits in root (8 words of 64 bits each)
        for word_idx_chirho in 0..8 {
            let mut bits_chirho = new_root_chirho.0[word_idx_chirho];
            while bits_chirho != 0 {
                let bit_pos_chirho = bits_chirho.trailing_zeros();
                bits_chirho &= bits_chirho - 1;
                let i_chirho = word_idx_chirho * 64 + bit_pos_chirho as usize;

                let leaf_result_chirho = self.leaves_chirho[i_chirho]
                    .and_chirho(other_chirho.leaves_chirho[i_chirho]);

                if !leaf_result_chirho.is_zero_chirho() {
                    new_leaves_chirho[i_chirho] = leaf_result_chirho;
                    final_root_chirho = final_root_chirho.set_bit_chirho(i_chirho as u32);
                }
            }
        }

        Self {
            root_chirho: final_root_chirho,
            leaves_chirho: new_leaves_chirho,
        }
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        self.root_chirho.is_zero_chirho()
    }

    /// Count values in domain
    pub fn count_chirho(&self) -> u32 {
        let mut count_chirho = 0u32;
        for word_idx_chirho in 0..8 {
            let mut bits_chirho = self.root_chirho.0[word_idx_chirho];
            while bits_chirho != 0 {
                let bit_pos_chirho = bits_chirho.trailing_zeros();
                bits_chirho &= bits_chirho - 1;
                let i_chirho = word_idx_chirho * 64 + bit_pos_chirho as usize;
                count_chirho += self.leaves_chirho[i_chirho].popcount_chirho();
            }
        }
        count_chirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_hierarchical_4k_basic_chirho() {
        let a_chirho = Hierarchical4kChirho::range_chirho(100);
        assert!(a_chirho.contains_chirho(0));
        assert!(a_chirho.contains_chirho(99));
        assert!(!a_chirho.contains_chirho(100));
        assert_eq!(a_chirho.count_chirho(), 100);
    }

    #[test]
    fn test_hierarchical_4k_intersection_chirho() {
        let a_chirho = Hierarchical4kChirho::range_chirho(1000);
        let b_chirho = Hierarchical4kChirho::range_chirho(500);
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert_eq!(c_chirho.count_chirho(), 500);
        assert!(c_chirho.contains_chirho(499));
        assert!(!c_chirho.contains_chirho(500));
    }

    #[test]
    fn test_hierarchical_4k_sparse_intersection_chirho() {
        // Two domains that don't overlap at root level
        let mut a_chirho = Hierarchical4kChirho::empty_chirho();
        a_chirho.leaves_chirho[0] = BitVec64Chirho::ONES_CHIRHO; // Values 0-63
        a_chirho.root_chirho = BitVec64Chirho(1);

        let mut b_chirho = Hierarchical4kChirho::empty_chirho();
        b_chirho.leaves_chirho[10] = BitVec64Chirho::ONES_CHIRHO; // Values 640-703
        b_chirho.root_chirho = BitVec64Chirho(1 << 10);

        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        // Root AND is 0, so intersection should be empty
        // This is O(1) - no leaf checks needed!
        assert!(c_chirho.is_empty_chirho());
    }

    #[test]
    fn test_hierarchical_4k_iteration_chirho() {
        let domain_chirho = Hierarchical4kChirho::range_chirho(10);
        let values_chirho: Vec<u32> = domain_chirho.iter_chirho().collect();
        assert_eq!(values_chirho, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_hierarchical_256k_basic_chirho() {
        let a_chirho = Hierarchical256kChirho::range_chirho(1000);
        let b_chirho = Hierarchical256kChirho::range_chirho(500);
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert!(!c_chirho.is_empty_chirho());
    }

    #[test]
    fn test_hierarchical_16k_basic_chirho() {
        let a_chirho = Hierarchical16kChirho::range_chirho(1000);
        assert!(a_chirho.contains_chirho(0));
        assert!(a_chirho.contains_chirho(999));
        assert!(!a_chirho.contains_chirho(1000));
        assert_eq!(a_chirho.count_chirho(), 1000);
    }

    #[test]
    fn test_hierarchical_16k_intersection_chirho() {
        let a_chirho = Hierarchical16kChirho::range_chirho(5000);
        let b_chirho = Hierarchical16kChirho::range_chirho(3000);
        let c_chirho = a_chirho.intersect_chirho(&b_chirho);

        assert_eq!(c_chirho.count_chirho(), 3000);
        assert!(c_chirho.contains_chirho(2999));
        assert!(!c_chirho.contains_chirho(3000));
    }

    #[test]
    fn test_hierarchical_16k_large_values_chirho() {
        // Test values that span multiple 64-bit words in a 256-bit leaf
        let a_chirho = Hierarchical16kChirho::singleton_chirho(10000);
        assert!(a_chirho.contains_chirho(10000));
        assert!(!a_chirho.contains_chirho(9999));
        assert!(!a_chirho.contains_chirho(10001));
        assert_eq!(a_chirho.count_chirho(), 1);
    }

    #[test]
    fn test_hierarchical_16k_full_capacity_chirho() {
        let a_chirho = Hierarchical16kChirho::range_chirho(16384);
        assert_eq!(a_chirho.count_chirho(), 16384);
        assert!(a_chirho.contains_chirho(0));
        assert!(a_chirho.contains_chirho(16383));
    }
}
