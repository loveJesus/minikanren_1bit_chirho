// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Free Monad Goals ☧
//!
//! Goals as a free monad over the goal functor.
//! Enables multiple interpreters: Boolean, Probabilistic, SMT.
//!
//! Inspired by Edward Kmett's work on free monads.
//!
//! Key insight: Same AST, different backends (Bool, Prob, SMT).

use crate::terms_chirho::{TermIdChirho, TermStoreChirho, VarIdChirho};
use crate::SubstChirho;
use crate::stream_chirho::StreamChirho;
use crate::diff_semiring_chirho::{DiffProbChirho, soft_eq_chirho};
use std::sync::Arc;

// ============================================================================
// Goal Functor
// ============================================================================

/// Goal functor: the "shape" of goals without recursion
pub enum GoalFChirho<A> {
    /// Unification: (== t1 t2) then continue with A
    EqChirho(TermIdChirho, TermIdChirho, A),
    /// Fresh variable: (fresh (x) ...) - continuation receives new VarId
    FreshChirho(Arc<dyn Fn(VarIdChirho, TermIdChirho) -> A + Send + Sync>),
    /// Disjunction: (conde [g1] [g2])
    DisjChirho(A, A),
    /// Conjunction: (conj g1 g2)
    ConjChirho(A, A),
    /// Disequality: (!= t1 t2)
    DiseqChirho(TermIdChirho, TermIdChirho, A),
    /// Negation as failure: (not goal)
    NotChirho(A, A), // (negated_goal, continuation)
}

impl<A: Clone> Clone for GoalFChirho<A> {
    fn clone(&self) -> Self {
        match self {
            GoalFChirho::EqChirho(t1, t2, a) => GoalFChirho::EqChirho(*t1, *t2, a.clone()),
            GoalFChirho::FreshChirho(f) => GoalFChirho::FreshChirho(Arc::clone(f)),
            GoalFChirho::DisjChirho(a, b) => GoalFChirho::DisjChirho(a.clone(), b.clone()),
            GoalFChirho::ConjChirho(a, b) => GoalFChirho::ConjChirho(a.clone(), b.clone()),
            GoalFChirho::DiseqChirho(t1, t2, a) => GoalFChirho::DiseqChirho(*t1, *t2, a.clone()),
            GoalFChirho::NotChirho(a, k) => GoalFChirho::NotChirho(a.clone(), k.clone()),
        }
    }
}

// ============================================================================
// Free Monad
// ============================================================================

/// Free monad over GoalF
pub enum FreeGoalChirho {
    /// Pure: computation succeeded with this substitution
    PureChirho(SubstChirho),
    /// Free: suspended computation
    FreeChirho(Box<GoalFChirho<FreeGoalChirho>>),
    /// Fail: computation failed
    FailChirho,
}

impl Clone for FreeGoalChirho {
    fn clone(&self) -> Self {
        match self {
            FreeGoalChirho::PureChirho(s) => FreeGoalChirho::PureChirho(s.clone()),
            FreeGoalChirho::FreeChirho(gf) => FreeGoalChirho::FreeChirho(gf.clone()),
            FreeGoalChirho::FailChirho => FreeGoalChirho::FailChirho,
        }
    }
}

impl FreeGoalChirho {
    /// Lift a goal functor into the free monad
    pub fn lift_chirho(gf_chirho: GoalFChirho<FreeGoalChirho>) -> Self {
        FreeGoalChirho::FreeChirho(Box::new(gf_chirho))
    }

    /// Pure: succeed with substitution
    pub fn pure_chirho(subst_chirho: SubstChirho) -> Self {
        FreeGoalChirho::PureChirho(subst_chirho)
    }

    /// Fail: computation fails
    pub fn fail_chirho() -> Self {
        FreeGoalChirho::FailChirho
    }
}

// ============================================================================
// Smart constructors
// ============================================================================

/// Create unification goal
pub fn eq_free_chirho(t1_chirho: TermIdChirho, t2_chirho: TermIdChirho) -> FreeGoalChirho {
    FreeGoalChirho::lift_chirho(GoalFChirho::EqChirho(
        t1_chirho,
        t2_chirho,
        FreeGoalChirho::pure_chirho(SubstChirho::new()),
    ))
}

/// Create fresh variable goal
pub fn fresh_free_chirho<F>(body_chirho: F) -> FreeGoalChirho
where
    F: Fn(VarIdChirho, TermIdChirho) -> FreeGoalChirho + Send + Sync + 'static,
{
    FreeGoalChirho::lift_chirho(GoalFChirho::FreshChirho(Arc::new(body_chirho)))
}

/// Create disjunction
pub fn disj_free_chirho(g1_chirho: FreeGoalChirho, g2_chirho: FreeGoalChirho) -> FreeGoalChirho {
    FreeGoalChirho::lift_chirho(GoalFChirho::DisjChirho(g1_chirho, g2_chirho))
}

/// Create conjunction
pub fn conj_free_chirho(g1_chirho: FreeGoalChirho, g2_chirho: FreeGoalChirho) -> FreeGoalChirho {
    FreeGoalChirho::lift_chirho(GoalFChirho::ConjChirho(g1_chirho, g2_chirho))
}

/// Create disequality
pub fn diseq_free_chirho(t1_chirho: TermIdChirho, t2_chirho: TermIdChirho) -> FreeGoalChirho {
    FreeGoalChirho::lift_chirho(GoalFChirho::DiseqChirho(
        t1_chirho,
        t2_chirho,
        FreeGoalChirho::pure_chirho(SubstChirho::new()),
    ))
}

// ============================================================================
// Boolean Interpreter
// ============================================================================

/// Run goal with Boolean semantics (standard miniKanren)
pub fn run_bool_chirho(
    goal_chirho: FreeGoalChirho,
    subst_chirho: SubstChirho,
    store_chirho: &mut TermStoreChirho,
) -> StreamChirho {
    match goal_chirho {
        FreeGoalChirho::PureChirho(s) => {
            // Merge substitutions
            StreamChirho::unit_chirho(s)
        }
        FreeGoalChirho::FailChirho => StreamChirho::empty_chirho(),
        FreeGoalChirho::FreeChirho(gf) => match *gf {
            GoalFChirho::EqChirho(t1, t2, k) => {
                let mut new_subst_chirho = subst_chirho.clone();
                match crate::unify_chirho(t1, t2, &mut new_subst_chirho, store_chirho)
                {
                    crate::UnifyResultChirho::Success => {
                        run_bool_chirho(k, new_subst_chirho, store_chirho)
                    }
                    _ => StreamChirho::empty_chirho(),
                }
            }
            GoalFChirho::FreshChirho(f) => {
                let (var_id_chirho, term_id_chirho) = store_chirho.fresh_var_chirho();
                let k_chirho = f(var_id_chirho, term_id_chirho);
                run_bool_chirho(k_chirho, subst_chirho, store_chirho)
            }
            GoalFChirho::DisjChirho(g1, g2) => {
                let s1_chirho = run_bool_chirho(g1, subst_chirho.clone(), store_chirho);
                let s2_chirho = run_bool_chirho(g2, subst_chirho, store_chirho);
                s1_chirho.mplus_chirho(s2_chirho)
            }
            GoalFChirho::ConjChirho(g1, g2) => {
                // Flatten conjunction: run g1, collect results, run g2 on each
                let s1_results_chirho = run_bool_chirho(g1, subst_chirho, store_chirho).take_chirho(1000);
                let mut combined_chirho = StreamChirho::empty_chirho();
                for new_subst_chirho in s1_results_chirho {
                    let s2_chirho = run_bool_chirho(g2.clone(), new_subst_chirho, store_chirho);
                    combined_chirho = combined_chirho.mplus_chirho(s2_chirho);
                }
                combined_chirho
            }
            GoalFChirho::DiseqChirho(t1, t2, k) => {
                let mut new_subst_chirho = subst_chirho.clone();
                new_subst_chirho.add_diseq_chirho(t1, t2);
                if new_subst_chirho.check_diseqs_chirho(store_chirho) {
                    run_bool_chirho(k, new_subst_chirho, store_chirho)
                } else {
                    StreamChirho::empty_chirho()
                }
            }
            GoalFChirho::NotChirho(negated, k) => {
                // Negation as failure
                let results_chirho = run_bool_chirho(negated, subst_chirho.clone(), store_chirho)
                    .take_chirho(1);
                if results_chirho.is_empty() {
                    run_bool_chirho(k, subst_chirho, store_chirho)
                } else {
                    StreamChirho::empty_chirho()
                }
            }
        },
    }
}

// ============================================================================
// Probabilistic Interpreter
// ============================================================================

/// Run goal with probabilistic semantics
/// Returns probability of success
pub fn run_prob_chirho(
    goal_chirho: &FreeGoalChirho,
    subst_chirho: &SubstChirho,
    store_chirho: &TermStoreChirho,
    temp_chirho: f64,
) -> DiffProbChirho {
    match goal_chirho {
        FreeGoalChirho::PureChirho(_) => DiffProbChirho::new_chirho(1.0),
        FreeGoalChirho::FailChirho => DiffProbChirho::new_chirho(0.0),
        FreeGoalChirho::FreeChirho(gf) => match gf.as_ref() {
            GoalFChirho::EqChirho(t1, t2, k) => {
                // Soft equality based on term similarity
                let p_eq_chirho = soft_eq_term_chirho(*t1, *t2, subst_chirho, store_chirho, temp_chirho);
                let p_k_chirho = run_prob_chirho(k, subst_chirho, store_chirho, temp_chirho);
                p_eq_chirho.and_chirho(&p_k_chirho)
            }
            GoalFChirho::DisjChirho(g1, g2) => {
                let p1_chirho = run_prob_chirho(g1, subst_chirho, store_chirho, temp_chirho);
                let p2_chirho = run_prob_chirho(g2, subst_chirho, store_chirho, temp_chirho);
                p1_chirho.or_chirho(&p2_chirho)
            }
            GoalFChirho::ConjChirho(g1, g2) => {
                let p1_chirho = run_prob_chirho(g1, subst_chirho, store_chirho, temp_chirho);
                let p2_chirho = run_prob_chirho(g2, subst_chirho, store_chirho, temp_chirho);
                p1_chirho.and_chirho(&p2_chirho)
            }
            GoalFChirho::DiseqChirho(t1, t2, k) => {
                // Soft disequality: 1 - soft_eq
                let p_eq_chirho = soft_eq_term_chirho(*t1, *t2, subst_chirho, store_chirho, temp_chirho);
                let p_neq_chirho = DiffProbChirho::new_chirho(1.0 - p_eq_chirho.value_chirho);
                let p_k_chirho = run_prob_chirho(k, subst_chirho, store_chirho, temp_chirho);
                p_neq_chirho.and_chirho(&p_k_chirho)
            }
            GoalFChirho::FreshChirho(_) => {
                // Fresh variables don't affect probability
                DiffProbChirho::new_chirho(1.0)
            }
            GoalFChirho::NotChirho(negated, k) => {
                // Soft negation
                let p_neg_chirho = run_prob_chirho(negated, subst_chirho, store_chirho, temp_chirho);
                let p_not_chirho = DiffProbChirho::new_chirho(1.0 - p_neg_chirho.value_chirho);
                let p_k_chirho = run_prob_chirho(k, subst_chirho, store_chirho, temp_chirho);
                p_not_chirho.and_chirho(&p_k_chirho)
            }
        },
    }
}

/// Soft equality for terms (used by probabilistic interpreter)
fn soft_eq_term_chirho(
    t1_chirho: TermIdChirho,
    t2_chirho: TermIdChirho,
    _subst_chirho: &SubstChirho,
    _store_chirho: &TermStoreChirho,
    temp_chirho: f64,
) -> DiffProbChirho {
    // For now, use term ID distance as proxy
    // In full implementation, would walk terms and compute structural similarity
    if t1_chirho == t2_chirho {
        DiffProbChirho::new_chirho(1.0)
    } else {
        let diff_chirho = (t1_chirho as f64 - t2_chirho as f64).abs();
        let prob_chirho = soft_eq_chirho(0.0, diff_chirho, temp_chirho);
        DiffProbChirho::new_chirho(prob_chirho)
    }
}

// ============================================================================
// SMT Export (placeholder)
// ============================================================================

/// SMT formula representation
#[derive(Debug, Clone)]
pub enum SmtFormulaChirho {
    /// Equality
    EqSmtChirho(TermIdChirho, TermIdChirho),
    /// Disequality
    NeqSmtChirho(TermIdChirho, TermIdChirho),
    /// Conjunction
    AndSmtChirho(Box<SmtFormulaChirho>, Box<SmtFormulaChirho>),
    /// Disjunction
    OrSmtChirho(Box<SmtFormulaChirho>, Box<SmtFormulaChirho>),
    /// Negation
    NotSmtChirho(Box<SmtFormulaChirho>),
    /// True
    TrueSmtChirho,
    /// False
    FalseSmtChirho,
}

/// Export goal to SMT formula
pub fn goal_to_smt_chirho(goal_chirho: &FreeGoalChirho) -> SmtFormulaChirho {
    match goal_chirho {
        FreeGoalChirho::PureChirho(_) => SmtFormulaChirho::TrueSmtChirho,
        FreeGoalChirho::FailChirho => SmtFormulaChirho::FalseSmtChirho,
        FreeGoalChirho::FreeChirho(gf) => match gf.as_ref() {
            GoalFChirho::EqChirho(t1, t2, k) => {
                let eq_chirho = SmtFormulaChirho::EqSmtChirho(*t1, *t2);
                let k_smt_chirho = goal_to_smt_chirho(k);
                SmtFormulaChirho::AndSmtChirho(Box::new(eq_chirho), Box::new(k_smt_chirho))
            }
            GoalFChirho::DisjChirho(g1, g2) => {
                let s1_chirho = goal_to_smt_chirho(g1);
                let s2_chirho = goal_to_smt_chirho(g2);
                SmtFormulaChirho::OrSmtChirho(Box::new(s1_chirho), Box::new(s2_chirho))
            }
            GoalFChirho::ConjChirho(g1, g2) => {
                let s1_chirho = goal_to_smt_chirho(g1);
                let s2_chirho = goal_to_smt_chirho(g2);
                SmtFormulaChirho::AndSmtChirho(Box::new(s1_chirho), Box::new(s2_chirho))
            }
            GoalFChirho::DiseqChirho(t1, t2, k) => {
                let neq_chirho = SmtFormulaChirho::NeqSmtChirho(*t1, *t2);
                let k_smt_chirho = goal_to_smt_chirho(k);
                SmtFormulaChirho::AndSmtChirho(Box::new(neq_chirho), Box::new(k_smt_chirho))
            }
            GoalFChirho::FreshChirho(_) => SmtFormulaChirho::TrueSmtChirho,
            GoalFChirho::NotChirho(negated, k) => {
                let neg_smt_chirho = goal_to_smt_chirho(negated);
                let k_smt_chirho = goal_to_smt_chirho(k);
                SmtFormulaChirho::AndSmtChirho(
                    Box::new(SmtFormulaChirho::NotSmtChirho(Box::new(neg_smt_chirho))),
                    Box::new(k_smt_chirho),
                )
            }
        },
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_eq_free_bool_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let one_chirho = store_chirho.int_chirho(1);
        let goal_chirho = eq_free_chirho(one_chirho, one_chirho);

        let results_chirho = run_bool_chirho(goal_chirho, subst_chirho, &mut store_chirho)
            .take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_disj_free_bool_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        let g1_chirho = eq_free_chirho(one_chirho, one_chirho);
        let g2_chirho = eq_free_chirho(two_chirho, two_chirho);
        let goal_chirho = disj_free_chirho(g1_chirho, g2_chirho);

        let results_chirho = run_bool_chirho(goal_chirho, subst_chirho, &mut store_chirho)
            .take_chirho(10);
        assert_eq!(results_chirho.len(), 2);
    }

    #[test]
    fn test_conj_free_bool_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        let g1_chirho = eq_free_chirho(one_chirho, one_chirho);
        let g2_chirho = eq_free_chirho(two_chirho, two_chirho);
        let goal_chirho = conj_free_chirho(g1_chirho, g2_chirho);

        let results_chirho = run_bool_chirho(goal_chirho, subst_chirho, &mut store_chirho)
            .take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_eq_free_prob_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        // Same terms → probability 1.0
        let goal_chirho = eq_free_chirho(0, 0);
        let prob_chirho = run_prob_chirho(&goal_chirho, &subst_chirho, &store_chirho, 1.0);
        assert!((prob_chirho.value_chirho - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_disj_free_prob_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        // P(A ∨ B) = P(A) + P(B) - P(A)P(B)
        let g1_chirho = eq_free_chirho(0, 0); // p = 1
        let g2_chirho = eq_free_chirho(0, 0); // p = 1
        let goal_chirho = disj_free_chirho(g1_chirho, g2_chirho);

        let prob_chirho = run_prob_chirho(&goal_chirho, &subst_chirho, &store_chirho, 1.0);
        // 1 + 1 - 1*1 = 1
        assert!((prob_chirho.value_chirho - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_goal_to_smt_chirho() {
        let g1_chirho = eq_free_chirho(0, 1);
        let g2_chirho = eq_free_chirho(2, 3);
        let goal_chirho = disj_free_chirho(g1_chirho, g2_chirho);

        let smt_chirho = goal_to_smt_chirho(&goal_chirho);
        match smt_chirho {
            SmtFormulaChirho::OrSmtChirho(_, _) => {} // Expected
            _ => panic!("Expected OrSmtChirho"),
        }
    }

    #[test]
    fn test_fresh_free_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        // fresh x. x == 1
        let one_chirho = store_chirho.int_chirho(1);
        let goal_chirho = fresh_free_chirho(move |_var_id, term_id| eq_free_chirho(term_id, one_chirho));

        let results_chirho = run_bool_chirho(goal_chirho, subst_chirho, &mut store_chirho)
            .take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }
}
