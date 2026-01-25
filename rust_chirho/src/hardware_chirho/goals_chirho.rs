//! Hardware-accelerated miniKanren goals ☧
//!
//! Goals over bit-parallel search states. Each goal transforms
//! a SearchStateHwChirho by intersecting domains (unification)
//! or forking states (disjunction).
//!
//! ## Mapping to reference implementation
//!
//! | reference_chirho | hardware_chirho |
//! |------------------|-----------------|
//! | SubstChirho (HashMap) | SearchStateHwChirho (bit domains) |
//! | StreamChirho (lazy list) | Vec<SearchStateHwChirho> (eager) |
//! | eq_chirho | eq_hw_chirho (AND domains) |
//! | conde_chirho | conde_hw_chirho (fork states) |

use super::hardware_chirho::{BitVec64Chirho, SearchStateHwChirho};

/// A hardware goal: transforms search states
///
/// Unlike reference_chirho which uses lazy streams, hardware goals
/// are eager and operate on vectors of states for SIMD parallelism.
pub type GoalHwChirho<const N: usize> =
    Box<dyn Fn(Vec<SearchStateHwChirho<N>>) -> Vec<SearchStateHwChirho<N>>>;

/// Constrain variable to specific domain (== for finite domains)
///
/// `eq_hw_chirho(0, domain)` means "variable 0 must be in domain"
pub fn eq_hw_chirho<const N: usize>(
    var_chirho: usize,
    domain_chirho: BitVec64Chirho,
) -> GoalHwChirho<N> {
    Box::new(move |states_chirho| {
        states_chirho
            .into_iter()
            .filter_map(|mut state_chirho| {
                if var_chirho < N {
                    state_chirho.domains_chirho[var_chirho] =
                        state_chirho.domains_chirho[var_chirho].and_chirho(domain_chirho);
                    if state_chirho.domains_chirho[var_chirho].is_zero_chirho() {
                        state_chirho.valid_chirho = false;
                    }
                }
                if state_chirho.valid_chirho {
                    Some(state_chirho)
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Unify two variables (== between variables)
pub fn unify_hw_chirho<const N: usize>(
    var1_chirho: usize,
    var2_chirho: usize,
) -> GoalHwChirho<N> {
    Box::new(move |states_chirho| {
        states_chirho
            .into_iter()
            .filter_map(|mut state_chirho| {
                state_chirho.unify_chirho(var1_chirho, var2_chirho);
                if state_chirho.valid_chirho {
                    Some(state_chirho)
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Conjunction: g1 AND g2 (both must succeed)
pub fn conj_hw_chirho<const N: usize>(
    g1_chirho: GoalHwChirho<N>,
    g2_chirho: GoalHwChirho<N>,
) -> GoalHwChirho<N> {
    Box::new(move |states_chirho| {
        g2_chirho(g1_chirho(states_chirho))
    })
}

/// Disjunction: g1 OR g2 (either may succeed)
///
/// Forks the search: tries g1 on all states, tries g2 on all states,
/// concatenates results.
pub fn disj_hw_chirho<const N: usize>(
    g1_chirho: GoalHwChirho<N>,
    g2_chirho: GoalHwChirho<N>,
) -> GoalHwChirho<N> {
    Box::new(move |states_chirho| {
        let mut results_chirho = g1_chirho(states_chirho.clone());
        results_chirho.extend(g2_chirho(states_chirho));
        results_chirho
    })
}

/// Conde: try multiple branches (OR of ANDs)
///
/// Each branch is a conjunction of goals.
pub fn conde_hw_chirho<const N: usize>(
    branches_chirho: Vec<Vec<GoalHwChirho<N>>>,
) -> GoalHwChirho<N> {
    Box::new(move |states_chirho| {
        let mut all_results_chirho = Vec::new();

        for branch_chirho in &branches_chirho {
            let mut branch_states_chirho = states_chirho.clone();
            for goal_chirho in branch_chirho {
                branch_states_chirho = goal_chirho(branch_states_chirho);
            }
            all_results_chirho.extend(branch_states_chirho);
        }

        all_results_chirho
    })
}

/// Always succeed (identity goal)
pub fn succeed_hw_chirho<const N: usize>() -> GoalHwChirho<N> {
    Box::new(|states_chirho| states_chirho)
}

/// Always fail (empty goal)
pub fn fail_hw_chirho<const N: usize>() -> GoalHwChirho<N> {
    Box::new(|_| Vec::new())
}

/// Run goal and collect solutions
///
/// Returns up to `n_chirho` solutions, where each solution assigns
/// a single value to each variable (domain becomes singleton).
pub fn run_hw_chirho<const N: usize>(
    n_chirho: usize,
    query_var_chirho: usize,
    goal_chirho: GoalHwChirho<N>,
) -> Vec<u32> {
    let initial_chirho = vec![SearchStateHwChirho::<N>::new_chirho()];
    let final_states_chirho = goal_chirho(initial_chirho);

    let mut solutions_chirho = Vec::new();

    for state_chirho in final_states_chirho {
        if !state_chirho.valid_chirho || query_var_chirho >= N {
            continue;
        }

        // Extract all values from domain
        let mut domain_chirho = state_chirho.domains_chirho[query_var_chirho];
        while !domain_chirho.is_zero_chirho() && solutions_chirho.len() < n_chirho {
            let value_chirho = domain_chirho.ctz_chirho();
            solutions_chirho.push(value_chirho);
            domain_chirho = domain_chirho.clear_lowest_chirho();
        }

        if solutions_chirho.len() >= n_chirho {
            break;
        }
    }

    solutions_chirho.truncate(n_chirho);
    solutions_chirho
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_eq_hw_chirho() {
        // x ∈ {1, 2, 3}
        let goal_chirho = eq_hw_chirho::<4>(0, BitVec64Chirho(0b1110)); // bits 1,2,3
        let results_chirho = run_hw_chirho(10, 0, goal_chirho);
        assert_eq!(results_chirho, vec![1, 2, 3]);
    }

    #[test]
    fn test_conde_hw_chirho() {
        // x == 1 OR x == 2
        let goal_chirho = conde_hw_chirho::<4>(vec![
            vec![eq_hw_chirho(0, BitVec64Chirho(0b10))],   // x == 1
            vec![eq_hw_chirho(0, BitVec64Chirho(0b100))],  // x == 2
        ]);
        let results_chirho = run_hw_chirho(10, 0, goal_chirho);
        assert_eq!(results_chirho, vec![1, 2]);
    }

    #[test]
    fn test_conj_hw_chirho() {
        // x ∈ {1,2,3} AND x ∈ {2,3,4} → x ∈ {2,3}
        let goal_chirho = conj_hw_chirho::<4>(
            eq_hw_chirho(0, BitVec64Chirho(0b1110)),   // {1,2,3}
            eq_hw_chirho(0, BitVec64Chirho(0b11100)),  // {2,3,4}
        );
        let results_chirho = run_hw_chirho(10, 0, goal_chirho);
        assert_eq!(results_chirho, vec![2, 3]);
    }

    #[test]
    fn test_disj_hw_chirho() {
        // x == 5 OR x == 7
        let goal_chirho = disj_hw_chirho::<4>(
            eq_hw_chirho(0, BitVec64Chirho(1 << 5)),
            eq_hw_chirho(0, BitVec64Chirho(1 << 7)),
        );
        let results_chirho = run_hw_chirho(10, 0, goal_chirho);
        assert_eq!(results_chirho, vec![5, 7]);
    }

    #[test]
    fn test_fail_hw_chirho() {
        // Empty domain = failure
        let goal_chirho = eq_hw_chirho::<4>(0, BitVec64Chirho::ZERO_CHIRHO);
        let results_chirho = run_hw_chirho(10, 0, goal_chirho);
        assert!(results_chirho.is_empty());
    }
}
