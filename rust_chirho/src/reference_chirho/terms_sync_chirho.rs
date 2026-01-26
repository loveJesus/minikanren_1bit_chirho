// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Thread-safe Term Store ☧
//!
//! Concurrent hash-consed term store for parallel search.
//! Uses RwLock for interior mutability while maintaining hash-consing invariants.
//!
//! For God so loved the world that he gave his only begotten Son,
//! that whoever believes in him should not perish but have eternal life.
//! John 3:16

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;

use super::terms_chirho::{TermChirho, TermIdChirho, VarIdChirho};

/// Thread-safe hash-consed term store
///
/// # Design
///
/// Uses interior mutability pattern:
/// - `RwLock<HashMap>` for term interning (many readers, occasional writer)
/// - `RwLock<Vec>` for ID lookup (many readers, occasional writer)
/// - `AtomicU32` for fresh variable generation (lock-free)
///
/// # Performance
///
/// - Read operations (get, is_var, is_ground): acquire read lock only
/// - Write operations (intern, fresh_var): acquire write lock
/// - Variable ID generation is lock-free
///
/// # Usage
///
/// ```ignore
/// let store = TermStoreSyncChirho::new();
///
/// // Can be shared across threads
/// let store_clone = store.clone();
/// std::thread::spawn(move || {
///     let id = store_clone.int_chirho(42);
/// });
/// ```
#[derive(Debug, Default)]
pub struct TermStoreSyncChirho {
    /// Term → ID mapping (hash consing)
    term_to_id_chirho: RwLock<HashMap<TermChirho, TermIdChirho>>,
    /// ID → Term mapping
    id_to_term_chirho: RwLock<Vec<TermChirho>>,
    /// Next variable ID (lock-free)
    next_var_chirho: AtomicU32,
    /// Symbol table: string → symbol ID
    sym_to_id_chirho: RwLock<HashMap<String, u32>>,
    /// Reverse: symbol ID → string
    id_to_sym_chirho: RwLock<Vec<String>>,
}

impl Clone for TermStoreSyncChirho {
    fn clone(&self) -> Self {
        Self {
            term_to_id_chirho: RwLock::new(
                self.term_to_id_chirho.read().unwrap().clone()
            ),
            id_to_term_chirho: RwLock::new(
                self.id_to_term_chirho.read().unwrap().clone()
            ),
            next_var_chirho: AtomicU32::new(
                self.next_var_chirho.load(Ordering::SeqCst)
            ),
            sym_to_id_chirho: RwLock::new(
                self.sym_to_id_chirho.read().unwrap().clone()
            ),
            id_to_sym_chirho: RwLock::new(
                self.id_to_sym_chirho.read().unwrap().clone()
            ),
        }
    }
}

impl TermStoreSyncChirho {
    /// Create new empty thread-safe term store
    pub fn new_chirho() -> Self {
        Self::default()
    }

    /// Intern a term, returning its unique ID
    ///
    /// Thread-safe: uses double-checked locking pattern
    pub fn intern_chirho(&self, term_chirho: TermChirho) -> TermIdChirho {
        // Fast path: check if already interned (read lock only)
        {
            let read_guard_chirho = self.term_to_id_chirho.read().unwrap();
            if let Some(&id_chirho) = read_guard_chirho.get(&term_chirho) {
                return id_chirho;
            }
        }

        // Slow path: need to intern (write lock)
        let mut term_map_chirho = self.term_to_id_chirho.write().unwrap();
        let mut id_vec_chirho = self.id_to_term_chirho.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(&id_chirho) = term_map_chirho.get(&term_chirho) {
            return id_chirho;
        }

        let id_chirho = id_vec_chirho.len() as TermIdChirho;
        id_vec_chirho.push(term_chirho.clone());
        term_map_chirho.insert(term_chirho, id_chirho);
        id_chirho
    }

    /// Get term by ID (read-only, fast)
    pub fn get_chirho(&self, id_chirho: TermIdChirho) -> Option<TermChirho> {
        self.id_to_term_chirho
            .read()
            .unwrap()
            .get(id_chirho as usize)
            .cloned()
    }

    /// Create fresh variable (lock-free ID generation)
    pub fn fresh_var_chirho(&self) -> (VarIdChirho, TermIdChirho) {
        let var_id_chirho = self.next_var_chirho.fetch_add(1, Ordering::SeqCst);
        let term_chirho = TermChirho::VarChirho(var_id_chirho);
        let term_id_chirho = self.intern_chirho(term_chirho);
        (var_id_chirho, term_id_chirho)
    }

    /// Create integer term
    pub fn int_chirho(&self, val_chirho: i64) -> TermIdChirho {
        self.intern_chirho(TermChirho::IntChirho(val_chirho))
    }

    /// Create nil term
    pub fn nil_chirho(&self) -> TermIdChirho {
        self.intern_chirho(TermChirho::NilChirho)
    }

    /// Create cons cell
    pub fn cons_chirho(&self, head_chirho: TermIdChirho, tail_chirho: TermIdChirho) -> TermIdChirho {
        self.intern_chirho(TermChirho::ConsChirho(head_chirho, tail_chirho))
    }

    /// Build list from slice of term IDs
    pub fn list_chirho(&self, elems_chirho: &[TermIdChirho]) -> TermIdChirho {
        let mut result_chirho = self.nil_chirho();
        for &elem_chirho in elems_chirho.iter().rev() {
            result_chirho = self.cons_chirho(elem_chirho, result_chirho);
        }
        result_chirho
    }

    /// Build list from integers
    pub fn list_ints_chirho(&self, vals_chirho: &[i64]) -> TermIdChirho {
        let ids_chirho: Vec<_> = vals_chirho.iter().map(|&v| self.int_chirho(v)).collect();
        self.list_chirho(&ids_chirho)
    }

    /// Create or lookup a symbol by name
    pub fn sym_chirho(&self, name_chirho: &str) -> TermIdChirho {
        // Fast path: check if symbol exists
        {
            let read_guard_chirho = self.sym_to_id_chirho.read().unwrap();
            if let Some(&sym_id_chirho) = read_guard_chirho.get(name_chirho) {
                return self.intern_chirho(TermChirho::SymChirho(sym_id_chirho));
            }
        }

        // Slow path: create new symbol
        let mut sym_map_chirho = self.sym_to_id_chirho.write().unwrap();
        let mut sym_vec_chirho = self.id_to_sym_chirho.write().unwrap();

        // Double-check
        if let Some(&sym_id_chirho) = sym_map_chirho.get(name_chirho) {
            return self.intern_chirho(TermChirho::SymChirho(sym_id_chirho));
        }

        let sym_id_chirho = sym_vec_chirho.len() as u32;
        sym_vec_chirho.push(name_chirho.to_string());
        sym_map_chirho.insert(name_chirho.to_string(), sym_id_chirho);
        self.intern_chirho(TermChirho::SymChirho(sym_id_chirho))
    }

    /// Get symbol name by symbol ID
    pub fn sym_name_chirho(&self, sym_id_chirho: u32) -> Option<String> {
        self.id_to_sym_chirho
            .read()
            .unwrap()
            .get(sym_id_chirho as usize)
            .cloned()
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
                self.is_ground_chirho(h) && self.is_ground_chirho(t)
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
                    if let Some(TermChirho::IntChirho(v)) = self.get_chirho(h) {
                        result_chirho.push(v);
                        curr_chirho = t;
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
        self.id_to_term_chirho.read().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty_chirho(&self) -> bool {
        self.id_to_term_chirho.read().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_hash_cons_sync_chirho() {
        let store_chirho = TermStoreSyncChirho::new_chirho();

        let a_chirho = store_chirho.int_chirho(42);
        let b_chirho = store_chirho.int_chirho(42);
        assert_eq!(a_chirho, b_chirho); // Same ID due to hash consing

        let c_chirho = store_chirho.int_chirho(43);
        assert_ne!(a_chirho, c_chirho);
    }

    #[test]
    fn test_list_sync_chirho() {
        let store_chirho = TermStoreSyncChirho::new_chirho();

        let list_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);
        let extracted_chirho = store_chirho.extract_list_chirho(list_chirho);

        assert_eq!(extracted_chirho, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_parallel_intern_chirho() {
        let store_chirho = Arc::new(TermStoreSyncChirho::new_chirho());
        let mut handles_chirho = vec![];

        // Spawn 8 threads, each interning 1000 integers
        for thread_id_chirho in 0..8 {
            let store_clone_chirho = Arc::clone(&store_chirho);
            handles_chirho.push(thread::spawn(move || {
                let mut ids_chirho = vec![];
                for i in 0..1000 {
                    // Each thread interns overlapping values
                    let val_chirho = (thread_id_chirho * 100 + i % 200) as i64;
                    ids_chirho.push(store_clone_chirho.int_chirho(val_chirho));
                }
                ids_chirho
            }));
        }

        // Collect results
        let mut all_ids_chirho = vec![];
        for handle_chirho in handles_chirho {
            all_ids_chirho.extend(handle_chirho.join().unwrap());
        }

        // Verify hash-consing: same values should have same IDs
        let id_42_a_chirho = store_chirho.int_chirho(42);
        let id_42_b_chirho = store_chirho.int_chirho(42);
        assert_eq!(id_42_a_chirho, id_42_b_chirho);

        // Verify no duplicate terms
        let store_len_chirho = store_chirho.len_chirho();
        println!("Interned {} unique terms from 8000 operations", store_len_chirho);
        assert!(store_len_chirho < 8000); // Hash consing should reduce count
    }

    #[test]
    fn test_parallel_fresh_var_chirho() {
        let store_chirho = Arc::new(TermStoreSyncChirho::new_chirho());
        let mut handles_chirho = vec![];

        // Spawn 4 threads, each creating 100 fresh variables
        for _ in 0..4 {
            let store_clone_chirho = Arc::clone(&store_chirho);
            handles_chirho.push(thread::spawn(move || {
                let mut var_ids_chirho = vec![];
                for _ in 0..100 {
                    let (var_id_chirho, _) = store_clone_chirho.fresh_var_chirho();
                    var_ids_chirho.push(var_id_chirho);
                }
                var_ids_chirho
            }));
        }

        // Collect all variable IDs
        let mut all_var_ids_chirho = vec![];
        for handle_chirho in handles_chirho {
            all_var_ids_chirho.extend(handle_chirho.join().unwrap());
        }

        // All 400 variable IDs should be unique
        all_var_ids_chirho.sort();
        all_var_ids_chirho.dedup();
        assert_eq!(all_var_ids_chirho.len(), 400, "All var IDs should be unique");
    }
}

// Soli Deo Gloria ☧
