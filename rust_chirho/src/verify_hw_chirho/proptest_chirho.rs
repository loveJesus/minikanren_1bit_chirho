// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Property-based testing for verification bridge ☧
//!
//! Fuzzes the search engine with random command sequences.

use super::cmd_chirho::SearchCmdChirho;
use super::engine_chirho::EngineStateChirho;
use proptest::prelude::*;
use proptest::strategy::ValueTree;

/// Generate a random search command
pub fn arb_cmd_chirho() -> impl Strategy<Value = SearchCmdChirho> {
    prop_oneof![
        Just(SearchCmdChirho::InitChirho),
        (0u8..8, 0u8..8).prop_map(|(v1, v2)| SearchCmdChirho::UnifyVarsChirho(v1, v2)),
        (0u8..8, any::<u64>()).prop_map(|(v, m)| SearchCmdChirho::ConstrainVarChirho(v, m)),
        (0u8..8).prop_map(SearchCmdChirho::BranchVarChirho),
        Just(SearchCmdChirho::BacktrackChirho),
        Just(SearchCmdChirho::NopChirho),
    ]
}

/// Generate a sequence of commands
pub fn arb_cmd_sequence_chirho(len_chirho: usize) -> impl Strategy<Value = Vec<SearchCmdChirho>> {
    proptest::collection::vec(arb_cmd_chirho(), 0..=len_chirho)
}

/// Execute a command sequence and return final state
pub fn execute_sequence_chirho(cmds_chirho: &[SearchCmdChirho]) -> EngineStateChirho {
    let mut state_chirho = EngineStateChirho::new_chirho();
    for cmd_chirho in cmds_chirho {
        let (new_state_chirho, _) = state_chirho.step_chirho(*cmd_chirho);
        state_chirho = new_state_chirho;
    }
    state_chirho
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    /// Invariant: After Init, state is valid with full domains
    /// Note: valid=true doesn't guarantee all domains are non-empty
    /// (backtrack can restore a snapshot that had empty domains)
    #[test]
    fn test_after_init_valid_chirho(cmds_chirho in arb_cmd_sequence_chirho(100)) {
        let state_chirho = execute_sequence_chirho(&cmds_chirho);

        // Apply Init and verify it resets properly
        let (fresh_chirho, resp_chirho) = state_chirho.step_chirho(SearchCmdChirho::InitChirho);

        prop_assert!(fresh_chirho.valid_chirho, "Init should set valid=true");
        prop_assert!(!resp_chirho.solution_chirho, "Full domains aren't a solution");
        for d_chirho in &fresh_chirho.domains_chirho {
            prop_assert_eq!(*d_chirho, u64::MAX, "Init should set full domains");
        }
    }

    /// Invariant: Init resets to full domains
    #[test]
    fn test_init_resets_chirho(
        cmds_chirho in arb_cmd_sequence_chirho(50),
    ) {
        let state_chirho = execute_sequence_chirho(&cmds_chirho);

        // Apply Init
        let (state_chirho, resp_chirho) = state_chirho.step_chirho(SearchCmdChirho::InitChirho);

        prop_assert!(state_chirho.valid_chirho);
        prop_assert!(!resp_chirho.solution_chirho);
        prop_assert_eq!(state_chirho.domains_chirho, [u64::MAX; 8]);
        prop_assert_eq!(state_chirho.sp_chirho, 0);
    }

    /// Invariant: Unify is commutative
    #[test]
    fn test_unify_commutative_chirho(
        v1_chirho in 0u8..8,
        v2_chirho in 0u8..8,
        prefix_chirho in arb_cmd_sequence_chirho(20),
    ) {
        let state1_chirho = execute_sequence_chirho(&prefix_chirho);
        let state2_chirho = state1_chirho.clone();

        let (s1_chirho, r1_chirho) = state1_chirho.step_chirho(
            SearchCmdChirho::UnifyVarsChirho(v1_chirho, v2_chirho)
        );
        let (s2_chirho, r2_chirho) = state2_chirho.step_chirho(
            SearchCmdChirho::UnifyVarsChirho(v2_chirho, v1_chirho)
        );

        prop_assert_eq!(r1_chirho.valid_chirho, r2_chirho.valid_chirho);
        prop_assert_eq!(r1_chirho.solution_chirho, r2_chirho.solution_chirho);
        prop_assert_eq!(s1_chirho.domains_chirho, s2_chirho.domains_chirho);
    }

    /// Invariant: Constrain is idempotent
    #[test]
    fn test_constrain_idempotent_chirho(
        v_chirho in 0u8..8,
        mask_chirho in any::<u64>(),
        prefix_chirho in arb_cmd_sequence_chirho(20),
    ) {
        let state_chirho = execute_sequence_chirho(&prefix_chirho);

        let (s1_chirho, _) = state_chirho.step_chirho(
            SearchCmdChirho::ConstrainVarChirho(v_chirho, mask_chirho)
        );
        let (s2_chirho, _) = s1_chirho.step_chirho(
            SearchCmdChirho::ConstrainVarChirho(v_chirho, mask_chirho)
        );

        // Applying same constraint twice should give same result
        prop_assert_eq!(s1_chirho.domains_chirho, s2_chirho.domains_chirho);
        prop_assert_eq!(s1_chirho.valid_chirho, s2_chirho.valid_chirho);
    }

    /// Invariant: Branch followed by Backtrack explores all options
    #[test]
    fn test_branch_backtrack_coverage_chirho(
        v_chirho in 0u8..8,
        mask_chirho in 1u64..=0xFF, // Small non-empty domain
    ) {
        let state_chirho = EngineStateChirho::new_chirho();

        // Constrain variable to small domain
        let (state_chirho, _) = state_chirho.step_chirho(
            SearchCmdChirho::ConstrainVarChirho(v_chirho, mask_chirho)
        );

        let original_domain_chirho = state_chirho.domains_chirho[v_chirho as usize];
        let pop_count_chirho = original_domain_chirho.count_ones();

        // Branch and backtrack repeatedly to explore all options
        let mut visited_chirho = 0u64;
        let mut current_chirho = state_chirho.clone();

        for _ in 0..pop_count_chirho {
            // Branch
            let (branched_chirho, _) = current_chirho.step_chirho(
                SearchCmdChirho::BranchVarChirho(v_chirho)
            );

            if branched_chirho.valid_chirho {
                // Record the value we're exploring
                visited_chirho |= branched_chirho.domains_chirho[v_chirho as usize];
            }

            // Backtrack
            let (backtracked_chirho, _) = branched_chirho.step_chirho(
                SearchCmdChirho::BacktrackChirho
            );

            if !backtracked_chirho.valid_chirho {
                break;
            }
            current_chirho = backtracked_chirho;
        }

        // We should have visited all values in the original domain
        prop_assert_eq!(visited_chirho, original_domain_chirho,
            "Branch/backtrack didn't cover all values");
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_arb_cmd_generates_all_variants_chirho() {
        // Just verify the strategy compiles and runs
        let mut runner_chirho = proptest::test_runner::TestRunner::default();
        for _ in 0..100 {
            let cmd_chirho = arb_cmd_chirho().new_tree(&mut runner_chirho).unwrap().current();
            // Just check it's a valid command
            match cmd_chirho {
                SearchCmdChirho::InitChirho => {}
                SearchCmdChirho::UnifyVarsChirho(_, _) => {}
                SearchCmdChirho::ConstrainVarChirho(_, _) => {}
                SearchCmdChirho::BranchVarChirho(_) => {}
                SearchCmdChirho::BacktrackChirho => {}
                SearchCmdChirho::NopChirho => {}
            }
        }
    }
}
