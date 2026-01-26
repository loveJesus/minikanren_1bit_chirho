// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Verilator FFI Bindings ☧
//!
//! Rust interface to the Verilator simulation of searchEngineChirho.
//! Part of P5-00: Bridge of Truth.
//!
//! When the `verilator_chirho` feature is enabled and Verilator is properly
//! set up, this module provides co-simulation with the Clash-generated Verilog.

use super::cmd_chirho::{SearchCmdChirho, SearchRespChirho};

/// C-compatible command struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdFFIChirho {
    pub tag_chirho: u8,
    pub var1_chirho: u8,
    pub var2_chirho: u8,
    pub mask_chirho: u64,
}

/// C-compatible response struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RespFFIChirho {
    pub valid_chirho: u8,
    pub solution_chirho: u8,
    pub domains_chirho: [u64; 8],
}

impl From<SearchCmdChirho> for CmdFFIChirho {
    fn from(cmd_chirho: SearchCmdChirho) -> Self {
        match cmd_chirho {
            SearchCmdChirho::InitChirho => CmdFFIChirho {
                tag_chirho: 0,
                var1_chirho: 0,
                var2_chirho: 0,
                mask_chirho: 0,
            },
            SearchCmdChirho::UnifyVarsChirho(v1, v2) => CmdFFIChirho {
                tag_chirho: 1,
                var1_chirho: v1,
                var2_chirho: v2,
                mask_chirho: 0,
            },
            SearchCmdChirho::ConstrainVarChirho(v, mask) => CmdFFIChirho {
                tag_chirho: 2,
                var1_chirho: v,
                var2_chirho: 0,
                mask_chirho: mask,
            },
            SearchCmdChirho::BranchVarChirho(v) => CmdFFIChirho {
                tag_chirho: 3,
                var1_chirho: v,
                var2_chirho: 0,
                mask_chirho: 0,
            },
            SearchCmdChirho::BacktrackChirho => CmdFFIChirho {
                tag_chirho: 4,
                var1_chirho: 0,
                var2_chirho: 0,
                mask_chirho: 0,
            },
            SearchCmdChirho::NopChirho => CmdFFIChirho {
                tag_chirho: 5,
                var1_chirho: 0,
                var2_chirho: 0,
                mask_chirho: 0,
            },
        }
    }
}

impl From<RespFFIChirho> for SearchRespChirho {
    fn from(resp_chirho: RespFFIChirho) -> Self {
        SearchRespChirho {
            valid_chirho: resp_chirho.valid_chirho != 0,
            solution_chirho: resp_chirho.solution_chirho != 0,
            domains_chirho: resp_chirho.domains_chirho,
        }
    }
}

// FFI declarations (only available when verilator_chirho feature is enabled)
#[cfg(feature = "verilator_chirho")]
extern "C" {
    fn verilator_init_chirho() -> i32;
    fn verilator_cleanup_chirho();
    fn verilator_step_chirho(cmd_chirho: *const CmdFFIChirho, resp_chirho: *mut RespFFIChirho) -> i32;
    fn verilator_available_chirho() -> i32;
}

/// Verilator simulation wrapper
pub struct VerilatorSimChirho {
    _private: (), // Prevent direct construction
}

impl VerilatorSimChirho {
    /// Check if Verilator is available
    #[cfg(feature = "verilator_chirho")]
    pub fn is_available_chirho() -> bool {
        unsafe { verilator_available_chirho() != 0 }
    }

    #[cfg(not(feature = "verilator_chirho"))]
    pub fn is_available_chirho() -> bool {
        false
    }

    /// Initialize the Verilator simulation
    #[cfg(feature = "verilator_chirho")]
    pub fn new_chirho() -> Result<Self, &'static str> {
        let result_chirho = unsafe { verilator_init_chirho() };
        if result_chirho == 0 {
            Ok(Self { _private: () })
        } else {
            Err("Failed to initialize Verilator simulation")
        }
    }

    #[cfg(not(feature = "verilator_chirho"))]
    pub fn new_chirho() -> Result<Self, &'static str> {
        Err("Verilator not available (compile with --features verilator_chirho)")
    }

    /// Execute one step in the Verilator simulation
    #[cfg(feature = "verilator_chirho")]
    pub fn step_chirho(&mut self, cmd_chirho: SearchCmdChirho) -> Result<SearchRespChirho, &'static str> {
        let ffi_cmd_chirho = CmdFFIChirho::from(cmd_chirho);
        let mut ffi_resp_chirho = RespFFIChirho {
            valid_chirho: 0,
            solution_chirho: 0,
            domains_chirho: [0; 8],
        };

        let result_chirho = unsafe {
            verilator_step_chirho(&ffi_cmd_chirho, &mut ffi_resp_chirho)
        };

        if result_chirho == 0 {
            Ok(SearchRespChirho::from(ffi_resp_chirho))
        } else {
            Err("Verilator step failed")
        }
    }

    #[cfg(not(feature = "verilator_chirho"))]
    pub fn step_chirho(&mut self, _cmd_chirho: SearchCmdChirho) -> Result<SearchRespChirho, &'static str> {
        Err("Verilator not available")
    }
}

#[cfg(feature = "verilator_chirho")]
impl Drop for VerilatorSimChirho {
    fn drop(&mut self) {
        unsafe { verilator_cleanup_chirho() };
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_ffi_struct_sizes_chirho() {
        // Verify FFI struct sizes match C expectations
        assert_eq!(std::mem::size_of::<CmdFFIChirho>(), 16);
        assert_eq!(std::mem::size_of::<RespFFIChirho>(), 72); // 2 + 6 padding + 64
    }

    #[test]
    fn test_cmd_conversion_chirho() {
        let cmd_chirho = SearchCmdChirho::UnifyVarsChirho(3, 5);
        let ffi_chirho = CmdFFIChirho::from(cmd_chirho);
        assert_eq!(ffi_chirho.tag_chirho, 1);
        assert_eq!(ffi_chirho.var1_chirho, 3);
        assert_eq!(ffi_chirho.var2_chirho, 5);
    }

    #[test]
    fn test_verilator_not_available_chirho() {
        // Without the feature, Verilator should not be available
        #[cfg(not(feature = "verilator_chirho"))]
        {
            assert!(!VerilatorSimChirho::is_available_chirho());
            assert!(VerilatorSimChirho::new_chirho().is_err());
        }
    }
}
