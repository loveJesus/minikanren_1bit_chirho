//! Differentiable Learning for miniKanren ☧
//!
//! Enables gradient-based learning through logic programs using soft relaxation.
//!
//! # Architecture
//!
//! - `learnable_chirho`: Relations with learnable tuple weights
//! - `anneal_chirho`: Temperature annealing for soft-to-hard transitions
//! - `gumbel_chirho`: Gumbel-softmax for differentiable discrete choices

pub mod learnable_chirho;
pub mod anneal_chirho;
pub mod gumbel_chirho;

pub use learnable_chirho::*;
pub use anneal_chirho::*;
pub use gumbel_chirho::*;
