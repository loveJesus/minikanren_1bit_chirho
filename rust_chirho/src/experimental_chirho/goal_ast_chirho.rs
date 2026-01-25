//! Goal as Data (AST) ☧
//!
//! Goals as algebraic data type instead of closures.
//! Kmett-approved: interpret the AST, don't execute functions.
//!
//! Benefits:
//! - Introspectable: can analyze goal structure
//! - Optimizable: rewrite rules before execution
//! - Serializable: can save/load goal trees
//! - Testable: property tests on goal structure

use crate::terms_chirho::{TermIdChirho, TermStoreChirho};
use crate::{SubstChirho, UnifyResultChirho, unify_chirho};
use std::rc::Rc;

/// Goal AST - goals as data, not functions ☧
#[derive(Clone)]
pub enum GoalAstChirho {
    /// Unification: t1 == t2
    EqChirho(TermIdChirho, TermIdChirho),

    /// Conjunction: g1 AND g2
    ConjChirho(Rc<GoalAstChirho>, Rc<GoalAstChirho>),

    /// Disjunction: g1 OR g2
    DisjChirho(Rc<GoalAstChirho>, Rc<GoalAstChirho>),

    /// Fresh variable introduction
    /// The function takes a fresh var and returns a goal
    FreshChirho(Rc<dyn Fn(TermIdChirho) -> GoalAstChirho>),

    /// Always succeed
    SucceedChirho,

    /// Always fail
    FailChirho,

    /// Delayed goal (for interleaving)
    DelayChirho(Rc<dyn Fn() -> GoalAstChirho>),

    /// Named goal (for debugging/tracing)
    NamedChirho(String, Rc<GoalAstChirho>),

    /// Conde (list of disjunctions, each a list of conjunctions)
    CondeChirho(Vec<Vec<GoalAstChirho>>),

    /// Negation as failure
    NotChirho(Rc<GoalAstChirho>),

    /// If-then-else: if g1 succeeds, run g2, else run g3
    CondaChirho(Rc<GoalAstChirho>, Rc<GoalAstChirho>, Rc<GoalAstChirho>),
}

impl std::fmt::Debug for GoalAstChirho {
    fn fmt(&self, f_chirho: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalAstChirho::EqChirho(t1_chirho, t2_chirho) => {
                write!(f_chirho, "(== {} {})", t1_chirho, t2_chirho)
            }
            GoalAstChirho::ConjChirho(g1_chirho, g2_chirho) => {
                write!(f_chirho, "(conj {:?} {:?})", g1_chirho, g2_chirho)
            }
            GoalAstChirho::DisjChirho(g1_chirho, g2_chirho) => {
                write!(f_chirho, "(disj {:?} {:?})", g1_chirho, g2_chirho)
            }
            GoalAstChirho::FreshChirho(_) => write!(f_chirho, "(fresh ...)"),
            GoalAstChirho::SucceedChirho => write!(f_chirho, "succeed"),
            GoalAstChirho::FailChirho => write!(f_chirho, "fail"),
            GoalAstChirho::DelayChirho(_) => write!(f_chirho, "(delay ...)"),
            GoalAstChirho::NamedChirho(name_chirho, g_chirho) => {
                write!(f_chirho, "(named {} {:?})", name_chirho, g_chirho)
            }
            GoalAstChirho::CondeChirho(clauses_chirho) => {
                write!(f_chirho, "(conde {:?})", clauses_chirho)
            }
            GoalAstChirho::NotChirho(g_chirho) => write!(f_chirho, "(not {:?})", g_chirho),
            GoalAstChirho::CondaChirho(cond_chirho, then_chirho, else_chirho) => {
                write!(f_chirho, "(conda {:?} {:?} {:?})", cond_chirho, then_chirho, else_chirho)
            }
        }
    }
}

// === Smart constructors ===

/// Create equality goal
pub fn eq_ast_chirho(t1_chirho: TermIdChirho, t2_chirho: TermIdChirho) -> GoalAstChirho {
    GoalAstChirho::EqChirho(t1_chirho, t2_chirho)
}

/// Create conjunction
pub fn conj_ast_chirho(g1_chirho: GoalAstChirho, g2_chirho: GoalAstChirho) -> GoalAstChirho {
    // Optimize: succeed is identity for conj
    match (&g1_chirho, &g2_chirho) {
        (GoalAstChirho::SucceedChirho, _) => g2_chirho,
        (_, GoalAstChirho::SucceedChirho) => g1_chirho,
        (GoalAstChirho::FailChirho, _) | (_, GoalAstChirho::FailChirho) => GoalAstChirho::FailChirho,
        _ => GoalAstChirho::ConjChirho(Rc::new(g1_chirho), Rc::new(g2_chirho)),
    }
}

/// Create disjunction
pub fn disj_ast_chirho(g1_chirho: GoalAstChirho, g2_chirho: GoalAstChirho) -> GoalAstChirho {
    // Optimize: fail is identity for disj
    match (&g1_chirho, &g2_chirho) {
        (GoalAstChirho::FailChirho, _) => g2_chirho,
        (_, GoalAstChirho::FailChirho) => g1_chirho,
        (GoalAstChirho::SucceedChirho, _) | (_, GoalAstChirho::SucceedChirho) => GoalAstChirho::SucceedChirho,
        _ => GoalAstChirho::DisjChirho(Rc::new(g1_chirho), Rc::new(g2_chirho)),
    }
}

/// Create fresh variable goal
pub fn fresh_ast_chirho<F>(f_chirho: F) -> GoalAstChirho
where
    F: Fn(TermIdChirho) -> GoalAstChirho + 'static,
{
    GoalAstChirho::FreshChirho(Rc::new(f_chirho))
}

/// Create conde
pub fn conde_ast_chirho(clauses_chirho: Vec<Vec<GoalAstChirho>>) -> GoalAstChirho {
    if clauses_chirho.is_empty() {
        return GoalAstChirho::FailChirho;
    }
    if clauses_chirho.len() == 1 && clauses_chirho[0].is_empty() {
        return GoalAstChirho::SucceedChirho;
    }
    GoalAstChirho::CondeChirho(clauses_chirho)
}

/// Conjunction of multiple goals
pub fn conj_all_ast_chirho(goals_chirho: Vec<GoalAstChirho>) -> GoalAstChirho {
    goals_chirho.into_iter().fold(GoalAstChirho::SucceedChirho, conj_ast_chirho)
}

/// Disjunction of multiple goals
pub fn disj_all_ast_chirho(goals_chirho: Vec<GoalAstChirho>) -> GoalAstChirho {
    goals_chirho.into_iter().fold(GoalAstChirho::FailChirho, disj_ast_chirho)
}

// === Goal analysis (introspection) ===

/// Count nodes in goal AST
pub fn goal_size_chirho(goal_chirho: &GoalAstChirho) -> usize {
    match goal_chirho {
        GoalAstChirho::EqChirho(_, _) => 1,
        GoalAstChirho::ConjChirho(g1_chirho, g2_chirho) => {
            1 + goal_size_chirho(g1_chirho) + goal_size_chirho(g2_chirho)
        }
        GoalAstChirho::DisjChirho(g1_chirho, g2_chirho) => {
            1 + goal_size_chirho(g1_chirho) + goal_size_chirho(g2_chirho)
        }
        GoalAstChirho::FreshChirho(_) => 1,
        GoalAstChirho::SucceedChirho | GoalAstChirho::FailChirho => 1,
        GoalAstChirho::DelayChirho(_) => 1,
        GoalAstChirho::NamedChirho(_, g_chirho) => 1 + goal_size_chirho(g_chirho),
        GoalAstChirho::CondeChirho(clauses_chirho) => {
            1 + clauses_chirho.iter()
                .map(|clause_chirho| clause_chirho.iter().map(goal_size_chirho).sum::<usize>())
                .sum::<usize>()
        }
        GoalAstChirho::NotChirho(g_chirho) => 1 + goal_size_chirho(g_chirho),
        GoalAstChirho::CondaChirho(c_chirho, t_chirho, e_chirho) => {
            1 + goal_size_chirho(c_chirho) + goal_size_chirho(t_chirho) + goal_size_chirho(e_chirho)
        }
    }
}

/// Count unification goals
pub fn count_unifications_chirho(goal_chirho: &GoalAstChirho) -> usize {
    match goal_chirho {
        GoalAstChirho::EqChirho(_, _) => 1,
        GoalAstChirho::ConjChirho(g1_chirho, g2_chirho) => {
            count_unifications_chirho(g1_chirho) + count_unifications_chirho(g2_chirho)
        }
        GoalAstChirho::DisjChirho(g1_chirho, g2_chirho) => {
            count_unifications_chirho(g1_chirho) + count_unifications_chirho(g2_chirho)
        }
        GoalAstChirho::NamedChirho(_, g_chirho) => count_unifications_chirho(g_chirho),
        GoalAstChirho::CondeChirho(clauses_chirho) => {
            clauses_chirho.iter()
                .map(|clause_chirho| clause_chirho.iter().map(count_unifications_chirho).sum::<usize>())
                .sum()
        }
        GoalAstChirho::NotChirho(g_chirho) => count_unifications_chirho(g_chirho),
        GoalAstChirho::CondaChirho(c_chirho, t_chirho, e_chirho) => {
            count_unifications_chirho(c_chirho) + count_unifications_chirho(t_chirho) + count_unifications_chirho(e_chirho)
        }
        _ => 0,
    }
}

/// Collect all term IDs referenced in goal
pub fn collect_terms_chirho(goal_chirho: &GoalAstChirho) -> Vec<TermIdChirho> {
    let mut terms_chirho = Vec::new();
    collect_terms_impl_chirho(goal_chirho, &mut terms_chirho);
    terms_chirho
}

fn collect_terms_impl_chirho(goal_chirho: &GoalAstChirho, acc_chirho: &mut Vec<TermIdChirho>) {
    match goal_chirho {
        GoalAstChirho::EqChirho(t1_chirho, t2_chirho) => {
            acc_chirho.push(*t1_chirho);
            acc_chirho.push(*t2_chirho);
        }
        GoalAstChirho::ConjChirho(g1_chirho, g2_chirho) | GoalAstChirho::DisjChirho(g1_chirho, g2_chirho) => {
            collect_terms_impl_chirho(g1_chirho, acc_chirho);
            collect_terms_impl_chirho(g2_chirho, acc_chirho);
        }
        GoalAstChirho::NamedChirho(_, g_chirho) | GoalAstChirho::NotChirho(g_chirho) => {
            collect_terms_impl_chirho(g_chirho, acc_chirho);
        }
        GoalAstChirho::CondeChirho(clauses_chirho) => {
            for clause_chirho in clauses_chirho {
                for g_chirho in clause_chirho {
                    collect_terms_impl_chirho(g_chirho, acc_chirho);
                }
            }
        }
        GoalAstChirho::CondaChirho(c_chirho, t_chirho, e_chirho) => {
            collect_terms_impl_chirho(c_chirho, acc_chirho);
            collect_terms_impl_chirho(t_chirho, acc_chirho);
            collect_terms_impl_chirho(e_chirho, acc_chirho);
        }
        _ => {}
    }
}

// === Interpreter ===

/// Search state for AST interpreter
#[derive(Clone)]
pub struct SearchStateChirho {
    pub store_chirho: TermStoreChirho,
    pub subst_chirho: SubstChirho,
}

impl SearchStateChirho {
    pub fn new_chirho() -> Self {
        Self {
            store_chirho: TermStoreChirho::new(),
            subst_chirho: SubstChirho::new(),
        }
    }

    pub fn with_store_chirho(store_chirho: TermStoreChirho) -> Self {
        Self {
            store_chirho,
            subst_chirho: SubstChirho::new(),
        }
    }
}

/// Stream of search states (lazy)
pub enum StreamAstChirho {
    EmptyChirho,
    ConsChirho(SearchStateChirho, Box<StreamAstChirho>),
    SuspendChirho(Box<dyn FnOnce() -> StreamAstChirho>),
}

impl StreamAstChirho {
    /// Take up to n results
    pub fn take_chirho(self, n_chirho: usize) -> Vec<SearchStateChirho> {
        let mut results_chirho = Vec::new();
        let mut stream_chirho = self;

        while results_chirho.len() < n_chirho {
            match stream_chirho {
                StreamAstChirho::EmptyChirho => break,
                StreamAstChirho::ConsChirho(state_chirho, rest_chirho) => {
                    results_chirho.push(state_chirho);
                    stream_chirho = *rest_chirho;
                }
                StreamAstChirho::SuspendChirho(thunk_chirho) => {
                    stream_chirho = thunk_chirho();
                }
            }
        }

        results_chirho
    }

    /// Take all results (may not terminate!)
    pub fn take_all_chirho(self) -> Vec<SearchStateChirho> {
        self.take_chirho(usize::MAX)
    }

    /// Interleave two streams (fair scheduling)
    pub fn mplus_chirho(self, other_chirho: StreamAstChirho) -> StreamAstChirho {
        match self {
            StreamAstChirho::EmptyChirho => other_chirho,
            StreamAstChirho::ConsChirho(head_chirho, tail_chirho) => {
                StreamAstChirho::ConsChirho(head_chirho, Box::new(other_chirho.mplus_chirho(*tail_chirho)))
            }
            StreamAstChirho::SuspendChirho(thunk_chirho) => {
                StreamAstChirho::SuspendChirho(Box::new(move || other_chirho.mplus_chirho(thunk_chirho())))
            }
        }
    }

    /// Bind (flatMap) over stream
    pub fn bind_chirho<F>(self, f_chirho: F) -> StreamAstChirho
    where
        F: Fn(SearchStateChirho) -> StreamAstChirho + Clone + 'static,
    {
        match self {
            StreamAstChirho::EmptyChirho => StreamAstChirho::EmptyChirho,
            StreamAstChirho::ConsChirho(head_chirho, tail_chirho) => {
                let f_clone_chirho = f_chirho.clone();
                f_chirho(head_chirho).mplus_chirho(tail_chirho.bind_chirho(f_clone_chirho))
            }
            StreamAstChirho::SuspendChirho(thunk_chirho) => {
                StreamAstChirho::SuspendChirho(Box::new(move || thunk_chirho().bind_chirho(f_chirho)))
            }
        }
    }
}

/// Interpret goal AST, producing stream of results
pub fn run_goal_chirho(goal_chirho: &GoalAstChirho, state_chirho: SearchStateChirho) -> StreamAstChirho {
    match goal_chirho {
        GoalAstChirho::EqChirho(t1_chirho, t2_chirho) => {
            let mut new_state_chirho = state_chirho.clone();
            let result_chirho = unify_chirho(
                *t1_chirho,
                *t2_chirho,
                &mut new_state_chirho.subst_chirho,
                &new_state_chirho.store_chirho,
            );
            match result_chirho {
                UnifyResultChirho::Success => {
                    StreamAstChirho::ConsChirho(new_state_chirho, Box::new(StreamAstChirho::EmptyChirho))
                }
                UnifyResultChirho::Failure | UnifyResultChirho::OccursCheckFailed => {
                    StreamAstChirho::EmptyChirho
                }
            }
        }

        GoalAstChirho::SucceedChirho => {
            StreamAstChirho::ConsChirho(state_chirho, Box::new(StreamAstChirho::EmptyChirho))
        }

        GoalAstChirho::FailChirho => StreamAstChirho::EmptyChirho,

        GoalAstChirho::ConjChirho(g1_chirho, g2_chirho) => {
            let g2_clone_chirho = g2_chirho.clone();
            run_goal_chirho(g1_chirho, state_chirho).bind_chirho(move |s_chirho| {
                run_goal_chirho(&g2_clone_chirho, s_chirho)
            })
        }

        GoalAstChirho::DisjChirho(g1_chirho, g2_chirho) => {
            let stream1_chirho = run_goal_chirho(g1_chirho, state_chirho.clone());
            let stream2_chirho = run_goal_chirho(g2_chirho, state_chirho);
            stream1_chirho.mplus_chirho(stream2_chirho)
        }

        GoalAstChirho::FreshChirho(f_chirho) => {
            let mut new_state_chirho = state_chirho.clone();
            let (_, var_term_chirho) = new_state_chirho.store_chirho.fresh_var_chirho();
            let inner_goal_chirho = f_chirho(var_term_chirho);
            run_goal_chirho(&inner_goal_chirho, new_state_chirho)
        }

        GoalAstChirho::DelayChirho(thunk_chirho) => {
            let goal_chirho = thunk_chirho();
            StreamAstChirho::SuspendChirho(Box::new(move || run_goal_chirho(&goal_chirho, state_chirho)))
        }

        GoalAstChirho::NamedChirho(_, inner_chirho) => {
            run_goal_chirho(inner_chirho, state_chirho)
        }

        GoalAstChirho::CondeChirho(clauses_chirho) => {
            let mut result_chirho = StreamAstChirho::EmptyChirho;
            for clause_chirho in clauses_chirho.iter().rev() {
                let clause_goal_chirho = conj_all_ast_chirho(clause_chirho.clone());
                let clause_stream_chirho = run_goal_chirho(&clause_goal_chirho, state_chirho.clone());
                result_chirho = clause_stream_chirho.mplus_chirho(result_chirho);
            }
            result_chirho
        }

        GoalAstChirho::NotChirho(g_chirho) => {
            let results_chirho = run_goal_chirho(g_chirho, state_chirho.clone()).take_chirho(1);
            if results_chirho.is_empty() {
                // g failed, so (not g) succeeds
                StreamAstChirho::ConsChirho(state_chirho, Box::new(StreamAstChirho::EmptyChirho))
            } else {
                // g succeeded, so (not g) fails
                StreamAstChirho::EmptyChirho
            }
        }

        GoalAstChirho::CondaChirho(cond_chirho, then_chirho, else_chirho) => {
            let cond_results_chirho = run_goal_chirho(cond_chirho, state_chirho.clone()).take_chirho(1);
            if let Some(cond_state_chirho) = cond_results_chirho.into_iter().next() {
                run_goal_chirho(then_chirho, cond_state_chirho)
            } else {
                run_goal_chirho(else_chirho, state_chirho)
            }
        }
    }
}

/// High-level run function
pub fn run_ast_chirho(
    n_chirho: usize,
    store_chirho: TermStoreChirho,
    goal_chirho: GoalAstChirho,
) -> Vec<SubstChirho> {
    let state_chirho = SearchStateChirho::with_store_chirho(store_chirho);
    run_goal_chirho(&goal_chirho, state_chirho)
        .take_chirho(n_chirho)
        .into_iter()
        .map(|s_chirho| s_chirho.subst_chirho)
        .collect()
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_goal_ast_eq_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let forty_two_chirho = store_chirho.int_chirho(42);

        let goal_chirho = eq_ast_chirho(x_chirho, forty_two_chirho);
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_goal_ast_conj_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let (_, y_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        let goal_chirho = conj_ast_chirho(
            eq_ast_chirho(x_chirho, one_chirho),
            eq_ast_chirho(y_chirho, two_chirho),
        );
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_goal_ast_disj_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        let goal_chirho = disj_ast_chirho(
            eq_ast_chirho(x_chirho, one_chirho),
            eq_ast_chirho(x_chirho, two_chirho),
        );
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 2);
    }

    #[test]
    fn test_goal_ast_fresh_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let one_chirho = store_chirho.int_chirho(1);

        let goal_chirho = fresh_ast_chirho(move |x_chirho| {
            eq_ast_chirho(x_chirho, one_chirho)
        });
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_goal_ast_conde_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);
        let three_chirho = store_chirho.int_chirho(3);

        let goal_chirho = conde_ast_chirho(vec![
            vec![eq_ast_chirho(x_chirho, one_chirho)],
            vec![eq_ast_chirho(x_chirho, two_chirho)],
            vec![eq_ast_chirho(x_chirho, three_chirho)],
        ]);
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 3);
    }

    #[test]
    fn test_goal_ast_fail_chirho() {
        let store_chirho = TermStoreChirho::new();
        let goal_chirho = GoalAstChirho::FailChirho;
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 0);
    }

    #[test]
    fn test_goal_ast_succeed_chirho() {
        let store_chirho = TermStoreChirho::new();
        let goal_chirho = GoalAstChirho::SucceedChirho;
        let results_chirho = run_ast_chirho(10, store_chirho, goal_chirho);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_goal_size_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);
        let three_chirho = store_chirho.int_chirho(3);

        // Use Rc directly to avoid smart constructor optimization
        let goal_chirho = GoalAstChirho::ConjChirho(
            Rc::new(eq_ast_chirho(one_chirho, two_chirho)),
            Rc::new(GoalAstChirho::DisjChirho(
                Rc::new(eq_ast_chirho(two_chirho, three_chirho)),
                Rc::new(eq_ast_chirho(one_chirho, three_chirho)),
            )),
        );

        // conj(eq, disj(eq, eq)) = 1 + 1 + 1 + 1 + 1 = 5
        assert_eq!(goal_size_chirho(&goal_chirho), 5);
        assert_eq!(count_unifications_chirho(&goal_chirho), 3);
    }

    #[test]
    fn test_goal_optimization_chirho() {
        // conj(succeed, x) should simplify to x
        let mut store_chirho = TermStoreChirho::new();
        let one_chirho = store_chirho.int_chirho(1);

        let goal_chirho = conj_ast_chirho(
            GoalAstChirho::SucceedChirho,
            eq_ast_chirho(one_chirho, one_chirho),
        );

        // Should be just the eq, not a conj
        assert!(matches!(goal_chirho, GoalAstChirho::EqChirho(_, _)));
    }
}
