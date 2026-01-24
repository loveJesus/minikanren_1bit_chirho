//! miniKanren as 1-Bit Matrix Operations ☧
//!
//! Rust implementation for performance, with hardware-friendly abstractions.
//!
//! Core equation: miniKanren search = sparse Boolean tensor network contraction

pub mod terms_chirho;
pub mod union_find_chirho;
pub mod unify_chirho;
pub mod bitmatrix_chirho;
pub mod relations_chirho;
pub mod contraction_chirho;

pub use terms_chirho::*;
pub use union_find_chirho::*;
pub use unify_chirho::*;
pub use bitmatrix_chirho::*;
