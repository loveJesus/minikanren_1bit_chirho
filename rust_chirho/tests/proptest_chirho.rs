//! Property-based tests ☧
//!
//! Kmett-approved: test algebraic laws, not just examples.
//! These tests find edge cases humans miss.

use proptest::prelude::*;
use minikanren_1bit_chirho::{
    UnionFindChirho,
    BitMatrix64Chirho,
    Word64Chirho,
};

// === Union-Find Properties ===

proptest! {
    /// Reflexivity: find(x) after make_set(x) returns x's equivalence class
    #[test]
    fn prop_union_find_reflexive_chirho(x_chirho in 0u32..1000) {
        let mut uf_chirho = UnionFindChirho::new();
        uf_chirho.make_set_chirho(x_chirho);
        let root_chirho = uf_chirho.find_chirho(x_chirho);
        // Root should be consistent
        prop_assert_eq!(uf_chirho.find_chirho(x_chirho), root_chirho);
    }

    /// Symmetry: if x ~ y, then y ~ x
    #[test]
    fn prop_union_find_symmetric_chirho(x_chirho in 0u32..100, y_chirho in 0u32..100) {
        let mut uf_chirho = UnionFindChirho::new();
        uf_chirho.union_chirho(x_chirho, y_chirho);
        prop_assert_eq!(
            uf_chirho.same_class_chirho(x_chirho, y_chirho),
            uf_chirho.same_class_chirho(y_chirho, x_chirho)
        );
    }

    /// Transitivity: if x ~ y and y ~ z, then x ~ z
    #[test]
    fn prop_union_find_transitive_chirho(
        x_chirho in 0u32..50,
        y_chirho in 0u32..50,
        z_chirho in 0u32..50
    ) {
        let mut uf_chirho = UnionFindChirho::new();
        uf_chirho.union_chirho(x_chirho, y_chirho);
        uf_chirho.union_chirho(y_chirho, z_chirho);
        prop_assert!(uf_chirho.same_class_chirho(x_chirho, z_chirho));
    }

    /// Idempotence: union(x, x) doesn't change anything
    #[test]
    fn prop_union_find_idempotent_chirho(x_chirho in 0u32..100) {
        let mut uf_chirho = UnionFindChirho::new();
        uf_chirho.make_set_chirho(x_chirho);
        let before_chirho = uf_chirho.find_chirho(x_chirho);
        uf_chirho.union_chirho(x_chirho, x_chirho);
        let after_chirho = uf_chirho.find_chirho(x_chirho);
        prop_assert_eq!(before_chirho, after_chirho);
    }
}

// === Bit Matrix Properties ===

proptest! {
    /// AND is commutative: A & B = B & A
    #[test]
    fn prop_bitmatrix_and_commutative_chirho(
        bits_a_chirho in prop::collection::vec(any::<bool>(), 64),
        bits_b_chirho in prop::collection::vec(any::<bool>(), 64)
    ) {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(8, 8);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(8, 8);

        for (i_chirho, &val_chirho) in bits_a_chirho.iter().enumerate() {
            if val_chirho {
                a_chirho.set_chirho((i_chirho / 8) as u8, (i_chirho % 8) as u8);
            }
        }
        for (i_chirho, &val_chirho) in bits_b_chirho.iter().enumerate() {
            if val_chirho {
                b_chirho.set_chirho((i_chirho / 8) as u8, (i_chirho % 8) as u8);
            }
        }

        let ab_chirho = a_chirho.and_chirho(&b_chirho);
        let ba_chirho = b_chirho.and_chirho(&a_chirho);

        prop_assert_eq!(ab_chirho, ba_chirho);
    }

    /// OR is commutative: A | B = B | A
    #[test]
    fn prop_bitmatrix_or_commutative_chirho(
        bits_a_chirho in prop::collection::vec(any::<bool>(), 64),
        bits_b_chirho in prop::collection::vec(any::<bool>(), 64)
    ) {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(8, 8);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(8, 8);

        for (i_chirho, &val_chirho) in bits_a_chirho.iter().enumerate() {
            if val_chirho {
                a_chirho.set_chirho((i_chirho / 8) as u8, (i_chirho % 8) as u8);
            }
        }
        for (i_chirho, &val_chirho) in bits_b_chirho.iter().enumerate() {
            if val_chirho {
                b_chirho.set_chirho((i_chirho / 8) as u8, (i_chirho % 8) as u8);
            }
        }

        let ab_chirho = a_chirho.or_chirho(&b_chirho);
        let ba_chirho = b_chirho.or_chirho(&a_chirho);

        prop_assert_eq!(ab_chirho, ba_chirho);
    }

    /// AND with self is identity: A & A = A
    #[test]
    fn prop_bitmatrix_and_idempotent_chirho(
        bits_chirho in prop::collection::vec(any::<bool>(), 64)
    ) {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(8, 8);
        for (i_chirho, &val_chirho) in bits_chirho.iter().enumerate() {
            if val_chirho {
                a_chirho.set_chirho((i_chirho / 8) as u8, (i_chirho % 8) as u8);
            }
        }

        let aa_chirho = a_chirho.and_chirho(&a_chirho);
        prop_assert_eq!(aa_chirho, a_chirho);
    }

    /// De Morgan: NOT(A AND B) = NOT(A) OR NOT(B)
    /// We test via popcount since we don't have NOT for matrices yet
    #[test]
    fn prop_word64_demorgan_chirho(a_chirho in any::<u64>(), b_chirho in any::<u64>()) {
        let wa_chirho = Word64Chirho::new_chirho(a_chirho);
        let wb_chirho = Word64Chirho::new_chirho(b_chirho);

        let lhs_chirho = !(wa_chirho & wb_chirho);
        let rhs_chirho = (!wa_chirho) | (!wb_chirho);

        prop_assert_eq!(lhs_chirho, rhs_chirho);
    }

    /// Transitive closure is idempotent: TC(TC(A)) = TC(A)
    #[test]
    fn prop_transitive_closure_idempotent_chirho(
        edges_chirho in prop::collection::vec((0u8..8, 0u8..8), 0..20)
    ) {
        let mut adj_chirho = BitMatrix64Chirho::new_chirho(8, 8);
        for (src_chirho, dst_chirho) in edges_chirho {
            adj_chirho.set_chirho(src_chirho, dst_chirho);
        }

        let tc1_chirho = adj_chirho.transitive_closure_chirho();
        let tc2_chirho = tc1_chirho.transitive_closure_chirho();

        prop_assert_eq!(tc1_chirho, tc2_chirho);
    }

    /// Transitive closure contains original: A ⊆ TC(A)
    #[test]
    fn prop_transitive_closure_contains_original_chirho(
        edges_chirho in prop::collection::vec((0u8..8, 0u8..8), 0..20)
    ) {
        let mut adj_chirho = BitMatrix64Chirho::new_chirho(8, 8);
        for (src_chirho, dst_chirho) in &edges_chirho {
            adj_chirho.set_chirho(*src_chirho, *dst_chirho);
        }

        let tc_chirho = adj_chirho.transitive_closure_chirho();

        // Every edge in original should be in TC
        for (src_chirho, dst_chirho) in edges_chirho {
            prop_assert!(tc_chirho.get_chirho(src_chirho, dst_chirho));
        }
    }
}

// === Boolean Semiring Laws ===

proptest! {
    /// Matrix multiply associativity: (A * B) * C = A * (B * C)
    #[test]
    fn prop_matmul_associative_chirho(
        edges_a_chirho in prop::collection::vec((0u8..4, 0u8..4), 0..8),
        edges_b_chirho in prop::collection::vec((0u8..4, 0u8..4), 0..8),
        edges_c_chirho in prop::collection::vec((0u8..4, 0u8..4), 0..8),
    ) {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        let mut b_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        let mut c_chirho = BitMatrix64Chirho::new_chirho(4, 4);

        for (r_chirho, c_chirho) in edges_a_chirho { a_chirho.set_chirho(r_chirho, c_chirho); }
        for (r_chirho, c_chirho) in edges_b_chirho { b_chirho.set_chirho(r_chirho, c_chirho); }
        for (r_chirho, col_chirho) in edges_c_chirho { c_chirho.set_chirho(r_chirho, col_chirho); }

        let ab_c_chirho = a_chirho.matmul_chirho(&b_chirho).matmul_chirho(&c_chirho);
        let a_bc_chirho = a_chirho.matmul_chirho(&b_chirho.matmul_chirho(&c_chirho));

        prop_assert_eq!(ab_c_chirho, a_bc_chirho);
    }

    /// Identity matrix is multiplicative identity: I * A = A = A * I
    #[test]
    fn prop_identity_matmul_chirho(
        edges_chirho in prop::collection::vec((0u8..4, 0u8..4), 0..10)
    ) {
        let mut a_chirho = BitMatrix64Chirho::new_chirho(4, 4);
        for (r_chirho, c_chirho) in edges_chirho {
            a_chirho.set_chirho(r_chirho, c_chirho);
        }

        let identity_chirho = BitMatrix64Chirho::identity_chirho(4);

        let ia_chirho = identity_chirho.matmul_chirho(&a_chirho);
        let ai_chirho = a_chirho.matmul_chirho(&identity_chirho);

        prop_assert_eq!(ia_chirho, a_chirho.clone());
        prop_assert_eq!(ai_chirho, a_chirho);
    }
}
