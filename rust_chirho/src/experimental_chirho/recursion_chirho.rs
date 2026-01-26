// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Recursion Schemes ☧
//!
//! Kmett-style algebraic approach to term traversal.
//! Define structure once, get fold/unfold/hylo for free.
//!
//! Core idea: Separate recursion from the "one layer" functor.
//! TermF<A> is one layer, Fix<TermF> is the recursive structure.


/// One layer of term structure (the "base functor")
/// Parametric in the recursive position A
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TermFChirho<A> {
    /// Logic variable (leaf)
    VarFChirho(u32),
    /// Integer constant (leaf)
    IntFChirho(i64),
    /// Nil (leaf)
    NilFChirho,
    /// Cons cell with recursive positions
    ConsFChirho(A, A),
    /// Symbol (leaf)
    SymFChirho(u32),
    /// Application with recursive positions
    AppFChirho(A, A),
}

impl<A> TermFChirho<A> {
    /// Functor map: apply f to all recursive positions
    pub fn map_chirho<B, F>(self, mut f_chirho: F) -> TermFChirho<B>
    where
        F: FnMut(A) -> B,
    {
        match self {
            TermFChirho::VarFChirho(v_chirho) => TermFChirho::VarFChirho(v_chirho),
            TermFChirho::IntFChirho(n_chirho) => TermFChirho::IntFChirho(n_chirho),
            TermFChirho::NilFChirho => TermFChirho::NilFChirho,
            TermFChirho::ConsFChirho(h_chirho, t_chirho) => {
                TermFChirho::ConsFChirho(f_chirho(h_chirho), f_chirho(t_chirho))
            }
            TermFChirho::SymFChirho(s_chirho) => TermFChirho::SymFChirho(s_chirho),
            TermFChirho::AppFChirho(f_val_chirho, a_chirho) => {
                TermFChirho::AppFChirho(f_chirho(f_val_chirho), f_chirho(a_chirho))
            }
        }
    }

    /// Traverse with fallible function
    pub fn traverse_chirho<B, E, F>(self, mut f_chirho: F) -> Result<TermFChirho<B>, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        match self {
            TermFChirho::VarFChirho(v_chirho) => Ok(TermFChirho::VarFChirho(v_chirho)),
            TermFChirho::IntFChirho(n_chirho) => Ok(TermFChirho::IntFChirho(n_chirho)),
            TermFChirho::NilFChirho => Ok(TermFChirho::NilFChirho),
            TermFChirho::ConsFChirho(h_chirho, t_chirho) => {
                Ok(TermFChirho::ConsFChirho(f_chirho(h_chirho)?, f_chirho(t_chirho)?))
            }
            TermFChirho::SymFChirho(s_chirho) => Ok(TermFChirho::SymFChirho(s_chirho)),
            TermFChirho::AppFChirho(f_val_chirho, a_chirho) => {
                Ok(TermFChirho::AppFChirho(f_chirho(f_val_chirho)?, f_chirho(a_chirho)?))
            }
        }
    }

    /// Check if this is a leaf (no recursive positions)
    pub fn is_leaf_chirho(&self) -> bool {
        matches!(
            self,
            TermFChirho::VarFChirho(_) | TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_)
        )
    }

    /// Get children (recursive positions)
    pub fn children_chirho(&self) -> Vec<&A> {
        match self {
            TermFChirho::VarFChirho(_) | TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => {
                vec![]
            }
            TermFChirho::ConsFChirho(h_chirho, t_chirho) => vec![h_chirho, t_chirho],
            TermFChirho::AppFChirho(f_chirho, a_chirho) => vec![f_chirho, a_chirho],
        }
    }
}

/// Fixed point of a functor: Fix<F> = F<Fix<F>>
/// This is the recursive term type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixChirho<F> {
    pub unfix_chirho: Box<F>,
}

/// Recursive term = Fix<TermF<_>>
pub type TermRecChirho = FixChirho<TermFChirho<FixChirho<TermFChirho<()>>>>;

/// Simpler: just use TermFChirho<TermIdChirho> with a store
/// This is the practical approach used in the rest of the codebase

/// Algebra: F<A> -> A
/// Used by catamorphism (fold)
pub trait AlgebraChirho<F, A> {
    fn apply_chirho(&self, layer_chirho: F) -> A;
}

/// Coalgebra: A -> F<A>
/// Used by anamorphism (unfold)
pub trait CoalgebraChirho<F, A> {
    fn apply_chirho(&self, seed_chirho: A) -> F;
}

/// Catamorphism (generalized fold)
/// Given an algebra (TermF<A> -> A), fold a term to A
///
/// Note: This is a placeholder for the proper Fix-based implementation.
/// Use `cata_indexed_chirho` for the practical indexed version.
pub fn cata_chirho<A, F>(
    _alg_chirho: F,
    _term_chirho: &TermFChirho<Box<TermFChirho<()>>>,
) -> A
where
    F: Fn(TermFChirho<A>) -> A + Copy,
{
    // For a real implementation, we'd need proper recursion
    // This is a simplified version for the indexed representation
    todo!("Full cata requires proper Fix type - use cata_indexed_chirho instead")
}

/// Practical catamorphism over indexed terms
/// Uses a term store for indirection
pub fn cata_indexed_chirho<A, F>(
    store_chirho: &TermStoreIndexedChirho,
    alg_chirho: F,
    term_id_chirho: u32,
) -> A
where
    F: Fn(TermFChirho<A>) -> A,
    A: Clone,
{
    let mut cache_chirho: std::collections::HashMap<u32, A> = std::collections::HashMap::new();
    cata_indexed_impl_chirho(store_chirho, &alg_chirho, term_id_chirho, &mut cache_chirho)
}

fn cata_indexed_impl_chirho<A, F>(
    store_chirho: &TermStoreIndexedChirho,
    alg_chirho: &F,
    term_id_chirho: u32,
    cache_chirho: &mut std::collections::HashMap<u32, A>,
) -> A
where
    F: Fn(TermFChirho<A>) -> A,
    A: Clone,
{
    if let Some(cached_chirho) = cache_chirho.get(&term_id_chirho) {
        return cached_chirho.clone();
    }

    let layer_chirho = store_chirho.get_layer_chirho(term_id_chirho);
    let mapped_chirho = layer_chirho.map_chirho(|child_id_chirho| {
        cata_indexed_impl_chirho(store_chirho, alg_chirho, child_id_chirho, cache_chirho)
    });

    let result_chirho = alg_chirho(mapped_chirho);
    cache_chirho.insert(term_id_chirho, result_chirho.clone());
    result_chirho
}

/// Indexed term store for recursion schemes
#[derive(Debug, Default)]
pub struct TermStoreIndexedChirho {
    terms_chirho: Vec<TermFChirho<u32>>,
}

impl TermStoreIndexedChirho {
    pub fn new_chirho() -> Self {
        Self::default()
    }

    pub fn intern_chirho(&mut self, layer_chirho: TermFChirho<u32>) -> u32 {
        let id_chirho = self.terms_chirho.len() as u32;
        self.terms_chirho.push(layer_chirho);
        id_chirho
    }

    pub fn get_layer_chirho(&self, id_chirho: u32) -> TermFChirho<u32> {
        self.terms_chirho[id_chirho as usize].clone()
    }

    // Convenience constructors
    pub fn var_chirho(&mut self, v_chirho: u32) -> u32 {
        self.intern_chirho(TermFChirho::VarFChirho(v_chirho))
    }

    pub fn int_chirho(&mut self, n_chirho: i64) -> u32 {
        self.intern_chirho(TermFChirho::IntFChirho(n_chirho))
    }

    pub fn nil_chirho(&mut self) -> u32 {
        self.intern_chirho(TermFChirho::NilFChirho)
    }

    pub fn cons_chirho(&mut self, h_chirho: u32, t_chirho: u32) -> u32 {
        self.intern_chirho(TermFChirho::ConsFChirho(h_chirho, t_chirho))
    }

    pub fn sym_chirho(&mut self, s_chirho: u32) -> u32 {
        self.intern_chirho(TermFChirho::SymFChirho(s_chirho))
    }

    pub fn app_chirho(&mut self, f_chirho: u32, a_chirho: u32) -> u32 {
        self.intern_chirho(TermFChirho::AppFChirho(f_chirho, a_chirho))
    }
}

// === Standard algebras ===

/// Check if term is ground (no variables)
pub fn is_ground_algebra_chirho(layer_chirho: TermFChirho<bool>) -> bool {
    match layer_chirho {
        TermFChirho::VarFChirho(_) => false,
        TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => true,
        TermFChirho::ConsFChirho(h_chirho, t_chirho) => h_chirho && t_chirho,
        TermFChirho::AppFChirho(f_chirho, a_chirho) => f_chirho && a_chirho,
    }
}

/// Collect all variable IDs
pub fn vars_algebra_chirho(layer_chirho: TermFChirho<std::collections::HashSet<u32>>) -> std::collections::HashSet<u32> {
    match layer_chirho {
        TermFChirho::VarFChirho(v_chirho) => {
            let mut set_chirho = std::collections::HashSet::new();
            set_chirho.insert(v_chirho);
            set_chirho
        }
        TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => {
            std::collections::HashSet::new()
        }
        TermFChirho::ConsFChirho(h_chirho, t_chirho) => {
            h_chirho.union(&t_chirho).cloned().collect()
        }
        TermFChirho::AppFChirho(f_chirho, a_chirho) => {
            f_chirho.union(&a_chirho).cloned().collect()
        }
    }
}

/// Count term size (number of nodes)
pub fn size_algebra_chirho(layer_chirho: TermFChirho<usize>) -> usize {
    match layer_chirho {
        TermFChirho::VarFChirho(_) | TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => 1,
        TermFChirho::ConsFChirho(h_chirho, t_chirho) => 1 + h_chirho + t_chirho,
        TermFChirho::AppFChirho(f_chirho, a_chirho) => 1 + f_chirho + a_chirho,
    }
}

/// Compute depth
pub fn depth_algebra_chirho(layer_chirho: TermFChirho<usize>) -> usize {
    match layer_chirho {
        TermFChirho::VarFChirho(_) | TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => 0,
        TermFChirho::ConsFChirho(h_chirho, t_chirho) => 1 + h_chirho.max(t_chirho),
        TermFChirho::AppFChirho(f_chirho, a_chirho) => 1 + f_chirho.max(a_chirho),
    }
}

/// Occurs check algebra: returns (contains_var, is_var_at_root)
/// If we're checking var V, we pass V as context
pub struct OccursCheckAlgebraChirho {
    pub target_var_chirho: u32,
}

impl OccursCheckAlgebraChirho {
    pub fn check_chirho(&self, layer_chirho: TermFChirho<bool>) -> bool {
        match layer_chirho {
            TermFChirho::VarFChirho(v_chirho) => v_chirho == self.target_var_chirho,
            TermFChirho::IntFChirho(_) | TermFChirho::NilFChirho | TermFChirho::SymFChirho(_) => false,
            TermFChirho::ConsFChirho(h_chirho, t_chirho) => h_chirho || t_chirho,
            TermFChirho::AppFChirho(f_chirho, a_chirho) => f_chirho || a_chirho,
        }
    }
}

/// Paramorphism: like catamorphism but also has access to the original subterm
/// Para: F<(A, Fix<F>)> -> A
pub fn para_indexed_chirho<A, F>(
    store_chirho: &TermStoreIndexedChirho,
    alg_chirho: F,
    term_id_chirho: u32,
) -> A
where
    F: Fn(TermFChirho<(A, u32)>) -> A,
    A: Clone,
{
    let mut cache_chirho: std::collections::HashMap<u32, A> = std::collections::HashMap::new();
    para_indexed_impl_chirho(store_chirho, &alg_chirho, term_id_chirho, &mut cache_chirho)
}

fn para_indexed_impl_chirho<A, F>(
    store_chirho: &TermStoreIndexedChirho,
    alg_chirho: &F,
    term_id_chirho: u32,
    cache_chirho: &mut std::collections::HashMap<u32, A>,
) -> A
where
    F: Fn(TermFChirho<(A, u32)>) -> A,
    A: Clone,
{
    if let Some(cached_chirho) = cache_chirho.get(&term_id_chirho) {
        return cached_chirho.clone();
    }

    let layer_chirho = store_chirho.get_layer_chirho(term_id_chirho);
    let mapped_chirho = layer_chirho.map_chirho(|child_id_chirho| {
        let child_result_chirho = para_indexed_impl_chirho(store_chirho, alg_chirho, child_id_chirho, cache_chirho);
        (child_result_chirho, child_id_chirho)
    });

    let result_chirho = alg_chirho(mapped_chirho);
    cache_chirho.insert(term_id_chirho, result_chirho.clone());
    result_chirho
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_term_f_map_chirho() {
        let cons_chirho = TermFChirho::ConsFChirho(1u32, 2u32);
        let mapped_chirho = cons_chirho.map_chirho(|x_chirho| x_chirho * 10);
        assert_eq!(mapped_chirho, TermFChirho::ConsFChirho(10, 20));
    }

    #[test]
    fn test_cata_is_ground_chirho() {
        let mut store_chirho = TermStoreIndexedChirho::new_chirho();

        // Build: cons(1, nil)
        let one_chirho = store_chirho.int_chirho(1);
        let nil_chirho = store_chirho.nil_chirho();
        let list_chirho = store_chirho.cons_chirho(one_chirho, nil_chirho);

        let is_ground_chirho = cata_indexed_chirho(&store_chirho, is_ground_algebra_chirho, list_chirho);
        assert!(is_ground_chirho);

        // Build: cons(x, nil) where x is var
        let var_chirho = store_chirho.var_chirho(0);
        let list_with_var_chirho = store_chirho.cons_chirho(var_chirho, nil_chirho);

        let is_ground2_chirho = cata_indexed_chirho(&store_chirho, is_ground_algebra_chirho, list_with_var_chirho);
        assert!(!is_ground2_chirho);
    }

    #[test]
    fn test_cata_vars_chirho() {
        let mut store_chirho = TermStoreIndexedChirho::new_chirho();

        let v0_chirho = store_chirho.var_chirho(0);
        let v1_chirho = store_chirho.var_chirho(1);
        let nil_chirho = store_chirho.nil_chirho();
        let cons1_chirho = store_chirho.cons_chirho(v1_chirho, nil_chirho);
        let cons0_chirho = store_chirho.cons_chirho(v0_chirho, cons1_chirho);

        let vars_chirho = cata_indexed_chirho(&store_chirho, vars_algebra_chirho, cons0_chirho);
        assert!(vars_chirho.contains(&0));
        assert!(vars_chirho.contains(&1));
        assert_eq!(vars_chirho.len(), 2);
    }

    #[test]
    fn test_cata_size_chirho() {
        let mut store_chirho = TermStoreIndexedChirho::new_chirho();

        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);
        let nil_chirho = store_chirho.nil_chirho();
        let cons1_chirho = store_chirho.cons_chirho(two_chirho, nil_chirho);
        let cons0_chirho = store_chirho.cons_chirho(one_chirho, cons1_chirho);

        // Size: cons(1, cons(2, nil)) = 1 + (1 + 1) + (1 + (1 + 1)) = 5
        // Actually: cons + 1 + (cons + 2 + nil) = 1 + 1 + 1 + 1 + 1 = 5
        let size_chirho = cata_indexed_chirho(&store_chirho, size_algebra_chirho, cons0_chirho);
        assert_eq!(size_chirho, 5);
    }

    #[test]
    fn test_occurs_check_algebraic_chirho() {
        let mut store_chirho = TermStoreIndexedChirho::new_chirho();

        // cons(x, cons(y, nil)) - does it contain x (var 0)?
        let v0_chirho = store_chirho.var_chirho(0);
        let v1_chirho = store_chirho.var_chirho(1);
        let nil_chirho = store_chirho.nil_chirho();
        let inner_chirho = store_chirho.cons_chirho(v1_chirho, nil_chirho);
        let outer_chirho = store_chirho.cons_chirho(v0_chirho, inner_chirho);

        let occurs_x_chirho = OccursCheckAlgebraChirho { target_var_chirho: 0 };
        let contains_x_chirho = cata_indexed_chirho(&store_chirho, |l_chirho| occurs_x_chirho.check_chirho(l_chirho), outer_chirho);
        assert!(contains_x_chirho);

        // Does inner contain x?
        let inner_contains_x_chirho = cata_indexed_chirho(&store_chirho, |l_chirho| occurs_x_chirho.check_chirho(l_chirho), inner_chirho);
        assert!(!inner_contains_x_chirho);
    }
}
