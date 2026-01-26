// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Sparse Bit Matrix ☧
//!
//! COO (Coordinate) format for sparse Boolean tensors.
//! Hardware-friendly: can be implemented with TCAM or parallel comparators.

use std::collections::{HashMap, HashSet};

/// Sparse bit matrix in COO format
/// Stores only the (row, col) pairs where value = 1
#[derive(Debug, Clone, Default)]
pub struct BitMatrixChirho {
    /// Set of (row, col) entries that are 1
    entries_chirho: HashSet<(u32, u32)>,
    /// Number of rows (for bounds checking)
    num_rows_chirho: u32,
    /// Number of columns
    num_cols_chirho: u32,
}

impl BitMatrixChirho {
    pub fn new(rows_chirho: u32, cols_chirho: u32) -> Self {
        Self {
            entries_chirho: HashSet::new(),
            num_rows_chirho: rows_chirho,
            num_cols_chirho: cols_chirho,
        }
    }

    /// Set entry to 1
    pub fn set_chirho(&mut self, row_chirho: u32, col_chirho: u32) {
        self.entries_chirho.insert((row_chirho, col_chirho));
        self.num_rows_chirho = self.num_rows_chirho.max(row_chirho + 1);
        self.num_cols_chirho = self.num_cols_chirho.max(col_chirho + 1);
    }

    /// Get entry value
    pub fn get_chirho(&self, row_chirho: u32, col_chirho: u32) -> bool {
        self.entries_chirho.contains(&(row_chirho, col_chirho))
    }

    /// Clear entry (set to 0)
    pub fn clear_chirho(&mut self, row_chirho: u32, col_chirho: u32) {
        self.entries_chirho.remove(&(row_chirho, col_chirho));
    }

    /// Get all entries in a row
    pub fn row_chirho(&self, row_chirho: u32) -> Vec<u32> {
        self.entries_chirho
            .iter()
            .filter(|(r, _)| *r == row_chirho)
            .map(|(_, c)| *c)
            .collect()
    }

    /// Get all entries in a column
    pub fn col_chirho(&self, col_chirho: u32) -> Vec<u32> {
        self.entries_chirho
            .iter()
            .filter(|(_, c)| *c == col_chirho)
            .map(|(r, _)| *r)
            .collect()
    }

    /// Number of 1s (nnz = number of non-zeros)
    pub fn nnz_chirho(&self) -> usize {
        self.entries_chirho.len()
    }

    /// Density = nnz / (rows * cols)
    pub fn density_chirho(&self) -> f64 {
        if self.num_rows_chirho == 0 || self.num_cols_chirho == 0 {
            return 0.0;
        }
        self.entries_chirho.len() as f64 / (self.num_rows_chirho as f64 * self.num_cols_chirho as f64)
    }

    /// Bitwise AND of two matrices (intersection)
    pub fn and_chirho(&self, other_chirho: &BitMatrixChirho) -> BitMatrixChirho {
        let mut result_chirho = BitMatrixChirho::new(
            self.num_rows_chirho.max(other_chirho.num_rows_chirho),
            self.num_cols_chirho.max(other_chirho.num_cols_chirho),
        );

        for entry_chirho in &self.entries_chirho {
            if other_chirho.entries_chirho.contains(entry_chirho) {
                result_chirho.entries_chirho.insert(*entry_chirho);
            }
        }

        result_chirho
    }

    /// Bitwise OR of two matrices (union)
    pub fn or_chirho(&self, other_chirho: &BitMatrixChirho) -> BitMatrixChirho {
        let mut result_chirho = BitMatrixChirho::new(
            self.num_rows_chirho.max(other_chirho.num_rows_chirho),
            self.num_cols_chirho.max(other_chirho.num_cols_chirho),
        );

        result_chirho.entries_chirho = self.entries_chirho.union(&other_chirho.entries_chirho).copied().collect();

        result_chirho
    }

    /// Matrix multiplication (Boolean semiring: OR for +, AND for *)
    pub fn matmul_chirho(&self, other_chirho: &BitMatrixChirho) -> BitMatrixChirho {
        let mut result_chirho = BitMatrixChirho::new(self.num_rows_chirho, other_chirho.num_cols_chirho);

        // Group entries by row for self, by col for other
        let mut self_by_row_chirho: HashMap<u32, Vec<u32>> = HashMap::new();
        for (r, c) in &self.entries_chirho {
            self_by_row_chirho.entry(*r).or_default().push(*c);
        }

        let mut other_by_col_chirho: HashMap<u32, Vec<u32>> = HashMap::new();
        for (r, c) in &other_chirho.entries_chirho {
            other_by_col_chirho.entry(*c).or_default().push(*r);
        }

        // For each row in self
        for (row_chirho, cols_chirho) in &self_by_row_chirho {
            // For each column in other
            for (col_chirho, rows_chirho) in &other_by_col_chirho {
                // Check if there's any shared index (AND gives 1)
                let cols_set_chirho: HashSet<_> = cols_chirho.iter().collect();
                for r_chirho in rows_chirho {
                    if cols_set_chirho.contains(r_chirho) {
                        result_chirho.set_chirho(*row_chirho, *col_chirho);
                        break; // OR: one match is enough
                    }
                }
            }
        }

        result_chirho
    }

    /// Transpose
    pub fn transpose_chirho(&self) -> BitMatrixChirho {
        let mut result_chirho = BitMatrixChirho::new(self.num_cols_chirho, self.num_rows_chirho);
        for (r, c) in &self.entries_chirho {
            result_chirho.entries_chirho.insert((*c, *r));
        }
        result_chirho
    }

    /// Get dimensions
    pub fn shape_chirho(&self) -> (u32, u32) {
        (self.num_rows_chirho, self.num_cols_chirho)
    }

    /// Iterate over all entries
    pub fn iter_chirho(&self) -> impl Iterator<Item = &(u32, u32)> {
        self.entries_chirho.iter()
    }
}

/// Sparse 3D Boolean tensor (for relations like appendo)
#[derive(Debug, Clone, Default)]
pub struct BitTensor3Chirho {
    /// Set of (i, j, k) entries that are 1
    entries_chirho: HashSet<(u32, u32, u32)>,
}

impl BitTensor3Chirho {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_chirho(&mut self, i_chirho: u32, j_chirho: u32, k_chirho: u32) {
        self.entries_chirho.insert((i_chirho, j_chirho, k_chirho));
    }

    pub fn get_chirho(&self, i_chirho: u32, j_chirho: u32, k_chirho: u32) -> bool {
        self.entries_chirho.contains(&(i_chirho, j_chirho, k_chirho))
    }

    pub fn nnz_chirho(&self) -> usize {
        self.entries_chirho.len()
    }

    /// Contract along index k: result[i,j] = OR_k (self[i,k] AND other[k,j])
    /// This is relation composition!
    pub fn contract_chirho(&self, dim_chirho: usize, values_chirho: &HashSet<u32>) -> BitMatrixChirho {
        let mut result_chirho = BitMatrixChirho::new(0, 0);

        for (i, j, k) in &self.entries_chirho {
            let selected_chirho = match dim_chirho {
                0 => values_chirho.contains(i),
                1 => values_chirho.contains(j),
                2 => values_chirho.contains(k),
                _ => false,
            };

            if selected_chirho {
                let (r, c) = match dim_chirho {
                    0 => (*j, *k),
                    1 => (*i, *k),
                    2 => (*i, *j),
                    _ => continue,
                };
                result_chirho.set_chirho(r, c);
            }
        }

        result_chirho
    }

    /// Query: given two fixed dimensions, get matching third dimension values
    pub fn query_chirho(&self, fixed_dims_chirho: [(usize, u32); 2]) -> Vec<u32> {
        let mut results_chirho = Vec::new();

        for (i, j, k) in &self.entries_chirho {
            let vals_chirho = [*i, *j, *k];
            let mut matches_chirho = true;

            for (dim_chirho, val_chirho) in fixed_dims_chirho {
                if vals_chirho[dim_chirho] != val_chirho {
                    matches_chirho = false;
                    break;
                }
            }

            if matches_chirho {
                let free_dim_chirho = 3 - fixed_dims_chirho[0].0 - fixed_dims_chirho[1].0;
                results_chirho.push(vals_chirho[free_dim_chirho]);
            }
        }

        results_chirho
    }

    pub fn iter_chirho(&self) -> impl Iterator<Item = &(u32, u32, u32)> {
        self.entries_chirho.iter()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bitmatrix_basic_chirho() {
        let mut m_chirho = BitMatrixChirho::new(10, 10);

        m_chirho.set_chirho(0, 0);
        m_chirho.set_chirho(0, 1);
        m_chirho.set_chirho(1, 1);

        assert!(m_chirho.get_chirho(0, 0));
        assert!(m_chirho.get_chirho(0, 1));
        assert!(!m_chirho.get_chirho(0, 2));
        assert_eq!(m_chirho.nnz_chirho(), 3);
    }

    #[test]
    fn test_bitmatrix_and_chirho() {
        let mut a_chirho = BitMatrixChirho::new(10, 10);
        let mut b_chirho = BitMatrixChirho::new(10, 10);

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
    fn test_tensor3_chirho() {
        let mut t_chirho = BitTensor3Chirho::new();

        // appendo: [0] ++ [1] = [0, 1]
        // Encode as (list_id, suffix_id, result_id)
        t_chirho.set_chirho(1, 2, 3); // [0] ++ [1] = [0,1]
        t_chirho.set_chirho(0, 3, 3); // [] ++ [0,1] = [0,1]

        assert!(t_chirho.get_chirho(1, 2, 3));
        assert!(!t_chirho.get_chirho(1, 1, 1));
    }
}
