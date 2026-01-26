// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! SLG Completion for Mutual Recursion ☧
//!
//! Full SLG resolution protocol for mutually recursive predicates.
//!
//! The problem with naive tabling:
//!   even(0).
//!   even(s(X)) :- odd(X).
//!   odd(s(X)) :- even(X).
//!
//! Without proper completion, mutual recursion can loop or miss answers.
//!
//! SLG Solution:
//! 1. Track dependency graph between subgoals
//! 2. Detect strongly connected components (SCCs) via Tarjan's algorithm
//! 3. Complete SCCs together (not individual goals)
//! 4. Propagate answers across SCC boundaries
//!
//! References:
//! - "Efficient Access Mechanisms for Tabled Logic Programs" (Swift & Warren)
//! - "A Survey of Tabling in Logic Programming" (Zhou & Sato)

use std::collections::{HashMap, HashSet};

/// Goal status in SLG resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalStatusChirho {
    /// Still collecting answers
    IncompleteChirho,
    /// All answers found
    CompleteChirho,
}

/// A goal pattern (relation name + arguments pattern)
pub type GoalPatternChirho = (u32, Vec<u32>); // (relation_id, arg_ids)

/// A single subgoal in the SLG table
#[derive(Debug, Clone)]
pub struct SlgGoalChirho {
    /// The goal pattern
    pub pattern_chirho: GoalPatternChirho,
    /// Collected answers (each answer is a tuple of term IDs)
    pub answers_chirho: HashSet<Vec<u32>>,
    /// Goals this goal depends on
    pub depends_on_chirho: HashSet<GoalPatternChirho>,
    /// Goals that depend on this goal
    pub depended_by_chirho: HashSet<GoalPatternChirho>,
    /// Current status
    pub status_chirho: GoalStatusChirho,
    /// SCC ID (assigned after Tarjan's)
    pub scc_id_chirho: Option<usize>,
}

impl SlgGoalChirho {
    pub fn new_chirho(pattern_chirho: GoalPatternChirho) -> Self {
        Self {
            pattern_chirho,
            answers_chirho: HashSet::new(),
            depends_on_chirho: HashSet::new(),
            depended_by_chirho: HashSet::new(),
            status_chirho: GoalStatusChirho::IncompleteChirho,
            scc_id_chirho: None,
        }
    }
}

/// SLG Table with completion protocol
#[derive(Debug)]
pub struct SlgTableChirho {
    /// All goals indexed by pattern
    goals_chirho: HashMap<GoalPatternChirho, SlgGoalChirho>,
    /// Computed SCCs (each SCC is a set of goal patterns)
    sccs_chirho: Vec<HashSet<GoalPatternChirho>>,
    /// Tarjan's algorithm state
    tarjan_index_chirho: HashMap<GoalPatternChirho, usize>,
    tarjan_lowlink_chirho: HashMap<GoalPatternChirho, usize>,
    tarjan_on_stack_chirho: HashSet<GoalPatternChirho>,
    tarjan_stack_chirho: Vec<GoalPatternChirho>,
    tarjan_counter_chirho: usize,
}

impl SlgTableChirho {
    pub fn new_chirho() -> Self {
        Self {
            goals_chirho: HashMap::new(),
            sccs_chirho: Vec::new(),
            tarjan_index_chirho: HashMap::new(),
            tarjan_lowlink_chirho: HashMap::new(),
            tarjan_on_stack_chirho: HashSet::new(),
            tarjan_stack_chirho: Vec::new(),
            tarjan_counter_chirho: 0,
        }
    }

    /// Get or create a goal entry. Returns (goal, is_new)
    pub fn get_or_create_chirho(
        &mut self,
        pattern_chirho: GoalPatternChirho,
    ) -> (&mut SlgGoalChirho, bool) {
        let is_new_chirho = !self.goals_chirho.contains_key(&pattern_chirho);
        if is_new_chirho {
            self.goals_chirho
                .insert(pattern_chirho.clone(), SlgGoalChirho::new_chirho(pattern_chirho.clone()));
        }
        (self.goals_chirho.get_mut(&pattern_chirho).unwrap(), is_new_chirho)
    }

    /// Record dependency: from_pattern depends on to_pattern
    pub fn add_dependency_chirho(
        &mut self,
        from_pattern_chirho: GoalPatternChirho,
        to_pattern_chirho: GoalPatternChirho,
    ) {
        if from_pattern_chirho == to_pattern_chirho {
            return; // Self-dependency (direct recursion)
        }

        // Ensure both exist
        self.get_or_create_chirho(from_pattern_chirho.clone());
        self.get_or_create_chirho(to_pattern_chirho.clone());

        // Add edges
        self.goals_chirho
            .get_mut(&from_pattern_chirho)
            .unwrap()
            .depends_on_chirho
            .insert(to_pattern_chirho.clone());

        self.goals_chirho
            .get_mut(&to_pattern_chirho)
            .unwrap()
            .depended_by_chirho
            .insert(from_pattern_chirho);
    }

    /// Add answer to a goal. Returns true if new.
    pub fn add_answer_chirho(
        &mut self,
        pattern_chirho: &GoalPatternChirho,
        answer_chirho: Vec<u32>,
    ) -> bool {
        if let Some(goal_chirho) = self.goals_chirho.get_mut(pattern_chirho) {
            if goal_chirho.answers_chirho.contains(&answer_chirho) {
                false
            } else {
                goal_chirho.answers_chirho.insert(answer_chirho);
                true
            }
        } else {
            false
        }
    }

    /// Get answers for a goal
    pub fn get_answers_chirho(&self, pattern_chirho: &GoalPatternChirho) -> Vec<Vec<u32>> {
        self.goals_chirho
            .get(pattern_chirho)
            .map(|g_chirho| g_chirho.answers_chirho.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if goal is complete
    pub fn is_complete_chirho(&self, pattern_chirho: &GoalPatternChirho) -> bool {
        self.goals_chirho
            .get(pattern_chirho)
            .map(|g_chirho| g_chirho.status_chirho == GoalStatusChirho::CompleteChirho)
            .unwrap_or(false)
    }

    /// Compute SCCs using Tarjan's algorithm
    pub fn compute_sccs_chirho(&mut self) {
        // Reset Tarjan state
        self.tarjan_index_chirho.clear();
        self.tarjan_lowlink_chirho.clear();
        self.tarjan_on_stack_chirho.clear();
        self.tarjan_stack_chirho.clear();
        self.tarjan_counter_chirho = 0;
        self.sccs_chirho.clear();

        // Get all patterns (clone to avoid borrow issues)
        let patterns_chirho: Vec<_> = self.goals_chirho.keys().cloned().collect();

        for pattern_chirho in patterns_chirho {
            if !self.tarjan_index_chirho.contains_key(&pattern_chirho) {
                self.tarjan_visit_chirho(pattern_chirho);
            }
        }

        // Assign SCC IDs to goals
        for (i_chirho, scc_chirho) in self.sccs_chirho.iter().enumerate() {
            for pattern_chirho in scc_chirho {
                if let Some(goal_chirho) = self.goals_chirho.get_mut(pattern_chirho) {
                    goal_chirho.scc_id_chirho = Some(i_chirho);
                }
            }
        }
    }

    /// Tarjan's algorithm visit step
    fn tarjan_visit_chirho(&mut self, pattern_chirho: GoalPatternChirho) {
        let index_chirho = self.tarjan_counter_chirho;
        self.tarjan_index_chirho.insert(pattern_chirho.clone(), index_chirho);
        self.tarjan_lowlink_chirho.insert(pattern_chirho.clone(), index_chirho);
        self.tarjan_counter_chirho += 1;
        self.tarjan_stack_chirho.push(pattern_chirho.clone());
        self.tarjan_on_stack_chirho.insert(pattern_chirho.clone());

        // Get dependencies (clone to avoid borrow issues)
        let deps_chirho: Vec<_> = self
            .goals_chirho
            .get(&pattern_chirho)
            .map(|g_chirho| g_chirho.depends_on_chirho.iter().cloned().collect())
            .unwrap_or_default();

        for dep_chirho in deps_chirho {
            if !self.goals_chirho.contains_key(&dep_chirho) {
                continue;
            }

            if !self.tarjan_index_chirho.contains_key(&dep_chirho) {
                // Not visited yet
                self.tarjan_visit_chirho(dep_chirho.clone());
                let dep_lowlink_chirho = self.tarjan_lowlink_chirho[&dep_chirho];
                let curr_lowlink_chirho = self.tarjan_lowlink_chirho[&pattern_chirho];
                self.tarjan_lowlink_chirho
                    .insert(pattern_chirho.clone(), curr_lowlink_chirho.min(dep_lowlink_chirho));
            } else if self.tarjan_on_stack_chirho.contains(&dep_chirho) {
                // Back edge
                let dep_index_chirho = self.tarjan_index_chirho[&dep_chirho];
                let curr_lowlink_chirho = self.tarjan_lowlink_chirho[&pattern_chirho];
                self.tarjan_lowlink_chirho
                    .insert(pattern_chirho.clone(), curr_lowlink_chirho.min(dep_index_chirho));
            }
        }

        // Root of SCC?
        if self.tarjan_lowlink_chirho[&pattern_chirho] == self.tarjan_index_chirho[&pattern_chirho] {
            let mut scc_chirho = HashSet::new();
            loop {
                let w_chirho = self.tarjan_stack_chirho.pop().unwrap();
                self.tarjan_on_stack_chirho.remove(&w_chirho);
                scc_chirho.insert(w_chirho.clone());
                if w_chirho == pattern_chirho {
                    break;
                }
            }
            self.sccs_chirho.push(scc_chirho);
        }
    }

    /// Mark an entire SCC as complete
    pub fn complete_scc_chirho(&mut self, scc_id_chirho: usize) {
        if scc_id_chirho < self.sccs_chirho.len() {
            let patterns_chirho: Vec<_> = self.sccs_chirho[scc_id_chirho].iter().cloned().collect();
            for pattern_chirho in patterns_chirho {
                if let Some(goal_chirho) = self.goals_chirho.get_mut(&pattern_chirho) {
                    goal_chirho.status_chirho = GoalStatusChirho::CompleteChirho;
                }
            }
        }
    }

    /// Get SCC for a goal
    pub fn get_scc_chirho(&self, pattern_chirho: &GoalPatternChirho) -> Option<&HashSet<GoalPatternChirho>> {
        let scc_id_chirho = self.goals_chirho.get(pattern_chirho)?.scc_id_chirho?;
        self.sccs_chirho.get(scc_id_chirho)
    }

    /// Number of goals
    pub fn num_goals_chirho(&self) -> usize {
        self.goals_chirho.len()
    }

    /// Number of SCCs
    pub fn num_sccs_chirho(&self) -> usize {
        self.sccs_chirho.len()
    }
}

/// Even/Odd mutual recursion as bit sets ☧
/// Demonstrates the tensor view of mutually recursive relations
#[derive(Debug, Default)]
pub struct EvenOddTensorChirho {
    /// Bit set of even numbers (bit i = 1 iff i is even)
    pub even_bits_chirho: u64,
    /// Bit set of odd numbers (bit i = 1 iff i is odd)
    pub odd_bits_chirho: u64,
    /// Maximum computed so far
    pub max_computed_chirho: u32,
}

impl EvenOddTensorChirho {
    pub fn new_chirho() -> Self {
        Self::default()
    }

    /// Compute even/odd up to n using the recurrence
    /// even(0) = true
    /// even(n+1) = odd(n)
    /// odd(n+1) = even(n)
    pub fn compute_up_to_chirho(&mut self, n_chirho: u32) {
        if n_chirho <= self.max_computed_chirho {
            return;
        }

        // Bootstrap: 0 is even
        self.even_bits_chirho |= 1;

        for i_chirho in (self.max_computed_chirho + 1)..=n_chirho {
            if i_chirho >= 64 {
                break; // Limit to 64 bits
            }
            let prev_chirho = i_chirho - 1;

            // even(i) = odd(i-1)
            if (self.odd_bits_chirho >> prev_chirho) & 1 == 1 {
                self.even_bits_chirho |= 1 << i_chirho;
            }

            // odd(i) = even(i-1)
            if (self.even_bits_chirho >> prev_chirho) & 1 == 1 {
                self.odd_bits_chirho |= 1 << i_chirho;
            }
        }

        self.max_computed_chirho = n_chirho.min(63);
    }

    /// Check if n is even
    pub fn is_even_chirho(&self, n_chirho: u32) -> bool {
        if n_chirho >= 64 {
            return n_chirho % 2 == 0; // Fallback
        }
        (self.even_bits_chirho >> n_chirho) & 1 == 1
    }

    /// Check if n is odd
    pub fn is_odd_chirho(&self, n_chirho: u32) -> bool {
        if n_chirho >= 64 {
            return n_chirho % 2 == 1; // Fallback
        }
        (self.odd_bits_chirho >> n_chirho) & 1 == 1
    }

    /// Get all even numbers as a vector
    pub fn get_evens_chirho(&self) -> Vec<u32> {
        (0..=self.max_computed_chirho)
            .filter(|&n_chirho| self.is_even_chirho(n_chirho))
            .collect()
    }

    /// Get all odd numbers as a vector
    pub fn get_odds_chirho(&self) -> Vec<u32> {
        (0..=self.max_computed_chirho)
            .filter(|&n_chirho| self.is_odd_chirho(n_chirho))
            .collect()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_slg_table_basic_chirho() {
        let mut table_chirho = SlgTableChirho::new_chirho();

        // Create two goals
        let even_0_chirho: GoalPatternChirho = (0, vec![0]); // even(0)
        let odd_1_chirho: GoalPatternChirho = (1, vec![1]); // odd(1)

        table_chirho.get_or_create_chirho(even_0_chirho.clone());
        table_chirho.get_or_create_chirho(odd_1_chirho.clone());

        assert_eq!(table_chirho.num_goals_chirho(), 2);
    }

    #[test]
    fn test_slg_dependency_chirho() {
        let mut table_chirho = SlgTableChirho::new_chirho();

        let even_2_chirho: GoalPatternChirho = (0, vec![2]); // even(2)
        let odd_1_chirho: GoalPatternChirho = (1, vec![1]); // odd(1)

        // even(2) depends on odd(1)
        table_chirho.add_dependency_chirho(even_2_chirho.clone(), odd_1_chirho.clone());

        let goal_chirho = &table_chirho.goals_chirho[&even_2_chirho];
        assert!(goal_chirho.depends_on_chirho.contains(&odd_1_chirho));
    }

    #[test]
    fn test_slg_scc_chirho() {
        let mut table_chirho = SlgTableChirho::new_chirho();

        // Create mutual dependency: A -> B -> A
        let goal_a_chirho: GoalPatternChirho = (0, vec![]);
        let goal_b_chirho: GoalPatternChirho = (1, vec![]);

        table_chirho.add_dependency_chirho(goal_a_chirho.clone(), goal_b_chirho.clone());
        table_chirho.add_dependency_chirho(goal_b_chirho.clone(), goal_a_chirho.clone());

        table_chirho.compute_sccs_chirho();

        // Should be 1 SCC containing both
        assert_eq!(table_chirho.num_sccs_chirho(), 1);
        let scc_chirho = table_chirho.get_scc_chirho(&goal_a_chirho).unwrap();
        assert!(scc_chirho.contains(&goal_a_chirho));
        assert!(scc_chirho.contains(&goal_b_chirho));
    }

    #[test]
    fn test_even_odd_tensor_chirho() {
        let mut tensor_chirho = EvenOddTensorChirho::new_chirho();

        tensor_chirho.compute_up_to_chirho(10);

        // Check evens: 0, 2, 4, 6, 8, 10
        assert!(tensor_chirho.is_even_chirho(0));
        assert!(tensor_chirho.is_even_chirho(2));
        assert!(tensor_chirho.is_even_chirho(4));
        assert!(!tensor_chirho.is_even_chirho(1));
        assert!(!tensor_chirho.is_even_chirho(3));

        // Check odds: 1, 3, 5, 7, 9
        assert!(tensor_chirho.is_odd_chirho(1));
        assert!(tensor_chirho.is_odd_chirho(3));
        assert!(!tensor_chirho.is_odd_chirho(0));
        assert!(!tensor_chirho.is_odd_chirho(2));

        assert_eq!(tensor_chirho.get_evens_chirho(), vec![0, 2, 4, 6, 8, 10]);
        assert_eq!(tensor_chirho.get_odds_chirho(), vec![1, 3, 5, 7, 9]);
    }

    #[test]
    fn test_slg_completion_chirho() {
        let mut table_chirho = SlgTableChirho::new_chirho();

        let goal_chirho: GoalPatternChirho = (0, vec![0]);
        table_chirho.get_or_create_chirho(goal_chirho.clone());

        assert!(!table_chirho.is_complete_chirho(&goal_chirho));

        // Compute SCCs and complete
        table_chirho.compute_sccs_chirho();
        table_chirho.complete_scc_chirho(0);

        assert!(table_chirho.is_complete_chirho(&goal_chirho));
    }
}
