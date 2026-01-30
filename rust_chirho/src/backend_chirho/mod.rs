// For God so loved the world that He gave His only begotten Son ☧
//! Pluggable Solver Backends
//!
//! This module provides a unified interface for constraint solving that can
//! be backed by either CPU (Rust/SIMD) or FPGA (AWS F2) implementations.
//!
//! # Usage
//!
//! ```rust
//! use minikanren_1bit_chirho::backend_chirho::{
//!     SolverBackendChirho,
//!     BackendConfigChirho,
//! };
//!
//! // Option 1: Auto-detect (checks env var + hardware)
//! let backend_chirho = BackendConfigChirho::auto_chirho().build_chirho();
//!
//! // Option 2: Explicit preference
//! let backend_chirho = BackendConfigChirho::new_chirho()
//!     .prefer_fpga_chirho(true)
//!     .build_chirho();
//!
//! // Option 3: Force CPU (useful for testing)
//! let backend_chirho = BackendConfigChirho::cpu_only_chirho().build_chirho();
//!
//! // Use the backend
//! let result_chirho = backend_chirho.intersect_64_chirho(0xFF00, 0x0FF0);
//! assert_eq!(result_chirho, 0x0F00);
//! ```

mod trait_chirho;
mod cpu_chirho;

#[cfg(feature = "fpga_chirho")]
mod fpga_chirho;

pub use trait_chirho::{
    SolverBackendChirho,
    BackendInfoChirho,
    ConstraintChirho,
    SolutionChirho,
    DomainVecChirho,
};

pub use cpu_chirho::CpuBackendChirho;

#[cfg(feature = "fpga_chirho")]
pub use fpga_chirho::FpgaBackendChirho;

use std::sync::Arc;

/// Backend selection configuration
///
/// # Examples
///
/// ```rust
/// use minikanren_1bit_chirho::backend_chirho::BackendConfigChirho;
///
/// // Auto-detect best backend
/// let config_chirho = BackendConfigChirho::auto_chirho();
///
/// // Prefer FPGA if available
/// let config_chirho = BackendConfigChirho::new_chirho()
///     .prefer_fpga_chirho(true);
///
/// // Force CPU only
/// let config_chirho = BackendConfigChirho::cpu_only_chirho();
/// ```
#[derive(Debug, Clone)]
pub struct BackendConfigChirho {
    /// Try to use FPGA if available
    prefer_fpga_chirho: bool,
    /// Check environment variable for FPGA setting
    check_env_chirho: bool,
    /// FPGA slot number (usually 0)
    #[cfg(feature = "fpga_chirho")]
    fpga_slot_chirho: u32,
}

impl Default for BackendConfigChirho {
    fn default() -> Self {
        Self::new_chirho()
    }
}

impl BackendConfigChirho {
    /// Create a new config with defaults (CPU only, no env check)
    pub fn new_chirho() -> Self {
        Self {
            prefer_fpga_chirho: false,
            check_env_chirho: false,
            #[cfg(feature = "fpga_chirho")]
            fpga_slot_chirho: 0,
        }
    }

    /// Auto-detect: check env var, then try FPGA, fall back to CPU
    pub fn auto_chirho() -> Self {
        Self {
            prefer_fpga_chirho: true,
            check_env_chirho: true,
            #[cfg(feature = "fpga_chirho")]
            fpga_slot_chirho: 0,
        }
    }

    /// Force CPU only (useful for testing, benchmarking)
    pub fn cpu_only_chirho() -> Self {
        Self {
            prefer_fpga_chirho: false,
            check_env_chirho: false,
            #[cfg(feature = "fpga_chirho")]
            fpga_slot_chirho: 0,
        }
    }

    /// Set FPGA preference
    pub fn prefer_fpga_chirho(mut self, prefer_chirho: bool) -> Self {
        self.prefer_fpga_chirho = prefer_chirho;
        self
    }

    /// Set whether to check `FPGA_ENABLED_CHIRHO` env var
    pub fn check_env_chirho(mut self, check_chirho: bool) -> Self {
        self.check_env_chirho = check_chirho;
        self
    }

    /// Set FPGA slot number (only relevant with fpga_chirho feature)
    #[cfg(feature = "fpga_chirho")]
    pub fn fpga_slot_chirho(mut self, slot_chirho: u32) -> Self {
        self.fpga_slot_chirho = slot_chirho;
        self
    }

    /// Build the backend based on configuration
    pub fn build_chirho(&self) -> Arc<dyn SolverBackendChirho> {
        // Check if FPGA should be attempted
        let try_fpga_chirho = self.prefer_fpga_chirho
            || (self.check_env_chirho && std::env::var("FPGA_ENABLED_CHIRHO").is_ok());

        #[cfg(feature = "fpga_chirho")]
        if try_fpga_chirho {
            match fpga_chirho::FpgaBackendChirho::connect_chirho() {
                Ok(fpga_backend_chirho) => {
                    return Arc::new(fpga_backend_chirho);
                }
                Err(_e_chirho) => {
                    // Fall through to CPU
                }
            }
        }

        #[cfg(not(feature = "fpga_chirho"))]
        let _ = try_fpga_chirho; // suppress warning

        Arc::new(CpuBackendChirho::new_chirho())
    }

    /// Check if FPGA backend would be available with current config
    pub fn fpga_available_chirho(&self) -> bool {
        #[cfg(feature = "fpga_chirho")]
        {
            fpga_chirho::FpgaBackendChirho::connect_chirho().is_ok()
        }
        #[cfg(not(feature = "fpga_chirho"))]
        {
            false
        }
    }
}

/// Create the best available backend (convenience function)
///
/// Equivalent to `BackendConfigChirho::auto_chirho().build_chirho()`
pub fn create_backend_chirho() -> Arc<dyn SolverBackendChirho> {
    BackendConfigChirho::auto_chirho().build_chirho()
}

/// Create CPU backend explicitly
pub fn create_cpu_backend_chirho() -> Arc<dyn SolverBackendChirho> {
    BackendConfigChirho::cpu_only_chirho().build_chirho()
}

/// Create FPGA backend explicitly (fails if unavailable)
#[cfg(feature = "fpga_chirho")]
pub fn create_fpga_backend_chirho() -> Result<Arc<dyn SolverBackendChirho>, String> {
    let fpga_chirho = fpga_chirho::FpgaBackendChirho::connect_chirho()?;
    Ok(Arc::new(fpga_chirho))
}
