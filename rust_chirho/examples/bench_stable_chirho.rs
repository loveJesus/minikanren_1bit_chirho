//! Benchmarks for stable Rust ☧
//!
//! Run with: cargo run --release --example bench_stable_chirho

use std::time::Instant;
use minikanren_1bit_chirho::*;

fn bench_chirho<F>(name_chirho: &str, iterations_chirho: u32, mut f_chirho: F)
where F: FnMut()
{
    // Warmup
    for _ in 0..100 { f_chirho(); }

    let start_chirho = Instant::now();
    for _ in 0..iterations_chirho { f_chirho(); }
    let elapsed_chirho = start_chirho.elapsed();

    let ns_per_op_chirho = elapsed_chirho.as_nanos() / iterations_chirho as u128;
    println!("{:45} {:>10} ns/op", name_chirho, ns_per_op_chirho);
}

fn main() {
    println!("miniKanren 1-bit Benchmarks ☧\n");
    println!("{:45} {:>10}", "Operation", "Time");
    println!("{:-<57}", "");

    // Domain operations (hardware-level)
    bench_chirho("BitVec64 AND (unify)", 1_000_000, || {
        let a_chirho = BitVec64Chirho(0xFFFF_0000_FFFF_0000);
        let b_chirho = BitVec64Chirho(0x0F0F_0F0F_0F0F_0F0F);
        std::hint::black_box(a_chirho.and_chirho(b_chirho));
    });

    bench_chirho("BitVec64 popcount", 1_000_000, || {
        let d_chirho = BitVec64Chirho(0xDEAD_BEEF_CAFE_BABE);
        std::hint::black_box(d_chirho.popcount_chirho());
    });

    bench_chirho("BitVec256 AND (unify)", 1_000_000, || {
        let a_chirho = BitVec256Chirho([0xFFFF_FFFF_FFFF_FFFF; 4]);
        let b_chirho = BitVec256Chirho([0x0F0F_0F0F_0F0F_0F0F; 4]);
        std::hint::black_box(a_chirho.and_chirho(b_chirho));
    });

    // Union-Find
    bench_chirho("Union-Find 100 unions + find", 10_000, || {
        let mut uf_chirho = UnionFindChirho::new();
        for i_chirho in 0..100u32 { uf_chirho.make_set_chirho(i_chirho); }
        for i_chirho in 0..99u32 { uf_chirho.union_chirho(i_chirho, i_chirho + 1); }
        std::hint::black_box(uf_chirho.find_chirho(0));
    });

    // Term operations
    bench_chirho("TermStore intern 100 ints", 10_000, || {
        let mut store_chirho = TermStoreChirho::new();
        for i_chirho in 0..100i64 {
            std::hint::black_box(store_chirho.int_chirho(i_chirho));
        }
    });

    let mut store_chirho = TermStoreChirho::new();
    let list1_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5]);
    let list2_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5]);

    bench_chirho("Unify equal lists (5 elem)", 100_000, || {
        let mut subst_chirho = SubstChirho::new();
        std::hint::black_box(unify_chirho(list1_chirho, list2_chirho, &mut subst_chirho, &store_chirho));
    });

    let list3_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let list4_chirho = store_chirho.list_ints_chirho(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    bench_chirho("Unify equal lists (10 elem)", 100_000, || {
        let mut subst_chirho = SubstChirho::new();
        std::hint::black_box(unify_chirho(list3_chirho, list4_chirho, &mut subst_chirho, &store_chirho));
    });

    // Goals
    let mut store2_chirho = TermStoreChirho::new();
    let (_, x_chirho) = store2_chirho.fresh_var_chirho();
    let one_chirho = store2_chirho.int_chirho(1);
    let two_chirho = store2_chirho.int_chirho(2);
    let three_chirho = store2_chirho.int_chirho(3);

    let goal3_chirho = conde_chirho(vec![
        vec![eq_chirho(x_chirho, one_chirho)],
        vec![eq_chirho(x_chirho, two_chirho)],
        vec![eq_chirho(x_chirho, three_chirho)],
    ]);

    bench_chirho("conde 3 branches, run 10", 10_000, || {
        std::hint::black_box(run_chirho(10, x_chirho, goal3_chirho.clone(), &store2_chirho));
    });

    // Larger conde
    let mut store3_chirho = TermStoreChirho::new();
    let (_, y_chirho) = store3_chirho.fresh_var_chirho();
    let vals_chirho: Vec<_> = (0..10).map(|i| store3_chirho.int_chirho(i)).collect();
    let clauses_chirho: Vec<Vec<_>> = vals_chirho.iter().map(|&v| vec![eq_chirho(y_chirho, v)]).collect();
    let goal10_chirho = conde_chirho(clauses_chirho);

    bench_chirho("conde 10 branches, run 20", 1_000, || {
        std::hint::black_box(run_chirho(20, y_chirho, goal10_chirho.clone(), &store3_chirho));
    });

    // Constraint propagation
    bench_chirho("AC-3 propagate (4 vars, 3 constraints)", 10_000, || {
        let mut cstore_chirho = ConstraintStoreChirho::new();
        cstore_chirho.add_var_chirho(0, DomainChirho(0b1111_1111));
        cstore_chirho.add_var_chirho(1, DomainChirho(0b0000_1111));
        cstore_chirho.add_var_chirho(2, DomainChirho(0b1111_0000));
        cstore_chirho.add_var_chirho(3, DomainChirho(0b0011_1100));
        cstore_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(0, 1, 8));
        cstore_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(1, 2, 8));
        cstore_chirho.add_constraint_chirho(BinaryConstraintChirho::equality_chirho(2, 3, 8));
        std::hint::black_box(cstore_chirho.propagate_chirho());
    });

    // Hardware search state
    bench_chirho("SearchStateHw<8> unify", 100_000, || {
        let mut state_chirho = SearchStateHwChirho::<8>::new_chirho();
        state_chirho.set_domain_chirho(0, BitVec64Chirho(0xFF00));
        state_chirho.set_domain_chirho(1, BitVec64Chirho(0x0FF0));
        state_chirho.unify_chirho(0, 1);
        std::hint::black_box(state_chirho.domains_chirho[0]);
    });

    bench_chirho("SearchState256Hw<8> unify", 100_000, || {
        let mut state_chirho = SearchState256HwChirho::<8>::new_chirho();
        state_chirho.set_domain_chirho(0, BitVec256Chirho([0xFF00, 0, 0, 0]));
        state_chirho.set_domain_chirho(1, BitVec256Chirho([0x0FF0, 0, 0, 0]));
        state_chirho.unify_chirho(0, 1);
        std::hint::black_box(state_chirho.domains_chirho[0]);
    });

    // CAM lookup
    let mut cam_chirho = CamHwChirho::<64>::new_chirho();
    for i_chirho in 0..60u32 { cam_chirho.insert_chirho(i_chirho % 10, i_chirho); }

    bench_chirho("CAM lookup (64 entries)", 100_000, || {
        std::hint::black_box(cam_chirho.lookup_chirho(5));
    });

    // Weighted matrix multiply
    bench_chirho("WeightedMatrix 10x10 matmul (count)", 1_000, || {
        let mut a_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(10, 10);
        let mut b_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(10, 10);
        for i in 0..10u32 {
            for j in 0..10u32 {
                if (i + j) % 3 == 0 { a_chirho.set_chirho(i, j, CountSemiringChirho(1)); }
                if (i * j) % 2 == 0 { b_chirho.set_chirho(i, j, CountSemiringChirho(1)); }
            }
        }
        std::hint::black_box(a_chirho.matmul_chirho(&b_chirho));
    });

    // Neural/soft operations
    bench_chirho("SoftDomain unify (10 values)", 100_000, || {
        let a_chirho = SoftDomainChirho::uniform_chirho(10);
        let b_chirho = SoftDomainChirho::uniform_chirho(10);
        std::hint::black_box(a_chirho.unify_chirho(&b_chirho));
    });

    println!("\n☧ Soli Deo Gloria ☧");
}
