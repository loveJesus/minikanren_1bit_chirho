// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Union-Find for variable equivalence classes ☧
//!
//! O(α(n)) ≈ O(1) amortized operations.
//! Hardware-friendly: can be implemented with CAM (content addressable memory).

use std::collections::HashMap;

/// Union-Find with path compression and union by rank
#[derive(Debug, Clone, Default)]
pub struct UnionFindChirho {
    parent_chirho: HashMap<u32, u32>,
    rank_chirho: HashMap<u32, u32>,
}

impl UnionFindChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Make singleton set
    pub fn make_set_chirho(&mut self, x_chirho: u32) {
        use std::collections::hash_map::Entry;
        if let Entry::Vacant(e_chirho) = self.parent_chirho.entry(x_chirho) {
            e_chirho.insert(x_chirho);
            self.rank_chirho.insert(x_chirho, 0);
        }
    }

    /// Find representative with path compression
    pub fn find_chirho(&mut self, x_chirho: u32) -> u32 {
        if !self.parent_chirho.contains_key(&x_chirho) {
            self.make_set_chirho(x_chirho);
            return x_chirho;
        }

        let parent_chirho = self.parent_chirho[&x_chirho];
        if parent_chirho != x_chirho {
            // Path compression
            let root_chirho = self.find_chirho(parent_chirho);
            self.parent_chirho.insert(x_chirho, root_chirho);
            root_chirho
        } else {
            x_chirho
        }
    }

    /// Find representative WITHOUT path compression (immutable access)
    pub fn find_immut_chirho(&self, x_chirho: u32) -> u32 {
        let mut current_chirho = x_chirho;
        while let Some(&parent_chirho) = self.parent_chirho.get(&current_chirho) {
            if parent_chirho == current_chirho {
                return current_chirho;
            }
            current_chirho = parent_chirho;
        }
        x_chirho // Not in structure, is its own root
    }

    /// Union by rank, returns new root
    pub fn union_chirho(&mut self, x_chirho: u32, y_chirho: u32) -> u32 {
        let root_x_chirho = self.find_chirho(x_chirho);
        let root_y_chirho = self.find_chirho(y_chirho);

        if root_x_chirho == root_y_chirho {
            return root_x_chirho;
        }

        let rank_x_chirho = self.rank_chirho.get(&root_x_chirho).copied().unwrap_or(0);
        let rank_y_chirho = self.rank_chirho.get(&root_y_chirho).copied().unwrap_or(0);

        let (new_root_chirho, child_chirho) = if rank_x_chirho < rank_y_chirho {
            (root_y_chirho, root_x_chirho)
        } else {
            (root_x_chirho, root_y_chirho)
        };

        self.parent_chirho.insert(child_chirho, new_root_chirho);

        if rank_x_chirho == rank_y_chirho {
            *self.rank_chirho.entry(new_root_chirho).or_insert(0) += 1;
        }

        new_root_chirho
    }

    /// Check if same class
    pub fn same_class_chirho(&mut self, x_chirho: u32, y_chirho: u32) -> bool {
        self.find_chirho(x_chirho) == self.find_chirho(y_chirho)
    }

    /// Get all equivalence classes
    pub fn all_classes_chirho(&mut self) -> HashMap<u32, Vec<u32>> {
        let keys_chirho: Vec<u32> = self.parent_chirho.keys().copied().collect();
        let mut classes_chirho: HashMap<u32, Vec<u32>> = HashMap::new();

        for key_chirho in keys_chirho {
            let root_chirho = self.find_chirho(key_chirho);
            classes_chirho.entry(root_chirho).or_default().push(key_chirho);
        }

        classes_chirho
    }
}

/// Hardware-oriented Union-Find using arrays (fixed size)
/// More suitable for FPGA implementation
#[derive(Debug, Clone)]
pub struct UnionFindHwChirho {
    /// Parent array (index = element, value = parent)
    parent_chirho: Vec<u32>,
    /// Rank array
    rank_chirho: Vec<u8>,
    /// Size (reserved for future use)
    #[allow(dead_code)]
    size_chirho: usize,
}

impl UnionFindHwChirho {
    pub fn new(size_chirho: usize) -> Self {
        Self {
            parent_chirho: (0..size_chirho as u32).collect(),
            rank_chirho: vec![0; size_chirho],
            size_chirho,
        }
    }

    /// Find with path compression (iterative for hardware)
    pub fn find_chirho(&mut self, mut x_chirho: u32) -> u32 {
        // Find root
        let mut root_chirho = x_chirho;
        while self.parent_chirho[root_chirho as usize] != root_chirho {
            root_chirho = self.parent_chirho[root_chirho as usize];
        }

        // Path compression
        while self.parent_chirho[x_chirho as usize] != root_chirho {
            let next_chirho = self.parent_chirho[x_chirho as usize];
            self.parent_chirho[x_chirho as usize] = root_chirho;
            x_chirho = next_chirho;
        }

        root_chirho
    }

    /// Union by rank
    pub fn union_chirho(&mut self, x_chirho: u32, y_chirho: u32) -> u32 {
        let root_x_chirho = self.find_chirho(x_chirho);
        let root_y_chirho = self.find_chirho(y_chirho);

        if root_x_chirho == root_y_chirho {
            return root_x_chirho;
        }

        let rank_x_chirho = self.rank_chirho[root_x_chirho as usize];
        let rank_y_chirho = self.rank_chirho[root_y_chirho as usize];

        if rank_x_chirho < rank_y_chirho {
            self.parent_chirho[root_x_chirho as usize] = root_y_chirho;
            root_y_chirho
        } else if rank_x_chirho > rank_y_chirho {
            self.parent_chirho[root_y_chirho as usize] = root_x_chirho;
            root_x_chirho
        } else {
            self.parent_chirho[root_y_chirho as usize] = root_x_chirho;
            self.rank_chirho[root_x_chirho as usize] += 1;
            root_x_chirho
        }
    }

    pub fn same_class_chirho(&mut self, x_chirho: u32, y_chirho: u32) -> bool {
        self.find_chirho(x_chirho) == self.find_chirho(y_chirho)
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_union_find_chirho() {
        let mut uf_chirho = UnionFindChirho::new();

        uf_chirho.make_set_chirho(1);
        uf_chirho.make_set_chirho(2);
        uf_chirho.make_set_chirho(3);

        assert!(!uf_chirho.same_class_chirho(1, 2));

        uf_chirho.union_chirho(1, 2);
        assert!(uf_chirho.same_class_chirho(1, 2));
        assert!(!uf_chirho.same_class_chirho(1, 3));

        uf_chirho.union_chirho(2, 3);
        assert!(uf_chirho.same_class_chirho(1, 3));
    }

    #[test]
    fn test_union_find_hw_chirho() {
        let mut uf_chirho = UnionFindHwChirho::new(100);

        assert!(!uf_chirho.same_class_chirho(1, 2));

        uf_chirho.union_chirho(1, 2);
        assert!(uf_chirho.same_class_chirho(1, 2));

        // Chain test
        for i_chirho in 0..50 {
            uf_chirho.union_chirho(i_chirho, i_chirho + 1);
        }
        assert!(uf_chirho.same_class_chirho(0, 50));
    }
}
