//! Example: appendo relation ☧
//!
//! Demonstrates list append as a relation that works in multiple directions.
//!
//! Run with: cargo run --example appendo_chirho

use minikanren_1bit_chirho::*;

fn main() {
    println!("miniKanren as 1-Bit Matrix Operations ☧\n");
    println!("Example: appendo relation (list append)\n");

    let mut store_chirho = TermStoreChirho::new();

    // Create some lists
    let nil_chirho = store_chirho.nil_chirho();
    let list_1_chirho = store_chirho.list_ints_chirho(&[1]);
    let list_2_chirho = store_chirho.list_ints_chirho(&[2]);
    let list_12_chirho = store_chirho.list_ints_chirho(&[1, 2]);
    let _list_123_chirho = store_chirho.list_ints_chirho(&[1, 2, 3]);

    println!("=== Forward mode: [1] ++ [2] = ? ===");

    // Build appendo tensor (precompute some known facts)
    let mut appendo_chirho = relations_chirho::AppendoChirho::new();

    // Base case: [] ++ s = s
    appendo_chirho.add_triple_chirho(nil_chirho, nil_chirho, nil_chirho);
    appendo_chirho.add_triple_chirho(nil_chirho, list_1_chirho, list_1_chirho);
    appendo_chirho.add_triple_chirho(nil_chirho, list_2_chirho, list_2_chirho);
    appendo_chirho.add_triple_chirho(nil_chirho, list_12_chirho, list_12_chirho);

    // Recursive case: [1] ++ [2] = [1, 2]
    appendo_chirho.add_triple_chirho(list_1_chirho, list_2_chirho, list_12_chirho);

    // Query forward: [1] ++ [2] = ?
    let results_chirho = appendo_chirho.forward_chirho(list_1_chirho, list_2_chirho);
    println!("Results: {:?}", results_chirho);
    if results_chirho.contains(&list_12_chirho) {
        println!("✓ Found [1, 2] (term_id={})", list_12_chirho);
    }

    println!("\n=== Backward mode: ? ++ ? = [1, 2] ===");

    // Query backward: find all ways to split [1, 2]
    let splits_chirho = appendo_chirho.backward_chirho(list_12_chirho);
    println!("Splits: {:?}", splits_chirho);
    for (l_chirho, s_chirho) in &splits_chirho {
        println!("  {} ++ {} = {}", l_chirho, s_chirho, list_12_chirho);
    }

    println!("\n=== Unification demo ===");

    let (_, x_chirho) = store_chirho.fresh_var_chirho();
    let (_, y_chirho) = store_chirho.fresh_var_chirho();

    // Create goal: x == [1] AND y == [2]
    let goal_chirho = conj_chirho(
        eq_chirho(x_chirho, list_1_chirho),
        eq_chirho(y_chirho, list_2_chirho),
    );

    let solutions_chirho = run_chirho(10, x_chirho, goal_chirho, &store_chirho);
    println!("Solutions for x: {:?}", solutions_chirho);

    println!("\n=== Conde (disjunction) demo ===");

    let (_, z_chirho) = store_chirho.fresh_var_chirho();

    // z could be [1], [2], or [1,2]
    let goal2_chirho = conde_chirho(vec![
        vec![eq_chirho(z_chirho, list_1_chirho)],
        vec![eq_chirho(z_chirho, list_2_chirho)],
        vec![eq_chirho(z_chirho, list_12_chirho)],
    ]);

    let solutions2_chirho = run_chirho(10, z_chirho, goal2_chirho, &store_chirho);
    println!("All values z can take: {:?}", solutions2_chirho);
    println!("  [1] = term {}", list_1_chirho);
    println!("  [2] = term {}", list_2_chirho);
    println!("  [1,2] = term {}", list_12_chirho);

    println!("\n=== Constraint propagation demo ===");

    let mut cstore_chirho = ConstraintStoreChirho::new();

    // x in {0,1,2,3}, y in {2,3,4,5}, x == y
    cstore_chirho.add_var_chirho(0, DomainChirho(0b1111));      // {0,1,2,3}
    cstore_chirho.add_var_chirho(1, DomainChirho(0b111100));    // {2,3,4,5}

    let eq_constraint_chirho = BinaryConstraintChirho::equality_chirho(0, 1, 10);
    cstore_chirho.add_constraint_chirho(eq_constraint_chirho);

    println!("Before propagation:");
    println!("  x domain: {:b} (0b{:b})", cstore_chirho.domain_chirho(0).0, cstore_chirho.domain_chirho(0).0);
    println!("  y domain: {:b} (0b{:b})", cstore_chirho.domain_chirho(1).0, cstore_chirho.domain_chirho(1).0);

    cstore_chirho.propagate_chirho();

    println!("After propagation (x == y):");
    println!("  x domain: {:b} = {{2,3}}", cstore_chirho.domain_chirho(0).0);
    println!("  y domain: {:b} = {{2,3}}", cstore_chirho.domain_chirho(1).0);

    println!("\n=== Semiring demo ===");

    // Counting semiring: count derivations
    let c1_chirho = CountSemiringChirho(3);
    let c2_chirho = CountSemiringChirho(4);
    println!("Counting: {} + {} = {}", c1_chirho.0, c2_chirho.0, (c1_chirho + c2_chirho).0);
    println!("Counting: {} * {} = {}", c1_chirho.0, c2_chirho.0, (c1_chirho * c2_chirho).0);

    // Tropical semiring: shortest paths
    let t1_chirho = TropicalSemiringChirho(3.0);
    let t2_chirho = TropicalSemiringChirho(5.0);
    println!("Tropical: min({}, {}) = {}", t1_chirho.0, t2_chirho.0, (t1_chirho + t2_chirho).0);
    println!("Tropical: {} + {} = {}", t1_chirho.0, t2_chirho.0, (t1_chirho * t2_chirho).0);

    println!("\n☧ Soli Deo Gloria ☧");
}
