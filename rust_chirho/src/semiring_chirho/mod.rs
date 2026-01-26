// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Semiring abstractions ☧
//!
//! Generalized provenance for logic programming:
//! - Boolean: Standard miniKanren (success/failure)
//! - Probability: Soft logic, weighted solutions
//! - Tropical: Shortest paths, optimization
//! - Count: Solution counting
//! - Differentiable: Gradient flow through logic

pub mod contraction_semiring_chirho;
pub mod diff_semiring_chirho;
pub mod semiring_chirho;

// Re-export key types
pub use contraction_semiring_chirho::{
    BoolTensorChirho, CountTensorChirho, ProbTensorChirho, SemiringNetworkChirho,
    SemiringTensorChirho, TropicalTensorChirho,
};
pub use diff_semiring_chirho::{
    annealed_temp_chirho, log_sum_exp_chirho, soft_eq_chirho, soft_eq_with_grad_chirho,
    DiffProbChirho, GumbelSoftmaxChirho, LearnableRelationChirho, StraightThroughChirho,
    WeightedTupleChirho,
};
pub use semiring_chirho::{
    BoolSemiringChirho, CountSemiringChirho, LogSemiringChirho, ProbSemiringChirho,
    SemiringChirho, TropicalSemiringChirho, WeightedMatrixChirho,
};
