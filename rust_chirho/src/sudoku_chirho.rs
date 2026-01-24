//! Sudoku Solver using 1-Bit Domain Propagation ☧
//!
//! Demonstrates practical use of miniKanren-style constraint propagation
//! with bit-parallel domain operations.
//!
//! Each cell has a 9-bit domain where bit i means value (i+1) is possible.
//! Constraints (row, column, box) are applied via bitwise AND.
//!
//! Advanced techniques:
//! - **Naked singles**: Cell with one possibility → assign it
//! - **Hidden singles**: Value possible in only one cell of unit → assign it
//! - **Naked pairs/triples**: N cells with same N values → eliminate from peers
//!
//! "Whether therefore ye eat, or drink, or whatsoever ye do,
//!  do all to the glory of God." — 1 Corinthians 10:31

/// Full domain: all values 1-9 possible (bits 0-8 set)
const FULL_DOMAIN_CHIRHO: u16 = 0b111111111;

/// Empty domain: no values possible (failure state)
const EMPTY_DOMAIN_CHIRHO: u16 = 0;

/// Row indices for each of the 9 rows
static ROWS_CHIRHO: [[u8; 9]; 9] = compute_rows_chirho();

/// Column indices for each of the 9 columns
static COLS_CHIRHO: [[u8; 9]; 9] = compute_cols_chirho();

/// Box indices for each of the 9 3x3 boxes
static BOXES_CHIRHO: [[u8; 9]; 9] = compute_boxes_chirho();

/// Peer indices for each cell (precomputed for performance)
/// Each cell has 20 peers: 8 in row + 8 in column + 4 in box (excluding overlaps)
static PEERS_CHIRHO: [[u8; 20]; 81] = compute_peers_chirho();

/// Which units (row, col, box) each cell belongs to (for future use)
#[allow(dead_code)]
static CELL_UNITS_CHIRHO: [[u8; 3]; 81] = compute_cell_units_chirho();

/// Precompute row indices
const fn compute_rows_chirho() -> [[u8; 9]; 9] {
    let mut result_chirho = [[0u8; 9]; 9];
    let mut row_chirho = 0usize;
    while row_chirho < 9 {
        let mut col_chirho = 0usize;
        while col_chirho < 9 {
            result_chirho[row_chirho][col_chirho] = (row_chirho * 9 + col_chirho) as u8;
            col_chirho += 1;
        }
        row_chirho += 1;
    }
    result_chirho
}

/// Precompute column indices
const fn compute_cols_chirho() -> [[u8; 9]; 9] {
    let mut result_chirho = [[0u8; 9]; 9];
    let mut col_chirho = 0usize;
    while col_chirho < 9 {
        let mut row_chirho = 0usize;
        while row_chirho < 9 {
            result_chirho[col_chirho][row_chirho] = (row_chirho * 9 + col_chirho) as u8;
            row_chirho += 1;
        }
        col_chirho += 1;
    }
    result_chirho
}

/// Precompute box indices
const fn compute_boxes_chirho() -> [[u8; 9]; 9] {
    let mut result_chirho = [[0u8; 9]; 9];
    let mut box_chirho = 0usize;
    while box_chirho < 9 {
        let box_row_chirho = (box_chirho / 3) * 3;
        let box_col_chirho = (box_chirho % 3) * 3;
        let mut i_chirho = 0usize;
        let mut r_chirho = 0usize;
        while r_chirho < 3 {
            let mut c_chirho = 0usize;
            while c_chirho < 3 {
                result_chirho[box_chirho][i_chirho] =
                    ((box_row_chirho + r_chirho) * 9 + box_col_chirho + c_chirho) as u8;
                i_chirho += 1;
                c_chirho += 1;
            }
            r_chirho += 1;
        }
        box_chirho += 1;
    }
    result_chirho
}

/// Precompute which units each cell belongs to
#[allow(dead_code)]
const fn compute_cell_units_chirho() -> [[u8; 3]; 81] {
    let mut result_chirho = [[0u8; 3]; 81];
    let mut idx_chirho = 0usize;
    while idx_chirho < 81 {
        let row_chirho = idx_chirho / 9;
        let col_chirho = idx_chirho % 9;
        let box_chirho = (row_chirho / 3) * 3 + col_chirho / 3;
        result_chirho[idx_chirho][0] = row_chirho as u8;
        result_chirho[idx_chirho][1] = col_chirho as u8;
        result_chirho[idx_chirho][2] = box_chirho as u8;
        idx_chirho += 1;
    }
    result_chirho
}

/// Precompute peer indices at compile time
const fn compute_peers_chirho() -> [[u8; 20]; 81] {
    let mut result_chirho = [[0u8; 20]; 81];
    let mut idx_chirho = 0usize;

    while idx_chirho < 81 {
        let row_chirho = idx_chirho / 9;
        let col_chirho = idx_chirho % 9;
        let box_row_chirho = (row_chirho / 3) * 3;
        let box_col_chirho = (col_chirho / 3) * 3;

        let mut peer_count_chirho = 0usize;

        // Same row
        let mut c_chirho = 0usize;
        while c_chirho < 9 {
            let peer_chirho = row_chirho * 9 + c_chirho;
            if peer_chirho != idx_chirho {
                result_chirho[idx_chirho][peer_count_chirho] = peer_chirho as u8;
                peer_count_chirho += 1;
            }
            c_chirho += 1;
        }

        // Same column
        let mut r_chirho = 0usize;
        while r_chirho < 9 {
            let peer_chirho = r_chirho * 9 + col_chirho;
            if peer_chirho != idx_chirho {
                result_chirho[idx_chirho][peer_count_chirho] = peer_chirho as u8;
                peer_count_chirho += 1;
            }
            r_chirho += 1;
        }

        // Same 3x3 box (only add if not already in row/col)
        let mut br_chirho = box_row_chirho;
        while br_chirho < box_row_chirho + 3 {
            let mut bc_chirho = box_col_chirho;
            while bc_chirho < box_col_chirho + 3 {
                let peer_chirho = br_chirho * 9 + bc_chirho;
                if peer_chirho != idx_chirho && br_chirho != row_chirho && bc_chirho != col_chirho {
                    result_chirho[idx_chirho][peer_count_chirho] = peer_chirho as u8;
                    peer_count_chirho += 1;
                }
                bc_chirho += 1;
            }
            br_chirho += 1;
        }

        idx_chirho += 1;
    }

    result_chirho
}

/// Convert value 1-9 to singleton domain
#[inline]
const fn value_to_domain_chirho(v_chirho: u8) -> u16 {
    1u16 << (v_chirho - 1)
}

/// Convert singleton domain to value (1-9), or 0 if not singleton
#[inline]
const fn domain_to_value_chirho(d_chirho: u16) -> u8 {
    if d_chirho == 0 || (d_chirho & (d_chirho - 1)) != 0 {
        0 // Empty or multiple values
    } else {
        d_chirho.trailing_zeros() as u8 + 1
    }
}

/// Check if exactly one value is possible
#[inline]
const fn is_singleton_chirho(d_chirho: u16) -> bool {
    d_chirho != 0 && (d_chirho & (d_chirho - 1)) == 0
}

/// Get lowest possible value as singleton domain
#[inline]
const fn lowest_bit_chirho(d_chirho: u16) -> u16 {
    d_chirho & d_chirho.wrapping_neg()
}

/// Sudoku solver using bit-parallel domain propagation
#[derive(Clone)]
pub struct SudokuSolverChirho {
    /// 81 cells, each with a 9-bit domain
    domains_chirho: [u16; 81],
    /// Statistics: propagation rounds
    pub propagation_count_chirho: u32,
    /// Statistics: search branches explored
    pub branch_count_chirho: u32,
}

impl Default for SudokuSolverChirho {
    fn default() -> Self {
        Self::new_chirho()
    }
}

impl SudokuSolverChirho {
    /// Create a new solver with all cells having full domains
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: [FULL_DOMAIN_CHIRHO; 81],
            propagation_count_chirho: 0,
            branch_count_chirho: 0,
        }
    }

    /// Set a cell to a specific value (1-9) without propagation
    /// Returns false if this leads to immediate contradiction
    #[inline]
    fn set_value_raw_chirho(&mut self, idx_chirho: usize, val_chirho: u8) -> bool {
        let domain_chirho = value_to_domain_chirho(val_chirho);
        self.domains_chirho[idx_chirho] &= domain_chirho;
        self.domains_chirho[idx_chirho] != EMPTY_DOMAIN_CHIRHO
    }

    /// Set a cell to a specific value (1-9) and propagate constraints
    /// Returns false if this leads to a contradiction
    #[inline]
    pub fn set_value_chirho(&mut self, idx_chirho: usize, val_chirho: u8) -> bool {
        if !self.set_value_raw_chirho(idx_chirho, val_chirho) {
            return false;
        }
        self.propagate_basic_chirho()
    }

    /// Naked singles: propagate singleton domains to peers
    /// Returns false if contradiction found
    fn propagate_naked_singles_chirho(&mut self) -> bool {
        for idx_chirho in 0..81 {
            let d_chirho = self.domains_chirho[idx_chirho];

            if d_chirho == EMPTY_DOMAIN_CHIRHO {
                return false;
            }

            if is_singleton_chirho(d_chirho) {
                // Remove this value from all peers
                let not_d_chirho = FULL_DOMAIN_CHIRHO ^ d_chirho;

                for &peer_idx_chirho in &PEERS_CHIRHO[idx_chirho] {
                    let peer_chirho = peer_idx_chirho as usize;
                    let old_chirho = self.domains_chirho[peer_chirho];
                    let new_chirho = old_chirho & not_d_chirho;

                    if new_chirho != old_chirho {
                        self.domains_chirho[peer_chirho] = new_chirho;
                        if new_chirho == EMPTY_DOMAIN_CHIRHO {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Hidden singles: if a value can only go in one cell of a unit, assign it
    /// Returns true if any assignment was made
    fn propagate_hidden_singles_chirho(&mut self) -> Result<bool, ()> {
        let mut changed_chirho = false;

        // Check all 27 units (9 rows + 9 cols + 9 boxes)
        for unit_type_chirho in 0..3 {
            let units_chirho: &[[u8; 9]; 9] = match unit_type_chirho {
                0 => &ROWS_CHIRHO,
                1 => &COLS_CHIRHO,
                _ => &BOXES_CHIRHO,
            };

            for unit_chirho in units_chirho.iter() {
                // For each value 1-9, find which cells can hold it
                for val_chirho in 0..9u16 {
                    let val_bit_chirho = 1u16 << val_chirho;
                    let mut count_chirho = 0u8;
                    let mut last_cell_chirho = 0usize;

                    for &cell_idx_chirho in unit_chirho.iter() {
                        let cell_chirho = cell_idx_chirho as usize;
                        if self.domains_chirho[cell_chirho] & val_bit_chirho != 0 {
                            count_chirho += 1;
                            last_cell_chirho = cell_chirho;
                        }
                    }

                    if count_chirho == 0 {
                        // Value cannot be placed anywhere in unit - contradiction
                        return Err(());
                    } else if count_chirho == 1 {
                        // Hidden single: only one cell can hold this value
                        if !is_singleton_chirho(self.domains_chirho[last_cell_chirho]) {
                            self.domains_chirho[last_cell_chirho] = val_bit_chirho;
                            changed_chirho = true;
                        }
                    }
                }
            }
        }

        Ok(changed_chirho)
    }

    /// Naked pairs: if N cells in a unit have exactly the same N values,
    /// eliminate those values from other cells in the unit
    /// Returns true if any elimination was made
    fn propagate_naked_pairs_chirho(&mut self) -> bool {
        let mut changed_chirho = false;

        // Check all 27 units
        for unit_type_chirho in 0..3 {
            let units_chirho: &[[u8; 9]; 9] = match unit_type_chirho {
                0 => &ROWS_CHIRHO,
                1 => &COLS_CHIRHO,
                _ => &BOXES_CHIRHO,
            };

            for unit_chirho in units_chirho.iter() {
                // Find cells with exactly 2 values (pairs)
                for i_chirho in 0..9 {
                    let cell_i_chirho = unit_chirho[i_chirho] as usize;
                    let domain_i_chirho = self.domains_chirho[cell_i_chirho];

                    if domain_i_chirho.count_ones() != 2 {
                        continue;
                    }

                    // Find matching pair
                    for j_chirho in (i_chirho + 1)..9 {
                        let cell_j_chirho = unit_chirho[j_chirho] as usize;
                        let domain_j_chirho = self.domains_chirho[cell_j_chirho];

                        if domain_i_chirho == domain_j_chirho {
                            // Found a naked pair! Eliminate from other cells
                            let not_pair_chirho = FULL_DOMAIN_CHIRHO ^ domain_i_chirho;

                            for k_chirho in 0..9 {
                                if k_chirho == i_chirho || k_chirho == j_chirho {
                                    continue;
                                }
                                let cell_k_chirho = unit_chirho[k_chirho] as usize;
                                let old_chirho = self.domains_chirho[cell_k_chirho];
                                let new_chirho = old_chirho & not_pair_chirho;

                                if new_chirho != old_chirho {
                                    self.domains_chirho[cell_k_chirho] = new_chirho;
                                    changed_chirho = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        changed_chirho
    }

    /// Basic propagation: only naked singles (fast path)
    fn propagate_basic_chirho(&mut self) -> bool {
        let mut changed_chirho = true;
        while changed_chirho {
            changed_chirho = false;
            self.propagation_count_chirho += 1;

            for idx_chirho in 0..81 {
                let d_chirho = self.domains_chirho[idx_chirho];

                if d_chirho == EMPTY_DOMAIN_CHIRHO {
                    return false;
                }

                if is_singleton_chirho(d_chirho) {
                    let not_d_chirho = FULL_DOMAIN_CHIRHO ^ d_chirho;

                    for &peer_idx_chirho in &PEERS_CHIRHO[idx_chirho] {
                        let peer_chirho = peer_idx_chirho as usize;
                        let old_chirho = self.domains_chirho[peer_chirho];
                        let new_chirho = old_chirho & not_d_chirho;

                        if new_chirho != old_chirho {
                            changed_chirho = true;
                            self.domains_chirho[peer_chirho] = new_chirho;
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

    /// Full propagation: naked singles + hidden singles + naked pairs
    /// Repeat until fixed point
    pub fn propagate_chirho(&mut self) -> bool {
        loop {
            self.propagation_count_chirho += 1;

            // Phase 1: Naked singles (basic constraint propagation)
            if !self.propagate_naked_singles_chirho() {
                return false;
            }

            // Phase 2: Hidden singles (more inference power)
            match self.propagate_hidden_singles_chirho() {
                Err(()) => return false,
                Ok(true) => continue, // Made progress, restart
                Ok(false) => {}
            }

            // Phase 3: Naked pairs (even more inference)
            if self.propagate_naked_pairs_chirho() {
                continue; // Made progress, restart
            }

            // Fixed point reached
            break;
        }

        true
    }

    /// Adaptive propagation: use simple propagation first,
    /// upgrade to full propagation only if search is needed
    pub fn propagate_adaptive_chirho(&mut self) -> bool {
        // Start with basic propagation
        if !self.propagate_basic_chirho() {
            return false;
        }

        // If solved or nearly solved, we're done
        let unsolved_chirho = self.domains_chirho.iter()
            .filter(|&&d_chirho| !is_singleton_chirho(d_chirho))
            .count();

        if unsolved_chirho == 0 {
            return true;
        }

        // If many cells unsolved, use advanced propagation
        if unsolved_chirho > 20 {
            return self.propagate_chirho();
        }

        true
    }

    /// Check if all cells are solved (singletons)
    #[inline]
    pub fn is_solved_chirho(&self) -> bool {
        self.domains_chirho.iter().all(|&d_chirho| is_singleton_chirho(d_chirho))
    }

    /// Find cell with smallest domain > 1 (fail-first heuristic)
    fn find_branch_cell_chirho(&self) -> Option<usize> {
        let mut best_idx_chirho = None;
        let mut best_count_chirho = 10u32;

        for (idx_chirho, &d_chirho) in self.domains_chirho.iter().enumerate() {
            let count_chirho = d_chirho.count_ones();
            if count_chirho > 1 && count_chirho < best_count_chirho {
                best_idx_chirho = Some(idx_chirho);
                best_count_chirho = count_chirho;

                if count_chirho == 2 {
                    break;
                }
            }
        }

        best_idx_chirho
    }

    /// Solve using full constraint propagation + search
    /// Best for hard puzzles that need advanced inference
    pub fn solve_chirho(&mut self) -> bool {
        if !self.propagate_chirho() {
            return false;
        }

        if self.is_solved_chirho() {
            return true;
        }

        // Branch on smallest domain
        let branch_idx_chirho = match self.find_branch_cell_chirho() {
            Some(idx_chirho) => idx_chirho,
            None => return self.is_solved_chirho(),
        };

        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        // Try each possible value
        while domain_chirho != 0 {
            self.branch_count_chirho += 1;

            // Fork: try lowest value first
            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1;

            // Clone state and try this value
            let mut attempt_chirho = self.clone();
            attempt_chirho.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if attempt_chirho.solve_chirho() {
                *self = attempt_chirho;
                return true;
            }
        }

        false
    }

    /// Solve using basic propagation + search (faster for easy puzzles)
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

        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        while domain_chirho != 0 {
            self.branch_count_chirho += 1;

            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1;

            let mut attempt_chirho = self.clone();
            attempt_chirho.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if attempt_chirho.solve_fast_chirho() {
                *self = attempt_chirho;
                return true;
            }
        }

        false
    }

    /// Adaptive solve: uses basic propagation for easy puzzles,
    /// escalates to full propagation only when needed
    pub fn solve_adaptive_chirho(&mut self) -> bool {
        // First try basic propagation
        if !self.propagate_basic_chirho() {
            return false;
        }

        if self.is_solved_chirho() {
            return true;
        }

        // Count unsolved cells to decide strategy
        let unsolved_chirho = self.domains_chirho.iter()
            .filter(|&&d_chirho| !is_singleton_chirho(d_chirho))
            .count();

        // If many cells remain after basic propagation, puzzle is hard
        // Use advanced propagation to reduce search space
        if unsolved_chirho > 15 {
            if !self.propagate_chirho() {
                return false;
            }
            if self.is_solved_chirho() {
                return true;
            }
            // After advanced propagation, continue with full solve
            return self.solve_after_propagation_chirho();
        }

        // Few cells remain - basic propagation is enough, just search
        self.solve_after_propagation_chirho()
    }

    /// Internal: solve after propagation is complete
    fn solve_after_propagation_chirho(&mut self) -> bool {
        if self.is_solved_chirho() {
            return true;
        }

        let branch_idx_chirho = match self.find_branch_cell_chirho() {
            Some(idx_chirho) => idx_chirho,
            None => return self.is_solved_chirho(),
        };

        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        while domain_chirho != 0 {
            self.branch_count_chirho += 1;

            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1;

            let mut attempt_chirho = self.clone();
            attempt_chirho.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if attempt_chirho.solve_adaptive_chirho() {
                *self = attempt_chirho;
                return true;
            }
        }

        false
    }

    /// Load puzzle from string (81 chars, 0 or . for empty)
    pub fn load_puzzle_chirho(&mut self, puzzle_chirho: &str) -> bool {
        let chars_chirho: Vec<char> = puzzle_chirho
            .chars()
            .filter(|c_chirho| !c_chirho.is_whitespace())
            .collect();

        if chars_chirho.len() != 81 {
            return false;
        }

        for (idx_chirho, ch_chirho) in chars_chirho.iter().enumerate() {
            if let Some(digit_chirho) = ch_chirho.to_digit(10) {
                if digit_chirho >= 1 && digit_chirho <= 9 {
                    if !self.set_value_chirho(idx_chirho, digit_chirho as u8) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Convert solution to 81-char string
    pub fn to_string_chirho(&self) -> String {
        self.domains_chirho
            .iter()
            .map(|&d_chirho| {
                let v_chirho = domain_to_value_chirho(d_chirho);
                if v_chirho > 0 { (b'0' + v_chirho) as char } else { '.' }
            })
            .collect()
    }

    /// Get value at cell (1-9, or 0 if unsolved)
    #[inline]
    pub fn get_value_chirho(&self, idx_chirho: usize) -> u8 {
        domain_to_value_chirho(self.domains_chirho[idx_chirho])
    }

    /// Get domain at cell (for debugging/visualization)
    #[inline]
    pub fn get_domain_chirho(&self, idx_chirho: usize) -> u16 {
        self.domains_chirho[idx_chirho]
    }
}

/// Standard test puzzles
pub mod puzzles_chirho {
    /// Easy puzzle (many givens)
    pub const EASY_CHIRHO: &str =
        "530070000\
         600195000\
         098000060\
         800060003\
         400803001\
         700020006\
         060000280\
         000419005\
         000080079";

    /// Medium puzzle
    pub const MEDIUM_CHIRHO: &str =
        "000000680\
         030000000\
         900800100\
         000002074\
         060090030\
         780500000\
         001007005\
         000000020\
         024000000";

    /// Hard puzzle (17 clues - minimal)
    pub const HARD_CHIRHO: &str =
        "000000010\
         400000000\
         020000000\
         000050407\
         008000300\
         001090000\
         300400200\
         050100000\
         000806000";

    /// Very hard ("AI Escargot" - one of the hardest known)
    pub const ESCARGOT_CHIRHO: &str =
        "100007090\
         030020008\
         009600500\
         005300900\
         010080002\
         600004000\
         300000010\
         040000007\
         007000300";

    /// Another hard puzzle for benchmarking
    pub const INKALA_CHIRHO: &str =
        "800000000\
         003600000\
         070090200\
         050007000\
         000045700\
         000100030\
         001000068\
         008500010\
         090000400";
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_easy_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::EASY_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());

        let solution_chirho = solver_chirho.to_string_chirho();
        assert_eq!(solution_chirho.len(), 81);
        assert!(!solution_chirho.contains('.'));
    }

    #[test]
    fn test_medium_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::MEDIUM_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());
    }

    #[test]
    fn test_hard_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::HARD_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());

        // Verify reduced branching with advanced propagation
        println!("Hard puzzle: {} branches", solver_chirho.branch_count_chirho);
    }

    #[test]
    fn test_escargot_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::ESCARGOT_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());
    }

    #[test]
    fn test_inkala_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::INKALA_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());
    }

    #[test]
    fn test_peers_count_chirho() {
        for idx_chirho in 0..81 {
            let peers_chirho = &PEERS_CHIRHO[idx_chirho];
            let unique_count_chirho = peers_chirho.iter()
                .filter(|&&p_chirho| p_chirho != idx_chirho as u8)
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(unique_count_chirho, 20, "Cell {} should have 20 peers", idx_chirho);
        }
    }

    #[test]
    fn test_value_domain_round_trip_chirho() {
        for v_chirho in 1..=9u8 {
            let d_chirho = value_to_domain_chirho(v_chirho);
            assert!(is_singleton_chirho(d_chirho));
            assert_eq!(domain_to_value_chirho(d_chirho), v_chirho);
        }
    }

    #[test]
    fn test_hidden_single_detection_chirho() {
        // Create a row where value 9 can only go in one cell
        let mut solver_chirho = SudokuSolverChirho::new_chirho();

        // Set first 8 cells to not allow 9
        for i_chirho in 0..8 {
            solver_chirho.domains_chirho[i_chirho] = 0b011111111; // All except 9
        }
        // Cell 8 allows all values including 9
        solver_chirho.domains_chirho[8] = FULL_DOMAIN_CHIRHO;

        // After propagation, cell 8 should be forced to 9
        assert!(solver_chirho.propagate_chirho());
        assert_eq!(solver_chirho.domains_chirho[8], 0b100000000); // Only 9
    }

    #[test]
    fn test_naked_pair_elimination_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();

        // In first row: cells 0,1 have domain {1,2}, cell 2 has {1,2,3}
        solver_chirho.domains_chirho[0] = 0b011; // 1,2
        solver_chirho.domains_chirho[1] = 0b011; // 1,2
        solver_chirho.domains_chirho[2] = 0b111; // 1,2,3

        // After naked pair propagation, cell 2 should lose 1,2
        solver_chirho.propagate_naked_pairs_chirho();
        assert_eq!(solver_chirho.domains_chirho[2], 0b100); // Only 3
    }
}
