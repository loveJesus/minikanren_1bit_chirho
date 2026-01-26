// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Type-safe ID wrappers ☧
//!
//! Newtype wrappers to prevent mixing up different kinds of IDs.
//! Kmett would approve: make illegal states unrepresentable.

use std::fmt;
use std::hash::Hash;

/// Macro to generate newtype ID wrappers with full _chirho compliance
macro_rules! define_id_chirho {
    ($name_chirho:ident, $doc_chirho:expr) => {
        #[doc = $doc_chirho]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[repr(transparent)]
        pub struct $name_chirho(pub u32);

        impl $name_chirho {
            #[inline]
            pub const fn new_chirho(val_chirho: u32) -> Self {
                Self(val_chirho)
            }

            #[inline]
            pub const fn raw_chirho(self) -> u32 {
                self.0
            }

            #[inline]
            pub const fn as_usize_chirho(self) -> usize {
                self.0 as usize
            }
        }

        impl fmt::Debug for $name_chirho {
            fn fmt(&self, f_chirho: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f_chirho, "{}({})", stringify!($name_chirho), self.0)
            }
        }

        impl fmt::Display for $name_chirho {
            fn fmt(&self, f_chirho: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f_chirho, "{}", self.0)
            }
        }

        impl From<u32> for $name_chirho {
            fn from(val_chirho: u32) -> Self {
                Self(val_chirho)
            }
        }

        impl From<usize> for $name_chirho {
            fn from(val_chirho: usize) -> Self {
                Self(val_chirho as u32)
            }
        }
    };
}

// Define all ID types
define_id_chirho!(TermIdChirhoSafe, "Term identifier (index into term store)");
define_id_chirho!(VarIdChirho, "Logic variable identifier");
define_id_chirho!(EClassIdChirhoSafe, "E-class identifier (equivalence class)");
define_id_chirho!(ENodeIdChirhoSafe, "E-node identifier (index into nodes array)");
define_id_chirho!(SymIdChirho, "Symbol identifier (interned string)");
define_id_chirho!(GoalIdChirho, "Goal identifier for tabling");
define_id_chirho!(TensorIdChirho, "Tensor identifier in network");

/// Type-safe index into a vector
/// Prevents indexing a Vec<A> with an ID meant for Vec<B>
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedIndexChirho<T> {
    idx_chirho: u32,
    _marker_chirho: std::marker::PhantomData<T>,
}

impl<T> TypedIndexChirho<T> {
    pub const fn new_chirho(idx_chirho: u32) -> Self {
        Self {
            idx_chirho,
            _marker_chirho: std::marker::PhantomData,
        }
    }

    pub const fn raw_chirho(self) -> u32 {
        self.idx_chirho
    }

    pub const fn as_usize_chirho(self) -> usize {
        self.idx_chirho as usize
    }
}

impl<T> fmt::Debug for TypedIndexChirho<T> {
    fn fmt(&self, f_chirho: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f_chirho, "Index<{}>({})", std::any::type_name::<T>(), self.idx_chirho)
    }
}

/// Type-safe wrapper for vectors that can only be indexed by correct ID type
pub struct TypedVecChirho<I, T> {
    inner_chirho: Vec<T>,
    _id_marker_chirho: std::marker::PhantomData<I>,
}

impl<I, T> TypedVecChirho<I, T> {
    pub fn new_chirho() -> Self {
        Self {
            inner_chirho: Vec::new(),
            _id_marker_chirho: std::marker::PhantomData,
        }
    }

    pub fn with_capacity_chirho(cap_chirho: usize) -> Self {
        Self {
            inner_chirho: Vec::with_capacity(cap_chirho),
            _id_marker_chirho: std::marker::PhantomData,
        }
    }

    pub fn push_chirho(&mut self, val_chirho: T) -> TypedIndexChirho<I> {
        let idx_chirho = self.inner_chirho.len() as u32;
        self.inner_chirho.push(val_chirho);
        TypedIndexChirho::new_chirho(idx_chirho)
    }

    pub fn get_chirho(&self, idx_chirho: TypedIndexChirho<I>) -> Option<&T> {
        self.inner_chirho.get(idx_chirho.as_usize_chirho())
    }

    pub fn get_mut_chirho(&mut self, idx_chirho: TypedIndexChirho<I>) -> Option<&mut T> {
        self.inner_chirho.get_mut(idx_chirho.as_usize_chirho())
    }

    pub fn len_chirho(&self) -> usize {
        self.inner_chirho.len()
    }

    pub fn is_empty_chirho(&self) -> bool {
        self.inner_chirho.is_empty()
    }

    pub fn iter_chirho(&self) -> impl Iterator<Item = &T> {
        self.inner_chirho.iter()
    }
}

impl<I, T> Default for TypedVecChirho<I, T> {
    fn default() -> Self {
        Self::new_chirho()
    }
}

impl<I, T: Clone> Clone for TypedVecChirho<I, T> {
    fn clone(&self) -> Self {
        Self {
            inner_chirho: self.inner_chirho.clone(),
            _id_marker_chirho: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_id_types_distinct_chirho() {
        let term_id_chirho = TermIdChirhoSafe::new_chirho(42);
        let var_id_chirho = VarIdChirho::new_chirho(42);

        // Same raw value but different types - cannot be compared at compile time
        assert_eq!(term_id_chirho.raw_chirho(), var_id_chirho.raw_chirho());

        // This would not compile (good!):
        // assert_eq!(term_id_chirho, var_id_chirho);
    }

    #[test]
    fn test_typed_vec_chirho() {
        struct NodeMarkerChirho;
        struct ClassMarkerChirho;

        let mut nodes_chirho: TypedVecChirho<NodeMarkerChirho, String> = TypedVecChirho::new_chirho();
        let mut classes_chirho: TypedVecChirho<ClassMarkerChirho, i32> = TypedVecChirho::new_chirho();

        let node_idx_chirho = nodes_chirho.push_chirho("hello".to_string());
        let class_idx_chirho = classes_chirho.push_chirho(42);

        // This works:
        assert_eq!(nodes_chirho.get_chirho(node_idx_chirho), Some(&"hello".to_string()));
        assert_eq!(classes_chirho.get_chirho(class_idx_chirho), Some(&42));

        // This would not compile (good!):
        // nodes_chirho.get_chirho(class_idx_chirho);
        // classes_chirho.get_chirho(node_idx_chirho);
    }
}
