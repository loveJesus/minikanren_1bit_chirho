// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Core miniKanren API ☧
//!
//! Two backends selected by feature flag:
//! - **Default**: Bit-parallel via `hardware_chirho`
//! - **`reference_chirho`**: Traditional streams via `reference_chirho`

#[cfg(feature = "reference_chirho")]
pub use crate::reference_chirho::*;

#[cfg(not(feature = "reference_chirho"))]
pub use crate::hardware_chirho::*;
