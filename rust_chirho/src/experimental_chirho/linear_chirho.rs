// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Linear Logic for Goals ☧
//!
//! Linear types for resource-aware logic programming.
//! Rust's ownership = linear logic's ! modality.
//!
//! Inspired by Girard's linear logic and session types.
//!
//! Key insight: Linear goals must be consumed exactly once.

use crate::SubstChirho;
use crate::stream_chirho::StreamChirho;
use crate::terms_chirho::TermStoreChirho;
use std::sync::Arc;

// ============================================================================
// Linear Goal
// ============================================================================

/// Goal function type (for internal use)
pub type GoalFnInternalChirho = Arc<dyn Fn(SubstChirho, &TermStoreChirho) -> StreamChirho + Send + Sync>;

/// Linear goal: must be consumed exactly once
/// This is enforced by Rust's ownership system
pub struct LinearGoalChirho(GoalFnInternalChirho);

impl LinearGoalChirho {
    /// Create a new linear goal
    pub fn new_chirho<F>(f_chirho: F) -> Self
    where
        F: Fn(SubstChirho, &TermStoreChirho) -> StreamChirho + Send + Sync + 'static,
    {
        LinearGoalChirho(Arc::new(f_chirho))
    }

    /// Consume the goal, producing a stream
    /// This takes ownership, so goal cannot be reused (linear!)
    pub fn run_once_chirho(self, subst_chirho: SubstChirho, store_chirho: &TermStoreChirho) -> StreamChirho {
        (self.0)(subst_chirho, store_chirho)
    }

    /// Peek at result without consuming (for testing)
    pub fn peek_chirho(&self, subst_chirho: SubstChirho, store_chirho: &TermStoreChirho) -> StreamChirho {
        (self.0)(subst_chirho, store_chirho)
    }
}

// ============================================================================
// Reusable Goal (! modality)
// ============================================================================

/// Reusable goal: the "bang" (!) modality of linear logic
/// Can be cloned and used multiple times
#[derive(Clone)]
pub struct ReusableGoalChirho(GoalFnInternalChirho);

impl ReusableGoalChirho {
    /// Create a reusable goal
    pub fn new_chirho<F>(f_chirho: F) -> Self
    where
        F: Fn(SubstChirho, &TermStoreChirho) -> StreamChirho + Send + Sync + 'static,
    {
        ReusableGoalChirho(Arc::new(f_chirho))
    }

    /// Run the goal (can be called multiple times)
    pub fn run_chirho(&self, subst_chirho: SubstChirho, store_chirho: &TermStoreChirho) -> StreamChirho {
        (self.0)(subst_chirho, store_chirho)
    }

    /// Dereliction: convert to linear (forget reusability)
    pub fn derelict_chirho(self) -> LinearGoalChirho {
        LinearGoalChirho(self.0)
    }
}

/// Promote a linear goal to reusable (requires the goal to be pure)
/// This is the ! (bang) introduction rule
pub fn bang_chirho<F>(f_chirho: F) -> ReusableGoalChirho
where
    F: Fn(SubstChirho, &TermStoreChirho) -> StreamChirho + Send + Sync + 'static,
{
    ReusableGoalChirho::new_chirho(f_chirho)
}

// ============================================================================
// Linear Connectives
// ============================================================================

/// Tensor (⊗): both goals must succeed (linear AND)
/// Consumes both goals
pub fn tensor_chirho(
    g1_chirho: LinearGoalChirho,
    g2_chirho: LinearGoalChirho,
) -> LinearGoalChirho {
    let g1_arc_chirho = g1_chirho.0;
    let g2_arc_chirho = g2_chirho.0;

    LinearGoalChirho::new_chirho(move |subst_chirho, store_chirho| {
        let s1_chirho = g1_arc_chirho(subst_chirho, store_chirho);
        let g2_clone_chirho = Arc::clone(&g2_arc_chirho);

        // Flatten: collect results from g1, run g2 on each
        let s1_results_chirho = s1_chirho.take_chirho(1000);
        let mut combined_chirho = StreamChirho::empty_chirho();
        for new_subst_chirho in s1_results_chirho {
            let s2_chirho = g2_clone_chirho(new_subst_chirho, store_chirho);
            combined_chirho = combined_chirho.mplus_chirho(s2_chirho);
        }
        combined_chirho
    })
}

/// Par (⅋): at least one goal succeeds (linear OR)
/// Consumes both goals, interleaves results
pub fn par_chirho(
    g1_chirho: LinearGoalChirho,
    g2_chirho: LinearGoalChirho,
) -> LinearGoalChirho {
    let g1_arc_chirho = g1_chirho.0;
    let g2_arc_chirho = g2_chirho.0;

    LinearGoalChirho::new_chirho(move |subst_chirho, store_chirho| {
        let s1_chirho = g1_arc_chirho(subst_chirho.clone(), store_chirho);
        let s2_chirho = g2_arc_chirho(subst_chirho, store_chirho);
        s1_chirho.mplus_chirho(s2_chirho)
    })
}

/// Plus (⊕): choice, exactly one branch taken (additive OR)
/// Returns goals that can be chosen between
pub struct ChoiceChirho {
    left_chirho: LinearGoalChirho,
    right_chirho: LinearGoalChirho,
}

impl ChoiceChirho {
    pub fn new_chirho(left_chirho: LinearGoalChirho, right_chirho: LinearGoalChirho) -> Self {
        ChoiceChirho {
            left_chirho,
            right_chirho,
        }
    }

    /// Choose left branch
    pub fn choose_left_chirho(self) -> LinearGoalChirho {
        self.left_chirho
    }

    /// Choose right branch
    pub fn choose_right_chirho(self) -> LinearGoalChirho {
        self.right_chirho
    }

    /// Choose both (becomes par)
    pub fn choose_both_chirho(self) -> LinearGoalChirho {
        par_chirho(self.left_chirho, self.right_chirho)
    }
}

/// With (&): both available, client chooses (additive AND)
/// Similar to Choice but represents conjunction
pub struct WithChirho {
    pub left_chirho: LinearGoalChirho,
    pub right_chirho: LinearGoalChirho,
}

impl WithChirho {
    pub fn new_chirho(left_chirho: LinearGoalChirho, right_chirho: LinearGoalChirho) -> Self {
        WithChirho {
            left_chirho,
            right_chirho,
        }
    }

    /// Project left component
    pub fn fst_chirho(self) -> LinearGoalChirho {
        self.left_chirho
    }

    /// Project right component
    pub fn snd_chirho(self) -> LinearGoalChirho {
        self.right_chirho
    }
}

// ============================================================================
// Linear Identity and Negation
// ============================================================================

/// One (1): unit for tensor (always succeeds)
pub fn one_chirho() -> LinearGoalChirho {
    LinearGoalChirho::new_chirho(|subst_chirho, _store_chirho| {
        StreamChirho::unit_chirho(subst_chirho)
    })
}

/// Zero (0): unit for plus (always fails)
pub fn zero_chirho() -> LinearGoalChirho {
    LinearGoalChirho::new_chirho(|_subst_chirho, _store_chirho| {
        StreamChirho::empty_chirho()
    })
}

/// Top (⊤): unit for with (accepts anything)
pub fn top_chirho() -> LinearGoalChirho {
    LinearGoalChirho::new_chirho(|subst_chirho, _store_chirho| {
        StreamChirho::unit_chirho(subst_chirho)
    })
}

/// Bottom (⊥): unit for par
pub fn bottom_chirho() -> LinearGoalChirho {
    LinearGoalChirho::new_chirho(|_subst_chirho, _store_chirho| {
        StreamChirho::empty_chirho()
    })
}

// ============================================================================
// Session Types (Communication Protocols)
// ============================================================================

/// Session type: a protocol for goal communication
pub enum SessionChirho<A> {
    /// Send a value then continue
    SendChirho(A, Box<SessionChirho<A>>),
    /// Receive a value then continue
    RecvChirho(Box<dyn Fn(A) -> SessionChirho<A>>),
    /// End of session
    EndChirho,
    /// Choice: offer multiple branches
    OfferChirho(Vec<SessionChirho<A>>),
    /// Selection: choose one branch
    SelectChirho(usize, Box<SessionChirho<A>>),
}

/// A simple goal session: send goal, receive results
pub struct GoalSessionChirho {
    /// Goals to send
    pub pending_chirho: Vec<LinearGoalChirho>,
    /// Results received
    pub results_chirho: Vec<SubstChirho>,
}

impl GoalSessionChirho {
    pub fn new_chirho() -> Self {
        GoalSessionChirho {
            pending_chirho: Vec::new(),
            results_chirho: Vec::new(),
        }
    }

    /// Enqueue a goal
    pub fn send_goal_chirho(&mut self, goal_chirho: LinearGoalChirho) {
        self.pending_chirho.push(goal_chirho);
    }

    /// Process one goal
    pub fn process_one_chirho(&mut self, store_chirho: &TermStoreChirho) -> bool {
        if let Some(goal_chirho) = self.pending_chirho.pop() {
            let subst_chirho = SubstChirho::new();
            let stream_chirho = goal_chirho.run_once_chirho(subst_chirho, store_chirho);
            let results_chirho = stream_chirho.take_chirho(10);
            self.results_chirho.extend(results_chirho);
            true
        } else {
            false
        }
    }

    /// Get all results
    pub fn drain_results_chirho(&mut self) -> Vec<SubstChirho> {
        std::mem::take(&mut self.results_chirho)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_linear_goal_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let goal_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));
        let results_chirho = goal_chirho.run_once_chirho(subst_chirho, &store_chirho).take_chirho(10);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_reusable_goal_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let goal_chirho = bang_chirho(|s, _| StreamChirho::unit_chirho(s));

        // Can run multiple times
        let r1_chirho = goal_chirho.run_chirho(subst_chirho.clone(), &store_chirho).take_chirho(10);
        let r2_chirho = goal_chirho.run_chirho(subst_chirho, &store_chirho).take_chirho(10);

        assert_eq!(r1_chirho.len(), 1);
        assert_eq!(r2_chirho.len(), 1);
    }

    #[test]
    fn test_tensor_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let g1_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));
        let g2_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));

        let combined_chirho = tensor_chirho(g1_chirho, g2_chirho);
        let results_chirho = combined_chirho.run_once_chirho(subst_chirho, &store_chirho).take_chirho(10);

        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_par_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let g1_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));
        let g2_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));

        let combined_chirho = par_chirho(g1_chirho, g2_chirho);
        let results_chirho = combined_chirho.run_once_chirho(subst_chirho, &store_chirho).take_chirho(10);

        // Should have results from both branches
        assert_eq!(results_chirho.len(), 2);
    }

    #[test]
    fn test_one_zero_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        // One always succeeds
        let one_chirho = one_chirho();
        let r1_chirho = one_chirho.run_once_chirho(subst_chirho.clone(), &store_chirho).take_chirho(10);
        assert_eq!(r1_chirho.len(), 1);

        // Zero always fails
        let zero_chirho = zero_chirho();
        let r2_chirho = zero_chirho.run_once_chirho(subst_chirho, &store_chirho).take_chirho(10);
        assert_eq!(r2_chirho.len(), 0);
    }

    #[test]
    fn test_choice_chirho() {
        let store_chirho = TermStoreChirho::new();
        let subst_chirho = SubstChirho::new();

        let g1_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));
        let g2_chirho = LinearGoalChirho::new_chirho(|_, _| StreamChirho::empty_chirho());

        let choice_chirho = ChoiceChirho::new_chirho(g1_chirho, g2_chirho);

        // Choose left (succeeds)
        let left_chirho = choice_chirho.choose_left_chirho();
        let results_chirho = left_chirho.run_once_chirho(subst_chirho, &store_chirho).take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_goal_session_chirho() {
        let store_chirho = TermStoreChirho::new();

        let mut session_chirho = GoalSessionChirho::new_chirho();

        let goal_chirho = LinearGoalChirho::new_chirho(|s, _| StreamChirho::unit_chirho(s));
        session_chirho.send_goal_chirho(goal_chirho);

        assert!(session_chirho.process_one_chirho(&store_chirho));

        let results_chirho = session_chirho.drain_results_chirho();
        assert_eq!(results_chirho.len(), 1);
    }
}
