//! E-Graph Integration via egg ☧
//!
//! Connects miniKanren terms to egg's e-graphs for:
//! - Equality saturation (finding all equivalent forms)
//! - Efficient equivalence queries
//! - Term rewriting with semantic equality
//!
//! Key insight: Unification is just adding to the e-graph and checking
//! if two e-classes can be merged without conflict.

use egg::{define_language, rewrite, Analysis, DidMerge, EGraph, Id, Rewrite, Runner};

// Define our term language for egg
// Note: egg variables (patterns like ?x) are different from miniKanren logic variables.
// We represent miniKanren variables as symbols with a special prefix.
define_language! {
    /// miniKanren term language for egg ☧
    pub enum TermLangChirho {
        // Atoms
        "nil" = NilChirho,
        Int(i64),
        // Symbols (including logic variables as "_v0", "_v1", etc.)
        Sym(egg::Symbol),

        // Compound terms
        "cons" = ConsChirho([Id; 2]),  // (cons head tail)

        // For application/function calls
        "app" = AppChirho([Id; 2]),  // (app func arg)

        // For list syntax sugar
        "list" = ListChirho(Box<[Id]>),  // (list e1 e2 ... en)
    }
}

/// Analysis for tracking ground-ness and canonical forms
#[derive(Debug, Clone, Default)]
pub struct TermAnalysisChirho;

/// Data computed for each e-class
#[derive(Debug, Clone)]
pub struct TermDataChirho {
    /// Is this e-class definitely ground (no variables)?
    pub is_ground_chirho: bool,
    /// Constant value if this is a singleton constant
    pub constant_chirho: Option<i64>,
}

impl Default for TermDataChirho {
    fn default() -> Self {
        Self {
            is_ground_chirho: false,
            constant_chirho: None,
        }
    }
}

impl Analysis<TermLangChirho> for TermAnalysisChirho {
    type Data = TermDataChirho;

    fn make(egraph_chirho: &EGraph<TermLangChirho, Self>, enode_chirho: &TermLangChirho) -> Self::Data {
        match enode_chirho {
            TermLangChirho::NilChirho => TermDataChirho {
                is_ground_chirho: true,
                constant_chirho: None,
            },
            TermLangChirho::Int(n_chirho) => TermDataChirho {
                is_ground_chirho: true,
                constant_chirho: Some(*n_chirho),
            },
            TermLangChirho::Sym(s_chirho) => {
                // Logic variables have names starting with "_v"
                let name_chirho = s_chirho.as_str();
                let is_var_chirho = name_chirho.starts_with("_v");
                TermDataChirho {
                    is_ground_chirho: !is_var_chirho,
                    constant_chirho: None,
                }
            },
            TermLangChirho::ConsChirho([h_chirho, t_chirho]) => {
                let h_data_chirho = &egraph_chirho[*h_chirho].data;
                let t_data_chirho = &egraph_chirho[*t_chirho].data;
                TermDataChirho {
                    is_ground_chirho: h_data_chirho.is_ground_chirho && t_data_chirho.is_ground_chirho,
                    constant_chirho: None,
                }
            }
            TermLangChirho::AppChirho([f_chirho, a_chirho]) => {
                let f_data_chirho = &egraph_chirho[*f_chirho].data;
                let a_data_chirho = &egraph_chirho[*a_chirho].data;
                TermDataChirho {
                    is_ground_chirho: f_data_chirho.is_ground_chirho && a_data_chirho.is_ground_chirho,
                    constant_chirho: None,
                }
            }
            TermLangChirho::ListChirho(elems_chirho) => {
                let all_ground_chirho = elems_chirho
                    .iter()
                    .all(|id_chirho| egraph_chirho[*id_chirho].data.is_ground_chirho);
                TermDataChirho {
                    is_ground_chirho: all_ground_chirho,
                    constant_chirho: None,
                }
            }
        }
    }

    fn merge(&mut self, a_chirho: &mut Self::Data, b_chirho: Self::Data) -> DidMerge {
        // Merge analysis data when e-classes are merged
        let did_change_chirho = a_chirho.is_ground_chirho != (a_chirho.is_ground_chirho || b_chirho.is_ground_chirho);
        a_chirho.is_ground_chirho = a_chirho.is_ground_chirho || b_chirho.is_ground_chirho;

        if a_chirho.constant_chirho.is_none() && b_chirho.constant_chirho.is_some() {
            a_chirho.constant_chirho = b_chirho.constant_chirho;
            DidMerge(true, true)
        } else {
            DidMerge(did_change_chirho, did_change_chirho)
        }
    }
}

/// E-graph based unification store
pub struct EggStoreChirho {
    pub egraph_chirho: EGraph<TermLangChirho, TermAnalysisChirho>,
    next_var_chirho: u32,
}

impl EggStoreChirho {
    pub fn new_chirho() -> Self {
        Self {
            egraph_chirho: EGraph::new(TermAnalysisChirho),
            next_var_chirho: 0,
        }
    }

    /// Create fresh logic variable (represented as symbol "_vN")
    pub fn fresh_var_chirho(&mut self) -> Id {
        let name_chirho = format!("_v{}", self.next_var_chirho);
        self.next_var_chirho += 1;
        let var_chirho = TermLangChirho::Sym(egg::Symbol::from(name_chirho));
        self.egraph_chirho.add(var_chirho)
    }

    /// Add integer constant
    pub fn int_chirho(&mut self, n_chirho: i64) -> Id {
        self.egraph_chirho.add(TermLangChirho::Int(n_chirho))
    }

    /// Add nil (empty list)
    pub fn nil_chirho(&mut self) -> Id {
        self.egraph_chirho.add(TermLangChirho::NilChirho)
    }

    /// Add symbol
    pub fn sym_chirho(&mut self, name_chirho: &str) -> Id {
        self.egraph_chirho.add(TermLangChirho::Sym(egg::Symbol::from(name_chirho)))
    }

    /// Add cons cell
    pub fn cons_chirho(&mut self, head_chirho: Id, tail_chirho: Id) -> Id {
        self.egraph_chirho.add(TermLangChirho::ConsChirho([head_chirho, tail_chirho]))
    }

    /// Build list from IDs
    pub fn list_chirho(&mut self, elems_chirho: &[Id]) -> Id {
        let mut result_chirho = self.nil_chirho();
        for &elem_chirho in elems_chirho.iter().rev() {
            result_chirho = self.cons_chirho(elem_chirho, result_chirho);
        }
        result_chirho
    }

    /// Build list from integers
    pub fn list_ints_chirho(&mut self, vals_chirho: &[i64]) -> Id {
        let ids_chirho: Vec<_> = vals_chirho.iter().map(|&v_chirho| self.int_chirho(v_chirho)).collect();
        self.list_chirho(&ids_chirho)
    }

    /// Unify two terms (merge their e-classes)
    /// Returns true if successful, false if conflict detected
    pub fn unify_chirho(&mut self, a_chirho: Id, b_chirho: Id) -> bool {
        let a_root_chirho = self.egraph_chirho.find(a_chirho);
        let b_root_chirho = self.egraph_chirho.find(b_chirho);

        if a_root_chirho == b_root_chirho {
            return true; // Already unified
        }

        // Check for constant conflicts
        let a_const_chirho = self.egraph_chirho[a_root_chirho].data.constant_chirho;
        let b_const_chirho = self.egraph_chirho[b_root_chirho].data.constant_chirho;

        if let (Some(ac_chirho), Some(bc_chirho)) = (a_const_chirho, b_const_chirho) {
            if ac_chirho != bc_chirho {
                return false; // Conflict: different constants
            }
        }

        // Merge the e-classes
        self.egraph_chirho.union(a_chirho, b_chirho);
        true
    }

    /// Check if two terms are equivalent
    pub fn equiv_chirho(&self, a_chirho: Id, b_chirho: Id) -> bool {
        self.egraph_chirho.find(a_chirho) == self.egraph_chirho.find(b_chirho)
    }

    /// Check if term is ground
    pub fn is_ground_chirho(&self, id_chirho: Id) -> bool {
        self.egraph_chirho[self.egraph_chirho.find(id_chirho)].data.is_ground_chirho
    }

    /// Run equality saturation with rewrite rules
    pub fn saturate_chirho(&mut self, rules_chirho: &[Rewrite<TermLangChirho, TermAnalysisChirho>]) {
        let runner_chirho = Runner::default()
            .with_egraph(std::mem::take(&mut self.egraph_chirho))
            .run(rules_chirho);
        self.egraph_chirho = runner_chirho.egraph;
    }

    /// Get number of e-classes
    pub fn num_classes_chirho(&self) -> usize {
        self.egraph_chirho.number_of_classes()
    }

    /// Get number of e-nodes
    pub fn num_nodes_chirho(&self) -> usize {
        self.egraph_chirho.total_size()
    }
}

/// Standard rewrite rules for list operations
pub fn list_rules_chirho() -> Vec<Rewrite<TermLangChirho, TermAnalysisChirho>> {
    vec![
        // (list) = nil
        rewrite!("list-empty-chirho"; "(list)" => "nil"),

        // (list a) = (cons a nil)
        rewrite!("list-single-chirho"; "(list ?a)" => "(cons ?a nil)"),

        // List associativity patterns could go here
    ]
}

/// Arithmetic rewrite rules (for expression synthesis)
pub fn arith_rules_chirho() -> Vec<Rewrite<TermLangChirho, TermAnalysisChirho>> {
    vec![
        // Commutativity
        rewrite!("add-comm-chirho"; "(app (app add ?a) ?b)" => "(app (app add ?b) ?a)"),
        rewrite!("mul-comm-chirho"; "(app (app mul ?a) ?b)" => "(app (app mul ?b) ?a)"),

        // Identity
        rewrite!("add-zero-chirho"; "(app (app add ?a) 0)" => "?a"),
        rewrite!("mul-one-chirho"; "(app (app mul ?a) 1)" => "?a"),
        rewrite!("mul-zero-chirho"; "(app (app mul ?a) 0)" => "0"),
    ]
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_egg_basic_chirho() {
        let mut store_chirho = EggStoreChirho::new_chirho();

        let x_chirho = store_chirho.fresh_var_chirho();
        let y_chirho = store_chirho.fresh_var_chirho();
        let forty_two_chirho = store_chirho.int_chirho(42);

        // x and y are different initially
        assert!(!store_chirho.equiv_chirho(x_chirho, y_chirho));

        // Unify x = y
        assert!(store_chirho.unify_chirho(x_chirho, y_chirho));
        assert!(store_chirho.equiv_chirho(x_chirho, y_chirho));

        // Unify x = 42
        assert!(store_chirho.unify_chirho(x_chirho, forty_two_chirho));

        // Now y should also equal 42
        assert!(store_chirho.equiv_chirho(y_chirho, forty_two_chirho));
    }

    #[test]
    fn test_egg_conflict_chirho() {
        let mut store_chirho = EggStoreChirho::new_chirho();

        let x_chirho = store_chirho.fresh_var_chirho();
        let one_chirho = store_chirho.int_chirho(1);
        let two_chirho = store_chirho.int_chirho(2);

        // Unify x = 1
        assert!(store_chirho.unify_chirho(x_chirho, one_chirho));

        // Try x = 2 - should fail
        assert!(!store_chirho.unify_chirho(x_chirho, two_chirho));
    }

    #[test]
    fn test_egg_list_chirho() {
        let mut store_chirho = EggStoreChirho::new_chirho();

        let list1_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);
        let list2_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);

        // Same structure should unify
        assert!(store_chirho.unify_chirho(list1_chirho, list2_chirho));
        assert!(store_chirho.equiv_chirho(list1_chirho, list2_chirho));
    }

    #[test]
    fn test_egg_ground_chirho() {
        let mut store_chirho = EggStoreChirho::new_chirho();

        let x_chirho = store_chirho.fresh_var_chirho();
        let forty_two_chirho = store_chirho.int_chirho(42);
        let list_chirho = store_chirho.list_ints_chirho(&[1, 2]);

        assert!(!store_chirho.is_ground_chirho(x_chirho));
        assert!(store_chirho.is_ground_chirho(forty_two_chirho));
        assert!(store_chirho.is_ground_chirho(list_chirho));
    }

    #[test]
    fn test_egg_saturation_chirho() {
        let mut store_chirho = EggStoreChirho::new_chirho();

        // Build: (app (app add x) 0)
        let add_sym_chirho = store_chirho.sym_chirho("add");
        let x_chirho = store_chirho.fresh_var_chirho();
        let zero_chirho = store_chirho.int_chirho(0);

        let add_x_chirho = store_chirho.egraph_chirho.add(TermLangChirho::AppChirho([add_sym_chirho, x_chirho]));
        let add_x_0_chirho = store_chirho.egraph_chirho.add(TermLangChirho::AppChirho([add_x_chirho, zero_chirho]));

        // Before saturation
        assert!(!store_chirho.equiv_chirho(add_x_0_chirho, x_chirho));

        // Saturate with arithmetic rules
        store_chirho.saturate_chirho(&arith_rules_chirho());

        // After saturation: (add x 0) should equal x
        assert!(store_chirho.equiv_chirho(add_x_0_chirho, x_chirho));
    }
}
