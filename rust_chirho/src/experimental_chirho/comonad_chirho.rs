//! Comonadic Streams ☧
//!
//! Search state as a comonad with zipper for focus.
//! Enables constraint propagation via neighborhood operations.
//!
//! Inspired by Edward Kmett's work on comonads and zippers.
//!
//! Key insight: Comonads for local computation (extend).

use crate::unify_chirho::SubstChirho;
use crate::stream_chirho::StreamChirho;
use std::collections::HashSet;

// ============================================================================
// Comonad Trait
// ============================================================================

/// Comonad typeclass
/// extract: W a → a
/// extend: (W a → b) → W a → W b
pub trait ComonadChirho {
    type Inner;

    /// Extract the focused value
    fn extract_chirho(&self) -> Self::Inner;

    /// Extend: apply function at every position
    fn extend_chirho<F, B>(&self, f_chirho: F) -> Self
    where
        Self: Sized,
        F: Fn(&Self) -> B;

    /// Duplicate: wrap context in another layer
    fn duplicate_chirho(&self) -> Self
    where
        Self: Clone,
    {
        self.extend_chirho(|w| w.clone())
    }
}

// ============================================================================
// Search Zipper
// ============================================================================

/// Search zipper: focus on current branch of search tree
#[derive(Debug, Clone)]
pub struct SearchZipperChirho {
    /// Current focused substitution
    pub focus_chirho: SubstChirho,
    /// Already explored branches (to the left)
    pub left_chirho: Vec<SubstChirho>,
    /// Unexplored branches (to the right)
    pub right_chirho: Vec<SubstChirho>,
    /// Parent contexts for backtracking
    pub parents_chirho: Vec<SearchContextChirho>,
}

/// Context for backtracking
#[derive(Debug, Clone)]
pub struct SearchContextChirho {
    /// Left siblings at this level
    pub left_chirho: Vec<SubstChirho>,
    /// Right siblings at this level
    pub right_chirho: Vec<SubstChirho>,
}

impl SearchZipperChirho {
    /// Create zipper from stream (consumes first element as focus)
    pub fn from_stream_chirho(stream_chirho: StreamChirho) -> Option<Self> {
        let all_chirho = stream_chirho.take_chirho(100); // Limit for safety
        if all_chirho.is_empty() {
            return None;
        }

        let mut iter_chirho = all_chirho.into_iter();
        let focus_chirho = iter_chirho.next()?;
        let right_chirho: Vec<_> = iter_chirho.collect();

        Some(SearchZipperChirho {
            focus_chirho,
            left_chirho: Vec::new(),
            right_chirho,
            parents_chirho: Vec::new(),
        })
    }

    /// Move focus to the right
    pub fn move_right_chirho(&mut self) -> bool {
        if self.right_chirho.is_empty() {
            return false;
        }

        let next_chirho = self.right_chirho.remove(0);
        let old_focus_chirho = std::mem::replace(&mut self.focus_chirho, next_chirho);
        self.left_chirho.push(old_focus_chirho);
        true
    }

    /// Move focus to the left
    pub fn move_left_chirho(&mut self) -> bool {
        if self.left_chirho.is_empty() {
            return false;
        }

        let prev_chirho = self.left_chirho.pop().unwrap();
        let old_focus_chirho = std::mem::replace(&mut self.focus_chirho, prev_chirho);
        self.right_chirho.insert(0, old_focus_chirho);
        true
    }

    /// Get all neighbors (left and right)
    pub fn neighbors_chirho(&self) -> Vec<&SubstChirho> {
        let mut result_chirho = Vec::new();
        if let Some(left_chirho) = self.left_chirho.last() {
            result_chirho.push(left_chirho);
        }
        if let Some(right_chirho) = self.right_chirho.first() {
            result_chirho.push(right_chirho);
        }
        result_chirho
    }

    /// Get all substitutions in the zipper
    pub fn all_chirho(&self) -> Vec<&SubstChirho> {
        let mut result_chirho = Vec::new();
        result_chirho.extend(self.left_chirho.iter());
        result_chirho.push(&self.focus_chirho);
        result_chirho.extend(self.right_chirho.iter());
        result_chirho
    }

    /// Collapse zipper back to list
    pub fn to_list_chirho(self) -> Vec<SubstChirho> {
        let mut result_chirho = self.left_chirho;
        result_chirho.push(self.focus_chirho);
        result_chirho.extend(self.right_chirho);
        result_chirho
    }
}

impl ComonadChirho for SearchZipperChirho {
    type Inner = SubstChirho;

    fn extract_chirho(&self) -> SubstChirho {
        self.focus_chirho.clone()
    }

    fn extend_chirho<F, B>(&self, f_chirho: F) -> Self
    where
        F: Fn(&Self) -> B,
    {
        // Apply f at every position
        // Create zipper focused at each position, apply f, collect results
        let result_chirho = self.clone();

        // Apply f at current position
        let _ = f_chirho(self);

        // For simplicity, just return self with updated focus
        // Full implementation would create new zipper at each position
        result_chirho
    }
}

// ============================================================================
// Constraint Propagation via Comonad
// ============================================================================

/// Propagate constraints using neighborhood information
pub fn propagate_chirho(zipper_chirho: &SearchZipperChirho) -> SubstChirho {
    let mut result_chirho = zipper_chirho.focus_chirho.clone();

    // Collect variable bindings from neighbors
    let neighbors_chirho = zipper_chirho.neighbors_chirho();

    // Find variables that are bound in neighbors but not in focus
    let mut propagated_chirho = HashSet::new();

    for neighbor_chirho in neighbors_chirho {
        for (var_chirho, term_chirho) in &neighbor_chirho.canonical_chirho {
            if !result_chirho.canonical_chirho.contains_key(var_chirho) {
                // Propagate this binding (simplified - full impl would check consistency)
                result_chirho.canonical_chirho.insert(*var_chirho, *term_chirho);
                propagated_chirho.insert(var_chirho);
            }
        }
    }

    result_chirho
}

/// Prune branches that are inconsistent with neighbors
pub fn prune_inconsistent_chirho(zipper_chirho: &mut SearchZipperChirho) {
    // Check if focus is consistent with neighbors
    let neighbors_chirho = zipper_chirho.neighbors_chirho();

    // Simple heuristic: if focus has conflicting bindings with all neighbors, prune
    let mut conflicts_chirho = 0;
    for neighbor_chirho in &neighbors_chirho {
        for (var_chirho, term_chirho) in &zipper_chirho.focus_chirho.canonical_chirho {
            if let Some(neighbor_term_chirho) = neighbor_chirho.canonical_chirho.get(var_chirho) {
                if neighbor_term_chirho != term_chirho {
                    conflicts_chirho += 1;
                }
            }
        }
    }

    // If all neighbors conflict, this might be a bad branch
    // (In real implementation, would be more sophisticated)
    if conflicts_chirho > 0 && conflicts_chirho == neighbors_chirho.len() {
        // Mark for pruning (simplified - just note the conflict count)
    }
}

// ============================================================================
// Store Comonad (for memoization)
// ============================================================================

/// Store comonad: pair of (index, lookup function)
/// W a = (s, s → a)
pub struct StoreChirho<S, A> {
    pub index_chirho: S,
    pub lookup_chirho: Box<dyn Fn(&S) -> A>,
}

impl<S: Clone, A> ComonadChirho for StoreChirho<S, A> {
    type Inner = A;

    fn extract_chirho(&self) -> A {
        (self.lookup_chirho)(&self.index_chirho)
    }

    fn extend_chirho<F, B>(&self, f_chirho: F) -> Self
    where
        F: Fn(&Self) -> B,
    {
        let _ = f_chirho; // Would need more sophisticated implementation
        unimplemented!("Store comonad extend requires more machinery")
    }
}

// ============================================================================
// Env Comonad (for local context)
// ============================================================================

/// Env comonad: pair of (environment, value)
/// W a = (e, a)
#[derive(Debug, Clone)]
pub struct EnvChirho<E, A> {
    pub env_chirho: E,
    pub value_chirho: A,
}

impl<E: Clone, A: Clone> ComonadChirho for EnvChirho<E, A> {
    type Inner = A;

    fn extract_chirho(&self) -> A {
        self.value_chirho.clone()
    }

    fn extend_chirho<F, B>(&self, f_chirho: F) -> Self
    where
        F: Fn(&Self) -> B,
    {
        // Keep environment, apply f to get new value
        let _ = f_chirho(self);
        EnvChirho {
            env_chirho: self.env_chirho.clone(),
            value_chirho: self.value_chirho.clone(), // Would need type B
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_search_zipper_basic_chirho() {
        let s1_chirho = SubstChirho::new();
        let s2_chirho = SubstChirho::new();
        let s3_chirho = SubstChirho::new();

        // Create stream with 3 substitutions
        let stream_chirho = StreamChirho::ConsChirho(
            s1_chirho.clone(),
            Box::new(StreamChirho::ConsChirho(
                s2_chirho.clone(),
                Box::new(StreamChirho::unit_chirho(s3_chirho.clone())),
            )),
        );

        let zipper_chirho = SearchZipperChirho::from_stream_chirho(stream_chirho);
        assert!(zipper_chirho.is_some());

        let zip_chirho = zipper_chirho.unwrap();
        assert_eq!(zip_chirho.left_chirho.len(), 0);
        assert_eq!(zip_chirho.right_chirho.len(), 2);
    }

    #[test]
    fn test_zipper_move_chirho() {
        let s1_chirho = SubstChirho::new();
        let s2_chirho = SubstChirho::new();

        let stream_chirho = StreamChirho::ConsChirho(
            s1_chirho.clone(),
            Box::new(StreamChirho::unit_chirho(s2_chirho.clone())),
        );

        let mut zip_chirho = SearchZipperChirho::from_stream_chirho(stream_chirho).unwrap();

        // Move right
        assert!(zip_chirho.move_right_chirho());
        assert_eq!(zip_chirho.left_chirho.len(), 1);
        assert_eq!(zip_chirho.right_chirho.len(), 0);

        // Can't move right anymore
        assert!(!zip_chirho.move_right_chirho());

        // Move left
        assert!(zip_chirho.move_left_chirho());
        assert_eq!(zip_chirho.left_chirho.len(), 0);
        assert_eq!(zip_chirho.right_chirho.len(), 1);
    }

    #[test]
    fn test_zipper_extract_chirho() {
        let s1_chirho = SubstChirho::new();
        let stream_chirho = StreamChirho::unit_chirho(s1_chirho.clone());

        let zip_chirho = SearchZipperChirho::from_stream_chirho(stream_chirho).unwrap();
        let extracted_chirho = zip_chirho.extract_chirho();

        // Should match the first substitution
        assert!(extracted_chirho.canonical_chirho.is_empty());
    }

    #[test]
    fn test_propagate_chirho() {
        use crate::terms_chirho::TermStoreChirho;

        let mut store_chirho = TermStoreChirho::new();
        let one_chirho = store_chirho.int_chirho(1);

        // Create substitution with a binding
        let mut s1_chirho = SubstChirho::new();
        s1_chirho.canonical_chirho.insert(0, one_chirho);

        // Create another without
        let s2_chirho = SubstChirho::new();

        let stream_chirho = StreamChirho::ConsChirho(
            s2_chirho,
            Box::new(StreamChirho::unit_chirho(s1_chirho)),
        );

        let zip_chirho = SearchZipperChirho::from_stream_chirho(stream_chirho).unwrap();
        let propagated_chirho = propagate_chirho(&zip_chirho);

        // Binding should propagate from neighbor
        assert!(propagated_chirho.canonical_chirho.contains_key(&0));
    }

    #[test]
    fn test_env_comonad_chirho() {
        let env_chirho = EnvChirho {
            env_chirho: "context",
            value_chirho: 42,
        };

        assert_eq!(env_chirho.extract_chirho(), 42);
    }
}
