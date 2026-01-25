//! Reference miniKanren implementation ☧
//!
//! Traditional stream-based miniKanren. Used for:
//! - Compatibility testing against hardware implementation
//! - Symbolic terms with arbitrary nesting
//! - Baseline benchmarking
//!
//! The foundational components:
//! - Term storage (hash-consed)
//! - Unification with occurs check
//! - Goal combinators (==, conde, conj, disj, not, conda, condu, =/=)
//! - Lazy streams with interleaving
//! - Union-Find for equivalence classes

pub mod constraint_chirho;
pub mod goals_chirho;
pub mod stream_chirho;
pub mod tabling_chirho;
pub mod terms_chirho;
pub mod types_chirho;
pub mod unify_chirho;
pub mod union_find_chirho;

// External library comparison (feature-gated)
#[cfg(feature = "egg_chirho")]
pub mod egg_chirho;

// Re-export key types for convenience
pub use constraint_chirho::{BinaryConstraintChirho, ConstraintStoreChirho, DomainChirho};
pub use goals_chirho::{
    conda_chirho, conde_chirho, conj_all_chirho, conj_chirho, diseq_chirho, disj_all_chirho,
    disj_chirho, eq_chirho, fail_chirho, not_chirho, condu_chirho, project_chirho, run_all_chirho,
    run_chirho, succeed_chirho, GoalFnChirho,
};
pub use stream_chirho::StreamChirho;
pub use tabling_chirho::{CallPatternChirho, LookupResultChirho, TableStoreChirho};
pub use terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
pub use types_chirho::{
    EClassIdChirhoSafe, ENodeIdChirhoSafe, GoalIdChirho, SymIdChirho, TensorIdChirho,
    TermIdChirhoSafe, TypedIndexChirho, TypedVecChirho, VarIdChirho,
};
pub use unify_chirho::{ground_eq_chirho, unify_chirho, SubstChirho, UnifyResultChirho};
pub use union_find_chirho::{UnionFindChirho, UnionFindHwChirho};

// External library comparison re-exports
#[cfg(feature = "egg_chirho")]
pub use egg_chirho::{
    arith_rules_chirho, list_rules_chirho, EggStoreChirho, TermAnalysisChirho, TermLangChirho,
};
