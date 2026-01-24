//! Benchmarks for miniKanren 1-bit operations ☧
//!
//! Run with: cargo bench

#![feature(test)]
extern crate test;

use test::Bencher;
use minikanren_1bit_chirho::*;

#[bench]
fn bench_unify_ints_chirho(b_chirho: &mut Bencher) {
    let mut store_chirho = TermStoreChirho::new();
    let a_chirho = store_chirho.int_chirho(42);
    let b_term_chirho = store_chirho.int_chirho(42);

    b_chirho.iter(|| {
        let mut subst_chirho = SubstChirho::new();
        unify_chirho(a_chirho, b_term_chirho, &mut subst_chirho, &store_chirho)
    });
}

#[bench]
fn bench_unify_vars_chirho(b_chirho: &mut Bencher) {
    let mut store_chirho = TermStoreChirho::new();
    let (_, x_chirho) = store_chirho.fresh_var_chirho();
    let val_chirho = store_chirho.int_chirho(42);

    b_chirho.iter(|| {
        let mut subst_chirho = SubstChirho::new();
        unify_chirho(x_chirho, val_chirho, &mut subst_chirho, &store_chirho)
    });
}

#[bench]
fn bench_unify_lists_chirho(b_chirho: &mut Bencher) {
    let mut store_chirho = TermStoreChirho::new();
    let list1_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5]);
    let list2_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5]);

    b_chirho.iter(|| {
        let mut subst_chirho = SubstChirho::new();
        unify_chirho(list1_chirho, list2_chirho, &mut subst_chirho, &store_chirho)
    });
}

#[bench]
fn bench_domain_and_chirho(b_chirho: &mut Bencher) {
    let d1_chirho = BitVec64Chirho(0xFFFF_0000_FFFF_0000);
    let d2_chirho = BitVec64Chirho(0x0F0F_0F0F_0F0F_0F0F);

    b_chirho.iter(|| {
        d1_chirho.and_chirho(d2_chirho)
    });
}

#[bench]
fn bench_domain_popcount_chirho(b_chirho: &mut Bencher) {
    let d_chirho = BitVec64Chirho(0xDEAD_BEEF_CAFE_BABE);

    b_chirho.iter(|| {
        d_chirho.popcount_chirho()
    });
}

#[bench]
fn bench_union_find_100_chirho(b_chirho: &mut Bencher) {
    b_chirho.iter(|| {
        let mut uf_chirho = UnionFindChirho::new();
        for i_chirho in 0..100u32 {
            uf_chirho.make_set_chirho(i_chirho);
        }
        for i_chirho in 0..99u32 {
            uf_chirho.union_chirho(i_chirho, i_chirho + 1);
        }
        uf_chirho.find_chirho(0)
    });
}

#[bench]
fn bench_constraint_propagate_chirho(b_chirho: &mut Bencher) {
    b_chirho.iter(|| {
        let mut store_chirho = ConstraintStoreChirho::new();

        // 4 variables with overlapping domains
        store_chirho.add_var_chirho(0, DomainChirho(0b1111_1111));
        store_chirho.add_var_chirho(1, DomainChirho(0b0000_1111));
        store_chirho.add_var_chirho(2, DomainChirho(0b1111_0000));
        store_chirho.add_var_chirho(3, DomainChirho(0b0011_1100));

        // Equality constraints
        store_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(0, 1, 8));
        store_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(1, 2, 8));
        store_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(2, 3, 8));

        store_chirho.propagate_chirho()
    });
}

#[bench]
fn bench_conde_3_branches_chirho(b_chirho: &mut Bencher) {
    let mut store_chirho = TermStoreChirho::new();
    let (_, x_chirho) = store_chirho.fresh_var_chirho();
    let one_chirho = store_chirho.int_chirho(1);
    let two_chirho = store_chirho.int_chirho(2);
    let three_chirho = store_chirho.int_chirho(3);

    let goal_chirho = conde_chirho(vec![
        vec![eq_chirho(x_chirho, one_chirho)],
        vec![eq_chirho(x_chirho, two_chirho)],
        vec![eq_chirho(x_chirho, three_chirho)],
    ]);

    b_chirho.iter(|| {
        run_chirho(10, x_chirho, goal_chirho.clone(), &store_chirho)
    });
}

#[bench]
fn bench_search_state_hw_unify_chirho(b_chirho: &mut Bencher) {
    b_chirho.iter(|| {
        let mut state_chirho = SearchStateHwChirho::<8>::new_chirho();
        state_chirho.set_domain_chirho(0, BitVec64Chirho(0xFF00));
        state_chirho.set_domain_chirho(1, BitVec64Chirho(0x0FF0));
        state_chirho.unify_chirho(0, 1);
        state_chirho.domains_chirho[0]
    });
}

#[bench]
fn bench_cam_lookup_chirho(b_chirho: &mut Bencher) {
    let mut cam_chirho = CamHwChirho::<64>::new_chirho();

    // Fill CAM with relation data
    for i_chirho in 0..60u32 {
        cam_chirho.insert_chirho(i_chirho % 10, i_chirho);
    }

    b_chirho.iter(|| {
        cam_chirho.lookup_chirho(5)
    });
}

#[bench]
fn bench_weighted_matmul_count_chirho(b_chirho: &mut Bencher) {
    let mut a_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(10, 10);
    let mut b_mat_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(10, 10);

    for i_chirho in 0..10u32 {
        for j_chirho in 0..10u32 {
            if (i_chirho + j_chirho) % 3 == 0 {
                a_chirho.set_chirho(i_chirho, j_chirho, CountSemiringChirho(1));
            }
            if (i_chirho * j_chirho) % 2 == 0 {
                b_mat_chirho.set_chirho(i_chirho, j_chirho, CountSemiringChirho(1));
            }
        }
    }

    b_chirho.iter(|| {
        a_chirho.matmul_chirho(&b_mat_chirho)
    });
}
