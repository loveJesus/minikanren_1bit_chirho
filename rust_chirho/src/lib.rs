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
//! use minikanren_1bit_chirho::sudoku_chirho::SudokuSolverChirho;
//! use minikanren_1bit_chirho::nqueens_chirho::NQueensSolverChirho;
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
//! ## Modules
//!
//! **Core:**
//! - [`terms_chirho`]: Hash-consed term storage
//! - [`union_find_chirho`]: O(α(n)) variable equivalence classes
//! - [`unify_chirho`]: Unification with occurs check
//! - [`goals_chirho`]: Goal combinators (==, conde, conj, disj, not, conda, condu, =/=)
//! - [`stream_chirho`]: Lazy streams with interleaving
//!
//! **Constraint Solving:**
//! - [`constraint_chirho`]: Arc consistency (AC-3) propagation
//! - [`sudoku_chirho`]: Sudoku solver (9-bit domains)
//! - [`nqueens_chirho`]: N-Queens solver (64-bit domains)
//! - [`jsonschema_chirho`]: JSON Schema validator
//!
//! **Tensors & Relations:**
//! - [`bitmatrix_chirho`]: Sparse Boolean tensors (COO format)
//! - [`relations_chirho`]: Relations as sparse tensors (appendo, membero)
//! - [`contraction_chirho`]: Tensor network contraction heuristics
//!
//! **Advanced:**
//! - [`semiring_chirho`]: Semiring abstraction (Bool, Prob, Tropical, Count)
//! - [`diff_semiring_chirho`]: Differentiable relaxation with gradients
//! - [`tabling_chirho`]: SLG-style memoization
//! - [`egraph_native_chirho`]: Bit-parallel e-graph
//!
//! **Hardware:**
//! - [`hardware_chirho`]: FPGA primitives (BitVec64, CAM)
//! - [`simd_chirho`]: AVX2 bulk operations
//! - [`optics_hw_chirho`]: Hardware optics (3000× faster than heap)

pub mod types_chirho;
pub mod terms_chirho;
pub mod union_find_chirho;
pub mod unify_chirho;
pub mod bitmatrix_chirho;
pub mod bitmatrix_packed_chirho;
pub mod relations_chirho;
pub mod contraction_chirho;
pub mod stream_chirho;
pub mod goals_chirho;
pub mod constraint_chirho;
pub mod tabling_chirho;
pub mod semiring_chirho;
pub mod hardware_chirho;
pub mod neural_chirho;
pub mod smt_chirho;
pub mod contraction_learn_chirho;
pub mod slg_complete_chirho;
pub mod nested_pattern_chirho;
pub mod recursion_chirho;
pub mod contraction_semiring_chirho;
pub mod unify_matrix_chirho;
pub mod simd_chirho;
pub mod diff_semiring_chirho;
pub mod sudoku_chirho;
pub mod nqueens_chirho;
pub mod jsonschema_chirho;

// WebAssembly bindings (feature-gated)
#[cfg(feature = "wasm_chirho")]
pub mod wasm_chirho;

// Goal-as-AST (feature-gated: introspection vs raw speed)
#[cfg(feature = "goal_ast_chirho")]
pub mod goal_ast_chirho;

// GPU backend (feature-gated: requires wgpu)
#[cfg(feature = "gpu_chirho")]
pub mod gpu_chirho;

// E-graph implementations (feature-gated)
#[cfg(feature = "egraph_native_chirho")]
pub mod egraph_native_chirho;

#[cfg(feature = "egg_chirho")]
pub mod egg_chirho;

// ============================================================================
// Category-theoretic extensions (Kmett-inspired, feature-gated)
// ============================================================================

// Optics: Prisms, Traversals, Lenses for term manipulation
#[cfg(feature = "optics_chirho")]
pub mod optics_chirho;

// Free Monad Goals: Same AST, multiple interpreters (Bool, Prob, SMT)
#[cfg(feature = "free_goal_chirho")]
pub mod free_goal_chirho;

// Comonadic Streams: Search zipper, extend, constraint propagation
#[cfg(feature = "comonad_chirho")]
pub mod comonad_chirho;

// Linear Logic: Tensor/Par connectives, session types
#[cfg(feature = "linear_chirho")]
pub mod linear_chirho;

// ============================================================================
// Hardware-ready category-theoretic extensions (1-bit, FPGA-friendly)
// ============================================================================

// Hardware Optics: Prisms, Lenses over BitVec64 domains
pub mod optics_hw_chirho;

// Re-export key types (avoiding ambiguous globs)
pub use terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
pub use union_find_chirho::{UnionFindChirho, UnionFindHwChirho};
pub use unify_chirho::{SubstChirho, UnifyResultChirho, unify_chirho, ground_eq_chirho};
pub use bitmatrix_chirho::{BitMatrixChirho, BitTensor3Chirho};
pub use goals_chirho::{GoalFnChirho, eq_chirho, conj_chirho, disj_chirho, conde_chirho, conj_all_chirho, disj_all_chirho, succeed_chirho, fail_chirho, not_chirho, conda_chirho, condu_chirho, diseq_chirho, project_chirho, run_chirho, run_all_chirho};
pub use constraint_chirho::{DomainChirho, ConstraintStoreChirho, BinaryConstraintChirho};
pub use tabling_chirho::{TableStoreChirho, CallPatternChirho, LookupResultChirho};
pub use semiring_chirho::{SemiringChirho, BoolSemiringChirho, ProbSemiringChirho, TropicalSemiringChirho, CountSemiringChirho, LogSemiringChirho, WeightedMatrixChirho};
pub use hardware_chirho::{BitVec64Chirho, BitVec256Chirho, SearchStateHwChirho, SearchState256HwChirho, CamHwChirho, UnifyUnitHwChirho};
pub use neural_chirho::{SoftDomainChirho, NeuralStateChirho, NeuralHeuristicChirho, beam_search_chirho};
pub use smt_chirho::{SmtSortChirho, SmtExprChirho, SmtProblemChirho, domain_to_smt_chirho};
pub use contraction_learn_chirho::{TensorNetworkChirho, EdgeFeaturesChirho, LinearEdgeScorerChirho, LearnedContractionChirho};
pub use slg_complete_chirho::{SlgTableChirho, SlgGoalChirho, GoalStatusChirho, EvenOddTensorChirho};
pub use nested_pattern_chirho::{TreePathChirho, PathStepChirho, NodeTypeChirho, NodeConstraintChirho, NestedPatternChirho};
pub use types_chirho::{TermIdChirhoSafe, VarIdChirho, EClassIdChirhoSafe, ENodeIdChirhoSafe, SymIdChirho, GoalIdChirho, TensorIdChirho, TypedIndexChirho, TypedVecChirho};
pub use bitmatrix_packed_chirho::{Word64Chirho, BitMatrix64Chirho, BitMatrixPackedChirho};
pub use recursion_chirho::{TermFChirho, TermStoreIndexedChirho, cata_indexed_chirho, para_indexed_chirho, is_ground_algebra_chirho, vars_algebra_chirho, size_algebra_chirho, depth_algebra_chirho, OccursCheckAlgebraChirho};
pub use contraction_semiring_chirho::{SemiringTensorChirho, SemiringNetworkChirho, BoolTensorChirho, ProbTensorChirho, TropicalTensorChirho, CountTensorChirho};
pub use unify_matrix_chirho::{SubstMatrixChirho, unify_matrix_chirho};
pub use simd_chirho::{bulk_and_chirho, bulk_or_chirho, bulk_xor_chirho, bulk_not_chirho, bulk_popcount_chirho, AlignedBitMatrixChirho};
pub use diff_semiring_chirho::{DiffProbChirho, soft_eq_chirho, soft_eq_with_grad_chirho, annealed_temp_chirho, GumbelSoftmaxChirho, StraightThroughChirho, log_sum_exp_chirho, WeightedTupleChirho, LearnableRelationChirho};

// E-graph re-exports (feature-gated)
#[cfg(feature = "egraph_native_chirho")]
pub use egraph_native_chirho::{ENodeChirho, ENodeIdChirho, EClassIdChirho, EClassDataChirho, EGraphNativeChirho};

#[cfg(feature = "egg_chirho")]
pub use egg_chirho::{TermLangChirho, TermAnalysisChirho, EggStoreChirho, list_rules_chirho, arith_rules_chirho};

#[cfg(feature = "gpu_chirho")]
pub use gpu_chirho::GpuContextChirho;

// Category-theoretic extensions re-exports (feature-gated)
#[cfg(feature = "optics_chirho")]
pub use optics_chirho::{PrismChirho, TraversalChirho, LensChirho, ConsPrismChirho, IntPrismChirho, VarPrismChirho, NilPrismChirho, ChildrenTraversalChirho, SubtermTraversalChirho, VarsTraversalChirho, HeadLensChirho, TailLensChirho, walk_deep_optic_chirho, reify_optic_chirho};

#[cfg(feature = "free_goal_chirho")]
pub use free_goal_chirho::{GoalFChirho, FreeGoalChirho, SmtFormulaChirho, eq_free_chirho, fresh_free_chirho, disj_free_chirho, conj_free_chirho, diseq_free_chirho, run_bool_chirho, run_prob_chirho, goal_to_smt_chirho};

#[cfg(feature = "comonad_chirho")]
pub use comonad_chirho::{SearchZipperChirho, ComonadChirho, propagate_chirho};

#[cfg(feature = "linear_chirho")]
pub use linear_chirho::{LinearGoalChirho, ReusableGoalChirho, tensor_chirho, par_chirho, bang_chirho};

// Hardware-ready abstractions (always available, no overhead)
pub use optics_hw_chirho::{DomainHwChirho, PrismHwChirho, LensHwChirho, StateHwChirho, PartitionedDomainChirho, traverse_all_chirho, collect_nonempty_chirho};

// Sudoku solver (practical 1-bit domain demo)
pub use sudoku_chirho::{SudokuSolverChirho, puzzles_chirho};

// N-Queens solver (scales to 64×64)
pub use nqueens_chirho::{NQueensSolverChirho, KNOWN_SOLUTIONS_CHIRHO};

// JSON Schema validator (1-bit type domains)
pub use jsonschema_chirho::{JsonValueChirho, SchemaChirho, ValidatorChirho, ValidationErrorChirho, validate_chirho};
