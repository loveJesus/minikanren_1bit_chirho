//! Nested Latent Patterns ☧
//!
//! Extends flat patterns to handle nested structure (cons cells/trees).
//!
//! Key insight: Instead of (position, value) encoding for flat lists,
//! use (path, constraint) encoding for trees.
//!
//! Path = sequence of car/cdr traversals
//! Constraint = type (nil/cons/atom) + value for atoms
//!
//! This enables pattern matching over arbitrary S-expressions,
//! not just flat lists.

use std::collections::HashMap;

/// Single step in a tree path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathStepChirho {
    /// Take the head (car) of a cons cell
    CarChirho,
    /// Take the tail (cdr) of a cons cell
    CdrChirho,
}

/// Path through a tree structure
/// Empty = root, [Car] = head, [Cdr, Car] = second element
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TreePathChirho {
    steps_chirho: Vec<PathStepChirho>,
}

impl TreePathChirho {
    /// Create empty path (root)
    pub fn root_chirho() -> Self {
        Self { steps_chirho: Vec::new() }
    }

    /// Create path from steps
    pub fn new_chirho(steps_chirho: Vec<PathStepChirho>) -> Self {
        Self { steps_chirho }
    }

    /// Extend path with car (take head)
    pub fn car_chirho(&self) -> Self {
        let mut new_steps_chirho = self.steps_chirho.clone();
        new_steps_chirho.push(PathStepChirho::CarChirho);
        Self { steps_chirho: new_steps_chirho }
    }

    /// Extend path with cdr (take tail)
    pub fn cdr_chirho(&self) -> Self {
        let mut new_steps_chirho = self.steps_chirho.clone();
        new_steps_chirho.push(PathStepChirho::CdrChirho);
        Self { steps_chirho: new_steps_chirho }
    }

    /// Path depth
    pub fn depth_chirho(&self) -> usize {
        self.steps_chirho.len()
    }

    /// Path to the n-th element of a proper list
    /// list_position(0) = [Car] (first element)
    /// list_position(1) = [Cdr, Car] (second element)
    pub fn list_position_chirho(index_chirho: usize) -> Self {
        let mut steps_chirho = vec![PathStepChirho::CdrChirho; index_chirho];
        steps_chirho.push(PathStepChirho::CarChirho);
        Self { steps_chirho }
    }

    /// Path to tail after n elements
    /// list_tail(0) = [] (root)
    /// list_tail(1) = [Cdr]
    /// list_tail(2) = [Cdr, Cdr]
    pub fn list_tail_chirho(after_chirho: usize) -> Self {
        Self {
            steps_chirho: vec![PathStepChirho::CdrChirho; after_chirho],
        }
    }

    /// Check if this path is a prefix of another
    pub fn is_prefix_of_chirho(&self, other_chirho: &TreePathChirho) -> bool {
        if self.steps_chirho.len() > other_chirho.steps_chirho.len() {
            return false;
        }
        self.steps_chirho
            .iter()
            .zip(other_chirho.steps_chirho.iter())
            .all(|(a_chirho, b_chirho)| a_chirho == b_chirho)
    }
}

/// Type of a node in the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeTypeChirho {
    /// Empty list (nil)
    NilChirho,
    /// Cons cell (pair)
    ConsChirho,
    /// Atomic value (integer)
    AtomChirho,
    /// Unknown (logic variable)
    AnyChirho,
}

/// Constraint on a single node in the tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeConstraintChirho {
    /// What kind of node
    pub type_chirho: NodeTypeChirho,
    /// If atom, the specific value (None = any atom)
    pub value_chirho: Option<i64>,
    /// If this is a logic variable, its ID
    pub var_id_chirho: Option<u32>,
}

impl NodeConstraintChirho {
    /// Create nil constraint
    pub fn nil_chirho() -> Self {
        Self {
            type_chirho: NodeTypeChirho::NilChirho,
            value_chirho: None,
            var_id_chirho: None,
        }
    }

    /// Create cons constraint
    pub fn cons_chirho() -> Self {
        Self {
            type_chirho: NodeTypeChirho::ConsChirho,
            value_chirho: None,
            var_id_chirho: None,
        }
    }

    /// Create atom constraint with specific value
    pub fn atom_chirho(value_chirho: i64) -> Self {
        Self {
            type_chirho: NodeTypeChirho::AtomChirho,
            value_chirho: Some(value_chirho),
            var_id_chirho: None,
        }
    }

    /// Create any-atom constraint (matches any atom)
    pub fn any_atom_chirho() -> Self {
        Self {
            type_chirho: NodeTypeChirho::AtomChirho,
            value_chirho: None,
            var_id_chirho: None,
        }
    }

    /// Create variable constraint
    pub fn var_chirho(id_chirho: u32) -> Self {
        Self {
            type_chirho: NodeTypeChirho::AnyChirho,
            value_chirho: None,
            var_id_chirho: Some(id_chirho),
        }
    }

    /// Create unconstrained (any type)
    pub fn any_chirho() -> Self {
        Self {
            type_chirho: NodeTypeChirho::AnyChirho,
            value_chirho: None,
            var_id_chirho: None,
        }
    }

    /// Check if this constraint is compatible with another
    /// Returns merged constraint if compatible, None if conflict
    pub fn unify_chirho(&self, other_chirho: &NodeConstraintChirho) -> Option<NodeConstraintChirho> {
        use NodeTypeChirho::*;

        match (self.type_chirho, other_chirho.type_chirho) {
            // Any matches anything
            (AnyChirho, _) => Some(other_chirho.clone()),
            (_, AnyChirho) => Some(self.clone()),

            // Same type checks
            (NilChirho, NilChirho) => Some(NodeConstraintChirho::nil_chirho()),
            (ConsChirho, ConsChirho) => Some(NodeConstraintChirho::cons_chirho()),

            (AtomChirho, AtomChirho) => {
                match (self.value_chirho, other_chirho.value_chirho) {
                    (Some(v1_chirho), Some(v2_chirho)) => {
                        if v1_chirho == v2_chirho {
                            Some(NodeConstraintChirho::atom_chirho(v1_chirho))
                        } else {
                            None // Conflict: different values
                        }
                    }
                    (Some(v_chirho), None) | (None, Some(v_chirho)) => {
                        Some(NodeConstraintChirho::atom_chirho(v_chirho))
                    }
                    (None, None) => Some(NodeConstraintChirho::any_atom_chirho()),
                }
            }

            // Incompatible types
            _ => None,
        }
    }
}

/// Nested pattern: partial specification of a tree
/// Maps paths to constraints
#[derive(Debug, Clone, Default)]
pub struct NestedPatternChirho {
    /// Constraints at each path
    pub constraints_chirho: HashMap<TreePathChirho, NodeConstraintChirho>,
}

impl NestedPatternChirho {
    pub fn new_chirho() -> Self {
        Self::default()
    }

    /// Set constraint at a path
    pub fn set_chirho(&mut self, path_chirho: TreePathChirho, constraint_chirho: NodeConstraintChirho) {
        self.constraints_chirho.insert(path_chirho, constraint_chirho);
    }

    /// Get constraint at a path (returns Any if not specified)
    pub fn get_chirho(&self, path_chirho: &TreePathChirho) -> NodeConstraintChirho {
        self.constraints_chirho
            .get(path_chirho)
            .cloned()
            .unwrap_or_else(NodeConstraintChirho::any_chirho)
    }

    /// Create pattern for flat list [a, b, c, ...]
    pub fn from_list_chirho(values_chirho: &[i64]) -> Self {
        let mut pattern_chirho = Self::new_chirho();

        for (i_chirho, &value_chirho) in values_chirho.iter().enumerate() {
            // Each position is a cons
            pattern_chirho.set_chirho(
                TreePathChirho::list_tail_chirho(i_chirho),
                NodeConstraintChirho::cons_chirho(),
            );
            // With this value as car
            pattern_chirho.set_chirho(
                TreePathChirho::list_position_chirho(i_chirho),
                NodeConstraintChirho::atom_chirho(value_chirho),
            );
        }

        // Final nil
        pattern_chirho.set_chirho(
            TreePathChirho::list_tail_chirho(values_chirho.len()),
            NodeConstraintChirho::nil_chirho(),
        );

        pattern_chirho
    }

    /// Create pattern for list with holes: [1, ?, 3] = Some(1), None, Some(3)
    pub fn from_list_with_holes_chirho(values_chirho: &[Option<i64>]) -> Self {
        let mut pattern_chirho = Self::new_chirho();

        for (i_chirho, value_opt_chirho) in values_chirho.iter().enumerate() {
            // Each position is a cons
            pattern_chirho.set_chirho(
                TreePathChirho::list_tail_chirho(i_chirho),
                NodeConstraintChirho::cons_chirho(),
            );

            // Value or hole
            if let Some(value_chirho) = value_opt_chirho {
                pattern_chirho.set_chirho(
                    TreePathChirho::list_position_chirho(i_chirho),
                    NodeConstraintChirho::atom_chirho(*value_chirho),
                );
            }
            // If None, we don't add a constraint (defaults to Any)
        }

        // Final nil
        pattern_chirho.set_chirho(
            TreePathChirho::list_tail_chirho(values_chirho.len()),
            NodeConstraintChirho::nil_chirho(),
        );

        pattern_chirho
    }

    /// Unify two patterns
    /// Returns merged pattern if compatible, None if conflict
    pub fn unify_chirho(&self, other_chirho: &NestedPatternChirho) -> Option<NestedPatternChirho> {
        let mut result_chirho = NestedPatternChirho::new_chirho();

        // Collect all paths from both patterns
        let mut all_paths_chirho: std::collections::HashSet<_> =
            self.constraints_chirho.keys().cloned().collect();
        all_paths_chirho.extend(other_chirho.constraints_chirho.keys().cloned());

        for path_chirho in all_paths_chirho {
            let c1_chirho = self.get_chirho(&path_chirho);
            let c2_chirho = other_chirho.get_chirho(&path_chirho);

            match c1_chirho.unify_chirho(&c2_chirho) {
                Some(merged_chirho) => {
                    result_chirho.set_chirho(path_chirho, merged_chirho);
                }
                None => return None, // Conflict
            }
        }

        Some(result_chirho)
    }

    /// Check if pattern is fully ground (no Any constraints)
    pub fn is_ground_chirho(&self) -> bool {
        self.constraints_chirho
            .values()
            .all(|c_chirho| c_chirho.type_chirho != NodeTypeChirho::AnyChirho)
    }

    /// Get max depth of constraints
    pub fn max_depth_chirho(&self) -> usize {
        self.constraints_chirho
            .keys()
            .map(|p_chirho| p_chirho.depth_chirho())
            .max()
            .unwrap_or(0)
    }

    /// Convert to bit representation for a flat list pattern
    /// Returns (value_bits, mask_bits) where bit i = position i's value/known
    pub fn to_list_bits_chirho(&self, max_len_chirho: usize) -> (u64, u64) {
        let mut values_chirho: u64 = 0;
        let mut mask_chirho: u64 = 0;

        for i_chirho in 0..max_len_chirho.min(64) {
            let path_chirho = TreePathChirho::list_position_chirho(i_chirho);
            if let Some(c_chirho) = self.constraints_chirho.get(&path_chirho) {
                if let Some(v_chirho) = c_chirho.value_chirho {
                    if v_chirho >= 0 && v_chirho < 64 {
                        values_chirho |= (v_chirho as u64) << (i_chirho * 8);
                        mask_chirho |= 0xFF << (i_chirho * 8);
                    }
                }
            }
        }

        (values_chirho, mask_chirho)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_path_basic_chirho() {
        let root_chirho = TreePathChirho::root_chirho();
        assert_eq!(root_chirho.depth_chirho(), 0);

        let car_chirho = root_chirho.car_chirho();
        assert_eq!(car_chirho.depth_chirho(), 1);

        let cdr_car_chirho = root_chirho.cdr_chirho().car_chirho();
        assert_eq!(cdr_car_chirho.depth_chirho(), 2);
    }

    #[test]
    fn test_list_position_chirho() {
        // Position 0 = first element = [Car]
        let pos0_chirho = TreePathChirho::list_position_chirho(0);
        assert_eq!(pos0_chirho.steps_chirho, vec![PathStepChirho::CarChirho]);

        // Position 1 = second element = [Cdr, Car]
        let pos1_chirho = TreePathChirho::list_position_chirho(1);
        assert_eq!(
            pos1_chirho.steps_chirho,
            vec![PathStepChirho::CdrChirho, PathStepChirho::CarChirho]
        );
    }

    #[test]
    fn test_constraint_unify_chirho() {
        let nil_chirho = NodeConstraintChirho::nil_chirho();
        let cons_chirho = NodeConstraintChirho::cons_chirho();
        let atom5_chirho = NodeConstraintChirho::atom_chirho(5);
        let any_chirho = NodeConstraintChirho::any_chirho();

        // Same type unifies
        assert!(nil_chirho.unify_chirho(&nil_chirho).is_some());
        assert!(cons_chirho.unify_chirho(&cons_chirho).is_some());

        // Different types fail
        assert!(nil_chirho.unify_chirho(&cons_chirho).is_none());

        // Any unifies with anything
        assert_eq!(any_chirho.unify_chirho(&atom5_chirho), Some(atom5_chirho.clone()));

        // Same atom unifies
        let atom5b_chirho = NodeConstraintChirho::atom_chirho(5);
        assert!(atom5_chirho.unify_chirho(&atom5b_chirho).is_some());

        // Different atoms fail
        let atom7_chirho = NodeConstraintChirho::atom_chirho(7);
        assert!(atom5_chirho.unify_chirho(&atom7_chirho).is_none());
    }

    #[test]
    fn test_nested_pattern_list_chirho() {
        let pattern_chirho = NestedPatternChirho::from_list_chirho(&[1, 2, 3]);

        // Check first element
        let pos0_chirho = TreePathChirho::list_position_chirho(0);
        let c0_chirho = pattern_chirho.get_chirho(&pos0_chirho);
        assert_eq!(c0_chirho.value_chirho, Some(1));

        // Check second element
        let pos1_chirho = TreePathChirho::list_position_chirho(1);
        let c1_chirho = pattern_chirho.get_chirho(&pos1_chirho);
        assert_eq!(c1_chirho.value_chirho, Some(2));

        // Check tail is nil
        let tail_chirho = TreePathChirho::list_tail_chirho(3);
        let ct_chirho = pattern_chirho.get_chirho(&tail_chirho);
        assert_eq!(ct_chirho.type_chirho, NodeTypeChirho::NilChirho);
    }

    #[test]
    fn test_nested_pattern_unify_chirho() {
        // [1, ?, 3]
        let p1_chirho = NestedPatternChirho::from_list_with_holes_chirho(&[Some(1), None, Some(3)]);

        // [?, 2, 3]
        let p2_chirho = NestedPatternChirho::from_list_with_holes_chirho(&[None, Some(2), Some(3)]);

        // Should unify to [1, 2, 3]
        let unified_chirho = p1_chirho.unify_chirho(&p2_chirho).unwrap();

        let pos0_chirho = TreePathChirho::list_position_chirho(0);
        let pos1_chirho = TreePathChirho::list_position_chirho(1);
        let pos2_chirho = TreePathChirho::list_position_chirho(2);

        assert_eq!(unified_chirho.get_chirho(&pos0_chirho).value_chirho, Some(1));
        assert_eq!(unified_chirho.get_chirho(&pos1_chirho).value_chirho, Some(2));
        assert_eq!(unified_chirho.get_chirho(&pos2_chirho).value_chirho, Some(3));
    }

    #[test]
    fn test_nested_pattern_conflict_chirho() {
        // [1, ?, ?]
        let p1_chirho = NestedPatternChirho::from_list_with_holes_chirho(&[Some(1), None, None]);

        // [2, ?, ?]
        let p2_chirho = NestedPatternChirho::from_list_with_holes_chirho(&[Some(2), None, None]);

        // Should fail - first elements conflict
        assert!(p1_chirho.unify_chirho(&p2_chirho).is_none());
    }
}
