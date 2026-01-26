// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Native E-Graph Implementation ☧
//!
//! Hardware-optimized e-graph with full _chirho compliance.
//! Designed for bit-parallel operations and FPGA synthesis.
//!
//! Key differences from egg crate:
//! - Bit-parallel e-class membership (bitmask per node)
//! - Fixed-size structures (no Box<[Id]>)
//! - Integrated with our union-find and term store
//! - SIMD-friendly memory layout
//!
//! Core insight: E-class membership can be a bitmask.
//! If we have N e-classes, each node stores an N-bit vector
//! indicating which classes it belongs to. Merge = OR.

use crate::union_find_chirho::UnionFindChirho;
use crate::hardware_chirho::BitVec64Chirho;
use std::collections::HashMap;

/// E-node identifier (index into nodes array)
pub type ENodeIdChirho = u32;

/// E-class identifier (index into union-find)
pub type EClassIdChirho = u32;

/// Term structure for e-graph nodes ☧
/// Fixed arity (max 2 children) for hardware friendliness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ENodeChirho {
    /// Nil/unit value
    NilChirho,
    /// Integer constant
    IntChirho(i64),
    /// Symbol (interned string ID)
    SymChirho(u32),
    /// Cons cell (head, tail)
    ConsChirho(EClassIdChirho, EClassIdChirho),
    /// Application (func, arg)
    AppChirho(EClassIdChirho, EClassIdChirho),
}

impl ENodeChirho {
    /// Get children of this node (for congruence closure)
    pub fn children_chirho(&self) -> [Option<EClassIdChirho>; 2] {
        match self {
            ENodeChirho::NilChirho | ENodeChirho::IntChirho(_) | ENodeChirho::SymChirho(_) => {
                [None, None]
            }
            ENodeChirho::ConsChirho(h_chirho, t_chirho) => [Some(*h_chirho), Some(*t_chirho)],
            ENodeChirho::AppChirho(f_chirho, a_chirho) => [Some(*f_chirho), Some(*a_chirho)],
        }
    }

    /// Canonicalize node using union-find (map children to roots)
    /// Uses immutable find (no path compression)
    pub fn canonicalize_chirho(&self, uf_chirho: &UnionFindChirho) -> Self {
        match self {
            ENodeChirho::NilChirho => ENodeChirho::NilChirho,
            ENodeChirho::IntChirho(n_chirho) => ENodeChirho::IntChirho(*n_chirho),
            ENodeChirho::SymChirho(s_chirho) => ENodeChirho::SymChirho(*s_chirho),
            ENodeChirho::ConsChirho(h_chirho, t_chirho) => ENodeChirho::ConsChirho(
                uf_chirho.find_immut_chirho(*h_chirho),
                uf_chirho.find_immut_chirho(*t_chirho),
            ),
            ENodeChirho::AppChirho(f_chirho, a_chirho) => ENodeChirho::AppChirho(
                uf_chirho.find_immut_chirho(*f_chirho),
                uf_chirho.find_immut_chirho(*a_chirho),
            ),
        }
    }
}

/// Analysis data for each e-class ☧
#[derive(Debug, Clone, Default)]
pub struct EClassDataChirho {
    /// Is this e-class ground (no variables)?
    pub is_ground_chirho: bool,
    /// Constant value if singleton integer
    pub constant_chirho: Option<i64>,
    /// Domain as bitmask (for constraint integration)
    pub domain_chirho: BitVec64Chirho,
}

/// Native E-Graph ☧
/// Hardware-optimized with bit-parallel operations
pub struct EGraphNativeChirho {
    /// Union-find for e-class equivalence
    uf_chirho: UnionFindChirho,
    /// All e-nodes (indexed by ENodeIdChirho)
    nodes_chirho: Vec<ENodeChirho>,
    /// Map from e-node to its e-class
    node_to_class_chirho: Vec<EClassIdChirho>,
    /// Hash-cons map: canonical node → e-class
    hashcons_chirho: HashMap<ENodeChirho, EClassIdChirho>,
    /// Analysis data per e-class
    class_data_chirho: Vec<EClassDataChirho>,
    /// Parent pointers for congruence closure
    /// parents_chirho[class] = list of (node_id, child_index)
    parents_chirho: Vec<Vec<(ENodeIdChirho, u8)>>,
    /// Pending e-class merges for worklist algorithm
    pending_chirho: Vec<(EClassIdChirho, EClassIdChirho)>,
    /// Symbol table for interning
    symbols_chirho: Vec<String>,
    symbol_map_chirho: HashMap<String, u32>,
    /// Next variable counter
    next_var_chirho: u32,
}

impl EGraphNativeChirho {
    /// Create new empty e-graph
    pub fn new_chirho() -> Self {
        Self {
            uf_chirho: UnionFindChirho::new(),
            nodes_chirho: Vec::new(),
            node_to_class_chirho: Vec::new(),
            hashcons_chirho: HashMap::new(),
            class_data_chirho: Vec::new(),
            parents_chirho: Vec::new(),
            pending_chirho: Vec::new(),
            symbols_chirho: Vec::new(),
            symbol_map_chirho: HashMap::new(),
            next_var_chirho: 0,
        }
    }

    /// Find canonical e-class for a class ID
    pub fn find_chirho(&mut self, class_chirho: EClassIdChirho) -> EClassIdChirho {
        self.uf_chirho.find_chirho(class_chirho)
    }

    /// Find without path compression (for read-only access)
    pub fn find_immut_chirho(&self, class_chirho: EClassIdChirho) -> EClassIdChirho {
        self.uf_chirho.find_immut_chirho(class_chirho)
    }

    /// Allocate a new e-class
    fn alloc_class_chirho(&mut self) -> EClassIdChirho {
        let id_chirho = self.class_data_chirho.len() as EClassIdChirho;
        self.uf_chirho.make_set_chirho(id_chirho);
        self.class_data_chirho.push(EClassDataChirho::default());
        self.parents_chirho.push(Vec::new());
        id_chirho
    }

    /// Add an e-node, returning its e-class (hash-consed)
    pub fn add_chirho(&mut self, node_chirho: ENodeChirho) -> EClassIdChirho {
        // Canonicalize the node first
        let canonical_chirho = node_chirho.canonicalize_chirho(&self.uf_chirho);

        // Check if already exists
        if let Some(&class_chirho) = self.hashcons_chirho.get(&canonical_chirho) {
            return self.find_chirho(class_chirho);
        }

        // Create new e-class
        let class_chirho = self.alloc_class_chirho();
        let node_id_chirho = self.nodes_chirho.len() as ENodeIdChirho;

        self.nodes_chirho.push(canonical_chirho);
        self.node_to_class_chirho.push(class_chirho);
        self.hashcons_chirho.insert(canonical_chirho, class_chirho);

        // Update analysis
        self.update_analysis_chirho(class_chirho, &canonical_chirho);

        // Register parent pointers for congruence
        let children_chirho = canonical_chirho.children_chirho();
        for (idx_chirho, child_opt_chirho) in children_chirho.iter().enumerate() {
            if let Some(child_chirho) = child_opt_chirho {
                let child_root_chirho = self.find_chirho(*child_chirho);
                self.parents_chirho[child_root_chirho as usize]
                    .push((node_id_chirho, idx_chirho as u8));
            }
        }

        class_chirho
    }

    /// Update analysis data for an e-class
    fn update_analysis_chirho(&mut self, class_chirho: EClassIdChirho, node_chirho: &ENodeChirho) {
        // Compute new values first (avoiding overlapping borrows)
        let (is_ground_chirho, constant_chirho) = match node_chirho {
            ENodeChirho::NilChirho => (true, None),
            ENodeChirho::IntChirho(n_chirho) => (true, Some(*n_chirho)),
            ENodeChirho::SymChirho(s_chirho) => {
                // Check if it's a logic variable (_v prefix)
                let name_chirho = &self.symbols_chirho[*s_chirho as usize];
                (!name_chirho.starts_with("_v"), None)
            }
            ENodeChirho::ConsChirho(h_chirho, t_chirho) => {
                let h_root_chirho = self.find_immut_chirho(*h_chirho) as usize;
                let t_root_chirho = self.find_immut_chirho(*t_chirho) as usize;
                let h_ground_chirho = self.class_data_chirho[h_root_chirho].is_ground_chirho;
                let t_ground_chirho = self.class_data_chirho[t_root_chirho].is_ground_chirho;
                (h_ground_chirho && t_ground_chirho, None)
            }
            ENodeChirho::AppChirho(f_chirho, a_chirho) => {
                let f_root_chirho = self.find_immut_chirho(*f_chirho) as usize;
                let a_root_chirho = self.find_immut_chirho(*a_chirho) as usize;
                let f_ground_chirho = self.class_data_chirho[f_root_chirho].is_ground_chirho;
                let a_ground_chirho = self.class_data_chirho[a_root_chirho].is_ground_chirho;
                (f_ground_chirho && a_ground_chirho, None)
            }
        };

        // Now assign
        let data_chirho = &mut self.class_data_chirho[class_chirho as usize];
        data_chirho.is_ground_chirho = is_ground_chirho;
        if constant_chirho.is_some() {
            data_chirho.constant_chirho = constant_chirho;
        }
    }

    /// Merge two e-classes (union)
    /// Returns false if conflict detected
    pub fn union_chirho(
        &mut self,
        a_chirho: EClassIdChirho,
        b_chirho: EClassIdChirho,
    ) -> bool {
        let a_root_chirho = self.find_chirho(a_chirho);
        let b_root_chirho = self.find_chirho(b_chirho);

        if a_root_chirho == b_root_chirho {
            return true; // Already equivalent
        }

        // Check for constant conflict
        let a_const_chirho = self.class_data_chirho[a_root_chirho as usize].constant_chirho;
        let b_const_chirho = self.class_data_chirho[b_root_chirho as usize].constant_chirho;

        if let (Some(ac_chirho), Some(bc_chirho)) = (a_const_chirho, b_const_chirho) {
            if ac_chirho != bc_chirho {
                return false; // Conflict: 1 != 2
            }
        }

        // Perform union
        self.uf_chirho.union_chirho(a_root_chirho, b_root_chirho);
        let new_root_chirho = self.find_chirho(a_root_chirho);

        // Merge analysis data
        let a_data_chirho = self.class_data_chirho[a_root_chirho as usize].clone();
        let b_data_chirho = self.class_data_chirho[b_root_chirho as usize].clone();
        let merged_chirho = &mut self.class_data_chirho[new_root_chirho as usize];

        merged_chirho.is_ground_chirho = a_data_chirho.is_ground_chirho || b_data_chirho.is_ground_chirho;
        merged_chirho.constant_chirho = a_data_chirho.constant_chirho.or(b_data_chirho.constant_chirho);
        merged_chirho.domain_chirho = a_data_chirho.domain_chirho.and_chirho(b_data_chirho.domain_chirho);

        // Queue for congruence closure
        self.pending_chirho.push((a_root_chirho, b_root_chirho));

        true
    }

    /// Run congruence closure (rebuild)
    pub fn rebuild_chirho(&mut self) {
        while let Some((a_chirho, b_chirho)) = self.pending_chirho.pop() {
            self.repair_chirho(a_chirho);
            self.repair_chirho(b_chirho);
        }
    }

    /// Repair an e-class after merge
    fn repair_chirho(&mut self, class_chirho: EClassIdChirho) {
        let root_chirho = self.find_chirho(class_chirho);

        // Re-canonicalize nodes that had this class as child
        // This is where congruence closure happens
        let parents_copy_chirho = std::mem::take(&mut self.parents_chirho[class_chirho as usize]);

        for (node_id_chirho, _child_idx_chirho) in parents_copy_chirho {
            let old_node_chirho = self.nodes_chirho[node_id_chirho as usize];
            let new_node_chirho = old_node_chirho.canonicalize_chirho(&self.uf_chirho);

            if old_node_chirho != new_node_chirho {
                // Node changed, may need to merge e-classes
                let old_class_chirho = self.node_to_class_chirho[node_id_chirho as usize];

                if let Some(&existing_class_chirho) = self.hashcons_chirho.get(&new_node_chirho) {
                    // Merge with existing
                    self.union_chirho(old_class_chirho, existing_class_chirho);
                } else {
                    // Update hash-cons
                    self.hashcons_chirho.remove(&old_node_chirho);
                    self.hashcons_chirho.insert(new_node_chirho, old_class_chirho);
                    self.nodes_chirho[node_id_chirho as usize] = new_node_chirho;
                }
            }
        }

        // Re-register parents at the root
        if root_chirho != class_chirho {
            let old_parents_chirho = std::mem::take(&mut self.parents_chirho[class_chirho as usize]);
            self.parents_chirho[root_chirho as usize].extend(old_parents_chirho);
        }
    }

    /// Check if two e-classes are equivalent
    pub fn equiv_chirho(&self, a_chirho: EClassIdChirho, b_chirho: EClassIdChirho) -> bool {
        self.find_immut_chirho(a_chirho) == self.find_immut_chirho(b_chirho)
    }

    /// Unify two e-classes (union + rebuild)
    /// Returns false if conflict
    pub fn unify_chirho(
        &mut self,
        a_chirho: EClassIdChirho,
        b_chirho: EClassIdChirho,
    ) -> bool {
        if !self.union_chirho(a_chirho, b_chirho) {
            return false;
        }
        self.rebuild_chirho();
        true
    }

    // === Convenience constructors ===

    /// Intern a symbol
    pub fn intern_symbol_chirho(&mut self, name_chirho: &str) -> u32 {
        if let Some(&id_chirho) = self.symbol_map_chirho.get(name_chirho) {
            id_chirho
        } else {
            let id_chirho = self.symbols_chirho.len() as u32;
            self.symbols_chirho.push(name_chirho.to_string());
            self.symbol_map_chirho.insert(name_chirho.to_string(), id_chirho);
            id_chirho
        }
    }

    /// Create fresh logic variable
    pub fn fresh_var_chirho(&mut self) -> EClassIdChirho {
        let name_chirho = format!("_v{}", self.next_var_chirho);
        self.next_var_chirho += 1;
        let sym_id_chirho = self.intern_symbol_chirho(&name_chirho);
        self.add_chirho(ENodeChirho::SymChirho(sym_id_chirho))
    }

    /// Add integer constant
    pub fn int_chirho(&mut self, n_chirho: i64) -> EClassIdChirho {
        self.add_chirho(ENodeChirho::IntChirho(n_chirho))
    }

    /// Add nil
    pub fn nil_chirho(&mut self) -> EClassIdChirho {
        self.add_chirho(ENodeChirho::NilChirho)
    }

    /// Add symbol
    pub fn sym_chirho(&mut self, name_chirho: &str) -> EClassIdChirho {
        let sym_id_chirho = self.intern_symbol_chirho(name_chirho);
        self.add_chirho(ENodeChirho::SymChirho(sym_id_chirho))
    }

    /// Add cons cell
    pub fn cons_chirho(
        &mut self,
        head_chirho: EClassIdChirho,
        tail_chirho: EClassIdChirho,
    ) -> EClassIdChirho {
        self.add_chirho(ENodeChirho::ConsChirho(head_chirho, tail_chirho))
    }

    /// Build list from e-class IDs
    pub fn list_chirho(&mut self, elems_chirho: &[EClassIdChirho]) -> EClassIdChirho {
        let mut result_chirho = self.nil_chirho();
        for &elem_chirho in elems_chirho.iter().rev() {
            result_chirho = self.cons_chirho(elem_chirho, result_chirho);
        }
        result_chirho
    }

    /// Build list from integers
    pub fn list_ints_chirho(&mut self, vals_chirho: &[i64]) -> EClassIdChirho {
        let ids_chirho: Vec<_> = vals_chirho
            .iter()
            .map(|&v_chirho| self.int_chirho(v_chirho))
            .collect();
        self.list_chirho(&ids_chirho)
    }

    /// Check if e-class is ground
    pub fn is_ground_chirho(&self, class_chirho: EClassIdChirho) -> bool {
        self.class_data_chirho[self.find_immut_chirho(class_chirho) as usize].is_ground_chirho
    }

    /// Get number of e-classes
    pub fn num_classes_chirho(&self) -> usize {
        self.class_data_chirho.len()
    }

    /// Get number of e-nodes
    pub fn num_nodes_chirho(&self) -> usize {
        self.nodes_chirho.len()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_native_egraph_basic_chirho() {
        let mut eg_chirho = EGraphNativeChirho::new_chirho();

        let x_chirho = eg_chirho.fresh_var_chirho();
        let y_chirho = eg_chirho.fresh_var_chirho();
        let forty_two_chirho = eg_chirho.int_chirho(42);

        // Different initially
        assert!(!eg_chirho.equiv_chirho(x_chirho, y_chirho));

        // Unify x = y
        assert!(eg_chirho.unify_chirho(x_chirho, y_chirho));
        assert!(eg_chirho.equiv_chirho(x_chirho, y_chirho));

        // Unify x = 42
        assert!(eg_chirho.unify_chirho(x_chirho, forty_two_chirho));

        // Now y should also equal 42
        assert!(eg_chirho.equiv_chirho(y_chirho, forty_two_chirho));
    }

    #[test]
    fn test_native_egraph_conflict_chirho() {
        let mut eg_chirho = EGraphNativeChirho::new_chirho();

        let x_chirho = eg_chirho.fresh_var_chirho();
        let one_chirho = eg_chirho.int_chirho(1);
        let two_chirho = eg_chirho.int_chirho(2);

        // Unify x = 1
        assert!(eg_chirho.unify_chirho(x_chirho, one_chirho));

        // Try x = 2 - should fail
        assert!(!eg_chirho.unify_chirho(x_chirho, two_chirho));
    }

    #[test]
    fn test_native_egraph_list_chirho() {
        let mut eg_chirho = EGraphNativeChirho::new_chirho();

        let list1_chirho = eg_chirho.list_ints_chirho(&[1, 2, 3]);
        let list2_chirho = eg_chirho.list_ints_chirho(&[1, 2, 3]);

        // Same structure should be hash-consed to same class
        assert!(eg_chirho.equiv_chirho(list1_chirho, list2_chirho));
    }

    #[test]
    fn test_native_egraph_ground_chirho() {
        let mut eg_chirho = EGraphNativeChirho::new_chirho();

        let x_chirho = eg_chirho.fresh_var_chirho();
        let forty_two_chirho = eg_chirho.int_chirho(42);
        let list_chirho = eg_chirho.list_ints_chirho(&[1, 2]);

        assert!(!eg_chirho.is_ground_chirho(x_chirho));
        assert!(eg_chirho.is_ground_chirho(forty_two_chirho));
        assert!(eg_chirho.is_ground_chirho(list_chirho));
    }

    #[test]
    fn test_native_egraph_congruence_chirho() {
        let mut eg_chirho = EGraphNativeChirho::new_chirho();

        // Build cons(x, nil) and cons(y, nil)
        let x_chirho = eg_chirho.fresh_var_chirho();
        let y_chirho = eg_chirho.fresh_var_chirho();
        let nil_chirho = eg_chirho.nil_chirho();

        let cons_x_chirho = eg_chirho.cons_chirho(x_chirho, nil_chirho);
        let cons_y_chirho = eg_chirho.cons_chirho(y_chirho, nil_chirho);

        // Different initially
        assert!(!eg_chirho.equiv_chirho(cons_x_chirho, cons_y_chirho));

        // Unify x = y
        assert!(eg_chirho.unify_chirho(x_chirho, y_chirho));

        // Now cons(x, nil) should equal cons(y, nil) by congruence
        assert!(eg_chirho.equiv_chirho(cons_x_chirho, cons_y_chirho));
    }
}
