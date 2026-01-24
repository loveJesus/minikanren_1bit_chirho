//! SIMD-Accelerated Bit Operations ☧
//!
//! Bulk AND/OR operations using wide registers.
//! Falls back to scalar when SIMD unavailable.
//!
//! Target: 256-bit AVX2 or 128-bit SSE for x86_64,
//! NEON for ARM.

/// Bulk AND of two slices of u64 words
/// Result stored in `dst_chirho`
#[inline]
pub fn bulk_and_chirho(dst_chirho: &mut [u64], a_chirho: &[u64], b_chirho: &[u64]) {
    debug_assert_eq!(dst_chirho.len(), a_chirho.len());
    debug_assert_eq!(a_chirho.len(), b_chirho.len());

    // Process 4 words at a time (256 bits = 4 × 64)
    let chunks_chirho = a_chirho.len() / 4;
    let remainder_chirho = a_chirho.len() % 4;

    // Main loop - compiler should auto-vectorize this
    for i_chirho in 0..chunks_chirho {
        let base_chirho = i_chirho * 4;
        dst_chirho[base_chirho] = a_chirho[base_chirho] & b_chirho[base_chirho];
        dst_chirho[base_chirho + 1] = a_chirho[base_chirho + 1] & b_chirho[base_chirho + 1];
        dst_chirho[base_chirho + 2] = a_chirho[base_chirho + 2] & b_chirho[base_chirho + 2];
        dst_chirho[base_chirho + 3] = a_chirho[base_chirho + 3] & b_chirho[base_chirho + 3];
    }

    // Handle remainder
    let base_chirho = chunks_chirho * 4;
    for i_chirho in 0..remainder_chirho {
        dst_chirho[base_chirho + i_chirho] = a_chirho[base_chirho + i_chirho] & b_chirho[base_chirho + i_chirho];
    }
}

/// Bulk OR of two slices of u64 words
#[inline]
pub fn bulk_or_chirho(dst_chirho: &mut [u64], a_chirho: &[u64], b_chirho: &[u64]) {
    debug_assert_eq!(dst_chirho.len(), a_chirho.len());
    debug_assert_eq!(a_chirho.len(), b_chirho.len());

    let chunks_chirho = a_chirho.len() / 4;
    let remainder_chirho = a_chirho.len() % 4;

    for i_chirho in 0..chunks_chirho {
        let base_chirho = i_chirho * 4;
        dst_chirho[base_chirho] = a_chirho[base_chirho] | b_chirho[base_chirho];
        dst_chirho[base_chirho + 1] = a_chirho[base_chirho + 1] | b_chirho[base_chirho + 1];
        dst_chirho[base_chirho + 2] = a_chirho[base_chirho + 2] | b_chirho[base_chirho + 2];
        dst_chirho[base_chirho + 3] = a_chirho[base_chirho + 3] | b_chirho[base_chirho + 3];
    }

    let base_chirho = chunks_chirho * 4;
    for i_chirho in 0..remainder_chirho {
        dst_chirho[base_chirho + i_chirho] = a_chirho[base_chirho + i_chirho] | b_chirho[base_chirho + i_chirho];
    }
}

/// Bulk XOR of two slices
#[inline]
pub fn bulk_xor_chirho(dst_chirho: &mut [u64], a_chirho: &[u64], b_chirho: &[u64]) {
    debug_assert_eq!(dst_chirho.len(), a_chirho.len());
    debug_assert_eq!(a_chirho.len(), b_chirho.len());

    let chunks_chirho = a_chirho.len() / 4;
    let remainder_chirho = a_chirho.len() % 4;

    for i_chirho in 0..chunks_chirho {
        let base_chirho = i_chirho * 4;
        dst_chirho[base_chirho] = a_chirho[base_chirho] ^ b_chirho[base_chirho];
        dst_chirho[base_chirho + 1] = a_chirho[base_chirho + 1] ^ b_chirho[base_chirho + 1];
        dst_chirho[base_chirho + 2] = a_chirho[base_chirho + 2] ^ b_chirho[base_chirho + 2];
        dst_chirho[base_chirho + 3] = a_chirho[base_chirho + 3] ^ b_chirho[base_chirho + 3];
    }

    let base_chirho = chunks_chirho * 4;
    for i_chirho in 0..remainder_chirho {
        dst_chirho[base_chirho + i_chirho] = a_chirho[base_chirho + i_chirho] ^ b_chirho[base_chirho + i_chirho];
    }
}

/// Bulk NOT
#[inline]
pub fn bulk_not_chirho(dst_chirho: &mut [u64], a_chirho: &[u64]) {
    debug_assert_eq!(dst_chirho.len(), a_chirho.len());

    let chunks_chirho = a_chirho.len() / 4;
    let remainder_chirho = a_chirho.len() % 4;

    for i_chirho in 0..chunks_chirho {
        let base_chirho = i_chirho * 4;
        dst_chirho[base_chirho] = !a_chirho[base_chirho];
        dst_chirho[base_chirho + 1] = !a_chirho[base_chirho + 1];
        dst_chirho[base_chirho + 2] = !a_chirho[base_chirho + 2];
        dst_chirho[base_chirho + 3] = !a_chirho[base_chirho + 3];
    }

    let base_chirho = chunks_chirho * 4;
    for i_chirho in 0..remainder_chirho {
        dst_chirho[base_chirho + i_chirho] = !a_chirho[base_chirho + i_chirho];
    }
}

/// Count total set bits across all words
#[inline]
pub fn bulk_popcount_chirho(words_chirho: &[u64]) -> usize {
    words_chirho.iter().map(|w_chirho| w_chirho.count_ones() as usize).sum()
}

/// Check if any bit is set
#[inline]
pub fn bulk_any_chirho(words_chirho: &[u64]) -> bool {
    let chunks_chirho = words_chirho.len() / 4;
    let remainder_chirho = words_chirho.len() % 4;

    // Process in chunks - OR together then check
    for i_chirho in 0..chunks_chirho {
        let base_chirho = i_chirho * 4;
        let combined_chirho = words_chirho[base_chirho]
            | words_chirho[base_chirho + 1]
            | words_chirho[base_chirho + 2]
            | words_chirho[base_chirho + 3];
        if combined_chirho != 0 {
            return true;
        }
    }

    let base_chirho = chunks_chirho * 4;
    for i_chirho in 0..remainder_chirho {
        if words_chirho[base_chirho + i_chirho] != 0 {
            return true;
        }
    }

    false
}

/// Check if all bits are zero
#[inline]
pub fn bulk_is_zero_chirho(words_chirho: &[u64]) -> bool {
    !bulk_any_chirho(words_chirho)
}

/// SIMD-friendly bit matrix (aligned storage)
#[derive(Clone)]
#[repr(align(32))] // 256-bit alignment for AVX
pub struct AlignedBitMatrixChirho {
    /// Rows stored as contiguous u64 words
    data_chirho: Vec<u64>,
    /// Words per row (padded to multiple of 4 for SIMD)
    words_per_row_chirho: usize,
    /// Actual dimensions
    num_rows_chirho: usize,
    num_cols_chirho: usize,
}

impl AlignedBitMatrixChirho {
    pub fn new_chirho(rows_chirho: usize, cols_chirho: usize) -> Self {
        // Pad words_per_row to multiple of 4 for SIMD
        let min_words_chirho = (cols_chirho + 63) / 64;
        let words_per_row_chirho = ((min_words_chirho + 3) / 4) * 4;

        Self {
            data_chirho: vec![0u64; rows_chirho * words_per_row_chirho],
            words_per_row_chirho,
            num_rows_chirho: rows_chirho,
            num_cols_chirho: cols_chirho,
        }
    }

    #[inline]
    fn row_slice_chirho(&self, row_chirho: usize) -> &[u64] {
        let start_chirho = row_chirho * self.words_per_row_chirho;
        &self.data_chirho[start_chirho..start_chirho + self.words_per_row_chirho]
    }

    #[inline]
    fn row_slice_mut_chirho(&mut self, row_chirho: usize) -> &mut [u64] {
        let start_chirho = row_chirho * self.words_per_row_chirho;
        &mut self.data_chirho[start_chirho..start_chirho + self.words_per_row_chirho]
    }

    #[inline]
    pub fn get_chirho(&self, row_chirho: usize, col_chirho: usize) -> bool {
        let word_idx_chirho = col_chirho / 64;
        let bit_idx_chirho = col_chirho % 64;
        let row_start_chirho = row_chirho * self.words_per_row_chirho;
        (self.data_chirho[row_start_chirho + word_idx_chirho] >> bit_idx_chirho) & 1 == 1
    }

    #[inline]
    pub fn set_chirho(&mut self, row_chirho: usize, col_chirho: usize) {
        let word_idx_chirho = col_chirho / 64;
        let bit_idx_chirho = col_chirho % 64;
        let row_start_chirho = row_chirho * self.words_per_row_chirho;
        self.data_chirho[row_start_chirho + word_idx_chirho] |= 1u64 << bit_idx_chirho;
    }

    #[inline]
    pub fn clear_chirho(&mut self, row_chirho: usize, col_chirho: usize) {
        let word_idx_chirho = col_chirho / 64;
        let bit_idx_chirho = col_chirho % 64;
        let row_start_chirho = row_chirho * self.words_per_row_chirho;
        self.data_chirho[row_start_chirho + word_idx_chirho] &= !(1u64 << bit_idx_chirho);
    }

    /// SIMD-accelerated element-wise AND
    pub fn and_chirho(&self, other_chirho: &Self) -> Self {
        assert_eq!(self.num_rows_chirho, other_chirho.num_rows_chirho);
        assert_eq!(self.words_per_row_chirho, other_chirho.words_per_row_chirho);

        let mut result_chirho = Self::new_chirho(self.num_rows_chirho, self.num_cols_chirho);
        bulk_and_chirho(&mut result_chirho.data_chirho, &self.data_chirho, &other_chirho.data_chirho);
        result_chirho
    }

    /// SIMD-accelerated element-wise OR
    pub fn or_chirho(&self, other_chirho: &Self) -> Self {
        assert_eq!(self.num_rows_chirho, other_chirho.num_rows_chirho);
        assert_eq!(self.words_per_row_chirho, other_chirho.words_per_row_chirho);

        let mut result_chirho = Self::new_chirho(self.num_rows_chirho, self.num_cols_chirho);
        bulk_or_chirho(&mut result_chirho.data_chirho, &self.data_chirho, &other_chirho.data_chirho);
        result_chirho
    }

    /// Boolean matrix multiplication
    pub fn matmul_chirho(&self, other_chirho: &Self) -> Self {
        assert_eq!(self.num_cols_chirho, other_chirho.num_rows_chirho);

        let mut result_chirho = Self::new_chirho(self.num_rows_chirho, other_chirho.num_cols_chirho);

        for i_chirho in 0..self.num_rows_chirho {
            let row_a_chirho = self.row_slice_chirho(i_chirho);

            // For each set bit in row_a, OR in the corresponding row from other
            for (word_idx_chirho, &word_chirho) in row_a_chirho.iter().enumerate() {
                if word_chirho == 0 {
                    continue;
                }

                let mut bits_chirho = word_chirho;
                while bits_chirho != 0 {
                    let bit_pos_chirho = bits_chirho.trailing_zeros() as usize;
                    let k_chirho = word_idx_chirho * 64 + bit_pos_chirho;

                    if k_chirho < other_chirho.num_rows_chirho {
                        // Copy row_b to avoid aliasing
                        let row_b_chirho: Vec<u64> = other_chirho.row_slice_chirho(k_chirho).to_vec();
                        let row_r_chirho = result_chirho.row_slice_mut_chirho(i_chirho);
                        // In-place OR
                        for (dst_chirho, src_chirho) in row_r_chirho.iter_mut().zip(&row_b_chirho) {
                            *dst_chirho |= *src_chirho;
                        }
                    }

                    bits_chirho &= bits_chirho - 1; // Clear lowest bit
                }
            }
        }

        result_chirho
    }

    /// Transitive closure using repeated squaring
    pub fn transitive_closure_chirho(&self) -> Self {
        assert_eq!(self.num_rows_chirho, self.num_cols_chirho);

        let mut current_chirho = self.clone();
        let n_chirho = self.num_rows_chirho;

        // log2(n) iterations of squaring
        let iterations_chirho = (n_chirho as f64).log2().ceil() as usize + 1;

        for _ in 0..iterations_chirho {
            let squared_chirho = current_chirho.matmul_chirho(&current_chirho);
            let next_chirho = current_chirho.or_chirho(&squared_chirho);

            // Check for fixpoint
            if next_chirho.data_chirho == current_chirho.data_chirho {
                break;
            }
            current_chirho = next_chirho;
        }

        current_chirho
    }

    pub fn popcount_chirho(&self) -> usize {
        bulk_popcount_chirho(&self.data_chirho)
    }

    pub fn shape_chirho(&self) -> (usize, usize) {
        (self.num_rows_chirho, self.num_cols_chirho)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bulk_and_chirho() {
        let a_chirho = vec![0xFFFF_0000_FFFF_0000u64; 8];
        let b_chirho = vec![0xFF00_FF00_FF00_FF00u64; 8];
        let mut dst_chirho = vec![0u64; 8];

        bulk_and_chirho(&mut dst_chirho, &a_chirho, &b_chirho);

        for d_chirho in dst_chirho {
            assert_eq!(d_chirho, 0xFF00_0000_FF00_0000u64);
        }
    }

    #[test]
    fn test_bulk_or_chirho() {
        let a_chirho = vec![0xF0F0_F0F0u64; 4];
        let b_chirho = vec![0x0F0F_0F0Fu64; 4];
        let mut dst_chirho = vec![0u64; 4];

        bulk_or_chirho(&mut dst_chirho, &a_chirho, &b_chirho);

        for d_chirho in dst_chirho {
            assert_eq!(d_chirho, 0xFFFF_FFFFu64);
        }
    }

    #[test]
    fn test_aligned_matrix_basic_chirho() {
        let mut m_chirho = AlignedBitMatrixChirho::new_chirho(100, 200);

        m_chirho.set_chirho(50, 150);
        assert!(m_chirho.get_chirho(50, 150));
        assert!(!m_chirho.get_chirho(50, 151));
    }

    #[test]
    fn test_aligned_matrix_matmul_chirho() {
        let mut a_chirho = AlignedBitMatrixChirho::new_chirho(4, 4);
        a_chirho.set_chirho(0, 1);
        a_chirho.set_chirho(1, 2);
        a_chirho.set_chirho(2, 3);

        let sq_chirho = a_chirho.matmul_chirho(&a_chirho);

        // A^2[0,2] = A[0,1] AND A[1,2] = true
        assert!(sq_chirho.get_chirho(0, 2));
        // A^2[1,3] = A[1,2] AND A[2,3] = true
        assert!(sq_chirho.get_chirho(1, 3));
    }

    #[test]
    fn test_aligned_tc_chirho() {
        let mut adj_chirho = AlignedBitMatrixChirho::new_chirho(5, 5);
        adj_chirho.set_chirho(0, 1);
        adj_chirho.set_chirho(1, 2);
        adj_chirho.set_chirho(2, 3);
        adj_chirho.set_chirho(3, 4);

        let tc_chirho = adj_chirho.transitive_closure_chirho();

        // 0 can reach all
        assert!(tc_chirho.get_chirho(0, 1));
        assert!(tc_chirho.get_chirho(0, 2));
        assert!(tc_chirho.get_chirho(0, 3));
        assert!(tc_chirho.get_chirho(0, 4));
    }
}
