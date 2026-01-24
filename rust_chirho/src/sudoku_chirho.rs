//! Sudoku Solver using 1-Bit Domain Propagation ☧
//!
//! Demonstrates practical use of miniKanren-style constraint propagation
//! with bit-parallel domain operations.
//!
//! Each cell has a 9-bit domain where bit i means value (i+1) is possible.
//! Constraints (row, column, box) are applied via bitwise AND.
//!
//! "Whether therefore ye eat, or drink, or whatsoever ye do,
//!  do all to the glory of God." — 1 Corinthians 10:31

/// Full domain: all values 1-9 possible (bits 0-8 set)
const FULL_DOMAIN_CHIRHO: u16 = 0b111111111;

/// Empty domain: no values possible (failure state)
const EMPTY_DOMAIN_CHIRHO: u16 = 0;

/// Peer indices for each cell (precomputed for performance)
/// Each cell has 20 peers: 8 in row + 8 in column + 4 in box (excluding overlaps)
static PEERS_CHIRHO: [[u8; 20]; 81] = compute_peers_chirho();

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

    /// Set a cell to a specific value (1-9) and propagate constraints
    /// Returns false if this leads to a contradiction
    #[inline]
    pub fn set_value_chirho(&mut self, idx_chirho: usize, val_chirho: u8) -> bool {
        let domain_chirho = value_to_domain_chirho(val_chirho);

        // Intersect with current domain (unification!)
        self.domains_chirho[idx_chirho] &= domain_chirho;

        if self.domains_chirho[idx_chirho] == EMPTY_DOMAIN_CHIRHO {
            return false;
        }

        self.propagate_chirho()
    }

    /// Arc consistency propagation using bit operations
    ///
    /// For each singleton domain, remove that value from all peers.
    /// Repeat until fixed point.
    pub fn propagate_chirho(&mut self) -> bool {
        let mut changed_chirho = true;

        while changed_chirho {
            changed_chirho = false;
            self.propagation_count_chirho += 1;

            for idx_chirho in 0..81 {
                let d_chirho = self.domains_chirho[idx_chirho];

                if d_chirho == EMPTY_DOMAIN_CHIRHO {
                    return false; // Failure
                }

                if is_singleton_chirho(d_chirho) {
                    // Remove this value from all peers
                    let not_d_chirho = FULL_DOMAIN_CHIRHO ^ d_chirho;

                    for &peer_idx_chirho in &PEERS_CHIRHO[idx_chirho] {
                        let peer_chirho = peer_idx_chirho as usize;
                        let old_chirho = self.domains_chirho[peer_chirho];
                        // Key operation: domain intersection via AND
                        let new_chirho = old_chirho & not_d_chirho;

                        if new_chirho != old_chirho {
                            changed_chirho = true;
                            self.domains_chirho[peer_chirho] = new_chirho;

                            if new_chirho == EMPTY_DOMAIN_CHIRHO {
                                return false; // Failure
                            }
                        }
                    }
                }
            }
        }

        true
    }

    /// Check if all cells are solved (singletons)
    #[inline]
    pub fn is_solved_chirho(&self) -> bool {
        self.domains_chirho.iter().all(|&d| is_singleton_chirho(d))
    }

    /// Find cell with smallest domain > 1 (fail-first heuristic)
    fn find_branch_cell_chirho(&self) -> Option<usize> {
        let mut best_idx_chirho = None;
        let mut best_count_chirho = 10u32; // More than max possible

        for (idx_chirho, &d_chirho) in self.domains_chirho.iter().enumerate() {
            let count_chirho = d_chirho.count_ones();
            if count_chirho > 1 && count_chirho < best_count_chirho {
                best_idx_chirho = Some(idx_chirho);
                best_count_chirho = count_chirho;

                // Early exit if we find a cell with only 2 possibilities
                if count_chirho == 2 {
                    break;
                }
            }
        }

        best_idx_chirho
    }

    /// Solve using constraint propagation + search
    /// Returns true if solved, false if unsolvable
    pub fn solve_chirho(&mut self) -> bool {
        if !self.propagate_chirho() {
            return false;
        }

        if self.is_solved_chirho() {
            return true;
        }

        // Branch on smallest domain
        let branch_idx_chirho = match self.find_branch_cell_chirho() {
            Some(idx) => idx,
            None => return self.is_solved_chirho(),
        };

        let mut domain_chirho = self.domains_chirho[branch_idx_chirho];

        // Try each possible value
        while domain_chirho != 0 {
            self.branch_count_chirho += 1;

            // Fork: try lowest value first
            let try_val_chirho = lowest_bit_chirho(domain_chirho);
            domain_chirho &= domain_chirho - 1; // Clear lowest bit

            // Clone state and try this value
            let mut attempt_chirho = self.clone();
            attempt_chirho.domains_chirho[branch_idx_chirho] = try_val_chirho;

            if attempt_chirho.solve_chirho() {
                // Success! Copy solution back
                *self = attempt_chirho;
                return true;
            }
        }

        false // All branches failed
    }

    /// Load puzzle from string (81 chars, 0 or . for empty)
    pub fn load_puzzle_chirho(&mut self, puzzle_chirho: &str) -> bool {
        let chars_chirho: Vec<char> = puzzle_chirho
            .chars()
            .filter(|c| !c.is_whitespace())
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
            .map(|&d| {
                let v = domain_to_value_chirho(d);
                if v > 0 { (b'0' + v) as char } else { '.' }
            })
            .collect()
    }

    /// Get value at cell (1-9, or 0 if unsolved)
    #[inline]
    pub fn get_value_chirho(&self, idx_chirho: usize) -> u8 {
        domain_to_value_chirho(self.domains_chirho[idx_chirho])
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

        // Verify solution is valid
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
    }

    #[test]
    fn test_escargot_puzzle_chirho() {
        let mut solver_chirho = SudokuSolverChirho::new_chirho();
        assert!(solver_chirho.load_puzzle_chirho(puzzles_chirho::ESCARGOT_CHIRHO));
        assert!(solver_chirho.solve_chirho());
        assert!(solver_chirho.is_solved_chirho());
    }

    #[test]
    fn test_peers_count_chirho() {
        // Each cell should have exactly 20 peers
        for idx_chirho in 0..81 {
            let peers_chirho = &PEERS_CHIRHO[idx_chirho];
            // Count non-duplicate peers
            let unique_count_chirho = peers_chirho.iter()
                .filter(|&&p| p != idx_chirho as u8)
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(unique_count_chirho, 20, "Cell {} should have 20 peers", idx_chirho);
        }
    }

    #[test]
    fn test_value_domain_round_trip_chirho() {
        for v in 1..=9u8 {
            let d = value_to_domain_chirho(v);
            assert!(is_singleton_chirho(d));
            assert_eq!(domain_to_value_chirho(d), v);
        }
    }
}
