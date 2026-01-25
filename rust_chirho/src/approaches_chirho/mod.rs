//! Domain Approaches for Infinite miniKanren ☧
//!
//! Different strategies for extending beyond 64-value domains:
//!
//! | Approach | Domain Size | Hardware Accel | Use Case |
//! |----------|-------------|----------------|----------|
//! | `paged_chirho` | Arbitrary integers | Yes (per page) | Large finite domains |
//! | `symbolic_chirho` | Infinite (ranges, mod) | Partial | Linear constraints |
//! | `hybrid_chirho` | Mixed finite/infinite | Yes (finite parts) | General miniKanren |
//! | `complement_chirho` | Cofinite sets | No | "All except X" |
//!
//! ## Unification with experimental_chirho
//!
//! - `smt_chirho`: Symbolic domains can export to SMT-LIB2
//! - `unify_matrix_chirho`: Occurs check via bit matrix
//! - `free_goal_chirho`: Goals as AST enable mode analysis

pub mod paged_chirho;
pub mod symbolic_chirho;
pub mod hybrid_chirho;
pub mod complement_chirho;
pub mod hw_symbolic_chirho;
pub mod hierarchical_chirho;
pub mod diff_hierarchical_chirho;

pub use paged_chirho::PagedDomainChirho;
pub use symbolic_chirho::{SymbolicDomainChirho, SymbolicConstraintChirho};
pub use hybrid_chirho::{HybridDomainChirho, HybridStateChirho};
pub use complement_chirho::ComplementDomainChirho;
pub use hw_symbolic_chirho::{HwSymbolicDomainChirho, SymbolicOpChirho};
pub use hierarchical_chirho::{Hierarchical4kChirho, Hierarchical256kChirho};
pub use diff_hierarchical_chirho::{DiffHierarchical4kChirho, DiffUnifyStateChirho};
