// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! WebAssembly Bindings for miniKanren 1-bit ☧
//!
//! Exposes Sudoku and N-Queens solvers to JavaScript.
//!
//! Build with: wasm-pack build --target web --features wasm_chirho
//!
//! "Go ye into all the world, and preach the gospel to every creature." — Mark 16:15

use wasm_bindgen::prelude::*;

// ============================================================================
// Sudoku WASM API
// ============================================================================

/// Sudoku solver exposed to JavaScript
#[wasm_bindgen]
pub struct SudokuWasmChirho {
    solver_chirho: crate::sudoku_chirho::SudokuSolverChirho,
}

#[wasm_bindgen]
impl SudokuWasmChirho {
    /// Create a new Sudoku solver
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            solver_chirho: crate::sudoku_chirho::SudokuSolverChirho::new_chirho(),
        }
    }

    /// Load a puzzle from an 81-character string (0 or . for empty)
    #[wasm_bindgen]
    pub fn load_puzzle_chirho(&mut self, puzzle_chirho: &str) -> bool {
        self.solver_chirho.load_puzzle_chirho(puzzle_chirho)
    }

    /// Solve the puzzle using adaptive strategy
    /// Returns the solution as an 81-character string, or empty on failure
    #[wasm_bindgen]
    pub fn solve_chirho(&mut self) -> String {
        if self.solver_chirho.solve_adaptive_chirho() {
            self.solver_chirho.to_string_chirho()
        } else {
            String::new()
        }
    }

    /// Get current board state as 81-character string
    #[wasm_bindgen]
    pub fn get_board_chirho(&self) -> String {
        self.solver_chirho.to_string_chirho()
    }

    /// Get domain bitmask for a cell (0-80)
    /// Returns 9-bit mask where bit i means (i+1) is possible
    #[wasm_bindgen]
    pub fn get_domain_chirho(&self, cell_chirho: usize) -> u16 {
        if cell_chirho < 81 {
            self.solver_chirho.get_domain_chirho(cell_chirho)
        } else {
            0
        }
    }

    /// Check if puzzle is solved (all cells have single value)
    #[wasm_bindgen]
    pub fn is_solved_chirho(&self) -> bool {
        self.solver_chirho.is_solved_chirho()
    }

    /// Get propagation count (for stats)
    #[wasm_bindgen]
    pub fn propagation_count_chirho(&self) -> u32 {
        self.solver_chirho.propagation_count_chirho
    }

    /// Get branch count (for stats)
    #[wasm_bindgen]
    pub fn branch_count_chirho(&self) -> u32 {
        self.solver_chirho.branch_count_chirho
    }
}

impl Default for SudokuWasmChirho {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// N-Queens WASM API
// ============================================================================

/// N-Queens solver exposed to JavaScript
#[wasm_bindgen]
pub struct NQueensWasmChirho {
    n_chirho: usize,
}

#[wasm_bindgen]
impl NQueensWasmChirho {
    /// Create solver for N×N board (N ≤ 64)
    #[wasm_bindgen(constructor)]
    pub fn new(n_chirho: usize) -> Self {
        Self { n_chirho: n_chirho.min(64) }
    }

    /// Count all solutions
    #[wasm_bindgen]
    pub fn count_solutions_chirho(&self) -> u64 {
        let mut solver_chirho = crate::nqueens_chirho::NQueensSolverChirho::new_chirho(self.n_chirho);
        solver_chirho.count_solutions_chirho()
    }

    /// Find one solution, returns array of column positions (or empty)
    #[wasm_bindgen]
    pub fn solve_one_chirho(&self) -> Vec<u8> {
        let mut solver_chirho = crate::nqueens_chirho::NQueensSolverChirho::new_chirho(self.n_chirho);
        solver_chirho.solve_one_chirho().unwrap_or_default()
    }

    /// Get board size
    #[wasm_bindgen]
    pub fn size_chirho(&self) -> usize {
        self.n_chirho
    }
}

// ============================================================================
// JSON Schema Validator WASM API
// ============================================================================

/// JSON Schema validation result
#[wasm_bindgen]
pub struct ValidationResultChirho {
    valid_chirho: bool,
    errors_chirho: Vec<String>,
}

#[wasm_bindgen]
impl ValidationResultChirho {
    /// Is the validation successful?
    #[wasm_bindgen]
    pub fn is_valid_chirho(&self) -> bool {
        self.valid_chirho
    }

    /// Get error messages (empty if valid)
    #[wasm_bindgen]
    pub fn errors_chirho(&self) -> Vec<String> {
        self.errors_chirho.clone()
    }

    /// Get number of errors
    #[wasm_bindgen]
    pub fn error_count_chirho(&self) -> usize {
        self.errors_chirho.len()
    }
}

// ============================================================================
// Utility functions
// ============================================================================

/// Initialize WASM module
#[wasm_bindgen(start)]
pub fn init_wasm_chirho() {
    // Initialization hook (can add console_error_panic_hook if needed)
}

/// Get library version
#[wasm_bindgen]
pub fn version_chirho() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Solve Sudoku directly (convenience function)
/// Input: 81-char string (0 or . for empty)
/// Output: 81-char solution string (empty on failure)
#[wasm_bindgen]
pub fn solve_sudoku_chirho(puzzle_chirho: &str) -> String {
    let mut solver_chirho = SudokuWasmChirho::new();
    if solver_chirho.load_puzzle_chirho(puzzle_chirho) {
        solver_chirho.solve_chirho()
    } else {
        String::new()
    }
}

/// Solve N-Queens directly (convenience function)
/// Returns array of column positions for first solution
#[wasm_bindgen]
pub fn solve_nqueens_chirho(n_chirho: usize) -> Vec<u8> {
    NQueensWasmChirho::new(n_chirho).solve_one_chirho()
}

/// Count N-Queens solutions
#[wasm_bindgen]
pub fn count_nqueens_chirho(n_chirho: usize) -> u64 {
    NQueensWasmChirho::new(n_chirho).count_solutions_chirho()
}
