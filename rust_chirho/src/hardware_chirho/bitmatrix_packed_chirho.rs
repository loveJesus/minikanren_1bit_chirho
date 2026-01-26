// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Packed Bit Matrix ☧
//!
//! ACTUAL 1-bit operations using u64 words.
//! This fulfills the "1-bit matrix operations" claim.
//!
//! For small matrices (≤64 cols): pure bit-parallel ops
//! For larger: word-striped with SIMD-friendly layout

use std::ops::{BitAnd, BitOr, BitXor, Not};

/// 64-bit word for bit-parallel operations
/// This is the fundamental unit - one CPU instruction per 64 bits
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Word64Chirho(pub u64);

impl Word64Chirho {
    pub const ZERO_CHIRHO: Self = Self(0);
    pub const ONES_CHIRHO: Self = Self(u64::MAX);

    #[inline]
    pub const fn new_chirho(val_chirho: u64) -> Self {
        Self(val_chirho)
    }

    #[inline]
    pub const fn popcount_chirho(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn is_zero_chirho(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn get_bit_chirho(self, idx_chirho: u32) -> bool {
        (self.0 >> idx_chirho) & 1 == 1
    }

    #[inline]
    pub const fn set_bit_chirho(self, idx_chirho: u32) -> Self {
        Self(self.0 | (1u64 << idx_chirho))
    }

    #[inline]
    pub const fn clear_bit_chirho(self, idx_chirho: u32) -> Self {
        Self(self.0 & !(1u64 << idx_chirho))
    }

    /// Trailing zeros = index of least significant 1
    #[inline]
    pub const fn trailing_zeros_chirho(self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Pop least significant 1 and return its index
    #[inline]
    pub fn pop_lsb_chirho(&mut self) -> Option<u32> {
        if self.0 == 0 {
            return None;
        }
        let idx_chirho = self.0.trailing_zeros();
        self.0 &= self.0 - 1; // Clear lowest bit
        Some(idx_chirho)
    }

    /// Iterate over set bit indices
    pub fn iter_ones_chirho(self) -> impl Iterator<Item = u32> {
        BitIterChirho { word_chirho: self }
    }
}

impl BitAnd for Word64Chirho {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs_chirho: Self) -> Self {
        Self(self.0 & rhs_chirho.0)
    }
}

impl BitOr for Word64Chirho {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs_chirho: Self) -> Self {
        Self(self.0 | rhs_chirho.0)
    }
}

impl BitXor for Word64Chirho {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs_chirho: Self) -> Self {
        Self(self.0 ^ rhs_chirho.0)
    }
}

impl Not for Word64Chirho {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::fmt::Debug for Word64Chirho {
    fn fmt(&self, f_chirho: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f_chirho, "0b{:064b}", self.0)
    }
}

/// Iterator over set bits in a word
struct BitIterChirho {
    word_chirho: Word64Chirho,
}

impl Iterator for BitIterChirho {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.word_chirho.pop_lsb_chirho()
    }
}

/// Small bit matrix (up to 64x64) - pure bit-parallel
/// One AND instruction does 64 element-wise operations
#[derive(Clone, PartialEq, Eq)]
pub struct BitMatrix64Chirho {
    /// Each row is a 64-bit word
    rows_chirho: [Word64Chirho; 64],
    /// Actual dimensions
    num_rows_chirho: u8,
    num_cols_chirho: u8,
}

impl BitMatrix64Chirho {
    pub fn new_chirho(rows_chirho: u8, cols_chirho: u8) -> Self {
        assert!(rows_chirho <= 64 && cols_chirho <= 64);
        Self {
            rows_chirho: [Word64Chirho::ZERO_CHIRHO; 64],
            num_rows_chirho: rows_chirho,
            num_cols_chirho: cols_chirho,
        }
    }

    pub fn zeros_chirho(rows_chirho: u8, cols_chirho: u8) -> Self {
        Self::new_chirho(rows_chirho, cols_chirho)
    }

    pub fn identity_chirho(n_chirho: u8) -> Self {
        let mut m_chirho = Self::new_chirho(n_chirho, n_chirho);
        for i_chirho in 0..n_chirho {
            m_chirho.rows_chirho[i_chirho as usize] = Word64Chirho::new_chirho(1u64 << i_chirho);
        }
        m_chirho
    }

    #[inline]
    pub fn get_chirho(&self, row_chirho: u8, col_chirho: u8) -> bool {
        self.rows_chirho[row_chirho as usize].get_bit_chirho(col_chirho as u32)
    }

    #[inline]
    pub fn set_chirho(&mut self, row_chirho: u8, col_chirho: u8) {
        self.rows_chirho[row_chirho as usize] =
            self.rows_chirho[row_chirho as usize].set_bit_chirho(col_chirho as u32);
    }

    #[inline]
    pub fn clear_chirho(&mut self, row_chirho: u8, col_chirho: u8) {
        self.rows_chirho[row_chirho as usize] =
            self.rows_chirho[row_chirho as usize].clear_bit_chirho(col_chirho as u32);
    }

    /// Element-wise AND - ONE instruction per row!
    pub fn and_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self::new_chirho(
            self.num_rows_chirho.max(other_chirho.num_rows_chirho),
            self.num_cols_chirho.max(other_chirho.num_cols_chirho),
        );
        for i_chirho in 0..64 {
            result_chirho.rows_chirho[i_chirho] =
                self.rows_chirho[i_chirho] & other_chirho.rows_chirho[i_chirho];
        }
        result_chirho
    }

    /// Element-wise OR - ONE instruction per row!
    pub fn or_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self::new_chirho(
            self.num_rows_chirho.max(other_chirho.num_rows_chirho),
            self.num_cols_chirho.max(other_chirho.num_cols_chirho),
        );
        for i_chirho in 0..64 {
            result_chirho.rows_chirho[i_chirho] =
                self.rows_chirho[i_chirho] | other_chirho.rows_chirho[i_chirho];
        }
        result_chirho
    }

    /// Boolean matrix multiply: C[i,j] = OR_k (A[i,k] AND B[k,j])
    /// This is where the magic happens for relation composition
    pub fn matmul_chirho(&self, other_chirho: &Self) -> Self {
        let mut result_chirho = Self::new_chirho(self.num_rows_chirho, other_chirho.num_cols_chirho);

        // For each row in self
        for i_chirho in 0..self.num_rows_chirho {
            let row_a_chirho = self.rows_chirho[i_chirho as usize];
            if row_a_chirho.is_zero_chirho() {
                continue;
            }

            // For each k where A[i,k] = 1
            for k_chirho in row_a_chirho.iter_ones_chirho() {
                if k_chirho >= 64 {
                    break;
                }
                // OR in the entire row B[k,*]
                result_chirho.rows_chirho[i_chirho as usize] =
                    result_chirho.rows_chirho[i_chirho as usize]
                        | other_chirho.rows_chirho[k_chirho as usize];
            }
        }

        result_chirho
    }

    /// Transpose - bit-level operation
    pub fn transpose_chirho(&self) -> Self {
        let mut result_chirho = Self::new_chirho(self.num_cols_chirho, self.num_rows_chirho);
        for i_chirho in 0..self.num_rows_chirho {
            for j_chirho in self.rows_chirho[i_chirho as usize].iter_ones_chirho() {
                if j_chirho < 64 {
                    result_chirho.set_chirho(j_chirho as u8, i_chirho);
                }
            }
        }
        result_chirho
    }

    /// Transitive closure (reachability) using repeated squaring
    /// Used for occurs check!
    pub fn transitive_closure_chirho(&self) -> Self {
        let mut current_chirho = self.clone();
        let mut prev_chirho = Self::new_chirho(self.num_rows_chirho, self.num_cols_chirho);

        // Iterate until fixpoint
        while current_chirho != prev_chirho {
            prev_chirho = current_chirho.clone();
            let squared_chirho = current_chirho.matmul_chirho(&current_chirho);
            current_chirho = current_chirho.or_chirho(&squared_chirho);
        }

        current_chirho
    }

    /// Number of 1s
    pub fn popcount_chirho(&self) -> u32 {
        self.rows_chirho
            .iter()
            .map(|w_chirho| w_chirho.popcount_chirho())
            .sum()
    }

    /// Check if any entry is set
    pub fn any_chirho(&self) -> bool {
        self.rows_chirho.iter().any(|w_chirho| !w_chirho.is_zero_chirho())
    }

    /// Check if all zero
    pub fn is_zero_chirho(&self) -> bool {
        !self.any_chirho()
    }

    /// Get row as bitmask
    pub fn row_chirho(&self, i_chirho: u8) -> Word64Chirho {
        self.rows_chirho[i_chirho as usize]
    }

    /// Get column as bitmask (slower - needs gather)
    pub fn col_chirho(&self, j_chirho: u8) -> Word64Chirho {
        let mut result_chirho = 0u64;
        for i_chirho in 0..64 {
            if self.rows_chirho[i_chirho].get_bit_chirho(j_chirho as u32) {
                result_chirho |= 1u64 << i_chirho;
            }
        }
        Word64Chirho::new_chirho(result_chirho)
    }

    pub fn shape_chirho(&self) -> (u8, u8) {
        (self.num_rows_chirho, self.num_cols_chirho)
    }
}

impl Default for BitMatrix64Chirho {
    fn default() -> Self {
        Self::new_chirho(64, 64)
    }
}

impl std::fmt::Debug for BitMatrix64Chirho {
    fn fmt(&self, f_chirho: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f_chirho, "BitMatrix64Chirho {}x{}:", self.num_rows_chirho, self.num_cols_chirho)?;
        for i_chirho in 0..self.num_rows_chirho {
            for j_chirho in 0..self.num_cols_chirho {
                write!(f_chirho, "{}", if self.get_chirho(i_chirho, j_chirho) { "1" } else { "." })?;
            }
            writeln!(f_chirho)?;
        }
        Ok(())
    }
}

/// Large bit matrix using word-striped storage
/// SIMD-friendly layout: words[row * num_words + word_idx]
#[derive(Clone, PartialEq, Eq)]
pub struct BitMatrixPackedChirho {
    /// Flattened word array
    words_chirho: Vec<Word64Chirho>,
    /// Number of 64-bit words per row
    words_per_row_chirho: usize,
    /// Dimensions
    num_rows_chirho: usize,
    num_cols_chirho: usize,
}

impl BitMatrixPackedChirho {
    pub fn new_chirho(rows_chirho: usize, cols_chirho: usize) -> Self {
        let words_per_row_chirho = (cols_chirho + 63) / 64;
        Self {
            words_chirho: vec![Word64Chirho::ZERO_CHIRHO; rows_chirho * words_per_row_chirho],
            words_per_row_chirho,
            num_rows_chirho: rows_chirho,
            num_cols_chirho: cols_chirho,
        }
    }

    #[inline]
    fn word_idx_chirho(&self, row_chirho: usize, col_chirho: usize) -> (usize, u32) {
        let word_offset_chirho = col_chirho / 64;
        let bit_offset_chirho = (col_chirho % 64) as u32;
        let flat_idx_chirho = row_chirho * self.words_per_row_chirho + word_offset_chirho;
        (flat_idx_chirho, bit_offset_chirho)
    }

    #[inline]
    pub fn get_chirho(&self, row_chirho: usize, col_chirho: usize) -> bool {
        let (flat_chirho, bit_chirho) = self.word_idx_chirho(row_chirho, col_chirho);
        self.words_chirho[flat_chirho].get_bit_chirho(bit_chirho)
    }

    #[inline]
    pub fn set_chirho(&mut self, row_chirho: usize, col_chirho: usize) {
        let (flat_chirho, bit_chirho) = self.word_idx_chirho(row_chirho, col_chirho);
        self.words_chirho[flat_chirho] = self.words_chirho[flat_chirho].set_bit_chirho(bit_chirho);
    }

    #[inline]
    pub fn clear_chirho(&mut self, row_chirho: usize, col_chirho: usize) {
        let (flat_chirho, bit_chirho) = self.word_idx_chirho(row_chirho, col_chirho);
        self.words_chirho[flat_chirho] = self.words_chirho[flat_chirho].clear_bit_chirho(bit_chirho);
    }

    /// AND - parallel across all words
    pub fn and_chirho(&self, other_chirho: &Self) -> Self {
        assert_eq!(self.words_chirho.len(), other_chirho.words_chirho.len());
        let mut result_chirho = self.clone();
        for (w_chirho, o_chirho) in result_chirho.words_chirho.iter_mut().zip(&other_chirho.words_chirho) {
            *w_chirho = *w_chirho & *o_chirho;
        }
        result_chirho
    }

    /// OR - parallel across all words
    pub fn or_chirho(&self, other_chirho: &Self) -> Self {
        assert_eq!(self.words_chirho.len(), other_chirho.words_chirho.len());
        let mut result_chirho = self.clone();
        for (w_chirho, o_chirho) in result_chirho.words_chirho.iter_mut().zip(&other_chirho.words_chirho) {
            *w_chirho = *w_chirho | *o_chirho;
        }
        result_chirho
    }

    pub fn popcount_chirho(&self) -> usize {
        self.words_chirho
            .iter()
            .map(|w_chirho| w_chirho.popcount_chirho() as usize)
            .sum()
    }

    pub fn shape_chirho(&self) -> (usize, usize) {
        (self.num_rows_chirho, self.num_cols_chirho)
    }
}

impl Default for BitMatrixPackedChirho {
    fn default() -> Self {
        Self::new_chirho(64, 64)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_word64_ops_chirho() {
        let a_chirho = Word64Chirho::new_chirho(0b1010);
        let b_chirho = Word64Chirho::new_chirho(0b1100);

        assert_eq!((a_chirho & b_chirho).0, 0b1000);
        assert_eq!((a_chirho | b_chirho).0, 0b1110);
        assert_eq!(a_chirho.popcount_chirho(), 2);
    }

    #[test]
    fn test_bit_iter_chirho() {
        let w_chirho = Word64Chirho::new_chirho(0b10101);
        let bits_chirho: Vec<_> = w_chirho.iter_ones_chirho().collect();
        assert_eq!(bits_chirho, vec![0, 2, 4]);
    }

    #[test]
    fn test_matrix64_and_chirho() {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(4, 4);

        a_chirho.set_chirho(0, 0);
        a_chirho.set_chirho(0, 1);
        b_chirho.set_chirho(0, 1);
        b_chirho.set_chirho(0, 2);

        let c_chirho = a_chirho.and_chirho(&b_chirho);
        assert!(!c_chirho.get_chirho(0, 0));
        assert!(c_chirho.get_chirho(0, 1));
        assert!(!c_chirho.get_chirho(0, 2));
    }

    #[test]
    fn test_matrix64_matmul_chirho() {
        // Graph adjacency: 0->1, 1->2
        let mut adj_chirho = BitMatrix64Chirho::new_chirho(3, 3);
        adj_chirho.set_chirho(0, 1); // 0 -> 1
        adj_chirho.set_chirho(1, 2); // 1 -> 2

        // Square = 2-hop paths: 0->2
        let sq_chirho = adj_chirho.matmul_chirho(&adj_chirho);
        assert!(sq_chirho.get_chirho(0, 2)); // 0 can reach 2 in 2 hops
        assert!(!sq_chirho.get_chirho(0, 0)); // No self-loop
    }

    #[test]
    fn test_transitive_closure_chirho() {
        // Chain: 0->1->2->3
        let mut adj_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        adj_chirho.set_chirho(0, 1);
        adj_chirho.set_chirho(1, 2);
        adj_chirho.set_chirho(2, 3);

        let tc_chirho = adj_chirho.transitive_closure_chirho();

        // 0 can reach everyone
        assert!(tc_chirho.get_chirho(0, 1));
        assert!(tc_chirho.get_chirho(0, 2));
        assert!(tc_chirho.get_chirho(0, 3));

        // 3 reaches nobody
        assert!(!tc_chirho.get_chirho(3, 0));
        assert!(!tc_chirho.get_chirho(3, 1));
        assert!(!tc_chirho.get_chirho(3, 2));
    }

    #[test]
    fn test_occurs_check_via_reachability_chirho() {
        // x = cons(x, nil) creates cycle: x -> x
        // In term graph: node for cons, edges to children
        // If transitive closure has self-loop, occurs check fails

        let mut term_graph_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        // Node 0 = x (var)
        // Node 1 = nil
        // Node 2 = cons(x, nil) = cons(0, 1)
        // Equation x = cons(x, nil) means 0 ≡ 2

        // Before unification: cons node points to children
        term_graph_chirho.set_chirho(2, 0); // cons -> x
        term_graph_chirho.set_chirho(2, 1); // cons -> nil

        // After x ≡ cons(x, nil): x (node 0) should point wherever node 2 points
        // This creates: x -> x (through cons structure)
        term_graph_chirho.set_chirho(0, 2); // x = cons(...) means x->cons

        let tc_chirho = term_graph_chirho.transitive_closure_chirho();

        // Check for cycle: can x reach itself?
        // x -> cons -> x creates self-loop
        assert!(tc_chirho.get_chirho(0, 0), "Occurs check: x contains itself!");
    }

    #[test]
    fn test_packed_matrix_chirho() {
        let mut m_chirho = BitMatrixPackedChirho::new_chirho(100, 100);

        m_chirho.set_chirho(50, 50);
        m_chirho.set_chirho(99, 99);

        assert!(m_chirho.get_chirho(50, 50));
        assert!(m_chirho.get_chirho(99, 99));
        assert!(!m_chirho.get_chirho(0, 0));

        assert_eq!(m_chirho.popcount_chirho(), 2);
    }
}
