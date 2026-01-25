//! Lazy Stream for miniKanren search ☧
//!
//! Streams represent potentially infinite sequences of solutions.
//! This enables interleaving search for completeness.

use super::unify_chirho::SubstChirho;

/// A lazy stream of substitutions (solutions)
pub enum StreamChirho {
    /// Empty stream - no solutions
    EmptyChirho,
    /// Single solution followed by more
    ConsChirho(SubstChirho, Box<StreamChirho>),
    /// Suspended computation (thunk) for laziness
    ThunkChirho(Box<dyn FnOnce() -> StreamChirho>),
}

impl StreamChirho {
    /// Create empty stream
    pub fn empty_chirho() -> Self {
        StreamChirho::EmptyChirho
    }

    /// Create singleton stream
    pub fn unit_chirho(subst_chirho: SubstChirho) -> Self {
        StreamChirho::ConsChirho(subst_chirho, Box::new(StreamChirho::EmptyChirho))
    }

    /// Suspend a computation
    pub fn suspend_chirho<F>(thunk_chirho: F) -> Self
    where
        F: FnOnce() -> StreamChirho + 'static,
    {
        StreamChirho::ThunkChirho(Box::new(thunk_chirho))
    }

    /// Force a thunk (one step)
    pub fn force_chirho(self) -> Self {
        match self {
            StreamChirho::ThunkChirho(thunk_chirho) => thunk_chirho(),
            other_chirho => other_chirho,
        }
    }

    /// Interleaving append (mplus)
    /// Fair: alternates between streams
    pub fn mplus_chirho(self, other_chirho: StreamChirho) -> StreamChirho {
        match self {
            StreamChirho::EmptyChirho => other_chirho,
            StreamChirho::ConsChirho(head_chirho, tail_chirho) => {
                StreamChirho::ConsChirho(
                    head_chirho,
                    Box::new(other_chirho.mplus_chirho(*tail_chirho)),
                )
            }
            StreamChirho::ThunkChirho(thunk_chirho) => {
                // Swap order for fairness
                StreamChirho::suspend_chirho(move || other_chirho.mplus_chirho(thunk_chirho()))
            }
        }
    }

    /// Bind (flatmap) with interleaving
    pub fn bind_chirho<F>(self, goal_chirho: F) -> StreamChirho
    where
        F: Fn(SubstChirho) -> StreamChirho + Clone + 'static,
    {
        match self {
            StreamChirho::EmptyChirho => StreamChirho::EmptyChirho,
            StreamChirho::ConsChirho(head_chirho, tail_chirho) => {
                let goal_clone_chirho = goal_chirho.clone();
                goal_chirho(head_chirho).mplus_chirho(tail_chirho.bind_chirho(goal_clone_chirho))
            }
            StreamChirho::ThunkChirho(thunk_chirho) => {
                StreamChirho::suspend_chirho(move || thunk_chirho().bind_chirho(goal_chirho))
            }
        }
    }

    /// Take n solutions
    pub fn take_chirho(self, n_chirho: usize) -> Vec<SubstChirho> {
        let mut results_chirho = Vec::new();
        let mut stream_chirho = self;

        for _ in 0..n_chirho {
            stream_chirho = stream_chirho.force_chirho();
            match stream_chirho {
                StreamChirho::EmptyChirho => break,
                StreamChirho::ConsChirho(head_chirho, tail_chirho) => {
                    results_chirho.push(head_chirho);
                    stream_chirho = *tail_chirho;
                }
                StreamChirho::ThunkChirho(_) => {
                    stream_chirho = stream_chirho.force_chirho();
                    continue;
                }
            }
        }

        results_chirho
    }

    /// Take all solutions (careful with infinite streams!)
    pub fn take_all_chirho(self) -> Vec<SubstChirho> {
        let mut results_chirho = Vec::new();
        let mut stream_chirho = self;

        loop {
            stream_chirho = stream_chirho.force_chirho();
            match stream_chirho {
                StreamChirho::EmptyChirho => break,
                StreamChirho::ConsChirho(head_chirho, tail_chirho) => {
                    results_chirho.push(head_chirho);
                    stream_chirho = *tail_chirho;
                }
                StreamChirho::ThunkChirho(_) => continue,
            }
        }

        results_chirho
    }

    /// Check if stream is empty (forces thunks)
    pub fn is_empty_chirho(self) -> bool {
        matches!(self.force_chirho(), StreamChirho::EmptyChirho)
    }
}

/// Goal: a function from substitution to stream of substitutions
pub type GoalChirho = Box<dyn Fn(SubstChirho) -> StreamChirho>;

/// Create a goal that succeeds with current substitution
pub fn succeed_chirho() -> GoalChirho {
    Box::new(|subst_chirho| StreamChirho::unit_chirho(subst_chirho))
}

/// Create a goal that always fails
pub fn fail_chirho() -> GoalChirho {
    Box::new(|_| StreamChirho::empty_chirho())
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_stream_basic_chirho() {
        let s1_chirho = StreamChirho::unit_chirho(SubstChirho::new());
        let results_chirho = s1_chirho.take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }

    #[test]
    fn test_stream_mplus_chirho() {
        let s1_chirho = StreamChirho::unit_chirho(SubstChirho::new());
        let s2_chirho = StreamChirho::unit_chirho(SubstChirho::new());
        let combined_chirho = s1_chirho.mplus_chirho(s2_chirho);
        let results_chirho = combined_chirho.take_chirho(10);
        assert_eq!(results_chirho.len(), 2);
    }

    #[test]
    fn test_stream_suspend_chirho() {
        let s_chirho = StreamChirho::suspend_chirho(|| {
            StreamChirho::unit_chirho(SubstChirho::new())
        });
        let results_chirho = s_chirho.take_chirho(10);
        assert_eq!(results_chirho.len(), 1);
    }
}
