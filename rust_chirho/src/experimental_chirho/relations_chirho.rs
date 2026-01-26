// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Relation definitions ☧
//!
//! Relations as sparse Boolean tensors.

use crate::bitmatrix_chirho::BitTensor3Chirho;

/// appendo relation: l ++ s = out
#[derive(Debug, Default)]
pub struct AppendoChirho {
    /// Tensor storing valid (l, s, out) triples
    pub tensor_chirho: BitTensor3Chirho,
}

impl AppendoChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a known triple
    pub fn add_triple_chirho(&mut self, l_chirho: u32, s_chirho: u32, out_chirho: u32) {
        self.tensor_chirho.set_chirho(l_chirho, s_chirho, out_chirho);
    }

    /// Query: given l and s, find possible out values
    pub fn forward_chirho(&self, l_chirho: u32, s_chirho: u32) -> Vec<u32> {
        self.tensor_chirho.query_chirho([(0, l_chirho), (1, s_chirho)])
    }

    /// Query: given out, find possible (l, s) pairs
    pub fn backward_chirho(&self, out_chirho: u32) -> Vec<(u32, u32)> {
        let mut results_chirho = Vec::new();
        for (l, s, o) in self.tensor_chirho.iter_chirho() {
            if *o == out_chirho {
                results_chirho.push((*l, *s));
            }
        }
        results_chirho
    }
}

/// membero relation: x is a member of list
#[derive(Debug, Default)]
pub struct MemberoChirho {
    /// Tensor storing valid (element, list) pairs
    /// element_id → set of list_ids containing it
    entries_chirho: std::collections::HashMap<u32, std::collections::HashSet<u32>>,
}

impl MemberoChirho {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pair_chirho(&mut self, elem_chirho: u32, list_chirho: u32) {
        self.entries_chirho
            .entry(elem_chirho)
            .or_default()
            .insert(list_chirho);
    }

    /// Query: which lists contain this element?
    pub fn lists_containing_chirho(&self, elem_chirho: u32) -> Vec<u32> {
        self.entries_chirho
            .get(&elem_chirho)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Query: what elements are in this list?
    pub fn elements_of_chirho(&self, list_chirho: u32) -> Vec<u32> {
        self.entries_chirho
            .iter()
            .filter(|(_, lists)| lists.contains(&list_chirho))
            .map(|(elem, _)| *elem)
            .collect()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_appendo_chirho() {
        let mut rel_chirho = AppendoChirho::new();

        // [] ++ [1] = [1] (assuming term IDs: nil=0, [1]=1)
        rel_chirho.add_triple_chirho(0, 1, 1);

        // [0] ++ [1] = [0,1] (assuming: [0]=2, [1]=1, [0,1]=3)
        rel_chirho.add_triple_chirho(2, 1, 3);

        let results_chirho = rel_chirho.forward_chirho(2, 1);
        assert_eq!(results_chirho, vec![3]);

        let splits_chirho = rel_chirho.backward_chirho(3);
        assert_eq!(splits_chirho, vec![(2, 1)]);
    }
}
