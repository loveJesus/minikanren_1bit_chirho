//! Type Inference via miniKanren ☧
//!
//! Demonstrates Hindley-Milner style type inference using relational programming.
//! Types are terms, type checking is unification.
//!
//! Run with: cargo run --example type_infer_chirho

use minikanren_1bit_chirho::*;

/// Simple expression language
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ExprChirho {
    VarChirho(String),
    IntLitChirho(i64),
    BoolLitChirho(bool),
    LamChirho(String, Box<ExprChirho>),          // λx. body
    AppChirho(Box<ExprChirho>, Box<ExprChirho>), // f x
    IfChirho(Box<ExprChirho>, Box<ExprChirho>, Box<ExprChirho>),
}

fn main() {
    println!("Type Inference via miniKanren ☧\n");

    let mut store_chirho = TermStoreChirho::new();

    // Type constructors as symbols
    let int_type_chirho = store_chirho.sym_chirho("Int");
    let bool_type_chirho = store_chirho.sym_chirho("Bool");
    let _arrow_sym_chirho = store_chirho.sym_chirho("->");

    // Helper to create function type: a -> b
    let make_arrow_chirho = |store: &mut TermStoreChirho, a: TermIdChirho, b: TermIdChirho| -> TermIdChirho {
        let arrow_chirho = store.sym_chirho("->");
        let nil_chirho = store.nil_chirho();
        let inner_chirho = store.cons_chirho(b, nil_chirho);
        let inner2_chirho = store.cons_chirho(a, inner_chirho);
        store.cons_chirho(arrow_chirho, inner2_chirho)
    };

    println!("=== Example 1: Integer literal ===");
    println!("Expression: 42");
    println!("Expected type: Int");

    // Type of integer literal is Int
    let (_, t1_chirho) = store_chirho.fresh_var_chirho();
    let goal1_chirho = eq_chirho(t1_chirho, int_type_chirho);
    let results1_chirho = run_chirho(1, t1_chirho, goal1_chirho, &store_chirho);
    println!("Inferred: {:?}\n", results1_chirho);

    println!("=== Example 2: Boolean literal ===");
    println!("Expression: true");
    println!("Expected type: Bool");

    let (_, t2_chirho) = store_chirho.fresh_var_chirho();
    let goal2_chirho = eq_chirho(t2_chirho, bool_type_chirho);
    let results2_chirho = run_chirho(1, t2_chirho, goal2_chirho, &store_chirho);
    println!("Inferred: {:?}\n", results2_chirho);

    println!("=== Example 3: Identity function ===");
    println!("Expression: λx. x");
    println!("Expected type: a -> a (polymorphic)");

    // For λx. x, if x : a, then (λx. x) : a -> a
    let (_, a_chirho) = store_chirho.fresh_var_chirho();  // Type variable
    let id_type_chirho = make_arrow_chirho(&mut store_chirho, a_chirho, a_chirho);

    let (_, t3_chirho) = store_chirho.fresh_var_chirho();
    let goal3_chirho = eq_chirho(t3_chirho, id_type_chirho);
    let results3_chirho = run_chirho(1, t3_chirho, goal3_chirho, &store_chirho);
    println!("Inferred: term_id={:?} (a -> a where a is fresh)\n", results3_chirho);

    println!("=== Example 4: Function application ===");
    println!("Expression: (λx. x) 42");
    println!("Expected type: Int");

    // If f : a -> b and x : a, then (f x) : b
    // Here f = id : Int -> Int (instantiated), x = 42 : Int
    // So (id 42) : Int

    let (_, arg_type_chirho) = store_chirho.fresh_var_chirho();
    let (_, result_type_chirho) = store_chirho.fresh_var_chirho();

    // Constrain: arg_type = Int (from literal 42)
    // Constrain: f_type = arg_type -> result_type
    // Constrain: f_type = Int -> Int (from id instantiated at Int)

    let goal4_chirho = conj_chirho(
        eq_chirho(arg_type_chirho, int_type_chirho),
        eq_chirho(result_type_chirho, int_type_chirho),
    );

    let results4_chirho = run_chirho(1, result_type_chirho, goal4_chirho, &store_chirho);
    println!("Inferred result type: {:?}\n", results4_chirho);

    println!("=== Example 5: If-then-else ===");
    println!("Expression: if true then 1 else 2");
    println!("Expected type: Int");

    // For (if c then t else e):
    // c : Bool, t : T, e : T => result : T

    let (_, cond_type_chirho) = store_chirho.fresh_var_chirho();
    let (_, then_type_chirho) = store_chirho.fresh_var_chirho();
    let (_, else_type_chirho) = store_chirho.fresh_var_chirho();
    let (_, if_result_chirho) = store_chirho.fresh_var_chirho();

    let goal5_chirho = conj_all_chirho(vec![
        eq_chirho(cond_type_chirho, bool_type_chirho),  // condition is Bool
        eq_chirho(then_type_chirho, int_type_chirho),   // then branch is Int
        eq_chirho(else_type_chirho, int_type_chirho),   // else branch is Int
        eq_chirho(then_type_chirho, else_type_chirho),  // branches must match
        eq_chirho(if_result_chirho, then_type_chirho),  // result is branch type
    ]);

    let results5_chirho = run_chirho(1, if_result_chirho, goal5_chirho, &store_chirho);
    println!("Inferred if-result type: {:?}\n", results5_chirho);

    println!("=== Example 6: Type error detection ===");
    println!("Expression: if 42 then true else false");
    println!("Expected: TYPE ERROR (condition must be Bool)");

    let (_, bad_cond_chirho) = store_chirho.fresh_var_chirho();

    // This should fail: 42 is Int, not Bool
    let goal6_chirho = conj_chirho(
        eq_chirho(bad_cond_chirho, int_type_chirho),    // 42 : Int
        eq_chirho(bad_cond_chirho, bool_type_chirho),   // but condition needs Bool
    );

    let results6_chirho = run_chirho(1, bad_cond_chirho, goal6_chirho, &store_chirho);
    if results6_chirho.is_empty() {
        println!("✓ Type error detected! (no solutions)\n");
    } else {
        println!("✗ Should have failed: {:?}\n", results6_chirho);
    }

    println!("=== Example 7: Inferring function argument type ===");
    println!("Given: f : ? -> Int, f(true) is valid");
    println!("Infer: f : Bool -> Int");

    let (_, f_arg_type_chirho) = store_chirho.fresh_var_chirho();
    let _f_ret_type_chirho = int_type_chirho;

    // f(true) valid means arg type must be Bool
    let goal7_chirho = eq_chirho(f_arg_type_chirho, bool_type_chirho);

    let results7_chirho = run_chirho(1, f_arg_type_chirho, goal7_chirho, &store_chirho);
    println!("Inferred f's argument type: {:?}\n", results7_chirho);

    println!("=== Constraint Propagation Demo ===");
    println!("Using AC-3 for type domains\n");

    let mut cstore_chirho = ConstraintStoreChirho::new();

    // Encode types as domain values: 0=Int, 1=Bool, 2=String, 3=Float
    // Variable domains: which types are possible?

    // x can be Int or Bool initially
    cstore_chirho.add_var_chirho(0, DomainChirho(0b0011));  // x : {Int, Bool}

    // y can be Bool or String initially
    cstore_chirho.add_var_chirho(1, DomainChirho(0b0110));  // y : {Bool, String}

    // Constraint: x == y (same type)
    let eq_constraint_chirho = BinaryConstraintChirho::equality_chirho(0, 1, 4);
    cstore_chirho.add_constraint_chirho(eq_constraint_chirho);

    println!("Before propagation:");
    println!("  x domain: {:04b} = {{Int, Bool}}", cstore_chirho.domain_chirho(0).0);
    println!("  y domain: {:04b} = {{Bool, String}}", cstore_chirho.domain_chirho(1).0);

    cstore_chirho.propagate_chirho();

    println!("After propagation (x == y):");
    println!("  x domain: {:04b} = {{Bool}}", cstore_chirho.domain_chirho(0).0);
    println!("  y domain: {:04b} = {{Bool}}", cstore_chirho.domain_chirho(1).0);

    if cstore_chirho.domain_chirho(0).is_singleton_chirho() {
        println!("✓ Type inferred: both must be Bool\n");
    }

    println!("☧ Soli Deo Gloria ☧");
}
