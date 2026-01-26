// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Ready-to-use constraint solvers ☧
//!
//! Practical demonstrations of 1-bit constraint propagation:
//! - Sudoku: 9-bit domains per cell, AC-3 + backtracking
//! - N-Queens: 64-bit domains, scales to 64×64

pub mod nqueens_chirho;
pub mod sudoku_chirho;

// Re-export key types
pub use nqueens_chirho::{NQueensSolverChirho, KNOWN_SOLUTIONS_CHIRHO};
pub use sudoku_chirho::{puzzles_chirho, SudokuSolverChirho};
