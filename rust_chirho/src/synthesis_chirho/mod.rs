//! Program Synthesis via Domain-Pruned Enumeration ☧
//!
//! This module implements SyGuS-style program synthesis using our 1-bit
//! domain representation for efficient pruning.
//!
//! # Architecture
//!
//! - `sygus_chirho`: Parser for SyGuS .sl files
//! - `grammar_chirho`: Grammar encoding as Hierarchical4kChirho domains
//! - `enumerate_chirho`: Domain-pruned enumeration with example propagation

pub mod sygus_chirho;
pub mod grammar_chirho;
pub mod enumerate_chirho;

pub use sygus_chirho::*;
pub use grammar_chirho::*;
pub use enumerate_chirho::*;
