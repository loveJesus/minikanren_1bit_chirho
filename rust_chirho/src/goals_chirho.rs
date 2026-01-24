//! Goal Combinators ☧
//!
//! The core miniKanren operators: ==, fresh, conde, conj

use std::sync::Arc;

use crate::stream_chirho::StreamChirho;
use crate::terms_chirho::{TermIdChirho, TermStoreChirho};
use crate::unify_chirho::{unify_chirho, SubstChirho, UnifyResultChirho};

/// Goal: substitution → stream of substitutions
pub type GoalFnChirho = Arc<dyn Fn(SubstChirho, &TermStoreChirho) -> StreamChirho + Send + Sync>;

/// Unification goal: (== t1 t2)
pub fn eq_chirho(t1_chirho: TermIdChirho, t2_chirho: TermIdChirho) -> GoalFnChirho {
    Arc::new(move |mut subst_chirho: SubstChirho, store_chirho: &TermStoreChirho| {
        match unify_chirho(t1_chirho, t2_chirho, &mut subst_chirho, store_chirho) {
            UnifyResultChirho::Success => StreamChirho::unit_chirho(subst_chirho),
            _ => StreamChirho::empty_chirho(),
        }
    })
}

/// Conjunction: (conj g1 g2) = g1 AND g2
pub fn conj_chirho(g1_chirho: GoalFnChirho, g2_chirho: GoalFnChirho) -> GoalFnChirho {
    Arc::new(move |subst_chirho: SubstChirho, store_chirho: &TermStoreChirho| {
        let g2_clone_chirho = g2_chirho.clone();
        let store_ptr_chirho = store_chirho as *const TermStoreChirho;

        g1_chirho(subst_chirho, store_chirho).bind_chirho(move |s_chirho| {
            // Safety: store_chirho lives for the duration of the search
            let store_ref_chirho = unsafe { &*store_ptr_chirho };
            g2_clone_chirho(s_chirho, store_ref_chirho)
        })
    })
}

/// Disjunction: (disj g1 g2) = g1 OR g2
pub fn disj_chirho(g1_chirho: GoalFnChirho, g2_chirho: GoalFnChirho) -> GoalFnChirho {
    Arc::new(move |subst_chirho: SubstChirho, store_chirho: &TermStoreChirho| {
        let s1_chirho = g1_chirho(subst_chirho.clone(), store_chirho);
        let s2_chirho = g2_chirho(subst_chirho, store_chirho);
        s1_chirho.mplus_chirho(s2_chirho)
    })
}

/// Conde: (conde [g1...] [g2...] ...) = disjunction of conjunctions
pub fn conde_chirho(clauses_chirho: Vec<Vec<GoalFnChirho>>) -> GoalFnChirho {
    let goals_chirho: Vec<GoalFnChirho> = clauses_chirho
        .into_iter()
        .map(|clause_chirho| conj_all_chirho(clause_chirho))
        .collect();

    disj_all_chirho(goals_chirho)
}

/// Conjunction of multiple goals
pub fn conj_all_chirho(goals_chirho: Vec<GoalFnChirho>) -> GoalFnChirho {
    if goals_chirho.is_empty() {
        return Arc::new(|subst_chirho, _| StreamChirho::unit_chirho(subst_chirho));
    }

    let mut iter_chirho = goals_chirho.into_iter();
    let first_chirho = iter_chirho.next().unwrap();

    iter_chirho.fold(first_chirho, |acc_chirho, g_chirho| conj_chirho(acc_chirho, g_chirho))
}

/// Disjunction of multiple goals
pub fn disj_all_chirho(goals_chirho: Vec<GoalFnChirho>) -> GoalFnChirho {
    if goals_chirho.is_empty() {
        return Arc::new(|_, _| StreamChirho::empty_chirho());
    }

    let mut iter_chirho = goals_chirho.into_iter();
    let first_chirho = iter_chirho.next().unwrap();

    iter_chirho.fold(first_chirho, |acc_chirho, g_chirho| disj_chirho(acc_chirho, g_chirho))
}

/// Success goal (always succeeds)
pub fn succeed_chirho() -> GoalFnChirho {
    Arc::new(|subst_chirho, _| StreamChirho::unit_chirho(subst_chirho))
}

/// Failure goal (always fails)
pub fn fail_chirho() -> GoalFnChirho {
    Arc::new(|_, _| StreamChirho::empty_chirho())
}

/// Run a goal and collect n solutions
pub fn run_chirho(
    n_chirho: usize,
    query_var_chirho: TermIdChirho,
    goal_chirho: GoalFnChirho,
    store_chirho: &TermStoreChirho,
) -> Vec<TermIdChirho> {
    let subst_chirho = SubstChirho::new();
    let stream_chirho = goal_chirho(subst_chirho, store_chirho);
    let solutions_chirho = stream_chirho.take_chirho(n_chirho);

    solutions_chirho
        .into_iter()
        .map(|mut s_chirho| s_chirho.walk_chirho(query_var_chirho, store_chirho))
        .collect()
}

/// Run all solutions
pub fn run_all_chirho(
    query_var_chirho: TermIdChirho,
    goal_chirho: GoalFnChirho,
    store_chirho: &TermStoreChirho,
) -> Vec<TermIdChirho> {
    let subst_chirho = SubstChirho::new();
    let stream_chirho = goal_chirho(subst_chirho, store_chirho);
    let solutions_chirho = stream_chirho.take_all_chirho();

    solutions_chirho
        .into_iter()
        .map(|mut s_chirho| s_chirho.walk_chirho(query_var_chirho, store_chirho))
        .collect()
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_eq_succeed_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let a_chirho = store_chirho.int_chirho(42);
        let b_chirho = store_chirho.int_chirho(42);

        let goal_chirho = eq_chirho(a_chirho, b_chirho);
        let results_chirho = run_chirho(10, a_chirho, goal_chirho, &store_chirho);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_eq_fail_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let a_chirho = store_chirho.int_chirho(42);
        let b_chirho = store_chirho.int_chirho(43);

        let goal_chirho = eq_chirho(a_chirho, b_chirho);
        let results_chirho = run_chirho(10, a_chirho, goal_chirho, &store_chirho);

        assert_eq!(results_chirho.len(), 0);
    }

    #[test]
    fn test_unify_var_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let val_chirho = store_chirho.int_chirho(42);

        let goal_chirho = eq_chirho(x_chirho, val_chirho);
        let results_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);

        assert_eq!(results_chirho.len(), 1);
        assert_eq!(results_chirho[0], val_chirho);
    }

    #[test]
    fn test_conde_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);
        let three_chirho = store_chirho.int_chirho(3);

        // (conde [(== x 1)] [(== x 2)] [(== x 3)])
        let goal_chirho = conde_chirho(vec![
            vec![eq_chirho(x_chirho, one_chirho)],
            vec![eq_chirho(x_chirho, two_chirho)],
            vec![eq_chirho(x_chirho, three_chirho)],
        ]);

        let results_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);

        assert_eq!(results_chirho.len(), 3);
        assert!(results_chirho.contains(&one_chirho));
        assert!(results_chirho.contains(&two_chirho));
        assert!(results_chirho.contains(&three_chirho));
    }

    #[test]
    fn test_conj_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let (_, y_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);

        // (conj (== x 1) (== y x)) => x=1, y=1
        let goal_chirho = conj_chirho(
            eq_chirho(x_chirho, one_chirho),
            eq_chirho(y_chirho, x_chirho),
        );

        let results_chirho = run_chirho(10, y_chirho, goal_chirho, &store_chirho);

        assert_eq!(results_chirho.len(), 1);
        assert_eq!(results_chirho[0], one_chirho);
    }
}
