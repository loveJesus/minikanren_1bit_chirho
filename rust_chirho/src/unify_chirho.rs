//! Unification ☧
//!
//! Core unification algorithm with occurs check.

use crate::terms_chirho::{TermChirho, TermIdChirho, TermStoreChirho, VarIdChirho};
use crate::union_find_chirho::UnionFindChirho;
use std::collections::HashMap;

/// Substitution: variable → term mapping via union-find
#[derive(Debug, Clone, Default)]
pub struct SubstChirho {
    /// Union-find for variable equivalence
    pub uf_chirho: UnionFindChirho,
    /// Canonical term for each class (root → term_id)
    pub canonical_chirho: HashMap<u32, TermIdChirho>,
}

impl SubstChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Walk a term through the substitution
    pub fn walk_chirho(&mut self, term_id_chirho: TermIdChirho, store_chirho: &TermStoreChirho) -> TermIdChirho {
        match store_chirho.get_chirho(term_id_chirho) {
            Some(TermChirho::VarChirho(var_id_chirho)) => {
                let root_chirho = self.uf_chirho.find_chirho(*var_id_chirho);
                if let Some(&canonical_chirho) = self.canonical_chirho.get(&root_chirho) {
                    if canonical_chirho != term_id_chirho {
                        return self.walk_chirho(canonical_chirho, store_chirho);
                    }
                }
                term_id_chirho
            }
            _ => term_id_chirho,
        }
    }

    /// Bind variable to term
    pub fn bind_chirho(&mut self, var_id_chirho: VarIdChirho, term_id_chirho: TermIdChirho) {
        let root_chirho = self.uf_chirho.find_chirho(var_id_chirho);
        self.canonical_chirho.insert(root_chirho, term_id_chirho);
    }
}

/// Unification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyResultChirho {
    Success,
    Failure,
    OccursCheckFailed,
}

/// Unify two terms
pub fn unify_chirho(
    t1_chirho: TermIdChirho,
    t2_chirho: TermIdChirho,
    subst_chirho: &mut SubstChirho,
    store_chirho: &TermStoreChirho,
) -> UnifyResultChirho {
    let t1_walked_chirho = subst_chirho.walk_chirho(t1_chirho, store_chirho);
    let t2_walked_chirho = subst_chirho.walk_chirho(t2_chirho, store_chirho);

    if t1_walked_chirho == t2_walked_chirho {
        return UnifyResultChirho::Success;
    }

    let term1_chirho = store_chirho.get_chirho(t1_walked_chirho);
    let term2_chirho = store_chirho.get_chirho(t2_walked_chirho);

    match (term1_chirho, term2_chirho) {
        // Variable cases
        (Some(TermChirho::VarChirho(v1)), Some(TermChirho::VarChirho(v2))) => {
            subst_chirho.uf_chirho.union_chirho(*v1, *v2);
            UnifyResultChirho::Success
        }
        (Some(TermChirho::VarChirho(v)), _) => {
            if occurs_chirho(*v, t2_walked_chirho, subst_chirho, store_chirho) {
                return UnifyResultChirho::OccursCheckFailed;
            }
            subst_chirho.bind_chirho(*v, t2_walked_chirho);
            UnifyResultChirho::Success
        }
        (_, Some(TermChirho::VarChirho(v))) => {
            if occurs_chirho(*v, t1_walked_chirho, subst_chirho, store_chirho) {
                return UnifyResultChirho::OccursCheckFailed;
            }
            subst_chirho.bind_chirho(*v, t1_walked_chirho);
            UnifyResultChirho::Success
        }

        // Structural cases
        (Some(TermChirho::IntChirho(a)), Some(TermChirho::IntChirho(b))) => {
            if a == b {
                UnifyResultChirho::Success
            } else {
                UnifyResultChirho::Failure
            }
        }
        (Some(TermChirho::NilChirho), Some(TermChirho::NilChirho)) => {
            UnifyResultChirho::Success
        }
        (Some(TermChirho::ConsChirho(h1, t1)), Some(TermChirho::ConsChirho(h2, t2))) => {
            let h1_chirho = *h1;
            let t1_chirho = *t1;
            let h2_chirho = *h2;
            let t2_chirho = *t2;

            match unify_chirho(h1_chirho, h2_chirho, subst_chirho, store_chirho) {
                UnifyResultChirho::Success => {
                    unify_chirho(t1_chirho, t2_chirho, subst_chirho, store_chirho)
                }
                other_chirho => other_chirho,
            }
        }
        (Some(TermChirho::SymChirho(a)), Some(TermChirho::SymChirho(b))) => {
            if a == b {
                UnifyResultChirho::Success
            } else {
                UnifyResultChirho::Failure
            }
        }

        // Type mismatch
        _ => UnifyResultChirho::Failure,
    }
}

/// Occurs check: does variable appear in term?
fn occurs_chirho(
    var_chirho: VarIdChirho,
    term_id_chirho: TermIdChirho,
    subst_chirho: &mut SubstChirho,
    store_chirho: &TermStoreChirho,
) -> bool {
    let walked_chirho = subst_chirho.walk_chirho(term_id_chirho, store_chirho);

    match store_chirho.get_chirho(walked_chirho) {
        Some(TermChirho::VarChirho(v)) => {
            subst_chirho.uf_chirho.same_class_chirho(var_chirho, *v)
        }
        Some(TermChirho::ConsChirho(h, t)) => {
            occurs_chirho(var_chirho, *h, subst_chirho, store_chirho)
                || occurs_chirho(var_chirho, *t, subst_chirho, store_chirho)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_unify_ints_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstChirho::new();

        let a_chirho = store_chirho.int_chirho(42);
        let b_chirho = store_chirho.int_chirho(42);
        let c_chirho = store_chirho.int_chirho(43);

        assert_eq!(
            unify_chirho(a_chirho, b_chirho, &mut subst_chirho, &store_chirho),
            UnifyResultChirho::Success
        );
        assert_eq!(
            unify_chirho(a_chirho, c_chirho, &mut subst_chirho, &store_chirho),
            UnifyResultChirho::Failure
        );
    }

    #[test]
    fn test_unify_var_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstChirho::new();

        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let val_chirho = store_chirho.int_chirho(42);

        assert_eq!(
            unify_chirho(x_chirho, val_chirho, &mut subst_chirho, &store_chirho),
            UnifyResultChirho::Success
        );

        let walked_chirho = subst_chirho.walk_chirho(x_chirho, &store_chirho);
        assert_eq!(walked_chirho, val_chirho);
    }

    #[test]
    fn test_unify_lists_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstChirho::new();

        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        // [x, 2]
        let nil_chirho = store_chirho.nil_chirho();
        let list1_chirho = store_chirho.cons_chirho(two_chirho, nil_chirho);
        let list1_chirho = store_chirho.cons_chirho(x_chirho, list1_chirho);

        // [1, 2]
        let list2_chirho = store_chirho.list_ints_chirho(&[1, 2]);

        assert_eq!(
            unify_chirho(list1_chirho, list2_chirho, &mut subst_chirho, &store_chirho),
            UnifyResultChirho::Success
        );

        let x_val_chirho = subst_chirho.walk_chirho(x_chirho, &store_chirho);
        assert_eq!(x_val_chirho, one_chirho);
    }

    #[test]
    fn test_occurs_check_chirho() {
        let mut store_chirho = TermStoreChirho::new();
        let mut subst_chirho = SubstChirho::new();

        let (_, x_chirho) = store_chirho.fresh_var_chirho();
        let nil_chirho = store_chirho.nil_chirho();

        // [x | nil] = cons(x, nil)
        let _cyclic_chirho = store_chirho.cons_chirho(x_chirho, nil_chirho);

        // x = [x | nil] should fail occurs check
        // Actually this would be x = cons(x, nil), so x appears in RHS
        let bad_list_chirho = store_chirho.cons_chirho(x_chirho, nil_chirho);

        // Unifying x with a list containing x
        let result_chirho = unify_chirho(x_chirho, bad_list_chirho, &mut subst_chirho, &store_chirho);
        assert_eq!(result_chirho, UnifyResultChirho::OccursCheckFailed);
    }
}
