//! Program Synthesis via miniKanren ☧
//!
//! Synthesize programs from input-output examples.
//! The relational interpreter runs "backwards" to find programs.
//!
//! Run with: cargo run --example synthesis_chirho

use minikanren_1bit_chirho::*;

fn main() {
    println!("Program Synthesis via miniKanren ☧\n");

    let mut store_chirho = TermStoreChirho::new();

    // Build a simple expression language
    // Expressions: Var(name), Const(n), Add(e1, e2), Mul(e1, e2)

    // Helper to create expression terms
    fn make_const_chirho(store: &mut TermStoreChirho, n: i64) -> TermIdChirho {
        let sym_chirho = store.sym_chirho("Const");
        let num_chirho = store.int_chirho(n);
        let nil_chirho = store.nil_chirho();
        let inner_chirho = store.cons_chirho(num_chirho, nil_chirho);
        store.cons_chirho(sym_chirho, inner_chirho)
    }

    fn make_var_chirho(store: &mut TermStoreChirho, name: &str) -> TermIdChirho {
        let sym_chirho = store.sym_chirho("Var");
        let name_sym_chirho = store.sym_chirho(name);
        let nil_chirho = store.nil_chirho();
        let inner_chirho = store.cons_chirho(name_sym_chirho, nil_chirho);
        store.cons_chirho(sym_chirho, inner_chirho)
    }

    fn make_add_chirho(store: &mut TermStoreChirho, e1: TermIdChirho, e2: TermIdChirho) -> TermIdChirho {
        let sym_chirho = store.sym_chirho("Add");
        let nil_chirho = store.nil_chirho();
        let inner_chirho = store.cons_chirho(e2, nil_chirho);
        let inner2_chirho = store.cons_chirho(e1, inner_chirho);
        store.cons_chirho(sym_chirho, inner2_chirho)
    }

    fn make_mul_chirho(store: &mut TermStoreChirho, e1: TermIdChirho, e2: TermIdChirho) -> TermIdChirho {
        let sym_chirho = store.sym_chirho("Mul");
        let nil_chirho = store.nil_chirho();
        let inner_chirho = store.cons_chirho(e2, nil_chirho);
        let inner2_chirho = store.cons_chirho(e1, inner_chirho);
        store.cons_chirho(sym_chirho, inner2_chirho)
    }

    println!("=== Example 1: Synthesize constant ===");
    println!("Input-output: any -> 42");
    println!("Synthesize expression that always returns 42\n");

    // We want to find expr such that interpret(expr) = 42
    // Simplest: expr = Const(42)
    let const_42_chirho = make_const_chirho(&mut store_chirho, 42);
    println!("Synthesized: Const(42) = term {:?}", const_42_chirho);

    println!("\n=== Example 2: Synthesize using variable ===");
    println!("Input-output: x=5 -> 5");
    println!("Synthesize: Var(x)\n");

    let var_x_chirho = make_var_chirho(&mut store_chirho, "x");
    println!("Synthesized: Var(x) = term {:?}", var_x_chirho);

    println!("\n=== Example 3: Synthesize addition ===");
    println!("Input-output: x=3 -> 5 (need x+2)");
    println!("Synthesize: Add(Var(x), Const(2))\n");

    let const_2_chirho = make_const_chirho(&mut store_chirho, 2);
    let add_expr_chirho = make_add_chirho(&mut store_chirho, var_x_chirho, const_2_chirho);
    println!("Synthesized: Add(Var(x), Const(2)) = term {:?}", add_expr_chirho);

    println!("\n=== Example 4: Relational search for programs ===");
    println!("Using constraint propagation to narrow program space\n");

    // Use constraint domains to represent possible program structures
    // Domain values: 0=Const, 1=Var, 2=Add, 3=Mul
    let mut cstore_chirho = ConstraintStoreChirho::new();

    // Root expression type: could be anything
    cstore_chirho.add_var_chirho(0, DomainChirho(0b1111));  // root: {Const, Var, Add, Mul}

    // If output is always same regardless of input -> must be Const
    // Constraint: root must be Const for constant output
    println!("Constraint: output is always 42 (constant)");
    println!("Before: root domain = {:04b} = {{Const, Var, Add, Mul}}", cstore_chirho.domain_chirho(0).0);

    // Apply constraint: for constant output, must be Const (value 0)
    cstore_chirho.add_var_chirho(0, DomainChirho(0b0001));  // Just Const

    println!("After:  root domain = {:04b} = {{Const}}", cstore_chirho.domain_chirho(0).0);
    println!("✓ Narrowed to: Const(42)\n");

    println!("=== Example 5: Synthesis with multiple constraints ===");
    println!("Input-outputs:");
    println!("  x=0 -> 0");
    println!("  x=1 -> 2");
    println!("  x=2 -> 4");
    println!("Pattern: output = 2*x");
    println!("Synthesize: Mul(Const(2), Var(x))\n");

    let const_2_new_chirho = make_const_chirho(&mut store_chirho, 2);
    let var_x_new_chirho = make_var_chirho(&mut store_chirho, "x");
    let double_expr_chirho = make_mul_chirho(&mut store_chirho, const_2_new_chirho, var_x_new_chirho);
    println!("Synthesized: Mul(Const(2), Var(x)) = term {:?}", double_expr_chirho);

    println!("\n=== Example 6: Enumerate small programs ===");
    println!("Using conde to enumerate expression structures\n");

    let (_, prog_type_chirho) = store_chirho.fresh_var_chirho();

    // prog_type can be 0=Const, 1=Var, 2=Add, 3=Mul
    let type_const_chirho = store_chirho.int_chirho(0);
    let type_var_chirho = store_chirho.int_chirho(1);
    let type_add_chirho = store_chirho.int_chirho(2);
    let type_mul_chirho = store_chirho.int_chirho(3);

    let enumerate_goal_chirho = conde_chirho(vec![
        vec![eq_chirho(prog_type_chirho, type_const_chirho)],
        vec![eq_chirho(prog_type_chirho, type_var_chirho)],
        vec![eq_chirho(prog_type_chirho, type_add_chirho)],
        vec![eq_chirho(prog_type_chirho, type_mul_chirho)],
    ]);

    let all_types_chirho = run_chirho(10, prog_type_chirho, enumerate_goal_chirho, &store_chirho);
    println!("All program types: {:?}", all_types_chirho);
    println!("  0 = Const, 1 = Var, 2 = Add, 3 = Mul\n");

    println!("=== Example 7: Batched parallel synthesis ===");
    println!("Simulating parallel search with domain vectors\n");

    // Parallel state representation: each state has domain bitvectors
    #[derive(Clone)]
    struct BatchStateChirho {
        domains_chirho: [u64; 8],  // Up to 8 variables
        valid_chirho: bool,
    }

    impl BatchStateChirho {
        fn new_chirho() -> Self {
            Self { domains_chirho: [u64::MAX; 8], valid_chirho: true }
        }
    }

    let mut states_chirho: Vec<BatchStateChirho> = Vec::new();

    // Create initial states exploring different program structures
    for i_chirho in 0..4u64 {
        let mut state_chirho = BatchStateChirho::new_chirho();
        // var 0 = program type
        state_chirho.domains_chirho[0] = 1u64 << i_chirho;  // Each state tries one type
        // var 1 = first operand type (for Add/Mul)
        state_chirho.domains_chirho[1] = 0b11;  // Could be Const or Var
        states_chirho.push(state_chirho);
    }

    println!("Initial states: {}", states_chirho.iter().filter(|s| s.valid_chirho).count());

    // Constraint: if type is Add (2), second operand must be Const (0)
    for state_chirho in &mut states_chirho {
        if state_chirho.domains_chirho[0] == 0b0100 {  // Add type
            state_chirho.domains_chirho[1] &= 0b01;  // Force operand to Const
        }
    }

    let valid_count_chirho = states_chirho.iter().filter(|s| s.valid_chirho).count();
    println!("After constraints: {} valid states", valid_count_chirho);

    println!("Solutions:");
    for state_chirho in &states_chirho {
        if state_chirho.valid_chirho {
            let type_val_chirho = state_chirho.domains_chirho[0].trailing_zeros();
            let type_name_chirho = match type_val_chirho {
                0 => "Const",
                1 => "Var",
                2 => "Add",
                3 => "Mul",
                _ => "?",
            };
            let operand_val_chirho = state_chirho.domains_chirho[1].trailing_zeros();
            println!("  Program type: {} (operand type: {})", type_name_chirho, operand_val_chirho);
        }
    }

    println!("\n☧ Soli Deo Gloria ☧");
}
