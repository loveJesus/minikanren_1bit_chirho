//! Unification with Matrix-Based Occurs Check ☧
//!
//! Integrate transitive closure for occurs check.
//! If the term graph has a cycle through a variable, fail.

use crate::bitmatrix_packed_chirho::BitMatrix64Chirho;
use crate::terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho};
use std::collections::HashMap;

/// Substitution mapping variables to terms
#[derive(Debug, Clone, Default)]
pub struct SubstMatrixChirho {
    /// Variable bindings
    bindings_chirho: HashMap<u32, TermIdChirho>,
    /// Term dependency graph for occurs check
    /// Edge (i, j) means term i contains reference to term j
    dep_graph_chirho: BitMatrix64Chirho,
    /// Map from TermId to matrix index
    term_to_idx_chirho: HashMap<TermIdChirho, u8>,
    /// Next available index
    next_idx_chirho: u8,
}

impl SubstMatrixChirho {
    pub fn new_chirho() -> Self {
        Self {
            bindings_chirho: HashMap::new(),
            dep_graph_chirho: BitMatrix64Chirho::new_chirho(64, 64),
            term_to_idx_chirho: HashMap::new(),
            next_idx_chirho: 0,
        }
    }

    /// Get or allocate matrix index for a term
    fn get_idx_chirho(&mut self, term_id_chirho: TermIdChirho) -> u8 {
        if let Some(&idx_chirho) = self.term_to_idx_chirho.get(&term_id_chirho) {
            idx_chirho
        } else {
            let idx_chirho = self.next_idx_chirho;
            self.next_idx_chirho += 1;
            self.term_to_idx_chirho.insert(term_id_chirho, idx_chirho);
            idx_chirho
        }
    }

    /// Walk a term to its bound value
    pub fn walk_chirho(&self, store_chirho: &TermStoreChirho, term_id_chirho: TermIdChirho) -> TermIdChirho {
        if let Some(TermChirho::VarChirho(v_chirho)) = store_chirho.get_chirho(term_id_chirho) {
            if let Some(&bound_chirho) = self.bindings_chirho.get(v_chirho) {
                return self.walk_chirho(store_chirho, bound_chirho);
            }
        }
        term_id_chirho
    }

    /// Add edges to dependency graph for a term's structure
    fn add_term_edges_chirho(&mut self, store_chirho: &TermStoreChirho, parent_id_chirho: TermIdChirho, term_id_chirho: TermIdChirho) {
        let parent_idx_chirho = self.get_idx_chirho(parent_id_chirho);
        let term_idx_chirho = self.get_idx_chirho(term_id_chirho);

        // Parent contains child
        self.dep_graph_chirho.set_chirho(parent_idx_chirho, term_idx_chirho);

        // Recurse into structure
        if let Some(term_chirho) = store_chirho.get_chirho(term_id_chirho) {
            match term_chirho {
                TermChirho::ConsChirho(h_chirho, t_chirho) => {
                    self.add_term_edges_chirho(store_chirho, term_id_chirho, *h_chirho);
                    self.add_term_edges_chirho(store_chirho, term_id_chirho, *t_chirho);
                }
                _ => {}
            }
        }
    }

    /// Check occurs via transitive closure
    /// Returns true if value_term contains var_term (would create infinite term)
    fn occurs_check_matrix_chirho(&mut self, var_term_id_chirho: TermIdChirho, value_term_id_chirho: TermIdChirho) -> bool {
        let var_idx_chirho = self.get_idx_chirho(var_term_id_chirho);
        let val_idx_chirho = self.get_idx_chirho(value_term_id_chirho);

        // Check if value can reach var (meaning var occurs in value's structure)
        // We use the term structure graph, NOT the binding graph
        let tc_chirho = self.dep_graph_chirho.transitive_closure_chirho();

        // If value can reach var, then var occurs in value
        tc_chirho.get_chirho(val_idx_chirho, var_idx_chirho)
    }

    /// Extend substitution with new binding, checking occurs
    pub fn extend_chirho(
        &mut self,
        store_chirho: &TermStoreChirho,
        var_id_chirho: u32,
        var_term_id_chirho: TermIdChirho,
        value_id_chirho: TermIdChirho,
    ) -> bool {
        // First add structure edges for the value
        self.add_term_edges_chirho(store_chirho, value_id_chirho, value_id_chirho);

        // Check if this binding would create a cycle
        if self.occurs_check_matrix_chirho(var_term_id_chirho, value_id_chirho) {
            return false; // Occurs check failed
        }

        self.bindings_chirho.insert(var_id_chirho, value_id_chirho);
        true
    }

    /// Get binding for a variable
    pub fn get_chirho(&self, var_id_chirho: u32) -> Option<TermIdChirho> {
        self.bindings_chirho.get(&var_id_chirho).copied()
    }
}

/// Unify two terms using matrix-based occurs check
pub fn unify_matrix_chirho(
    store_chirho: &TermStoreChirho,
    subst_chirho: &mut SubstMatrixChirho,
    t1_chirho: TermIdChirho,
    t2_chirho: TermIdChirho,
) -> bool {
    let w1_chirho = subst_chirho.walk_chirho(store_chirho, t1_chirho);
    let w2_chirho = subst_chirho.walk_chirho(store_chirho, t2_chirho);

    if w1_chirho == w2_chirho {
        return true;
    }

    let term1_chirho = store_chirho.get_chirho(w1_chirho);
    let term2_chirho = store_chirho.get_chirho(w2_chirho);

    match (term1_chirho, term2_chirho) {
        (Some(TermChirho::VarChirho(v_chirho)), _) => {
            subst_chirho.extend_chirho(store_chirho, *v_chirho, w1_chirho, w2_chirho)
        }
        (_, Some(TermChirho::VarChirho(v_chirho))) => {
            subst_chirho.extend_chirho(store_chirho, *v_chirho, w2_chirho, w1_chirho)
        }
        (Some(TermChirho::IntChirho(a_chirho)), Some(TermChirho::IntChirho(b_chirho))) => {
            a_chirho == b_chirho
        }
        (Some(TermChirho::NilChirho), Some(TermChirho::NilChirho)) => true,
        (Some(TermChirho::SymChirho(a_chirho)), Some(TermChirho::SymChirho(b_chirho))) => {
            a_chirho == b_chirho
        }
        (Some(TermChirho::ConsChirho(h1_chirho, t1_chirho)), Some(TermChirho::ConsChirho(h2_chirho, t2_chirho))) => {
            unify_matrix_chirho(store_chirho, subst_chirho, *h1_chirho, *h2_chirho)
                && unify_matrix_chirho(store_chirho, subst_chirho, *t1_chirho, *t2_chirho)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_unify_matrix_basic_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let forty_two_chirho = store_chirho.int_chirho(42);

        assert!(unify_matrix_chirho(&store_chirho, &mut subst_chirho, x_chirho, forty_two_chirho));

        let walked_chirho = subst_chirho.walk_chirho(&store_chirho, x_chirho);
        assert_eq!(walked_chirho, forty_two_chirho);
    }

    #[test]
    fn test_unify_matrix_lists_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        let list1_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);
        let list2_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);

        assert!(unify_matrix_chirho(&store_chirho, &mut subst_chirho, list1_chirho, list2_chirho));
    }

    #[test]
    fn test_unify_matrix_conflict_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        assert!(!unify_matrix_chirho(&store_chirho, &mut subst_chirho, one_chirho, two_chirho));
    }

    #[test]
    fn test_unify_matrix_occurs_check_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        // x = cons(x, nil) should fail occurs check
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let nil_chirho = store_chirho.nil_chirho();
        let cons_x_chirho = store_chirho.cons_chirho(x_chirho, nil_chirho);

        // This should fail because x occurs in cons(x, nil)
        assert!(!unify_matrix_chirho(&store_chirho, &mut subst_chirho, x_chirho, cons_x_chirho));
    }

    #[test]
    fn test_unify_matrix_nested_occurs_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        // x = cons(1, cons(x, nil)) should fail
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let nil_chirho = store_chirho.nil_chirho();
        let inner_chirho = store_chirho.cons_chirho(x_chirho, nil_chirho);
        let outer_chirho = store_chirho.cons_chirho(one_chirho, inner_chirho);

        assert!(!unify_matrix_chirho(&store_chirho, &mut subst_chirho, x_chirho, outer_chirho));
    }

    #[test]
    fn test_unify_matrix_deep_ok_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        // x = cons(1, cons(2, nil)) is fine (no cycle)
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let list_chirho = store_chirho.list_ints_chirho(&[1, 2]);

        assert!(unify_matrix_chirho(&store_chirho, &mut subst_chirho, x_chirho, list_chirho));
    }

    #[test]
    fn test_unify_matrix_transitive_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstMatrixChirho::new_chirho();

        // x = y, y = 42 => x = 42
        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let (_, y_chirho) = store_chirho.fresh_var_chirho();
        let forty_two_chirho = store_chirho.int_chirho(42);

        assert!(unify_matrix_chirho(&store_chirho, &mut subst_chirho, x_chirho, y_chirho));
        assert!(unify_matrix_chirho(&store_chirho, &mut subst_chirho, y_chirho, forty_two_chirho));

        let walked_x_chirho = subst_chirho.walk_chirho(&store_chirho, x_chirho);
        assert_eq!(walked_x_chirho, forty_two_chirho);
    }
}
