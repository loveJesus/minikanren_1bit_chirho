//! miniKanren as 1-Bit Matrix Operations ☧
//!
//! Rust implementation for performance, with hardware-friendly abstractions.
//!
//! Core equation: miniKanren search = sparse Boolean tensor network contraction
//!
//! # Features
//!
//! - `egraph_native_chirho` (default): Native hardware-optimized e-graph
//! - `egg_chirho`: External egg crate integration (more features, less hw-friendly)
//! - `goal_ast_chirho`: Goals as AST for introspection (15-70% slower, not for FPGA)
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
//! - `hardware_chirho`: FPGA/ASIC-oriented primitives (BitVec64, CAM, parallel ops)
//! - `gpu_chirho`: GPU backend sketch (SIMT-style parallel search)
//! - `egraph_native_chirho`: Native e-graph (hardware-optimized, bit-parallel)
//! - `egg_chirho`: External egg crate wrapper (feature-gated)

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
