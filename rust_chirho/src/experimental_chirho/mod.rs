//! Experimental modules ☧
//!
//! Research explorations and advanced features:
//! - E-graphs (native and egg-based)
//! - SMT integration
//! - Neural heuristics
//! - Category-theoretic abstractions (optics, free monads, comonads, linear logic)
//! - Tensor network learning
//! - Relations as bit-matrices

// Core experimental modules (always available)
pub mod contraction_chirho;
pub mod contraction_learn_chirho;
pub mod jsonschema_chirho;
pub mod nested_pattern_chirho;
pub mod neural_chirho;
pub mod recursion_chirho;
pub mod relation_chirho;
pub mod relations_chirho;
pub mod slg_complete_chirho;
pub mod smt_chirho;
pub mod unify_matrix_chirho;

// Feature-gated modules
#[cfg(feature = "egraph_native_chirho")]
pub mod egraph_native_chirho;

// egg_chirho moved to reference_chirho (external library comparison)

#[cfg(feature = "goal_ast_chirho")]
pub mod goal_ast_chirho;

#[cfg(feature = "gpu_chirho")]
pub mod gpu_chirho;

#[cfg(feature = "optics_chirho")]
pub mod optics_chirho;

#[cfg(feature = "free_goal_chirho")]
pub mod free_goal_chirho;

#[cfg(feature = "comonad_chirho")]
pub mod comonad_chirho;

#[cfg(feature = "linear_chirho")]
pub mod linear_chirho;

// Re-export key types from always-available modules
pub use contraction_chirho::{ContractionStepChirho, TensorNodeChirho, greedy_order_chirho, min_degree_order_chirho, total_cost_chirho};
pub use contraction_learn_chirho::{
    EdgeFeaturesChirho, LearnedContractionChirho, LinearEdgeScorerChirho, TensorNetworkChirho,
};
pub use jsonschema_chirho::{
    validate_chirho, JsonValueChirho, SchemaChirho, ValidationErrorChirho, ValidatorChirho,
};
pub use nested_pattern_chirho::{
    NestedPatternChirho, NodeConstraintChirho, NodeTypeChirho, PathStepChirho, TreePathChirho,
};
pub use neural_chirho::{
    beam_search_chirho, NeuralHeuristicChirho, NeuralStateChirho, SoftDomainChirho,
};
pub use recursion_chirho::{
    cata_indexed_chirho, depth_algebra_chirho, is_ground_algebra_chirho, para_indexed_chirho,
    size_algebra_chirho, vars_algebra_chirho, OccursCheckAlgebraChirho, TermFChirho,
    TermStoreIndexedChirho,
};
pub use relation_chirho::{DenseRelationChirho, SparseRelationChirho};
pub use slg_complete_chirho::{EvenOddTensorChirho, GoalStatusChirho, SlgGoalChirho, SlgTableChirho};
pub use smt_chirho::{domain_to_smt_chirho, SmtExprChirho, SmtProblemChirho, SmtSortChirho};
pub use unify_matrix_chirho::{unify_matrix_chirho, SubstMatrixChirho};

// Feature-gated re-exports
#[cfg(feature = "egraph_native_chirho")]
pub use egraph_native_chirho::{
    EClassDataChirho, EClassIdChirho, EGraphNativeChirho, ENodeChirho, ENodeIdChirho,
};

// egg_chirho re-exports moved to reference_chirho

#[cfg(feature = "gpu_chirho")]
pub use gpu_chirho::GpuContextChirho;

#[cfg(feature = "optics_chirho")]
pub use optics_chirho::{
    reify_optic_chirho, walk_deep_optic_chirho, ChildrenTraversalChirho, ConsPrismChirho,
    HeadLensChirho, IntPrismChirho, LensChirho, NilPrismChirho, PrismChirho,
    SubtermTraversalChirho, TailLensChirho, TraversalChirho, VarPrismChirho, VarsTraversalChirho,
};

#[cfg(feature = "free_goal_chirho")]
pub use free_goal_chirho::{
    conj_free_chirho, diseq_free_chirho, disj_free_chirho, eq_free_chirho, fresh_free_chirho,
    goal_to_smt_chirho, run_bool_chirho, run_prob_chirho, FreeGoalChirho, GoalFChirho,
    SmtFormulaChirho,
};

#[cfg(feature = "comonad_chirho")]
pub use comonad_chirho::{propagate_chirho, ComonadChirho, SearchZipperChirho};

#[cfg(feature = "linear_chirho")]
pub use linear_chirho::{bang_chirho, par_chirho, tensor_chirho, LinearGoalChirho, ReusableGoalChirho};
