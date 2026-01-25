//! Example: Family relationships as relational logic ☧
//!
//! Demonstrates symbolic reasoning with bit-matrix operations.
//!
//! ## Key Concepts
//!
//! 1. **Facts as sparse tensors** — `parent(tom, mary)` → `pairs[(0,1)] = 1`
//! 2. **Grandparent via tensor contraction** — Boolean matrix multiply: `grandparent = parent ∘ parent`
//! 3. **Ancestor via transitive closure** — `ancestor = parent*` (fixed point iteration)
//! 4. **Bidirectional queries** — Same relation answers "who are X's descendants?" and "who are X's ancestors?"
//! 5. **Bit representation** — Tom's grandchildren = `0b00111000` (bits 3,4,5 = ann, pat, bob)
//!
//! ## Family Tree (two parents per child)
//!
//! ```text
//!   tom === jane        ed === fay
//!       |                   |
//!    +--+--+             +--+--+
//!    |     |             |     |
//!  mary  james ======= alice  joe
//!            |
//!         +--+--+
//!         |     |
//!        bob   sue
//! ```
//!
//! - bob has parents: james, alice
//! - bob has grandparents: tom, jane, ed, fay
//!
//! ## Run
//!
//! ```bash
//! cargo run --example family_chirho
//! ```

use std::collections::{HashMap, HashSet};

/// A relation represented as sparse (row, col) pairs
/// This is the 1-bit matrix representation!
#[derive(Debug, Clone)]
struct RelationChirho {
    name_chirho: String,
    pairs_chirho: HashSet<(u32, u32)>,
}

impl RelationChirho {
    fn new_chirho(name_chirho: &str) -> Self {
        Self {
            name_chirho: name_chirho.to_string(),
            pairs_chirho: HashSet::new(),
        }
    }

    /// Add a fact: relation(a, b)
    fn add_chirho(&mut self, a_chirho: u32, b_chirho: u32) {
        self.pairs_chirho.insert((a_chirho, b_chirho));
    }

    /// Query: relation(a, ?) - find all b where relation(a, b) holds
    fn query_first_chirho(&self, a_chirho: u32) -> Vec<u32> {
        self.pairs_chirho
            .iter()
            .filter(|(x, _)| *x == a_chirho)
            .map(|(_, y)| *y)
            .collect()
    }

    /// Query: relation(?, b) - find all a where relation(a, b) holds
    fn query_second_chirho(&self, b_chirho: u32) -> Vec<u32> {
        self.pairs_chirho
            .iter()
            .filter(|(_, y)| *y == b_chirho)
            .map(|(x, _)| *x)
            .collect()
    }

    /// Compose two relations: (R1 ∘ R2)(a, c) = ∃b: R1(a, b) ∧ R2(b, c)
    /// This is Boolean matrix multiplication!
    fn compose_chirho(&self, other_chirho: &RelationChirho) -> RelationChirho {
        let mut result_chirho = RelationChirho::new_chirho(&format!(
            "({} ∘ {})",
            self.name_chirho, other_chirho.name_chirho
        ));

        for (a_chirho, b_chirho) in &self.pairs_chirho {
            for (b2_chirho, c_chirho) in &other_chirho.pairs_chirho {
                if b_chirho == b2_chirho {
                    // Found a path: a -> b -> c
                    result_chirho.add_chirho(*a_chirho, *c_chirho);
                }
            }
        }

        result_chirho
    }

    /// Union of two relations: (R1 ∪ R2)(a, b) = R1(a, b) ∨ R2(a, b)
    fn union_chirho(&self, other_chirho: &RelationChirho) -> RelationChirho {
        let mut result_chirho = RelationChirho::new_chirho(&format!(
            "({} ∪ {})",
            self.name_chirho, other_chirho.name_chirho
        ));
        result_chirho.pairs_chirho = self.pairs_chirho.union(&other_chirho.pairs_chirho).cloned().collect();
        result_chirho
    }

    /// Transitive closure: R* = R ∪ R∘R ∪ R∘R∘R ∪ ...
    fn transitive_closure_chirho(&self) -> RelationChirho {
        let mut result_chirho = self.clone();
        result_chirho.name_chirho = format!("{}*", self.name_chirho);

        loop {
            let extended_chirho = result_chirho.union_chirho(&result_chirho.compose_chirho(&self));
            if extended_chirho.pairs_chirho.len() == result_chirho.pairs_chirho.len() {
                break; // Fixed point reached
            }
            result_chirho = extended_chirho;
        }

        result_chirho
    }
}

fn main() {
    println!("miniKanren as 1-Bit Matrix Operations ☧\n");
    println!("Example: Family relationships as sparse Boolean tensors\n");

    // === Define the symbol table ===
    let mut names_chirho: HashMap<u32, &str> = HashMap::new();
    let mut ids_chirho: HashMap<&str, u32> = HashMap::new();

    let people_chirho = [
        "tom", "jane", "ed", "fay", "mary", "james", "alice", "joe", "bob", "sue",
    ];

    for (i_chirho, name_chirho) in people_chirho.iter().enumerate() {
        names_chirho.insert(i_chirho as u32, name_chirho);
        ids_chirho.insert(name_chirho, i_chirho as u32);
    }

    // Helper to get ID
    let id_chirho = |name_chirho: &str| -> u32 { *ids_chirho.get(name_chirho).unwrap() };
    let name_chirho = |id_chirho: u32| -> &str { names_chirho.get(&id_chirho).unwrap_or(&"?") };

    // === Define the parent relation as a sparse matrix ===
    //
    //   tom === jane        ed === fay
    //       |                   |
    //    +--+--+             +--+--+
    //    |     |             |     |
    //  mary  james ======= alice  joe
    //            |
    //         +--+--+
    //         |     |
    //        bob   sue

    let mut parent_chirho = RelationChirho::new_chirho("parent");

    // parent(X, Y) means X is a parent of Y
    // Tom & Jane's children
    parent_chirho.add_chirho(id_chirho("tom"), id_chirho("mary"));
    parent_chirho.add_chirho(id_chirho("jane"), id_chirho("mary"));
    parent_chirho.add_chirho(id_chirho("tom"), id_chirho("james"));
    parent_chirho.add_chirho(id_chirho("jane"), id_chirho("james"));

    // Ed & Fay's children
    parent_chirho.add_chirho(id_chirho("ed"), id_chirho("alice"));
    parent_chirho.add_chirho(id_chirho("fay"), id_chirho("alice"));
    parent_chirho.add_chirho(id_chirho("ed"), id_chirho("joe"));
    parent_chirho.add_chirho(id_chirho("fay"), id_chirho("joe"));

    // James & Alice's children (bob and sue have TWO parents each!)
    parent_chirho.add_chirho(id_chirho("james"), id_chirho("bob"));
    parent_chirho.add_chirho(id_chirho("alice"), id_chirho("bob"));
    parent_chirho.add_chirho(id_chirho("james"), id_chirho("sue"));
    parent_chirho.add_chirho(id_chirho("alice"), id_chirho("sue"));

    println!("=== Parent relation (sparse 2D tensor) ===");
    println!("parent = {{");
    for (a_chirho, b_chirho) in &parent_chirho.pairs_chirho {
        println!("  ({}, {}),  // {} is parent of {}", a_chirho, b_chirho, name_chirho(*a_chirho), name_chirho(*b_chirho));
    }
    println!("}}\n");

    // === Query: Who are tom's children? ===
    println!("=== Query: parent(tom, ?) ===");
    let toms_children_chirho = parent_chirho.query_first_chirho(id_chirho("tom"));
    print!("Tom's children: ");
    for c_chirho in &toms_children_chirho {
        print!("{} ", name_chirho(*c_chirho));
    }
    println!("\n");

    // === Grandparent = parent ∘ parent (tensor contraction!) ===
    println!("=== Grandparent via tensor contraction ===");
    println!("grandparent(X, Z) = ∃Y: parent(X, Y) ∧ parent(Y, Z)");
    println!("This is Boolean matrix multiplication!\n");

    let grandparent_chirho = parent_chirho.compose_chirho(&parent_chirho);

    println!("grandparent = {{");
    for (a_chirho, b_chirho) in &grandparent_chirho.pairs_chirho {
        println!("  ({}, {}),  // {} is grandparent of {}", a_chirho, b_chirho, name_chirho(*a_chirho), name_chirho(*b_chirho));
    }
    println!("}}\n");

    // === Query: Who are tom's grandchildren? (going DOWN the tree) ===
    println!("=== Query: grandparent(tom, ?) - going DOWN ===");
    let toms_grandchildren_chirho = grandparent_chirho.query_first_chirho(id_chirho("tom"));
    print!("Tom's grandchildren: ");
    for c_chirho in &toms_grandchildren_chirho {
        print!("{} ", name_chirho(*c_chirho));
    }
    println!("\n");

    // === Query: Who are bob's grandparents? (going UP the tree) ===
    println!("=== Query: grandparent(?, bob) - going UP ===");
    let bobs_grandparents_chirho = grandparent_chirho.query_second_chirho(id_chirho("bob"));
    print!("Bob's grandparents: ");
    for g_chirho in &bobs_grandparents_chirho {
        print!("{} ", name_chirho(*g_chirho));
    }
    println!("  (4 grandparents - two from each side!)\n");

    // === Query: Who are bob's parents? ===
    println!("=== Query: parent(?, bob) - bob has TWO parents ===");
    let bobs_parents_chirho = parent_chirho.query_second_chirho(id_chirho("bob"));
    print!("Bob's parents: ");
    for p_chirho in &bobs_parents_chirho {
        print!("{} ", name_chirho(*p_chirho));
    }
    println!("\n");

    // === Ancestor = transitive closure of parent ===
    println!("=== Ancestor via transitive closure ===");
    println!("ancestor = parent*  (reflexive transitive closure)\n");

    let ancestor_chirho = parent_chirho.transitive_closure_chirho();

    println!("ancestor = {{");
    for (a_chirho, b_chirho) in &ancestor_chirho.pairs_chirho {
        println!("  ({}, {}),  // {} is ancestor of {}", a_chirho, b_chirho, name_chirho(*a_chirho), name_chirho(*b_chirho));
    }
    println!("}}\n");

    // === Query: Who are tom's descendants? ===
    println!("=== Query: ancestor(tom, ?) - all of tom's descendants ===");
    let toms_descendants_chirho = ancestor_chirho.query_first_chirho(id_chirho("tom"));
    print!("Tom's descendants: ");
    for c_chirho in &toms_descendants_chirho {
        print!("{} ", name_chirho(*c_chirho));
    }
    println!("\n");

    // === Backward query: Who are bob's ancestors? ===
    println!("=== Query: ancestor(?, bob) - all of bob's ancestors ===");
    let bobs_ancestors_chirho = ancestor_chirho.query_second_chirho(id_chirho("bob"));
    print!("Bob's ancestors: ");
    for c_chirho in &bobs_ancestors_chirho {
        print!("{} ", name_chirho(*c_chirho));
    }
    println!("  (6 ancestors - parents + all 4 grandparents!)\n");

    // === Show the bit-matrix representation ===
    println!("=== Bit-matrix representation ===");
    println!("Each relation is a sparse Boolean matrix:");
    println!("  - Rows = first argument");
    println!("  - Cols = second argument");
    println!("  - Entry[i,j] = 1 iff relation(i, j) holds");
    println!();
    println!("Composition = Boolean matrix multiply (OR for +, AND for *)");
    println!("Query = Row/column selection");
    println!("Transitive closure = Repeated squaring until fixpoint");
    println!();

    // === Demonstrate with actual bit operations ===
    println!("=== Domain representation ===");
    // Set the bits directly - this IS the 1-bit matrix representation!
    let grandparents_bits_chirho: u64 = (1 << id_chirho("tom"))
        | (1 << id_chirho("jane"))
        | (1 << id_chirho("ed"))
        | (1 << id_chirho("fay"));
    println!("Bob's grandparents as bitmask: 0b{:016b}", grandparents_bits_chirho);
    println!("  bit {} = tom", id_chirho("tom"));
    println!("  bit {} = jane", id_chirho("jane"));
    println!("  bit {} = ed", id_chirho("ed"));
    println!("  bit {} = fay", id_chirho("fay"));
    println!();
    println!("Query 'grandparent(?, bob)' = select column 'bob' from grandparent matrix");
    println!("Result = bitmask with 1s for each grandparent!");

    println!("\n☧ Soli Deo Gloria ☧");
}
