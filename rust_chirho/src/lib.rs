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
pub mod gpu_chirho;
pub mod neural_chirho;
pub mod smt_chirho;
pub mod contraction_learn_chirho;
pub mod slg_complete_chirho;
pub mod nested_pattern_chirho;

// E-graph implementations (feature-gated)
#[cfg(feature = "egraph_native_chirho")]
pub mod egraph_native_chirho;

#[cfg(feature = "egg_chirho")]
pub mod egg_chirho;

// Re-export key types (avoiding ambiguous globs)
pub use terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
pub use union_find_chirho::{UnionFindChirho, UnionFindHwChirho};
pub use unify_chirho::{SubstChirho, UnifyResultChirho, unify_chirho};
pub use bitmatrix_chirho::{BitMatrixChirho, BitTensor3Chirho};
pub use goals_chirho::{GoalFnChirho, eq_chirho, conj_chirho, disj_chirho, conde_chirho, conj_all_chirho, disj_all_chirho, succeed_chirho, fail_chirho, run_chirho, run_all_chirho};
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

// E-graph re-exports (feature-gated)
#[cfg(feature = "egraph_native_chirho")]
pub use egraph_native_chirho::{ENodeChirho, ENodeIdChirho, EClassIdChirho, EClassDataChirho, EGraphNativeChirho};

#[cfg(feature = "egg_chirho")]
pub use egg_chirho::{TermLangChirho, TermAnalysisChirho, EggStoreChirho, list_rules_chirho, arith_rules_chirho};
