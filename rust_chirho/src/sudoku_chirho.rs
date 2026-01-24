//! Sudoku Solver using 1-Bit Domain Propagation ☧
//!
//! Demonstrates miniKanren-style constraint propagation with bit-parallel ops.
//! Each cell has a 9-bit domain where bit i means value (i+1) is possible.
//!
//! Techniques:
//! - **Naked singles**: Cell with one possibility → propagate to peers
//! - **Hidden singles**: Value in only one cell of unit → assign it
//! - **Naked pairs**: Two cells with same two values → eliminate from peers
//!
//! "Whether therefore ye eat, or drink, or whatsoever ye do,
//!  do all to the glory of God." — 1 Corinthians 10:31

// ============================================================================
// Constants
// ============================================================================

const GRID_SIZE_CHIRHO: usize = 9;
const NUM_CELLS_CHIRHO: usize = 81;
const FULL_DOMAIN_CHIRHO: u16 = 0b111111111;
const EMPTY_DOMAIN_CHIRHO: u16 = 0;

// ============================================================================
// Precomputed lookup tables (computed at compile time)
// ============================================================================

/// All 27 units: 9 rows + 9 columns + 9 boxes
static ALL_UNITS_CHIRHO: [[u8; 9]; 27] = compute_all_units_chirho();

/// Peer indices for each cell (20 peers per cell)
static PEERS_CHIRHO: [[u8; 20]; 81] = compute_peers_chirho();

const fn compute_all_units_chirho() -> [[u8; 9]; 27] {
    let mut units_chirho = [[0u8; 9]; 27];
    let mut unit_idx_chirho = 0usize;

    // Rows (units 0-8)
    let mut row_chirho = 0usize;
    while row_chirho < 9 {
        let mut col_chirho = 0usize;
        while col_chirho < 9 {
            units_chirho[unit_idx_chirho][col_chirho] = (row_chirho * 9 + col_chirho) as u8;
            col_chirho += 1;
        }
        unit_idx_chirho += 1;
        row_chirho += 1;
    }

    // Columns (units 9-17)
    let mut col_chirho = 0usize;
    while col_chirho < 9 {
        let mut row_chirho = 0usize;
        while row_chirho < 9 {
            units_chirho[unit_idx_chirho][row_chirho] = (row_chirho * 9 + col_chirho) as u8;
            row_chirho += 1;
        }
        unit_idx_chirho += 1;
        col_chirho += 1;
    }

    // Boxes (units 18-26)
    let mut box_chirho = 0usize;
    while box_chirho < 9 {
        let box_row_chirho = (box_chirho / 3) * 3;
        let box_col_chirho = (box_chirho % 3) * 3;
        let mut i_chirho = 0usize;
        let mut r_chirho = 0usize;
        while r_chirho < 3 {
            let mut c_chirho = 0usize;
            while c_chirho < 3 {
                units_chirho[unit_idx_chirho][i_chirho] =
                    ((box_row_chirho + r_chirho) * 9 + box_col_chirho + c_chirho) as u8;
                i_chirho += 1;
                c_chirho += 1;
            }
            r_chirho += 1;
        }
        unit_idx_chirho += 1;
        box_chirho += 1;
    }

    units_chirho
}

const fn compute_peers_chirho() -> [[u8; 20]; 81] {
    let mut result_chirho = [[0u8; 20]; 81];
    let mut idx_chirho = 0usize;

    while idx_chirho < 81 {
        let row_chirho = idx_chirho / 9;
        let col_chirho = idx_chirho % 9;
        let box_row_chirho = (row_chirho / 3) * 3;
        let box_col_chirho = (col_chirho / 3) * 3;
        let mut count_chirho = 0usize;

        // Row peers
        let mut c_chirho = 0usize;
        while c_chirho < 9 {
            let peer_chirho = row_chirho * 9 + c_chirho;
            if peer_chirho != idx_chirho {
                result_chirho[idx_chirho][count_chirho] = peer_chirho as u8;
                count_chirho += 1;
            }
            c_chirho += 1;
        }

        // Column peers
        let mut r_chirho = 0usize;
        while r_chirho < 9 {
            let peer_chirho = r_chirho * 9 + col_chirho;
            if peer_chirho != idx_chirho {
                result_chirho[idx_chirho][count_chirho] = peer_chirho as u8;
                count_chirho += 1;
            }
            r_chirho += 1;
        }

        // Box peers (excluding row/col overlap)
        let mut br_chirho = box_row_chirho;
        while br_chirho < box_row_chirho + 3 {
            let mut bc_chirho = box_col_chirho;
            while bc_chirho < box_col_chirho + 3 {
                let peer_chirho = br_chirho * 9 + bc_chirho;
                if peer_chirho != idx_chirho && br_chirho != row_chirho && bc_chirho != col_chirho {
                    result_chirho[idx_chirho][count_chirho] = peer_chirho as u8;
                    count_chirho += 1;
                }
                bc_chirho += 1;
            }
            br_chirho += 1;
        }

        idx_chirho += 1;
    }

    result_chirho
}

// ============================================================================
// Helper functions
// ============================================================================

#[inline]
const fn value_to_domain_chirho(v_chirho: u8) -> u16 {
    1u16 << (v_chirho - 1)
}

#[inline]
const fn domain_to_value_chirho(d_chirho: u16) -> u8 {
    if d_chirho == 0 || (d_chirho & (d_chirho - 1)) != 0 {
        0
    } else {
        d_chirho.trailing_zeros() as u8 + 1
    }
}

#[inline]
const fn is_singleton_chirho(d_chirho: u16) -> bool {
    d_chirho != 0 && (d_chirho & (d_chirho - 1)) == 0
}

#[inline]
const fn lowest_bit_chirho(d_chirho: u16) -> u16 {
    d_chirho & d_chirho.wrapping_neg()
}

// ============================================================================
// Solver
// ============================================================================

/// Sudoku solver using bit-parallel domain propagation
#[derive(Clone)]
pub struct SudokuSolverChirho {
    domains_chirho: [u16; NUM_CELLS_CHIRHO],
    pub propagation_count_chirho: u32,
    pub branch_count_chirho: u32,
}

impl Default for SudokuSolverChirho {
    fn default() -> Self {
        Self::new_chirho()
    }
}

impl SudokuSolverChirho {
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: [FULL_DOMAIN_CHIRHO; NUM_CELLS_CHIRHO],
            propagation_count_chirho: 0,
            branch_count_chirho: 0,
        }
    }

    // ========================================================================
    // Core propagation
    // ========================================================================

    /// Eliminate value from a cell's domain, return false if empty
    #[inline]
    fn eliminate_chirho(&mut self, idx_chirho: usize, val_bit_chirho: u16) -> bool {
        self.domains_chirho[idx_chirho] &= !val_bit_chirho;
        self.domains_chirho[idx_chirho] != EMPTY_DOMAIN_CHIRHO
    }

    /// Assign value to cell (set domain to singleton)
    #[inline]
    fn assign_chirho(&mut self, idx_chirho: usize, val_bit_chirho: u16) {
        self.domains_chirho[idx_chirho] = val_bit_chirho;
    }

    /// Propagate singleton constraints to peers
    fn propagate_naked_singles_chirho(&mut self) -> bool {
        for idx_chirho in 0..NUM_CELLS_CHIRHO {
            let d_chirho = self.domains_chirho[idx_chirho];

            if d_chirho == EMPTY_DOMAIN_CHIRHO {
                return false;
            }

            if is_singleton_chirho(d_chirho) {
                for &peer_chirho in &PEERS_CHIRHO[idx_chirho] {
                    if !self.eliminate_chirho(peer_chirho as usize, d_chirho) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Find hidden singles in all units
    fn propagate_hidden_singles_chirho(&mut self) -> Result<bool, ()> {
        let mut changed_chirho = false;

        for unit_chirho in &ALL_UNITS_CHIRHO {
            for val_chirho in 0..GRID_SIZE_CHIRHO as u16 {
                let val_bit_chirho = 1u16 << val_chirho;

                // Count cells that can hold this value
                let (count_chirho, last_chirho) = self.count_value_in_unit_chirho(unit_chirho, val_bit_chirho);

                match count_chirho {
                    0 => return Err(()), // No cell can hold value = contradiction
                    1 if !is_singleton_chirho(self.domains_chirho[last_chirho]) => {
                        self.assign_chirho(last_chirho, val_bit_chirho);
                        changed_chirho = true;
                    }
                    _ => {}
                }
            }
        }

        Ok(changed_chirho)
    }

    /// Count cells in unit that can hold a value
    #[inline]
    fn count_value_in_unit_chirho(&self, unit_chirho: &[u8; 9], val_bit_chirho: u16) -> (u8, usize) {
        let mut count_chirho = 0u8;
        let mut last_chirho = 0usize;

        for &cell_chirho in unit_chirho {
            if self.domains_chirho[cell_chirho as usize] & val_bit_chirho != 0 {
                count_chirho += 1;
                last_chirho = cell_chirho as usize;
            }
        }

        (count_chirho, last_chirho)
    }

    /// Find and eliminate naked pairs
    fn propagate_naked_pairs_chirho(&mut self) -> bool {
        let mut changed_chirho = false;

        for unit_chirho in &ALL_UNITS_CHIRHO {
            changed_chirho |= self.find_naked_pairs_in_unit_chirho(unit_chirho);
        }

        changed_chirho
    }

    /// Find naked pairs within a single unit
    fn find_naked_pairs_in_unit_chirho(&mut self, unit_chirho: &[u8; 9]) -> bool {
        let mut changed_chirho = false;

        for i_chirho in 0..GRID_SIZE_CHIRHO {
            let cell_i_chirho = unit_chirho[i_chirho] as usize;
            let domain_i_chirho = self.domains_chirho[cell_i_chirho];

            if domain_i_chirho.count_ones() != 2 {
                continue;
            }

            for j_chirho in (i_chirho + 1)..GRID_SIZE_CHIRHO {
                let cell_j_chirho = unit_chirho[j_chirho] as usize;

                if self.domains_chirho[cell_j_chirho] == domain_i_chirho {
                    // Found naked pair - eliminate from other cells
                    for k_chirho in 0..GRID_SIZE_CHIRHO {
                        if k_chirho != i_chirho && k_chirho != j_chirho {
                            let cell_k_chirho = unit_chirho[k_chirho] as usize;
                            let old_chirho = self.domains_chirho[cell_k_chirho];
                            self.domains_chirho[cell_k_chirho] &= !domain_i_chirho;
                            changed_chirho |= self.domains_chirho[cell_k_chirho] != old_chirho;
                        }
                    }
                }
            }
        }

        changed_chirho
    }

    // ========================================================================
    // High-level propagation strategies
    // ========================================================================

    /// Basic propagation (naked singles only) - fast
    pub fn propagate_basic_chirho(&mut self) -> bool {
        let mut changed_chirho = true;
        while changed_chirho {
            changed_chirho = false;
            self.propagation_count_chirho += 1;

            for idx_chirho in 0..NUM_CELLS_CHIRHO {
                let d_chirho = self.domains_chirho[idx_chirho];

                if d_chirho == EMPTY_DOMAIN_CHIRHO {
                    return false;
                }

                if is_singleton_chirho(d_chirho) {
                    for &peer_chirho in &PEERS_CHIRHO[idx_chirho] {
                        let p_chirho = peer_chirho as usize;
                        let old_chirho = self.domains_chirho[p_chirho];
                        let new_chirho = old_chirho & !d_chirho;
                        if new_chirho != old_chirho {
                            self.domains_chirho[p_chirho] = new_chirho;
                            changed_chirho = true;
                            if new_chirho == EMPTY_DOMAIN_CHIRHO {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Full propagation (all techniques) - thorough
    pub fn propagate_chirho(&mut self) -> bool {
        loop {
            self.propagation_count_chirho += 1;

            if !self.propagate_naked_singles_chirho() {
                return false;
            }

            match self.propagate_hidden_singles_chirho() {
                Err(()) => return false,
                Ok(true) => continue,
                Ok(false) => {}
            }

            if self.propagate_naked_pairs_chirho() {
                continue;
            }

            break;
        }
        true
    }

    // ========================================================================
    // Solving
    // ========================================================================

    /// Find cell with smallest domain > 1 (MRV heuristic)
    fn find_branch_cell_chirho(&self) -> Option<usize> {
        let mut best_chirho: Option<(usize, u32)> = None;

        for (idx_chirho, &d_chirho) in self.domains_chirho.iter().enumerate() {
            let count_chirho = d_chirho.count_ones();
            if count_chirho > 1 {
                match best_chirho {
                    None => best_chirho = Some((idx_chirho, count_chirho)),
                    Some((_, best_count_chirho)) if count_chirho < best_count_chirho => {
                        best_chirho = Some((idx_chirho, count_chirho));
                        if count_chirho == 2 {
                            break; // Can't do better than 2
                        }
                    }
                    _ => {}
                }
            }
        }

        best_chirho.map(|(idx_chirho, _)| idx_chirho)
    }

    #[inline]
    pub fn is_solved_chirho(&self) -> bool {
        self.domains_chirho.iter().all(|&d_chirho| is_singleton_chirho(d_chirho))
    }

    fn count_unsolved_chirho(&self) -> usize {
        self.domains_chirho.iter().filter(|&&d_chirho| !is_singleton_chirho(d_chirho)).count()
    }

    /// Search with backtracking
    fn search_chirho(&mut self) -> bool {
        let branch_idx_chirho = match self.find_branch_cell_chirho() {
            Some(idx_chirho) => idx_chirho,
            None => return self.is_solved_chirho(),
        };

        // Save domains for backtracking (stack copy, no heap allocation)
        let saved_domains_chirho = self.domains_chirho;
        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        while domain_chirho != 0 {
            self.branch_count_chirho += 1;

            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1;

            // Restore and try this value
            self.domains_chirho = saved_domains_chirho;
            self.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if self.solve_adaptive_chirho() {
                return true;
            }
        }

        // Restore on complete failure
        self.domains_chirho = saved_domains_chirho;
        false
    }

    /// Adaptive solve - chooses strategy based on difficulty
    pub fn solve_adaptive_chirho(&mut self) -> bool {
        if !self.propagate_basic_chirho() {
            return false;
        }

        if self.is_solved_chirho() {
            return true;
        }

        // Hard puzzle? Use full propagation
        if self.count_unsolved_chirho() > 15 {
            if !self.propagate_chirho() {
                return false;
            }
            if self.is_solved_chirho() {
                return true;
            }
        }

        self.search_chirho()
    }

    /// Full solve (always uses advanced propagation)
    pub fn solve_chirho(&mut self) -> bool {
        if !self.propagate_chirho() {
            return false;
        }
        if self.is_solved_chirho() {
            return true;
        }
        self.search_chirho()
    }

    /// Fast solve (basic propagation only)
    pub fn solve_fast_chirho(&mut self) -> bool {
        if !self.propagate_basic_chirho() {
            return false;
        }
        if self.is_solved_chirho() {
            return true;
        }

        let branch_idx_chirho = match self.find_branch_cell_chirho() {
            Some(idx_chirho) => idx_chirho,
            None => return self.is_solved_chirho(),
        };

        // Save domains for backtracking
        let saved_domains_chirho = self.domains_chirho;
        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        while domain_chirho != 0 {
            self.branch_count_chirho += 1;
            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1;

            self.domains_chirho = saved_domains_chirho;
            self.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if self.solve_fast_chirho() {
                return true;
            }
        }

        self.domains_chirho = saved_domains_chirho;
        false
    }

    // ========================================================================
    // I/O
    // ========================================================================

    pub fn load_puzzle_chirho(&mut self, puzzle_chirho: &str) -> bool {
        let chars_chirho: Vec<char> = puzzle_chirho
            .chars()
            .filter(|c_chirho| !c_chirho.is_whitespace())
            .collect();

        if chars_chirho.len() != NUM_CELLS_CHIRHO {
            return false;
        }

        for (idx_chirho, ch_chirho) in chars_chirho.iter().enumerate() {
            if let Some(digit_chirho) = ch_chirho.to_digit(10) {
                if (1..=9).contains(&digit_chirho) {
                    self.domains_chirho[idx_chirho] = value_to_domain_chirho(digit_chirho as u8);
                }
            }
        }

        self.propagate_basic_chirho()
    }

    pub fn to_string_chirho(&self) -> String {
        self.domains_chirho
            .iter()
            .map(|&d_chirho| {
                let v_chirho = domain_to_value_chirho(d_chirho);
                if v_chirho > 0 { (b'0' + v_chirho) as char } else { '.' }
            })
            .collect()
    }

    #[inline]
    pub fn get_value_chirho(&self, idx_chirho: usize) -> u8 {
        domain_to_value_chirho(self.domains_chirho[idx_chirho])
    }

    #[inline]
    pub fn get_domain_chirho(&self, idx_chirho: usize) -> u16 {
        self.domains_chirho[idx_chirho]
    }
}

// ============================================================================
// Test puzzles
// ============================================================================

pub mod puzzles_chirho {
    pub const EASY_CHIRHO: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    pub const MEDIUM_CHIRHO: &str = "000000680030000000900800100000002074060090030780500000001007005000000020024000000";
    pub const HARD_CHIRHO: &str = "000000010400000000020000000000050407008000300001090000300400200050100000000806000";
    pub const ESCARGOT_CHIRHO: &str = "100007090030020008009600500005300900010080002600004000300000010040000007007000300";
    pub const INKALA_CHIRHO: &str = "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_easy_chirho() {
        let mut s_chirho = SudokuSolverChirho::new_chirho();
        assert!(s_chirho.load_puzzle_chirho(puzzles_chirho::EASY_CHIRHO));
        assert!(s_chirho.solve_adaptive_chirho());
        assert!(s_chirho.is_solved_chirho());
    }

    #[test]
    fn test_hard_chirho() {
        let mut s_chirho = SudokuSolverChirho::new_chirho();
        assert!(s_chirho.load_puzzle_chirho(puzzles_chirho::HARD_CHIRHO));
        assert!(s_chirho.solve_adaptive_chirho());
        assert!(s_chirho.is_solved_chirho());
    }

    #[test]
    fn test_escargot_chirho() {
        let mut s_chirho = SudokuSolverChirho::new_chirho();
        assert!(s_chirho.load_puzzle_chirho(puzzles_chirho::ESCARGOT_CHIRHO));
        assert!(s_chirho.solve_adaptive_chirho());
        assert!(s_chirho.is_solved_chirho());
    }

    #[test]
    fn test_peers_chirho() {
        for idx_chirho in 0..NUM_CELLS_CHIRHO {
            let unique_chirho: std::collections::HashSet<_> = PEERS_CHIRHO[idx_chirho].iter().collect();
            assert_eq!(unique_chirho.len(), 20);
        }
    }

    #[test]
    fn test_domain_conversion_chirho() {
        for v_chirho in 1..=9u8 {
            let d_chirho = value_to_domain_chirho(v_chirho);
            assert!(is_singleton_chirho(d_chirho));
            assert_eq!(domain_to_value_chirho(d_chirho), v_chirho);
        }
    }
}
