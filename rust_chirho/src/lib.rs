//! # miniKanren as 1-Bit Matrix Operations ☧
//!
//! Hardware-accelerated logic programming through bit-parallel constraint propagation.
//!
//! **Core insight:** miniKanren search = sparse Boolean tensor network contraction
//!
//! ## Quick Example
//!
//! ```rust
//! use minikanren_1bit_chirho::*;
//!
//! // Create term store and define variables
//! let mut store_chirho = TermStoreChirho::new();
//! let (_, x_chirho) = store_chirho.fresh_var_chirho();
//! let one_chirho = store_chirho.int_chirho(1);
//! let two_chirho = store_chirho.int_chirho(2);
//!
//! // Goal: x ∈ {1, 2}
//! let goal_chirho = conde_chirho(vec![
//!     vec![eq_chirho(x_chirho, one_chirho)],
//!     vec![eq_chirho(x_chirho, two_chirho)],
//! ]);
//!
//! // Run and collect solutions
//! let results_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);
//! assert_eq!(results_chirho.len(), 2);
//! ```
//!
//! ## Solvers
//!
//! Ready-to-use constraint solvers:
//!
//! ```rust
//! use minikanren_1bit_chirho::solvers_chirho::{SudokuSolverChirho, NQueensSolverChirho};
//!
//! // Sudoku: 9-bit domains per cell
//! let mut sudoku_chirho = SudokuSolverChirho::new_chirho();
//! sudoku_chirho.load_puzzle_chirho("530070000600195000098000060800060003400803001700020006060000280000419005000080079");
//! sudoku_chirho.solve_adaptive_chirho();
//!
//! // N-Queens: 64-bit domains
//! let mut queens_chirho = NQueensSolverChirho::new_chirho(8);
//! let count_chirho = queens_chirho.count_solutions_chirho(); // 92 solutions
//! ```
//!
//! ## Performance
//!
//! | Operation | Time | Notes |
//! |-----------|------|-------|
//! | Unify (64-bit AND) | 2ns | Single SIMD instruction |
//! | Domain intersection | 8ns | 8 domains parallel (AVX2) |
//! | Sudoku (hard) | 10μs | 17-clue puzzles |
//! | N-Queens 8 (count 92) | 4μs | Bit-parallel |
//!
//! ## Features
//!
//! - `egraph_native_chirho` (default): Native hardware-optimized e-graph
//! - `egg_chirho`: External egg crate integration
//! - `wasm_chirho`: WebAssembly bindings
//! - `goal_ast_chirho`: Goals as AST for introspection
//!
//! ## Module Organization
//!
//! **Core** ([`core_chirho`]):
//! - [`core_chirho::terms_chirho`]: Hash-consed term storage
//! - [`core_chirho::union_find_chirho`]: O(α(n)) variable equivalence classes
//! - [`core_chirho::unify_chirho`]: Unification with occurs check
//! - [`core_chirho::goals_chirho`]: Goal combinators (==, conde, conj, disj, not, conda, condu, =/=)
//! - [`core_chirho::stream_chirho`]: Lazy streams with interleaving
//!
//! **Hardware** ([`hardware_chirho`]):
//! - [`hardware_chirho::hardware_chirho`]: FPGA primitives (BitVec64, CAM)
//! - [`hardware_chirho::simd_chirho`]: AVX2 bulk operations
//! - [`hardware_chirho::optics_hw_chirho`]: Hardware optics (3000× faster than heap)
//! - [`hardware_chirho::bitmatrix_chirho`]: Sparse Boolean tensors (COO format)
//!
//! **Solvers** ([`solvers_chirho`]):
//! - [`solvers_chirho::sudoku_chirho`]: Sudoku solver (9-bit domains)
//! - [`solvers_chirho::nqueens_chirho`]: N-Queens solver (64-bit domains)
//!
//! **Semirings** ([`semiring_chirho`]):
//! - [`semiring_chirho::semiring_chirho`]: Semiring abstraction (Bool, Prob, Tropical, Count)
//! - [`semiring_chirho::diff_semiring_chirho`]: Differentiable relaxation with gradients
//!
//! **Experimental** ([`experimental_chirho`]):
//! - E-graphs, SMT, neural heuristics, category-theoretic abstractions

// ============================================================================
// Module hierarchy ☧
// ============================================================================

/// Reference implementation: traditional stream-based miniKanren
pub mod reference_chirho;

/// Hardware-accelerated primitives (FPGA-friendly, SIMD)
pub mod hardware_chirho;

/// Core API: facade that selects reference or hardware backend
pub mod core_chirho;

/// Ready-to-use constraint solvers
pub mod solvers_chirho;

/// Semiring abstractions for generalized provenance
pub mod semiring_chirho;

/// Experimental and research modules
pub mod experimental_chirho;

/// Domain approaches for infinite/hybrid miniKanren
/// - paged_chirho: Hierarchical bit vectors for large finite domains
/// - symbolic_chirho: Algebraic constraints (ranges, modular arithmetic)
/// - hybrid_chirho: Mixed finite/infinite with mode analysis
/// - complement_chirho: Cofinite sets (all except X)
/// - hw_symbolic_chirho: Hardware-accelerated symbolic operations
pub mod approaches_chirho;

// WebAssembly bindings (feature-gated, stays at root)
#[cfg(feature = "wasm_chirho")]
pub mod wasm_chirho;

// ============================================================================
// Backward-compatible re-exports (crate root)
// ============================================================================

// These allow `use minikanren_1bit_chirho::TermStoreChirho` to keep working

// Core types (from reference implementation)
pub use reference_chirho::terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
pub use reference_chirho::union_find_chirho::{UnionFindChirho, UnionFindHwChirho};
pub use reference_chirho::unify_chirho::{ground_eq_chirho, unify_chirho, SubstChirho, UnifyResultChirho};
pub use reference_chirho::goals_chirho::{
    conda_chirho, conde_chirho, conj_all_chirho, conj_chirho, condu_chirho, diseq_chirho,
    disj_all_chirho, disj_chirho, eq_chirho, fail_chirho, not_chirho, project_chirho,
    run_all_chirho, run_chirho, succeed_chirho, GoalFnChirho,
};
pub use reference_chirho::constraint_chirho::{BinaryConstraintChirho, ConstraintStoreChirho, DomainChirho};
pub use reference_chirho::tabling_chirho::{CallPatternChirho, LookupResultChirho, TableStoreChirho};
pub use reference_chirho::types_chirho::{
    EClassIdChirhoSafe, ENodeIdChirhoSafe, GoalIdChirho, SymIdChirho, TensorIdChirho,
    TermIdChirhoSafe, TypedIndexChirho, TypedVecChirho, VarIdChirho,
};

// Hardware types
pub use hardware_chirho::bitmatrix_chirho::{BitMatrixChirho, BitTensor3Chirho};
pub use hardware_chirho::bitmatrix_packed_chirho::{BitMatrix64Chirho, BitMatrixPackedChirho, Word64Chirho};
pub use hardware_chirho::hardware_chirho::{
    BitVec256Chirho, BitVec64Chirho, CamHwChirho, SearchState256HwChirho, SearchStateHwChirho,
    UnifyUnitHwChirho,
};
pub use hardware_chirho::optics_hw_chirho::{
    collect_nonempty_chirho, traverse_all_chirho, DomainHwChirho, LensHwChirho,
    PartitionedDomainChirho, PrismHwChirho, StateHwChirho,
};
pub use hardware_chirho::simd_chirho::{
    bulk_and_chirho, bulk_not_chirho, bulk_or_chirho, bulk_popcount_chirho, bulk_xor_chirho,
    AlignedBitMatrixChirho,
};

// Solver types
pub use solvers_chirho::nqueens_chirho::{NQueensSolverChirho, KNOWN_SOLUTIONS_CHIRHO};
pub use solvers_chirho::sudoku_chirho::{puzzles_chirho, SudokuSolverChirho};

// Semiring types
pub use semiring_chirho::contraction_semiring_chirho::{
    BoolTensorChirho, CountTensorChirho, ProbTensorChirho, SemiringNetworkChirho,
    SemiringTensorChirho, TropicalTensorChirho,
};
pub use semiring_chirho::diff_semiring_chirho::{
    annealed_temp_chirho, log_sum_exp_chirho, soft_eq_chirho, soft_eq_with_grad_chirho,
    DiffProbChirho, GumbelSoftmaxChirho, LearnableRelationChirho, StraightThroughChirho,
    WeightedTupleChirho,
};
pub use semiring_chirho::semiring_chirho::{
    BoolSemiringChirho, CountSemiringChirho, LogSemiringChirho, ProbSemiringChirho,
    SemiringChirho, TropicalSemiringChirho, WeightedMatrixChirho,
};

// Experimental types (always available)
pub use experimental_chirho::contraction_learn_chirho::{
    EdgeFeaturesChirho, LearnedContractionChirho, LinearEdgeScorerChirho, TensorNetworkChirho,
};
pub use experimental_chirho::jsonschema_chirho::{
    validate_chirho, JsonValueChirho, SchemaChirho, ValidationErrorChirho, ValidatorChirho,
};
pub use experimental_chirho::nested_pattern_chirho::{
    NestedPatternChirho, NodeConstraintChirho, NodeTypeChirho, PathStepChirho, TreePathChirho,
};
pub use experimental_chirho::neural_chirho::{
    beam_search_chirho, NeuralHeuristicChirho, NeuralStateChirho, SoftDomainChirho,
};
pub use experimental_chirho::recursion_chirho::{
    cata_indexed_chirho, depth_algebra_chirho, is_ground_algebra_chirho, para_indexed_chirho,
    size_algebra_chirho, vars_algebra_chirho, OccursCheckAlgebraChirho, TermFChirho,
    TermStoreIndexedChirho,
};
pub use experimental_chirho::slg_complete_chirho::{
    EvenOddTensorChirho, GoalStatusChirho, SlgGoalChirho, SlgTableChirho,
};
pub use experimental_chirho::smt_chirho::{
    domain_to_smt_chirho, SmtExprChirho, SmtProblemChirho, SmtSortChirho,
};
pub use experimental_chirho::unify_matrix_chirho::{unify_matrix_chirho, SubstMatrixChirho};

// Feature-gated experimental re-exports
#[cfg(feature = "egraph_native_chirho")]
pub use experimental_chirho::egraph_native_chirho::{
    EClassDataChirho, EClassIdChirho, EGraphNativeChirho, ENodeChirho, ENodeIdChirho,
};

#[cfg(feature = "egg_chirho")]
pub use experimental_chirho::egg_chirho::{
    arith_rules_chirho, list_rules_chirho, EggStoreChirho, TermAnalysisChirho, TermLangChirho,
};

#[cfg(feature = "gpu_chirho")]
pub use experimental_chirho::gpu_chirho::GpuContextChirho;

#[cfg(feature = "optics_chirho")]
pub use experimental_chirho::optics_chirho::{
    reify_optic_chirho, walk_deep_optic_chirho, ChildrenTraversalChirho, ConsPrismChirho,
    HeadLensChirho, IntPrismChirho, LensChirho, NilPrismChirho, PrismChirho,
    SubtermTraversalChirho, TailLensChirho, TraversalChirho, VarPrismChirho, VarsTraversalChirho,
};

#[cfg(feature = "free_goal_chirho")]
pub use experimental_chirho::free_goal_chirho::{
    conj_free_chirho, diseq_free_chirho, disj_free_chirho, eq_free_chirho, fresh_free_chirho,
    goal_to_smt_chirho, run_bool_chirho, run_prob_chirho, FreeGoalChirho, GoalFChirho as GoalFChirhoFree,
    SmtFormulaChirho,
};

#[cfg(feature = "comonad_chirho")]
pub use experimental_chirho::comonad_chirho::{propagate_chirho, ComonadChirho, SearchZipperChirho};

#[cfg(feature = "linear_chirho")]
pub use experimental_chirho::linear_chirho::{
    bang_chirho, par_chirho, tensor_chirho, LinearGoalChirho, ReusableGoalChirho,
};

// ============================================================================
// Legacy module aliases (for old import paths)
// ============================================================================

// These allow `use minikanren_1bit_chirho::terms_chirho::TermStoreChirho` to keep working
pub use reference_chirho::constraint_chirho;
pub use reference_chirho::goals_chirho;
pub use reference_chirho::stream_chirho;
pub use reference_chirho::tabling_chirho;
pub use reference_chirho::terms_chirho;
pub use reference_chirho::types_chirho;
// Note: unify_chirho module aliased to avoid conflict with unify_chirho function
pub use reference_chirho::unify_chirho as unify_mod_chirho;
pub use reference_chirho::union_find_chirho;

pub use hardware_chirho::bitmatrix_chirho;
pub use hardware_chirho::bitmatrix_packed_chirho;
pub use hardware_chirho::hardware_chirho as hardware_mod_chirho;
pub use hardware_chirho::optics_hw_chirho;
pub use hardware_chirho::simd_chirho;

pub use solvers_chirho::nqueens_chirho;
pub use solvers_chirho::sudoku_chirho;

pub use semiring_chirho::contraction_semiring_chirho;
pub use semiring_chirho::diff_semiring_chirho;
pub use semiring_chirho::semiring_chirho as semiring_mod_chirho;

pub use experimental_chirho::contraction_chirho;
pub use experimental_chirho::contraction_learn_chirho;
pub use experimental_chirho::jsonschema_chirho;
pub use experimental_chirho::nested_pattern_chirho;
pub use experimental_chirho::neural_chirho;
pub use experimental_chirho::recursion_chirho;
pub use experimental_chirho::relation_chirho;
pub use experimental_chirho::relations_chirho;
pub use experimental_chirho::slg_complete_chirho;
pub use experimental_chirho::smt_chirho;
// Note: unify_matrix_chirho module aliased to avoid conflict with unify_matrix_chirho function
pub use experimental_chirho::unify_matrix_chirho as unify_matrix_mod_chirho;

#[cfg(feature = "egraph_native_chirho")]
pub use experimental_chirho::egraph_native_chirho;

#[cfg(feature = "egg_chirho")]
pub use experimental_chirho::egg_chirho;

#[cfg(feature = "goal_ast_chirho")]
pub use experimental_chirho::goal_ast_chirho;

#[cfg(feature = "gpu_chirho")]
pub use experimental_chirho::gpu_chirho;

#[cfg(feature = "optics_chirho")]
pub use experimental_chirho::optics_chirho;

#[cfg(feature = "free_goal_chirho")]
pub use experimental_chirho::free_goal_chirho;

#[cfg(feature = "comonad_chirho")]
pub use experimental_chirho::comonad_chirho;

#[cfg(feature = "linear_chirho")]
pub use experimental_chirho::linear_chirho;
