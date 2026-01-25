//! Tabling (Memoization) ☧
//!
//! SLG-style tabling for termination with recursive relations.
//! Tables store computed answers to avoid infinite loops.

use std::collections::{HashMap, HashSet};
use crate::terms_chirho::TermIdChirho;

/// A call pattern (relation + arguments)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CallPatternChirho {
    pub relation_chirho: String,
    pub args_chirho: Vec<TermIdChirho>,
}

/// Answer: a tuple of term IDs
pub type AnswerChirho = Vec<TermIdChirho>;

/// Table entry status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableStatusChirho {
    /// Currently being computed (for loop detection)
    ActiveChirho,
    /// Computation complete
    CompleteChirho,
}

/// Single table for one relation
#[derive(Debug, Clone, Default)]
pub struct TableChirho {
    /// Call → (status, answers)
    entries_chirho: HashMap<CallPatternChirho, (TableStatusChirho, HashSet<AnswerChirho>)>,
}

impl TableChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start computing a call (mark as active)
    pub fn start_call_chirho(&mut self, pattern_chirho: CallPatternChirho) -> bool {
        if self.entries_chirho.contains_key(&pattern_chirho) {
            return false; // Already seen
        }
        self.entries_chirho.insert(
            pattern_chirho,
            (TableStatusChirho::ActiveChirho, HashSet::new()),
        );
        true
    }

    /// Add an answer for a call
    pub fn add_answer_chirho(&mut self, pattern_chirho: &CallPatternChirho, answer_chirho: AnswerChirho) -> bool {
        if let Some((_, answers_chirho)) = self.entries_chirho.get_mut(pattern_chirho) {
            answers_chirho.insert(answer_chirho)
        } else {
            false
        }
    }

    /// Mark call as complete
    pub fn complete_call_chirho(&mut self, pattern_chirho: &CallPatternChirho) {
        if let Some((status_chirho, _)) = self.entries_chirho.get_mut(pattern_chirho) {
            *status_chirho = TableStatusChirho::CompleteChirho;
        }
    }

    /// Get answers for a call (if any)
    pub fn get_answers_chirho(&self, pattern_chirho: &CallPatternChirho) -> Option<&HashSet<AnswerChirho>> {
        self.entries_chirho.get(pattern_chirho).map(|(_, a)| a)
    }

    /// Check if call is complete
    pub fn is_complete_chirho(&self, pattern_chirho: &CallPatternChirho) -> bool {
        self.entries_chirho
            .get(pattern_chirho)
            .map(|(s, _)| *s == TableStatusChirho::CompleteChirho)
            .unwrap_or(false)
    }

    /// Check if call is active (being computed)
    pub fn is_active_chirho(&self, pattern_chirho: &CallPatternChirho) -> bool {
        self.entries_chirho
            .get(pattern_chirho)
            .map(|(s, _)| *s == TableStatusChirho::ActiveChirho)
            .unwrap_or(false)
    }
}

/// Global table store
#[derive(Debug, Clone, Default)]
pub struct TableStoreChirho {
    tables_chirho: HashMap<String, TableChirho>,
}

impl TableStoreChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create table for relation
    pub fn get_table_chirho(&mut self, relation_chirho: &str) -> &mut TableChirho {
        self.tables_chirho
            .entry(relation_chirho.to_string())
            .or_default()
    }

    /// Check if we should compute or reuse
    pub fn lookup_or_start_chirho(
        &mut self,
        pattern_chirho: CallPatternChirho,
    ) -> LookupResultChirho {
        let table_chirho = self.get_table_chirho(&pattern_chirho.relation_chirho);

        if table_chirho.is_complete_chirho(&pattern_chirho) {
            // Reuse cached answers
            let answers_chirho = table_chirho
                .get_answers_chirho(&pattern_chirho)
                .cloned()
                .unwrap_or_default();
            LookupResultChirho::CachedChirho(answers_chirho)
        } else if table_chirho.is_active_chirho(&pattern_chirho) {
            // Loop detected - return current answers (may be empty initially)
            let answers_chirho = table_chirho
                .get_answers_chirho(&pattern_chirho)
                .cloned()
                .unwrap_or_default();
            LookupResultChirho::LoopChirho(answers_chirho)
        } else {
            // New call - start computing
            table_chirho.start_call_chirho(pattern_chirho);
            LookupResultChirho::ComputeChirho
        }
    }

    /// Add answer to table
    pub fn add_answer_chirho(
        &mut self,
        pattern_chirho: &CallPatternChirho,
        answer_chirho: AnswerChirho,
    ) -> bool {
        let table_chirho = self.get_table_chirho(&pattern_chirho.relation_chirho);
        table_chirho.add_answer_chirho(pattern_chirho, answer_chirho)
    }

    /// Complete a call
    pub fn complete_chirho(&mut self, pattern_chirho: &CallPatternChirho) {
        let table_chirho = self.get_table_chirho(&pattern_chirho.relation_chirho);
        table_chirho.complete_call_chirho(pattern_chirho);
    }

    /// Clear all tables
    pub fn clear_chirho(&mut self) {
        self.tables_chirho.clear();
    }
}

/// Result of table lookup
#[derive(Debug, Clone)]
pub enum LookupResultChirho {
    /// Answers cached, no computation needed
    CachedChirho(HashSet<AnswerChirho>),
    /// Loop detected, return current answers
    LoopChirho(HashSet<AnswerChirho>),
    /// New call, compute answers
    ComputeChirho,
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_table_basic_chirho() {
        let mut store_chirho = TableStoreChirho::new();

        let pattern_chirho = CallPatternChirho {
            relation_chirho: "appendo".to_string(),
            args_chirho: vec![0, 1, 2], // term IDs
        };

        // First lookup should trigger compute
        match store_chirho.lookup_or_start_chirho(pattern_chirho.clone()) {
            LookupResultChirho::ComputeChirho => {}
            _ => panic!("Expected ComputeChirho"),
        }

        // Add an answer
        assert!(store_chirho.add_answer_chirho(&pattern_chirho, vec![0, 1, 2]));

        // Second lookup (still active) should detect loop
        match store_chirho.lookup_or_start_chirho(pattern_chirho.clone()) {
            LookupResultChirho::LoopChirho(answers_chirho) => {
                assert_eq!(answers_chirho.len(), 1);
            }
            _ => panic!("Expected LoopChirho"),
        }

        // Complete the call
        store_chirho.complete_chirho(&pattern_chirho);

        // Now should be cached
        match store_chirho.lookup_or_start_chirho(pattern_chirho.clone()) {
            LookupResultChirho::CachedChirho(answers_chirho) => {
                assert_eq!(answers_chirho.len(), 1);
            }
            _ => panic!("Expected CachedChirho"),
        }
    }
}
