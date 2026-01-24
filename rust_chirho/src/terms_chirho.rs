//! Term representation ☧
//!
//! Terms are hash-consed for efficient comparison and storage.
//! Each unique term gets a unique ID (u32).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Term ID - index into term store
pub type TermIdChirho = u32;

/// Variable ID - separate namespace from terms
pub type VarIdChirho = u32;

/// Term variants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TermChirho {
    /// Logic variable
    VarChirho(VarIdChirho),
    /// Integer constant
    IntChirho(i64),
    /// Nil (empty list)
    NilChirho,
    /// Cons cell (head, tail) - stores TermIds for hash consing
    ConsChirho(TermIdChirho, TermIdChirho),
    /// Symbol/atom
    SymChirho(u32), // Index into symbol table
}

/// Hash-consed term store
#[derive(Debug, Default)]
pub struct TermStoreChirho {
    /// Term → ID mapping (hash consing)
    term_to_id_chirho: HashMap<TermChirho, TermIdChirho>,
    /// ID → Term mapping
    id_to_term_chirho: Vec<TermChirho>,
    /// Next variable ID
    next_var_chirho: AtomicU32,
    /// Symbol table: string → symbol ID
    sym_to_id_chirho: HashMap<String, u32>,
    /// Reverse: symbol ID → string
    id_to_sym_chirho: Vec<String>,
}

impl Clone for TermStoreChirho {
    fn clone(&self) -> Self {
        Self {
            term_to_id_chirho: self.term_to_id_chirho.clone(),
            id_to_term_chirho: self.id_to_term_chirho.clone(),
            next_var_chirho: AtomicU32::new(self.next_var_chirho.load(Ordering::SeqCst)),
            sym_to_id_chirho: self.sym_to_id_chirho.clone(),
            id_to_sym_chirho: self.id_to_sym_chirho.clone(),
        }
    }
}

impl TermStoreChirho {
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a term, returning its unique ID
    pub fn intern_chirho(&mut self, term_chirho: TermChirho) -> TermIdChirho {
        if let Some(&id_chirho) = self.term_to_id_chirho.get(&term_chirho) {
            return id_chirho;
        }

        let id_chirho = self.id_to_term_chirho.len() as TermIdChirho;
        self.id_to_term_chirho.push(term_chirho.clone());
        self.term_to_id_chirho.insert(term_chirho, id_chirho);
        id_chirho
    }

    /// Get term by ID
    pub fn get_chirho(&self, id_chirho: TermIdChirho) -> Option<&TermChirho> {
        self.id_to_term_chirho.get(id_chirho as usize)
    }

    /// Create fresh variable
    pub fn fresh_var_chirho(&mut self) -> (VarIdChirho, TermIdChirho) {
        let var_id_chirho = self.next_var_chirho.fetch_add(1, Ordering::SeqCst);
        let term_chirho = TermChirho::VarChirho(var_id_chirho);
        let term_id_chirho = self.intern_chirho(term_chirho);
        (var_id_chirho, term_id_chirho)
    }

    /// Create integer term
    pub fn int_chirho(&mut self, val_chirho: i64) -> TermIdChirho {
        self.intern_chirho(TermChirho::IntChirho(val_chirho))
    }

    /// Create nil term
    pub fn nil_chirho(&mut self) -> TermIdChirho {
        self.intern_chirho(TermChirho::NilChirho)
    }

    /// Create cons cell
    pub fn cons_chirho(&mut self, head_chirho: TermIdChirho, tail_chirho: TermIdChirho) -> TermIdChirho {
        self.intern_chirho(TermChirho::ConsChirho(head_chirho, tail_chirho))
    }

    /// Build list from slice of term IDs
    pub fn list_chirho(&mut self, elems_chirho: &[TermIdChirho]) -> TermIdChirho {
        let mut result_chirho = self.nil_chirho();
        for &elem_chirho in elems_chirho.iter().rev() {
            result_chirho = self.cons_chirho(elem_chirho, result_chirho);
        }
        result_chirho
    }

    /// Build list from integers
    pub fn list_ints_chirho(&mut self, vals_chirho: &[i64]) -> TermIdChirho {
        let ids_chirho: Vec<_> = vals_chirho.iter().map(|&v| self.int_chirho(v)).collect();
        self.list_chirho(&ids_chirho)
    }

    /// Create or lookup a symbol by name
    pub fn sym_chirho(&mut self, name_chirho: &str) -> TermIdChirho {
        let sym_id_chirho = if let Some(&id_chirho) = self.sym_to_id_chirho.get(name_chirho) {
            id_chirho
        } else {
            let id_chirho = self.id_to_sym_chirho.len() as u32;
            self.id_to_sym_chirho.push(name_chirho.to_string());
            self.sym_to_id_chirho.insert(name_chirho.to_string(), id_chirho);
            id_chirho
        };
        self.intern_chirho(TermChirho::SymChirho(sym_id_chirho))
    }

    /// Get symbol name by symbol ID
    pub fn sym_name_chirho(&self, sym_id_chirho: u32) -> Option<&str> {
        self.id_to_sym_chirho.get(sym_id_chirho as usize).map(|s| s.as_str())
    }

    /// Check if term is a variable
    pub fn is_var_chirho(&self, id_chirho: TermIdChirho) -> bool {
        matches!(self.get_chirho(id_chirho), Some(TermChirho::VarChirho(_)))
    }

    /// Check if term is ground (no variables)
    pub fn is_ground_chirho(&self, id_chirho: TermIdChirho) -> bool {
        match self.get_chirho(id_chirho) {
            Some(TermChirho::VarChirho(_)) => false,
            Some(TermChirho::ConsChirho(h, t)) => {
                self.is_ground_chirho(*h) && self.is_ground_chirho(*t)
            }
            Some(_) => true,
            None => false,
        }
    }

    /// Extract list to Vec<i64> if ground
    pub fn extract_list_chirho(&self, id_chirho: TermIdChirho) -> Option<Vec<i64>> {
        let mut result_chirho = Vec::new();
        let mut curr_chirho = id_chirho;

        loop {
            match self.get_chirho(curr_chirho)? {
                TermChirho::NilChirho => return Some(result_chirho),
                TermChirho::ConsChirho(h, t) => {
                    if let TermChirho::IntChirho(v) = self.get_chirho(*h)? {
                        result_chirho.push(*v);
                        curr_chirho = *t;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    }

    /// Number of interned terms
    pub fn len_chirho(&self) -> usize {
        self.id_to_term_chirho.len()
    }

    pub fn is_empty_chirho(&self) -> bool {
        self.id_to_term_chirho.is_empty()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_hash_cons_chirho() {
        let mut store_chirho = TermStoreChirho::new();

        let a_chirho = store_chirho.int_chirho(42);
        let b_chirho = store_chirho.int_chirho(42);
        assert_eq!(a_chirho, b_chirho); // Same ID due to hash consing

        let c_chirho = store_chirho.int_chirho(43);
        assert_ne!(a_chirho, c_chirho);
    }

    #[test]
    fn test_list_chirho() {
        let mut store_chirho = TermStoreChirho::new();

        let list_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);
        let extracted_chirho = store_chirho.extract_list_chirho(list_chirho);

        assert_eq!(extracted_chirho, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_ground_check_chirho() {
        let mut store_chirho = TermStoreChirho::new();

        let ground_chirho = store_chirho.list_ints_chirho(&[1, 2]);
        assert!(store_chirho.is_ground_chirho(ground_chirho));

        let (_, var_id_chirho) = store_chirho.fresh_var_chirho();
        let nil_chirho = store_chirho.nil_chirho();
        let non_ground_chirho = store_chirho.cons_chirho(var_id_chirho, nil_chirho);
        assert!(!store_chirho.is_ground_chirho(non_ground_chirho));
    }
}
