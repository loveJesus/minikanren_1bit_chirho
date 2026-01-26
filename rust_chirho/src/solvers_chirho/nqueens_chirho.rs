// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! N-Queens Solver using 1-Bit Domain Propagation ☧
//!
//! Place N queens on an N×N board such that no two attack each other.
//! Uses bit-parallel constraint propagation for blazing speed.
//!
//! Supports boards up to 64×64 using 64-bit domains.
//! Each row has one queen; domain bits represent valid columns.
//!
//! "The queen's wrath is as messengers of death:
//!  but a wise man will pacify it." — Proverbs 16:14

// ============================================================================
// Solver
// ============================================================================

/// N-Queens solver using bit-parallel domain propagation
pub struct NQueensSolverChirho {
    n_chirho: usize,
    pub solution_count_chirho: u64,
    pub branch_count_chirho: u64,
}

impl NQueensSolverChirho {
    /// Create solver for N×N board (N ≤ 64)
    pub fn new_chirho(n_chirho: usize) -> Self {
        assert!(n_chirho <= 64, "Board size must be ≤ 64");
        Self {
            n_chirho,
            solution_count_chirho: 0,
            branch_count_chirho: 0,
        }
    }

    /// Full column mask for this board size
    #[inline]
    fn full_mask_chirho(&self) -> u64 {
        if self.n_chirho == 64 { u64::MAX } else { (1u64 << self.n_chirho) - 1 }
    }

    // ========================================================================
    // Core search (stack-based, no allocation in hot path)
    // ========================================================================

    /// Find first solution (returns column positions, or None)
    pub fn solve_one_chirho(&mut self) -> Option<Vec<u8>> {
        let mut cols_chirho = vec![0u8; self.n_chirho];

        if self.search_one_chirho(0, &mut cols_chirho, 0, 0, 0) {
            Some(cols_chirho)
        } else {
            None
        }
    }

    fn search_one_chirho(
        &mut self,
        row_chirho: usize,
        cols_chirho: &mut [u8],
        col_mask_chirho: u64,
        diag1_mask_chirho: u64,
        diag2_mask_chirho: u64,
    ) -> bool {
        if row_chirho == self.n_chirho {
            return true;
        }

        // Available columns = not attacked by any constraint
        let mut available_chirho = self.full_mask_chirho()
            & !col_mask_chirho
            & !diag1_mask_chirho
            & !diag2_mask_chirho;

        while available_chirho != 0 {
            self.branch_count_chirho += 1;

            // Pick lowest available column
            let col_bit_chirho = available_chirho & available_chirho.wrapping_neg();
            let col_chirho = col_bit_chirho.trailing_zeros() as u8;
            available_chirho &= available_chirho - 1;

            cols_chirho[row_chirho] = col_chirho;

            // Update constraints for next row
            let new_col_chirho = col_mask_chirho | col_bit_chirho;
            let new_diag1_chirho = (diag1_mask_chirho | col_bit_chirho) << 1;
            let new_diag2_chirho = (diag2_mask_chirho | col_bit_chirho) >> 1;

            if self.search_one_chirho(row_chirho + 1, cols_chirho, new_col_chirho, new_diag1_chirho, new_diag2_chirho) {
                return true;
            }
        }

        false
    }

    /// Count all solutions (fast, no collection overhead)
    pub fn count_solutions_chirho(&mut self) -> u64 {
        self.solution_count_chirho = 0;
        self.count_recursive_chirho(0, 0, 0, 0);
        self.solution_count_chirho
    }

    fn count_recursive_chirho(
        &mut self,
        row_chirho: usize,
        col_mask_chirho: u64,
        diag1_mask_chirho: u64,
        diag2_mask_chirho: u64,
    ) {
        if row_chirho == self.n_chirho {
            self.solution_count_chirho += 1;
            return;
        }

        let mut available_chirho = self.full_mask_chirho()
            & !col_mask_chirho
            & !diag1_mask_chirho
            & !diag2_mask_chirho;

        while available_chirho != 0 {
            self.branch_count_chirho += 1;

            let col_bit_chirho = available_chirho & available_chirho.wrapping_neg();
            available_chirho &= available_chirho - 1;

            self.count_recursive_chirho(
                row_chirho + 1,
                col_mask_chirho | col_bit_chirho,
                (diag1_mask_chirho | col_bit_chirho) << 1,
                (diag2_mask_chirho | col_bit_chirho) >> 1,
            );
        }
    }

    /// Collect all solutions
    pub fn solve_all_chirho(&mut self) -> Vec<Vec<u8>> {
        let mut solutions_chirho = Vec::new();
        let mut current_chirho = vec![0u8; self.n_chirho];
        self.collect_recursive_chirho(0, 0, 0, 0, &mut current_chirho, &mut solutions_chirho);
        solutions_chirho
    }

    fn collect_recursive_chirho(
        &mut self,
        row_chirho: usize,
        col_mask_chirho: u64,
        diag1_mask_chirho: u64,
        diag2_mask_chirho: u64,
        current_chirho: &mut Vec<u8>,
        solutions_chirho: &mut Vec<Vec<u8>>,
    ) {
        if row_chirho == self.n_chirho {
            solutions_chirho.push(current_chirho.clone());
            return;
        }

        let mut available_chirho = self.full_mask_chirho()
            & !col_mask_chirho
            & !diag1_mask_chirho
            & !diag2_mask_chirho;

        while available_chirho != 0 {
            let col_bit_chirho = available_chirho & available_chirho.wrapping_neg();
            let col_chirho = col_bit_chirho.trailing_zeros() as u8;
            available_chirho &= available_chirho - 1;

            current_chirho[row_chirho] = col_chirho;

            self.collect_recursive_chirho(
                row_chirho + 1,
                col_mask_chirho | col_bit_chirho,
                (diag1_mask_chirho | col_bit_chirho) << 1,
                (diag2_mask_chirho | col_bit_chirho) >> 1,
                current_chirho,
                solutions_chirho,
            );
        }
    }

    // ========================================================================
    // Display
    // ========================================================================

    /// Pretty-print a solution
    pub fn print_solution_chirho(cols_chirho: &[u8]) {
        let n_chirho = cols_chirho.len();
        for row_chirho in 0..n_chirho {
            for col_chirho in 0..n_chirho {
                print!("{} ", if cols_chirho[row_chirho] == col_chirho as u8 { 'Q' } else { '.' });
            }
            println!();
        }
    }
}

// ============================================================================
// Reference data
// ============================================================================

/// Known solution counts for N-Queens (for verification)
pub const KNOWN_SOLUTIONS_CHIRHO: &[u64] = &[
    1,        // N=1
    0,        // N=2
    0,        // N=3
    2,        // N=4
    10,       // N=5
    4,        // N=6
    40,       // N=7
    92,       // N=8
    352,      // N=9
    724,      // N=10
    2680,     // N=11
    14200,    // N=12
    73712,    // N=13
    365596,   // N=14
    2279184,  // N=15
];

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_4_queens_chirho() {
        let mut s_chirho = NQueensSolverChirho::new_chirho(4);
        assert_eq!(s_chirho.count_solutions_chirho(), 2);
    }

    #[test]
    fn test_8_queens_chirho() {
        let mut s_chirho = NQueensSolverChirho::new_chirho(8);
        assert_eq!(s_chirho.count_solutions_chirho(), 92);
    }

    #[test]
    fn test_12_queens_chirho() {
        let mut s_chirho = NQueensSolverChirho::new_chirho(12);
        assert_eq!(s_chirho.count_solutions_chirho(), 14200);
    }

    #[test]
    fn test_solve_one_valid_chirho() {
        let mut s_chirho = NQueensSolverChirho::new_chirho(8);
        let sol_chirho = s_chirho.solve_one_chirho().unwrap();

        // Verify no attacks
        for i_chirho in 0..8 {
            for j_chirho in (i_chirho + 1)..8 {
                let ci_chirho = sol_chirho[i_chirho] as i32;
                let cj_chirho = sol_chirho[j_chirho] as i32;
                let row_diff_chirho = (j_chirho - i_chirho) as i32;

                assert_ne!(ci_chirho, cj_chirho, "Same column");
                assert_ne!((cj_chirho - ci_chirho).abs(), row_diff_chirho, "Same diagonal");
            }
        }
    }

    #[test]
    fn test_known_solutions_chirho() {
        for n_chirho in 1..=10 {
            let mut s_chirho = NQueensSolverChirho::new_chirho(n_chirho);
            let count_chirho = s_chirho.count_solutions_chirho();
            assert_eq!(count_chirho, KNOWN_SOLUTIONS_CHIRHO[n_chirho - 1], "N={}", n_chirho);
        }
    }

    #[test]
    fn test_20_queens_chirho() {
        let mut s_chirho = NQueensSolverChirho::new_chirho(20);
        assert!(s_chirho.solve_one_chirho().is_some());
    }

    #[test]
    fn test_32_queens_chirho() {
        // 32×32 is fast; 64×64 works but takes longer
        let mut s_chirho = NQueensSolverChirho::new_chirho(32);
        assert!(s_chirho.solve_one_chirho().is_some());
    }
}
