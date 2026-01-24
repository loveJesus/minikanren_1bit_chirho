//! miniKanren as 1-Bit Matrix Operations ☧
//!
//! Rust implementation for performance, with hardware-friendly abstractions.
//!
//! Core equation: miniKanren search = sparse Boolean tensor network contraction
//!
//! # Modules
//!
//! - `terms_chirho`: Hash-consed term storage
//! - `union_find_chirho`: O(α(n)) variable equivalence classes
//! - `unify_chirho`: Unification with occurs check
//! - `bitmatrix_chirho`: Sparse Boolean tensors (COO format)
//! - `relations_chirho`: Relations as sparse tensors (appendo, membero)
//! - `contraction_chirho`: Tensor network contraction heuristics
//! - `stream_chirho`: Lazy streams for miniKanren search
//! - `goals_chirho`: Goal combinators (==, conde, conj, disj)
//! - `constraint_chirho`: Arc consistency constraint propagation
//! - `tabling_chirho`: SLG-style memoization for recursion
//! - `semiring_chirho`: Semiring abstraction (Bool, Prob, Tropical, Count)

pub mod terms_chirho;
pub mod union_find_chirho;
pub mod unify_chirho;
pub mod bitmatrix_chirho;
pub mod relations_chirho;
pub mod contraction_chirho;
pub mod stream_chirho;
pub mod goals_chirho;
pub mod constraint_chirho;
pub mod tabling_chirho;
pub mod semiring_chirho;

// Re-export key types (avoiding ambiguous globs)
pub use terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
pub use union_find_chirho::{UnionFindChirho, UnionFindHwChirho};
pub use unify_chirho::{SubstChirho, UnifyResultChirho, unify_chirho};
pub use bitmatrix_chirho::{BitMatrixChirho, BitTensor3Chirho};
pub use goals_chirho::{GoalFnChirho, eq_chirho, conj_chirho, disj_chirho, conde_chirho, run_chirho, run_all_chirho};
pub use constraint_chirho::{DomainChirho, ConstraintStoreChirho, BinaryConstraintChirho};
pub use tabling_chirho::{TableStoreChirho, CallPatternChirho, LookupResultChirho};
pub use semiring_chirho::{SemiringChirho, BoolSemiringChirho, ProbSemiringChirho, TropicalSemiringChirho, CountSemiringChirho, LogSemiringChirho, WeightedMatrixChirho};
