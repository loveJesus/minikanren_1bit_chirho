# Infinite Domains for Hardware ☧

## The Problem

`BitVec64Chirho` supports 64 values. Real miniKanren needs infinite domains (any integer, any list, any term).

## Option 1: Hierarchical Bit Vectors

```rust
/// Domain over arbitrary integers using paged bit vectors
pub struct PagedDomainChirho {
    // Sparse pages: only allocate pages that have bits set
    pages_chirho: HashMap<u64, BitVec64Chirho>,  // page_num → bits
    // page 0 = values 0-63
    // page 1 = values 64-127
    // etc.
}

impl PagedDomainChirho {
    fn contains_chirho(&self, value_chirho: u64) -> bool {
        let page_chirho = value_chirho / 64;
        let bit_chirho = value_chirho % 64;
        self.pages_chirho.get(&page_chirho)
            .map_or(false, |p| p.test_chirho(bit_chirho as u32))
    }

    fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        // Only intersect pages that exist in both
        let mut result_chirho = HashMap::new();
        for (&page_chirho, bits_chirho) in &self.pages_chirho {
            if let Some(other_bits_chirho) = other_chirho.pages_chirho.get(&page_chirho) {
                let intersection_chirho = bits_chirho.and_chirho(*other_bits_chirho);
                if !intersection_chirho.is_zero_chirho() {
                    result_chirho.insert(page_chirho, intersection_chirho);
                }
            }
        }
        Self { pages_chirho: result_chirho }
    }
}
```

**Pros:** Simple, fast AND on existing pages
**Cons:** Still finite (just larger), O(pages) intersection

---

## Option 2: Complement Representation

```rust
/// Domain can be finite or cofinite (all except finite set)
pub enum DomainChirho {
    /// Finite set of values
    Finite(HashSet<u64>),
    /// All values EXCEPT these (cofinite)
    Cofinite(HashSet<u64>),
    /// Everything
    All,
    /// Nothing
    Empty,
}

impl DomainChirho {
    fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        match (self, other_chirho) {
            (All, x) | (x, All) => x.clone(),
            (Empty, _) | (_, Empty) => Empty,
            (Finite(a), Finite(b)) => {
                Finite(a.intersection(b).copied().collect())
            }
            (Finite(a), Cofinite(b)) => {
                // a ∩ (ALL - b) = a - b
                Finite(a.difference(b).copied().collect())
            }
            (Cofinite(a), Cofinite(b)) => {
                // (ALL - a) ∩ (ALL - b) = ALL - (a ∪ b)
                Cofinite(a.union(b).copied().collect())
            }
            // ... symmetric cases
        }
    }
}
```

**Pros:** Can represent "all integers except 5" efficiently
**Cons:** Loses bit-parallel speedup, back to set operations

---

## Option 3: Symbolic Domains

```rust
/// Domain as symbolic constraint
pub enum SymbolicDomainChirho {
    /// Concrete set (small)
    Concrete(BitVec64Chirho),
    /// Range [lo, hi]
    Range { lo_chirho: i64, hi_chirho: i64 },
    /// Modular: x ≡ r (mod m)
    Modular { remainder_chirho: i64, modulus_chirho: i64 },
    /// Union of domains
    Union(Vec<SymbolicDomainChirho>),
    /// Intersection (lazy)
    Intersect(Box<SymbolicDomainChirho>, Box<SymbolicDomainChirho>),
}
```

**Pros:** Can represent infinite sets symbolically
**Cons:** Intersection may not simplify, need solver

---

## Option 4: Lazy Enumeration (Current Bridge)

The reference implementation already does this:

```rust
// Terms are created on-demand
let term_id = store.intern_chirho(ConsChirho(head_id, tail_id));

// Domain is "all term IDs created so far"
// Plus: can create new terms during search
```

To bridge to hardware:
1. Run reference until domains stabilize
2. Extract finite domain as BitVec
3. Run hardware propagation
4. If new terms needed, expand and repeat

```rust
fn hybrid_search_chirho(goal_chirho: GoalChirho) -> Vec<SubstChirho> {
    let mut store_chirho = TermStoreChirho::new();

    loop {
        // Get current term IDs
        let max_id_chirho = store_chirho.len();

        // Run hardware propagation on finite domain [0, max_id)
        let hw_results_chirho = hardware_propagate_chirho(&store_chirho, &goal_chirho);

        // Check if any result needs new terms
        let needs_expansion_chirho = hw_results_chirho.iter()
            .any(|r| r.needs_fresh_term_chirho());

        if !needs_expansion_chirho {
            return hw_results_chirho;
        }

        // Expand: create demanded terms
        expand_demanded_terms_chirho(&mut store_chirho, &hw_results_chirho);
    }
}
```

---

## Option 5: Type-Directed Domains

Different variables have different domain types:

```rust
pub enum TypedDomainChirho {
    /// Small integers: use BitVec64
    SmallInt(BitVec64Chirho),
    /// Large integers: use paged
    LargeInt(PagedDomainChirho),
    /// Booleans: 2 bits
    Bool(u8),  // bit 0 = false possible, bit 1 = true possible
    /// Finite enum: use BitVec
    Enum { size_chirho: u32, bits_chirho: BitVec64Chirho },
    /// List of T: structural
    List { element_domain_chirho: Box<TypedDomainChirho> },
    /// Any term: fall back to reference
    Term,
}
```

---

## Recommended Approach

**Short term:** Paged bit vectors (`PagedDomainChirho`) for larger finite domains

**Medium term:** Hybrid execution - hardware for finite, reference for infinite

**Long term:** Symbolic domains with SMT backend for true infinite reasoning

---

## How Sudoku/N-Queens Work Now

### Sudoku (9 values)
```
Each cell: domain ⊆ {1,2,3,4,5,6,7,8,9}
Represented as: bits 1-9 of a u16
Fits easily in BitVec64
```

### N-Queens (N columns)
```
NOT using SearchStateHwChirho!

Uses 3 bitmasks:
- col_mask: which columns have queens
- diag1_mask: which / diagonals attacked
- diag2_mask: which \ diagonals attacked

For N=32: needs 32 bits per mask → fits in u64
For N=64: needs 64 bits per mask → fits in u64
```

The N-Queens solver is a **specialized algorithm**, not the general miniKanren framework.

---

*Soli Deo Gloria* ☧
