//! Optics for Terms ☧
//!
//! Prisms, traversals, and lenses for term manipulation.
//! Inspired by Edward Kmett's `lens` library.
//!
//! Key insight: Optics unify walk, reify, and substitution application.

use crate::terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho, VarIdChirho};
use crate::unify_chirho::SubstChirho;

/// Prism: focus on one case of a sum type
/// preview_chirho: S → Option<A>
/// review_chirho: A → S
pub trait PrismChirho<S, A> {
    /// Try to extract A from S
    fn preview_chirho(&self, s: &S, store_chirho: &TermStoreChirho) -> Option<A>;
    /// Inject A into S
    fn review_chirho(&self, a: A, store_chirho: &mut TermStoreChirho) -> S;
}

/// Traversal: focus on zero or more targets
pub trait TraversalChirho<S, A> {
    /// Apply function to all targets
    fn traverse_chirho<F>(&self, s: S, f_chirho: F, store_chirho: &mut TermStoreChirho) -> S
    where
        F: Fn(A, &mut TermStoreChirho) -> A;

    /// Collect all targets
    fn to_list_chirho(&self, s: S, store_chirho: &TermStoreChirho) -> Vec<A>;
}

// ============================================================================
// Prisms for Term constructors
// ============================================================================

/// Prism into Cons cells: TermId → Option<(TermId, TermId)>
pub struct ConsPrismChirho;

impl PrismChirho<TermIdChirho, (TermIdChirho, TermIdChirho)> for ConsPrismChirho {
    fn preview_chirho(
        &self,
        term_chirho: &TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Option<(TermIdChirho, TermIdChirho)> {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::ConsChirho(h, t)) => Some((*h, *t)),
            _ => None,
        }
    }

    fn review_chirho(
        &self,
        (h_chirho, t_chirho): (TermIdChirho, TermIdChirho),
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho {
        store_chirho.cons_chirho(h_chirho, t_chirho)
    }
}

/// Prism into Integer values: TermId → Option<i64>
pub struct IntPrismChirho;

impl PrismChirho<TermIdChirho, i64> for IntPrismChirho {
    fn preview_chirho(
        &self,
        term_chirho: &TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Option<i64> {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::IntChirho(n)) => Some(*n),
            _ => None,
        }
    }

    fn review_chirho(&self, n_chirho: i64, store_chirho: &mut TermStoreChirho) -> TermIdChirho {
        store_chirho.int_chirho(n_chirho)
    }
}

/// Prism into Variables: TermId → Option<VarId>
pub struct VarPrismChirho;

impl PrismChirho<TermIdChirho, VarIdChirho> for VarPrismChirho {
    fn preview_chirho(
        &self,
        term_chirho: &TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Option<VarIdChirho> {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::VarChirho(v)) => Some(*v),
            _ => None,
        }
    }

    fn review_chirho(
        &self,
        var_chirho: VarIdChirho,
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho {
        store_chirho.intern_chirho(TermChirho::VarChirho(var_chirho))
    }
}

/// Prism into Nil: TermId → Option<()>
pub struct NilPrismChirho;

impl PrismChirho<TermIdChirho, ()> for NilPrismChirho {
    fn preview_chirho(
        &self,
        term_chirho: &TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Option<()> {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::NilChirho) => Some(()),
            _ => None,
        }
    }

    fn review_chirho(&self, _: (), store_chirho: &mut TermStoreChirho) -> TermIdChirho {
        store_chirho.nil_chirho()
    }
}

// ============================================================================
// Traversals for subterm access
// ============================================================================

/// Traversal over all immediate children
pub struct ChildrenTraversalChirho;

impl TraversalChirho<TermIdChirho, TermIdChirho> for ChildrenTraversalChirho {
    fn traverse_chirho<F>(
        &self,
        term_chirho: TermIdChirho,
        f_chirho: F,
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho
    where
        F: Fn(TermIdChirho, &mut TermStoreChirho) -> TermIdChirho,
    {
        match store_chirho.get_chirho(term_chirho).cloned() {
            Some(TermChirho::ConsChirho(h, t)) => {
                let h_new_chirho = f_chirho(h, store_chirho);
                let t_new_chirho = f_chirho(t, store_chirho);
                store_chirho.cons_chirho(h_new_chirho, t_new_chirho)
            }
            _ => term_chirho, // Atoms have no children
        }
    }

    fn to_list_chirho(
        &self,
        term_chirho: TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Vec<TermIdChirho> {
        match store_chirho.get_chirho(term_chirho) {
            Some(TermChirho::ConsChirho(h, t)) => vec![*h, *t],
            _ => vec![],
        }
    }
}

/// Traversal over all subterms (deep)
pub struct SubtermTraversalChirho;

impl TraversalChirho<TermIdChirho, TermIdChirho> for SubtermTraversalChirho {
    fn traverse_chirho<F>(
        &self,
        term_chirho: TermIdChirho,
        f_chirho: F,
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho
    where
        F: Fn(TermIdChirho, &mut TermStoreChirho) -> TermIdChirho,
    {
        // Bottom-up traversal: recurse first, then apply
        match store_chirho.get_chirho(term_chirho).cloned() {
            Some(TermChirho::ConsChirho(h, t)) => {
                let h_traversed_chirho = self.traverse_chirho(h, &f_chirho, store_chirho);
                let t_traversed_chirho = self.traverse_chirho(t, &f_chirho, store_chirho);
                let cons_chirho = store_chirho.cons_chirho(h_traversed_chirho, t_traversed_chirho);
                f_chirho(cons_chirho, store_chirho)
            }
            _ => f_chirho(term_chirho, store_chirho),
        }
    }

    fn to_list_chirho(
        &self,
        term_chirho: TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Vec<TermIdChirho> {
        let mut result_chirho = vec![term_chirho];
        match store_chirho.get_chirho(term_chirho) {
            Some(TermChirho::ConsChirho(h, t)) => {
                result_chirho.extend(self.to_list_chirho(*h, store_chirho));
                result_chirho.extend(self.to_list_chirho(*t, store_chirho));
            }
            _ => {}
        }
        result_chirho
    }
}

/// Traversal over all variables in a term
pub struct VarsTraversalChirho;

impl VarsTraversalChirho {
    /// Collect all variables in a term
    pub fn vars_chirho(
        &self,
        term_chirho: TermIdChirho,
        store_chirho: &TermStoreChirho,
    ) -> Vec<VarIdChirho> {
        let mut result_chirho = Vec::new();
        self.collect_vars_chirho(term_chirho, store_chirho, &mut result_chirho);
        result_chirho
    }

    fn collect_vars_chirho(
        &self,
        term_chirho: TermIdChirho,
        store_chirho: &TermStoreChirho,
        result_chirho: &mut Vec<VarIdChirho>,
    ) {
        match store_chirho.get_chirho(term_chirho) {
            Some(TermChirho::VarChirho(v)) => result_chirho.push(*v),
            Some(TermChirho::ConsChirho(h, t)) => {
                self.collect_vars_chirho(*h, store_chirho, result_chirho);
                self.collect_vars_chirho(*t, store_chirho, result_chirho);
            }
            _ => {}
        }
    }
}

// ============================================================================
// Optic composition
// ============================================================================

/// Lens: focus on exactly one part of a structure
/// get_chirho: S → A
/// set_chirho: (S, A) → S
pub trait LensChirho<S, A> {
    fn get_chirho(&self, s: &S, store_chirho: &TermStoreChirho) -> A;
    fn set_chirho(&self, s: S, a: A, store_chirho: &mut TermStoreChirho) -> S;

    /// Modify: get, apply function, set
    fn over_chirho<F>(&self, s: S, f_chirho: F, store_chirho: &mut TermStoreChirho) -> S
    where
        F: FnOnce(A) -> A,
    {
        let a_chirho = self.get_chirho(&s, store_chirho);
        self.set_chirho(s, f_chirho(a_chirho), store_chirho)
    }
}

/// Lens into the head of a Cons cell
pub struct HeadLensChirho;

impl LensChirho<TermIdChirho, TermIdChirho> for HeadLensChirho {
    fn get_chirho(&self, term_chirho: &TermIdChirho, store_chirho: &TermStoreChirho) -> TermIdChirho {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::ConsChirho(h, _)) => *h,
            _ => panic!("HeadLensChirho: not a Cons"),
        }
    }

    fn set_chirho(
        &self,
        term_chirho: TermIdChirho,
        new_head_chirho: TermIdChirho,
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho {
        match store_chirho.get_chirho(term_chirho).cloned() {
            Some(TermChirho::ConsChirho(_, t)) => {
                store_chirho.cons_chirho(new_head_chirho, t)
            }
            _ => panic!("HeadLensChirho: not a Cons"),
        }
    }
}

/// Lens into the tail of a Cons cell
pub struct TailLensChirho;

impl LensChirho<TermIdChirho, TermIdChirho> for TailLensChirho {
    fn get_chirho(&self, term_chirho: &TermIdChirho, store_chirho: &TermStoreChirho) -> TermIdChirho {
        match store_chirho.get_chirho(*term_chirho) {
            Some(TermChirho::ConsChirho(_, t)) => *t,
            _ => panic!("TailLensChirho: not a Cons"),
        }
    }

    fn set_chirho(
        &self,
        term_chirho: TermIdChirho,
        new_tail_chirho: TermIdChirho,
        store_chirho: &mut TermStoreChirho,
    ) -> TermIdChirho {
        match store_chirho.get_chirho(term_chirho).cloned() {
            Some(TermChirho::ConsChirho(h, _)) => {
                store_chirho.cons_chirho(h, new_tail_chirho)
            }
            _ => panic!("TailLensChirho: not a Cons"),
        }
    }
}

// ============================================================================
// Walk as optic application
// ============================================================================

/// Walk is a traversal that replaces variables with their bindings
pub fn walk_deep_optic_chirho(
    term_chirho: TermIdChirho,
    subst_chirho: &mut SubstChirho,
    store_chirho: &mut TermStoreChirho,
) -> TermIdChirho {
    let walked_chirho = subst_chirho.walk_chirho(term_chirho, store_chirho);

    match store_chirho.get_chirho(walked_chirho).cloned() {
        Some(TermChirho::ConsChirho(h, t)) => {
            let h_walked_chirho = walk_deep_optic_chirho(h, subst_chirho, store_chirho);
            let t_walked_chirho = walk_deep_optic_chirho(t, subst_chirho, store_chirho);
            store_chirho.cons_chirho(h_walked_chirho, t_walked_chirho)
        }
        _ => walked_chirho,
    }
}

/// Reify: walk + pretty print (placeholder for now)
pub fn reify_optic_chirho(
    term_chirho: TermIdChirho,
    subst_chirho: &mut SubstChirho,
    store_chirho: &mut TermStoreChirho,
) -> TermIdChirho {
    walk_deep_optic_chirho(term_chirho, subst_chirho, store_chirho)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_cons_prism_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let prism_chirho = ConsPrismChirho;

        // Create [1, 2]
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);
        let nil_chirho = store_chirho.nil_chirho();
        let cons2_chirho = store_chirho.cons_chirho(two_chirho, nil_chirho);
        let cons1_chirho = store_chirho.cons_chirho(one_chirho, cons2_chirho);

        // Preview should succeed
        let result_chirho = prism_chirho.preview_chirho(&cons1_chirho, &store_chirho);
        assert_eq!(result_chirho, Some((one_chirho, cons2_chirho)));

        // Preview on int should fail
        let result2_chirho = prism_chirho.preview_chirho(&one_chirho, &store_chirho);
        assert_eq!(result2_chirho, None);

        // Review should create a new cons
        let new_cons_chirho = prism_chirho.review_chirho((two_chirho, nil_chirho), &mut store_chirho);
        assert_eq!(new_cons_chirho, cons2_chirho); // Hash consing
    }

    #[test]
    fn test_int_prism_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let prism_chirho = IntPrismChirho;

        let n_chirho = store_chirho.int_chirho(42);
        assert_eq!(prism_chirho.preview_chirho(&n_chirho, &store_chirho), Some(42));

        let nil_chirho = store_chirho.nil_chirho();
        assert_eq!(prism_chirho.preview_chirho(&nil_chirho, &store_chirho), None);
    }

    #[test]
    fn test_subterm_traversal_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let traversal_chirho = SubtermTraversalChirho;

        // Create [1, 2]
        let list_chirho = store_chirho.list_ints_chirho(&[1, 2]);

        // Collect all subterms
        let subterms_chirho = traversal_chirho.to_list_chirho(list_chirho, &store_chirho);
        // Should include: root cons, 1, inner cons, 2, nil
        assert!(subterms_chirho.len() >= 5);
    }

    #[test]
    fn test_vars_traversal_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let traversal_chirho = VarsTraversalChirho;

        // Create [x, 1, y]
        let (x_var_chirho, x_chirho) = store_chirho.fresh_var_chirho();
        let (y_var_chirho, y_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let nil_chirho = store_chirho.nil_chirho();

        let t3_chirho = store_chirho.cons_chirho(y_chirho, nil_chirho);
        let t2_chirho = store_chirho.cons_chirho(one_chirho, t3_chirho);
        let t1_chirho = store_chirho.cons_chirho(x_chirho, t2_chirho);

        let vars_chirho = traversal_chirho.vars_chirho(t1_chirho, &store_chirho);
        assert_eq!(vars_chirho.len(), 2);
        assert!(vars_chirho.contains(&x_var_chirho));
        assert!(vars_chirho.contains(&y_var_chirho));
    }

    #[test]
    fn test_head_tail_lens_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let head_lens_chirho = HeadLensChirho;
        let tail_lens_chirho = TailLensChirho;

        // Create [1, 2]
        let list_chirho = store_chirho.list_ints_chirho(&[1, 2]);

        // Get head
        let head_chirho = head_lens_chirho.get_chirho(&list_chirho, &store_chirho);
        let one_chirho = store_chirho.int_chirho(1);
        assert_eq!(head_chirho, one_chirho);

        // Set head to 3
        let three_chirho = store_chirho.int_chirho(3);
        let new_list_chirho = head_lens_chirho.set_chirho(list_chirho, three_chirho, &mut store_chirho);
        let new_head_chirho = head_lens_chirho.get_chirho(&new_list_chirho, &store_chirho);
        assert_eq!(new_head_chirho, three_chirho);

        // Tail should be [2]
        let tail_chirho = tail_lens_chirho.get_chirho(&new_list_chirho, &store_chirho);
        let expected_tail_chirho = store_chirho.list_ints_chirho(&[2]);
        assert_eq!(tail_chirho, expected_tail_chirho);
    }

    #[test]
    fn test_walk_deep_optic_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstChirho::new();

        // x = 1, y = 2
        let (x_var_chirho, x_chirho) = store_chirho.fresh_var_chirho();
        let (y_var_chirho, y_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        subst_chirho.bind_chirho(x_var_chirho, one_chirho);
        subst_chirho.bind_chirho(y_var_chirho, two_chirho);

        // Create [x, y]
        let nil_chirho = store_chirho.nil_chirho();
        let t2_chirho = store_chirho.cons_chirho(y_chirho, nil_chirho);
        let t1_chirho = store_chirho.cons_chirho(x_chirho, t2_chirho);

        // Walk should give [1, 2]
        let walked_chirho = walk_deep_optic_chirho(t1_chirho, &mut subst_chirho, &mut store_chirho);
        let expected_chirho = store_chirho.list_ints_chirho(&[1, 2]);
        assert_eq!(walked_chirho, expected_chirho);
    }
}
