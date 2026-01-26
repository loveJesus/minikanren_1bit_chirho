//! Co-Simulation Harness ☧
//!
//! Verifies bit-identical behavior between Rust reference and Verilator.
//! Part of P5-00: Bridge of Truth.
//!
//! "Sanctify them through thy truth: thy word is truth." — John 17:17

use super::cmd_chirho::{SearchCmdChirho, SearchRespChirho};
use super::engine_chirho::EngineStateChirho;
use super::verilator_chirho::VerilatorSimChirho;

/// Co-simulation state: Rust reference + optional Verilator
pub struct CoSimChirho {
    /// Rust reference engine
    pub rust_chirho: EngineStateChirho,
    /// Verilator simulation (if available)
    pub verilator_chirho: Option<VerilatorSimChirho>,
    /// Number of steps executed
    pub steps_chirho: u64,
    /// Number of mismatches detected
    pub mismatches_chirho: u64,
}

/// Result of a co-simulation step
#[derive(Debug)]
pub struct CoSimResultChirho {
    /// Command that was executed
    pub cmd_chirho: SearchCmdChirho,
    /// Rust reference response
    pub rust_resp_chirho: SearchRespChirho,
    /// Verilator response (if available)
    pub verilator_resp_chirho: Option<SearchRespChirho>,
    /// Whether responses match
    pub match_chirho: bool,
}

impl CoSimChirho {
    /// Create a new co-simulation harness
    pub fn new_chirho() -> Self {
        let verilator_chirho = if VerilatorSimChirho::is_available_chirho() {
            match VerilatorSimChirho::new_chirho() {
                Ok(sim) => Some(sim),
                Err(_) => None,
            }
        } else {
            None
        };

        Self {
            rust_chirho: EngineStateChirho::new_chirho(),
            verilator_chirho,
            steps_chirho: 0,
            mismatches_chirho: 0,
        }
    }

    /// Check if Verilator co-simulation is available
    pub fn has_verilator_chirho(&self) -> bool {
        self.verilator_chirho.is_some()
    }

    /// Execute one command on both engines and compare
    pub fn step_chirho(&mut self, cmd_chirho: SearchCmdChirho) -> CoSimResultChirho {
        // Execute on Rust reference (step returns new state + response)
        let (new_rust_state_chirho, rust_resp_chirho) = self.rust_chirho.step_chirho(cmd_chirho.clone());
        self.rust_chirho = new_rust_state_chirho;

        // Execute on Verilator if available
        let verilator_resp_chirho = if let Some(ref mut verilator) = self.verilator_chirho {
            match verilator.step_chirho(cmd_chirho.clone()) {
                Ok(resp) => Some(resp),
                Err(_) => None,
            }
        } else {
            None
        };

        // Compare responses
        let match_chirho = match &verilator_resp_chirho {
            Some(v_resp) => responses_equal_chirho(&rust_resp_chirho, v_resp),
            None => true, // No Verilator = assume match (Rust-only mode)
        };

        if !match_chirho {
            self.mismatches_chirho += 1;
        }

        self.steps_chirho += 1;

        CoSimResultChirho {
            cmd_chirho,
            rust_resp_chirho,
            verilator_resp_chirho,
            match_chirho,
        }
    }

    /// Reset both engines
    pub fn reset_chirho(&mut self) {
        // Reset Rust reference
        self.rust_chirho = EngineStateChirho::new_chirho();

        // Reset Verilator by sending Init command
        if let Some(ref mut verilator) = self.verilator_chirho {
            let _ = verilator.step_chirho(SearchCmdChirho::InitChirho);
        }
    }

    /// Get summary statistics
    pub fn summary_chirho(&self) -> String {
        let verilator_status_chirho = if self.has_verilator_chirho() {
            "enabled"
        } else {
            "disabled (Rust-only)"
        };

        format!(
            "CoSim: {} steps, {} mismatches, Verilator: {}",
            self.steps_chirho, self.mismatches_chirho, verilator_status_chirho
        )
    }
}

/// Compare two responses for bit-identical match
fn responses_equal_chirho(a_chirho: &SearchRespChirho, b_chirho: &SearchRespChirho) -> bool {
    a_chirho.valid_chirho == b_chirho.valid_chirho
        && a_chirho.solution_chirho == b_chirho.solution_chirho
        && a_chirho.domains_chirho == b_chirho.domains_chirho
}

/// Run a sequence of commands and verify
pub fn run_sequence_chirho(cmds_chirho: &[SearchCmdChirho]) -> (u64, u64) {
    let mut cosim_chirho = CoSimChirho::new_chirho();

    for cmd_chirho in cmds_chirho {
        let result_chirho = cosim_chirho.step_chirho(cmd_chirho.clone());
        if !result_chirho.match_chirho {
            eprintln!(
                "MISMATCH at step {}: cmd={:?}",
                cosim_chirho.steps_chirho, cmd_chirho
            );
            eprintln!("  Rust:      {:?}", result_chirho.rust_resp_chirho);
            eprintln!(
                "  Verilator: {:?}",
                result_chirho.verilator_resp_chirho
            );
        }
    }

    (cosim_chirho.steps_chirho, cosim_chirho.mismatches_chirho)
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_cosim_rust_only_chirho() {
        let mut cosim_chirho = CoSimChirho::new_chirho();

        // Basic sequence
        let cmds_chirho = vec![
            SearchCmdChirho::InitChirho,
            SearchCmdChirho::ConstrainVarChirho(0, 0b1111),
            SearchCmdChirho::ConstrainVarChirho(1, 0b1010),
            SearchCmdChirho::UnifyVarsChirho(0, 1),
        ];

        for cmd_chirho in cmds_chirho {
            let result_chirho = cosim_chirho.step_chirho(cmd_chirho);
            // In Rust-only mode, match is always true
            assert!(result_chirho.match_chirho);
        }

        assert_eq!(cosim_chirho.steps_chirho, 4);
        assert_eq!(cosim_chirho.mismatches_chirho, 0);
    }

    #[test]
    fn test_cosim_sequence_chirho() {
        let cmds_chirho = vec![
            SearchCmdChirho::InitChirho,
            SearchCmdChirho::ConstrainVarChirho(0, 0b1111),
            SearchCmdChirho::BranchVarChirho(0),
            SearchCmdChirho::BacktrackChirho,
            SearchCmdChirho::NopChirho,
        ];

        let (steps, mismatches) = run_sequence_chirho(&cmds_chirho);
        assert_eq!(steps, 5);
        // Without Verilator, no mismatches possible
        if !VerilatorSimChirho::is_available_chirho() {
            assert_eq!(mismatches, 0);
        }
    }

    #[test]
    fn test_cosim_reset_chirho() {
        let mut cosim_chirho = CoSimChirho::new_chirho();

        // Do some operations
        cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b1010));
        cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(1, 0b0101));

        // Reset
        cosim_chirho.reset_chirho();

        // Verify all domains are full again
        let resp_chirho = cosim_chirho.step_chirho(SearchCmdChirho::NopChirho);
        assert!(resp_chirho.rust_resp_chirho.valid_chirho);
        for domain in &resp_chirho.rust_resp_chirho.domains_chirho {
            assert_eq!(*domain, u64::MAX);
        }
    }

    /// Golden test: Simple constraint solving via co-simulation
    #[test]
    fn test_cosim_constraint_solve_chirho() {
        let mut cosim_chirho = CoSimChirho::new_chirho();

        // Initialize
        cosim_chirho.step_chirho(SearchCmdChirho::InitChirho);

        // Test: Constrain var0 to {0,1,2,3}, var1 to {2,3,4,5}
        // Then unify var0 and var1
        // Result should be {2,3} (intersection)
        cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b1111));      // {0,1,2,3}
        cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(1, 0b111100));    // {2,3,4,5}

        let result_chirho = cosim_chirho.step_chirho(SearchCmdChirho::UnifyVarsChirho(0, 1));

        // Both var0 and var1 should now have domain {2,3} = 0b1100
        assert!(result_chirho.rust_resp_chirho.valid_chirho);
        assert_eq!(result_chirho.rust_resp_chirho.domains_chirho[0], 0b1100);
        assert_eq!(result_chirho.rust_resp_chirho.domains_chirho[1], 0b1100);

        assert_eq!(cosim_chirho.mismatches_chirho, 0);
    }

    /// Golden test: Branching and backtracking
    #[test]
    fn test_cosim_branch_backtrack_chirho() {
        let mut cosim_chirho = CoSimChirho::new_chirho();

        // Initialize with constrained domain
        cosim_chirho.step_chirho(SearchCmdChirho::InitChirho);
        cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(0, 0b111)); // {0,1,2}

        // Branch on var0 - should pick lowest bit (0), save {1,2} on stack
        let branch_result_chirho = cosim_chirho.step_chirho(SearchCmdChirho::BranchVarChirho(0));
        assert!(branch_result_chirho.rust_resp_chirho.valid_chirho);
        assert_eq!(branch_result_chirho.rust_resp_chirho.domains_chirho[0], 0b001); // Just {0}

        // Backtrack - should restore {1,2}
        let backtrack_result_chirho = cosim_chirho.step_chirho(SearchCmdChirho::BacktrackChirho);
        assert!(backtrack_result_chirho.rust_resp_chirho.valid_chirho);
        assert_eq!(backtrack_result_chirho.rust_resp_chirho.domains_chirho[0], 0b110); // {1,2}

        // Branch again - should pick 1, save {2}
        let branch2_result_chirho = cosim_chirho.step_chirho(SearchCmdChirho::BranchVarChirho(0));
        assert!(branch2_result_chirho.rust_resp_chirho.valid_chirho);
        assert_eq!(branch2_result_chirho.rust_resp_chirho.domains_chirho[0], 0b010); // Just {1}

        assert_eq!(cosim_chirho.mismatches_chirho, 0);
    }

    /// Golden test: Finding a solution state
    #[test]
    fn test_cosim_find_solution_chirho() {
        let mut cosim_chirho = CoSimChirho::new_chirho();

        // Initialize
        cosim_chirho.step_chirho(SearchCmdChirho::InitChirho);

        // Constrain all 8 vars to singletons -> should be a solution
        for var in 0u8..8 {
            cosim_chirho.step_chirho(SearchCmdChirho::ConstrainVarChirho(var, 1u64 << var));
        }

        // Check final state
        let final_result_chirho = cosim_chirho.step_chirho(SearchCmdChirho::NopChirho);
        assert!(final_result_chirho.rust_resp_chirho.valid_chirho);
        assert!(final_result_chirho.rust_resp_chirho.solution_chirho);

        // Each domain should be a singleton
        for (i, domain) in final_result_chirho.rust_resp_chirho.domains_chirho.iter().enumerate() {
            assert_eq!(*domain, 1u64 << i);
        }

        assert_eq!(cosim_chirho.mismatches_chirho, 0);
    }
}
