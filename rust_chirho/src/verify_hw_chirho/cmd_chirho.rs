//! Search Engine Commands ☧
//!
//! Mirrors Clash SearchCmdChirho exactly for bit-identical verification.

/// Search engine command (mirrors Clash SearchCmdChirho)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCmdChirho {
    /// Reset to initial state
    InitChirho,
    /// Unify two variables (var1, var2)
    UnifyVarsChirho(u8, u8),
    /// Constrain variable to domain mask
    ConstrainVarChirho(u8, u64),
    /// Branch on variable (push alternative to stack)
    BranchVarChirho(u8),
    /// Pop from stack and continue
    BacktrackChirho,
    /// No operation
    NopChirho,
}

/// Search engine response (mirrors Clash SearchRespChirho)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRespChirho {
    /// Valid flag (false = failed state)
    pub valid_chirho: bool,
    /// Solution flag (true = all domains are singletons)
    pub solution_chirho: bool,
    /// Current domains (8 variables × 64-bit)
    pub domains_chirho: [u64; 8],
}

impl SearchRespChirho {
    /// Create a new response
    pub fn new_chirho(valid_chirho: bool, solution_chirho: bool, domains_chirho: [u64; 8]) -> Self {
        Self {
            valid_chirho,
            solution_chirho,
            domains_chirho,
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_cmd_size_chirho() {
        // Ensure command enum is small for efficient simulation
        assert!(std::mem::size_of::<SearchCmdChirho>() <= 16);
    }
}
