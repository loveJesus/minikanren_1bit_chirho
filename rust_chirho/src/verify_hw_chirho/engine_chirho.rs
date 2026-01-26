//! Search Engine Reference Implementation ☧
//!
//! Rust implementation that mirrors Clash EngineStateChirho exactly.
//! Used as the "truth" side of the verification bridge.
//!
//! "Buy the truth, and sell it not." — Proverbs 23:23

use super::cmd_chirho::{SearchCmdChirho, SearchRespChirho};

/// Stack entry for backtracking (mirrors Clash StackEntryChirho)
#[derive(Debug, Clone, Copy)]
pub struct StackEntryChirho {
    /// Variable index that was branched on
    pub var_chirho: u8,
    /// Remaining domain (the "hi" branch)
    pub domain_chirho: u64,
    /// Snapshot of all domains at branch point
    pub domains_chirho: [u64; 8],
}

/// Search engine state (mirrors Clash EngineStateChirho exactly)
#[derive(Debug, Clone)]
pub struct EngineStateChirho {
    /// Current domains for 8 variables
    pub domains_chirho: [u64; 8],
    /// Valid flag
    pub valid_chirho: bool,
    /// Backtracking stack (max 16 entries)
    pub stack_chirho: [Option<StackEntryChirho>; 16],
    /// Stack pointer
    pub sp_chirho: usize,
}

impl Default for EngineStateChirho {
    fn default() -> Self {
        Self::new_chirho()
    }
}

impl EngineStateChirho {
    /// Full domain: all 64 values possible
    pub const FULL_DOMAIN_CHIRHO: u64 = u64::MAX;

    /// Create initial engine state (all full domains)
    pub fn new_chirho() -> Self {
        Self {
            domains_chirho: [Self::FULL_DOMAIN_CHIRHO; 8],
            valid_chirho: true,
            stack_chirho: [None; 16],
            sp_chirho: 0,
        }
    }

    /// Check if domain is empty (failure)
    #[inline]
    fn is_empty_chirho(domain_chirho: u64) -> bool {
        domain_chirho == 0
    }

    /// Check if domain is singleton (exactly one value)
    #[inline]
    fn is_singleton_chirho(domain_chirho: u64) -> bool {
        domain_chirho != 0 && (domain_chirho & (domain_chirho - 1)) == 0
    }

    /// Get lowest set bit (isolate one possible value)
    #[inline]
    fn lowest_bit_chirho(domain_chirho: u64) -> u64 {
        domain_chirho & domain_chirho.wrapping_neg()
    }

    /// Clear lowest set bit (remaining possibilities)
    #[inline]
    fn clear_lowest_chirho(domain_chirho: u64) -> u64 {
        domain_chirho & (domain_chirho - 1)
    }

    /// Fork: split domain into (lowest single value, remaining values)
    #[inline]
    fn fork_chirho(domain_chirho: u64) -> (u64, u64) {
        (
            Self::lowest_bit_chirho(domain_chirho),
            Self::clear_lowest_chirho(domain_chirho),
        )
    }

    /// Process a command and return (new_state, response)
    /// This mirrors Clash engineStepChirho EXACTLY
    pub fn step_chirho(&self, cmd_chirho: SearchCmdChirho) -> (Self, SearchRespChirho) {
        let mut new_state_chirho = self.clone();

        match cmd_chirho {
            SearchCmdChirho::InitChirho => {
                new_state_chirho = Self::new_chirho();
            }

            SearchCmdChirho::UnifyVarsChirho(v1_chirho, v2_chirho) => {
                let v1_chirho = (v1_chirho as usize) % 8;
                let v2_chirho = (v2_chirho as usize) % 8;

                let d1_chirho = new_state_chirho.domains_chirho[v1_chirho];
                let d2_chirho = new_state_chirho.domains_chirho[v2_chirho];
                let result_chirho = d1_chirho & d2_chirho;

                new_state_chirho.domains_chirho[v1_chirho] = result_chirho;
                new_state_chirho.domains_chirho[v2_chirho] = result_chirho;
                new_state_chirho.valid_chirho =
                    new_state_chirho.valid_chirho && !Self::is_empty_chirho(result_chirho);
            }

            SearchCmdChirho::ConstrainVarChirho(v_chirho, mask_chirho) => {
                let v_chirho = (v_chirho as usize) % 8;

                let d_chirho = new_state_chirho.domains_chirho[v_chirho];
                let result_chirho = d_chirho & mask_chirho;

                new_state_chirho.domains_chirho[v_chirho] = result_chirho;
                new_state_chirho.valid_chirho =
                    new_state_chirho.valid_chirho && !Self::is_empty_chirho(result_chirho);
            }

            SearchCmdChirho::BranchVarChirho(v_chirho) => {
                let v_chirho = (v_chirho as usize) % 8;

                let d_chirho = new_state_chirho.domains_chirho[v_chirho];
                let (lo_chirho, hi_chirho) = Self::fork_chirho(d_chirho);

                // Push alternative (hi branch) to stack
                if new_state_chirho.sp_chirho < 16 {
                    let entry_chirho = StackEntryChirho {
                        var_chirho: v_chirho as u8,
                        domain_chirho: hi_chirho,
                        domains_chirho: new_state_chirho.domains_chirho,
                    };
                    new_state_chirho.stack_chirho[new_state_chirho.sp_chirho] = Some(entry_chirho);
                    new_state_chirho.sp_chirho += 1;
                }

                // Continue with first choice (lo branch)
                new_state_chirho.domains_chirho[v_chirho] = lo_chirho;
                new_state_chirho.valid_chirho =
                    new_state_chirho.valid_chirho && !Self::is_empty_chirho(lo_chirho);
            }

            SearchCmdChirho::BacktrackChirho => {
                if new_state_chirho.sp_chirho == 0 {
                    // No more alternatives
                    new_state_chirho.valid_chirho = false;
                } else {
                    let new_sp_chirho = new_state_chirho.sp_chirho - 1;
                    if let Some(entry_chirho) = new_state_chirho.stack_chirho[new_sp_chirho] {
                        // Restore domains from stack entry
                        new_state_chirho.domains_chirho = entry_chirho.domains_chirho;
                        // Apply the alternative domain
                        let v_chirho = entry_chirho.var_chirho as usize;
                        new_state_chirho.domains_chirho[v_chirho] = entry_chirho.domain_chirho;
                        new_state_chirho.sp_chirho = new_sp_chirho;
                        new_state_chirho.valid_chirho =
                            !Self::is_empty_chirho(entry_chirho.domain_chirho);
                    } else {
                        new_state_chirho.valid_chirho = false;
                    }
                }
            }

            SearchCmdChirho::NopChirho => {
                // No change
            }
        }

        // Build response (from NEW state, matching Clash)
        let solution_chirho = new_state_chirho.valid_chirho
            && new_state_chirho
                .domains_chirho
                .iter()
                .all(|d| Self::is_singleton_chirho(*d));

        let resp_chirho = SearchRespChirho {
            valid_chirho: new_state_chirho.valid_chirho,
            solution_chirho,
            domains_chirho: new_state_chirho.domains_chirho,
        };

        (new_state_chirho, resp_chirho)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_init_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();
        assert!(state_chirho.valid_chirho);
        assert_eq!(state_chirho.domains_chirho, [u64::MAX; 8]);
        assert_eq!(state_chirho.sp_chirho, 0);
    }

    #[test]
    fn test_constrain_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();
        let (new_state_chirho, resp_chirho) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b1010));

        assert!(resp_chirho.valid_chirho);
        assert!(!resp_chirho.solution_chirho);
        assert_eq!(new_state_chirho.domains_chirho[0], 0b1010);
    }

    #[test]
    fn test_unify_vars_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();

        // Constrain var 0 to {1, 3}
        let (state_chirho, _) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b1010));

        // Constrain var 1 to {2, 3}
        let (state_chirho, _) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(1, 0b1100));

        // Unify var 0 and var 1 → intersection = {3}
        let (state_chirho, resp_chirho) =
            state_chirho.step_chirho(SearchCmdChirho::UnifyVarsChirho(0, 1));

        assert!(resp_chirho.valid_chirho);
        assert_eq!(state_chirho.domains_chirho[0], 0b1000); // {3}
        assert_eq!(state_chirho.domains_chirho[1], 0b1000); // {3}
    }

    #[test]
    fn test_branch_backtrack_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();

        // Constrain var 0 to {1, 2, 3} = 0b1110
        let (state_chirho, _) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b1110));

        // Branch on var 0
        let (state_chirho, resp_chirho) =
            state_chirho.step_chirho(SearchCmdChirho::BranchVarChirho(0));

        // Should take lowest bit (value 1 = 0b10)
        assert!(resp_chirho.valid_chirho);
        assert_eq!(state_chirho.domains_chirho[0], 0b0010);
        assert_eq!(state_chirho.sp_chirho, 1);

        // Backtrack
        let (state_chirho, resp_chirho) =
            state_chirho.step_chirho(SearchCmdChirho::BacktrackChirho);

        // Should restore to remaining {2, 3} = 0b1100
        assert!(resp_chirho.valid_chirho);
        assert_eq!(state_chirho.domains_chirho[0], 0b1100);
        assert_eq!(state_chirho.sp_chirho, 0);
    }

    #[test]
    fn test_failure_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();

        // Constrain var 0 to {1}
        let (state_chirho, _) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b0010));

        // Constrain var 0 to {2} (disjoint!) → failure
        let (state_chirho, resp_chirho) =
            state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b0100));

        assert!(!resp_chirho.valid_chirho);
        assert!(!state_chirho.valid_chirho);
    }

    #[test]
    fn test_solution_chirho() {
        let state_chirho = EngineStateChirho::new_chirho();

        // Constrain all 8 variables to singletons
        let mut state_chirho = state_chirho;
        for i in 0..8 {
            let (new_state_chirho, _) =
                state_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(i, 1u64 << i));
            state_chirho = new_state_chirho;
        }

        // Check final response
        let (_, resp_chirho) = state_chirho.step_chirho(SearchCmdChirho::NopChirho);
        assert!(resp_chirho.valid_chirho);
        assert!(resp_chirho.solution_chirho);
    }
}
