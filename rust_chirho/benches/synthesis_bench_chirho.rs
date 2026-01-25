//! Program Synthesis & Type Inference Benchmarks ☧
//!
//! Tests domain approaches on problems with >64 values:
//!
//! 1. **Type Inference**: Hindley-Milner style unification
//!    - Domain = type IDs (can be hundreds of types)
//!    - Operations: unification, occurs check, constraint solving
//!
//! 2. **Program Synthesis**: Generate programs from I/O examples
//!    - Domain = AST node IDs (grammar productions)
//!    - Operations: enumerate valid programs, prune invalid
//!
//! 3. **Language Verification**: Check properties of programs
//!    - Domain = program states
//!    - Operations: reachability, safety checking
//!
//! Run with: cargo bench --bench synthesis_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashSet;

use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;
use minikanren_1bit_chirho::approaches_chirho::{
    Hierarchical4kChirho,
    Hierarchical256kChirho,
    SymbolicDomainChirho,
};

// ============================================================================
// Type System for Type Inference
// ============================================================================

/// Type representation for Hindley-Milner
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeChirho {
    /// Type variable (to be unified)
    VarChirho(u32),
    /// Primitive: Int, Bool, String, etc.
    PrimChirho(u32),
    /// Function type: arg -> ret
    FuncChirho(Box<TypeChirho>, Box<TypeChirho>),
    /// Generic type: List<T>, Option<T>
    GenericChirho(u32, Vec<TypeChirho>),
}

/// Type constraint: two types must unify
#[derive(Clone, Debug)]
struct TypeConstraintChirho {
    left_chirho: TypeChirho,
    right_chirho: TypeChirho,
}

/// Generate random type inference problem
fn generate_type_problem_chirho(
    num_vars_chirho: u32,
    num_constraints_chirho: u32,
    seed_chirho: u64,
) -> Vec<TypeConstraintChirho> {
    let mut constraints_chirho = Vec::new();
    let mut rng_chirho = seed_chirho;

    let mut next_rand_chirho = || {
        rng_chirho = rng_chirho.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng_chirho
    };

    for _ in 0..num_constraints_chirho {
        let v1_chirho = (next_rand_chirho() % num_vars_chirho as u64) as u32;
        let v2_chirho = (next_rand_chirho() % num_vars_chirho as u64) as u32;

        // Mix of constraint types
        let kind_chirho = next_rand_chirho() % 4;
        let constraint_chirho = match kind_chirho {
            0 => TypeConstraintChirho {
                left_chirho: TypeChirho::VarChirho(v1_chirho),
                right_chirho: TypeChirho::VarChirho(v2_chirho),
            },
            1 => TypeConstraintChirho {
                left_chirho: TypeChirho::VarChirho(v1_chirho),
                right_chirho: TypeChirho::PrimChirho((next_rand_chirho() % 5) as u32),
            },
            2 => TypeConstraintChirho {
                left_chirho: TypeChirho::VarChirho(v1_chirho),
                right_chirho: TypeChirho::FuncChirho(
                    Box::new(TypeChirho::VarChirho(v2_chirho)),
                    Box::new(TypeChirho::VarChirho((next_rand_chirho() % num_vars_chirho as u64) as u32)),
                ),
            },
            _ => TypeConstraintChirho {
                left_chirho: TypeChirho::FuncChirho(
                    Box::new(TypeChirho::VarChirho(v1_chirho)),
                    Box::new(TypeChirho::VarChirho(v2_chirho)),
                ),
                right_chirho: TypeChirho::FuncChirho(
                    Box::new(TypeChirho::VarChirho((next_rand_chirho() % num_vars_chirho as u64) as u32)),
                    Box::new(TypeChirho::VarChirho((next_rand_chirho() % num_vars_chirho as u64) as u32)),
                ),
            },
        };

        constraints_chirho.push(constraint_chirho);
    }

    constraints_chirho
}

/// Domain of possible types for a type variable
/// In real HM, this starts as "any type" and narrows via unification
struct TypeDomainChirho {
    /// Set of type IDs this variable could be
    /// (In practice, we'd use symbolic constraints, but for benchmarking
    /// we enumerate a finite set)
    possible_types_chirho: HashSet<u32>,
}

impl TypeDomainChirho {
    fn full_chirho(num_types_chirho: u32) -> Self {
        Self {
            possible_types_chirho: (0..num_types_chirho).collect(),
        }
    }

    fn intersect_chirho(&self, other_chirho: &Self) -> Self {
        Self {
            possible_types_chirho: self.possible_types_chirho
                .intersection(&other_chirho.possible_types_chirho)
                .copied()
                .collect(),
        }
    }

    fn is_empty_chirho(&self) -> bool {
        self.possible_types_chirho.is_empty()
    }
}

// ============================================================================
// AST for Program Synthesis
// ============================================================================

/// Simple expression language for synthesis
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ExprChirho {
    /// Integer literal
    IntChirho(i32),
    /// Variable reference
    VarChirho(u32),
    /// Binary operation: +, -, *, /
    BinOpChirho(BinOpKindChirho, Box<ExprChirho>, Box<ExprChirho>),
    /// If-then-else
    IfChirho(Box<ExprChirho>, Box<ExprChirho>, Box<ExprChirho>),
    /// Function application
    AppChirho(Box<ExprChirho>, Box<ExprChirho>),
    /// Lambda
    LamChirho(u32, Box<ExprChirho>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BinOpKindChirho {
    AddChirho,
    SubChirho,
    MulChirho,
    EqChirho,
    LtChirho,
}

/// Grammar production ID (for enumeration)
/// Each production is a way to build an expression
#[derive(Clone, Copy, Debug)]
struct ProductionChirho {
    id_chirho: u32,
    /// 0 = Int, 1 = Var, 2 = BinOp, 3 = If, 4 = App, 5 = Lam
    kind_chirho: u8,
    /// Constraints on children (for typed synthesis)
    result_type_chirho: u32,
}

/// Generate synthesis problem: I/O examples
struct SynthesisProblemChirho {
    /// Input-output examples
    examples_chirho: Vec<(Vec<i32>, i32)>,
    /// Maximum AST depth
    max_depth_chirho: u32,
    /// Number of available variables
    num_vars_chirho: u32,
}

impl SynthesisProblemChirho {
    fn new_chirho(examples_chirho: Vec<(Vec<i32>, i32)>, max_depth_chirho: u32, num_vars_chirho: u32) -> Self {
        Self { examples_chirho, max_depth_chirho, num_vars_chirho }
    }

    /// Count total grammar productions for given depth
    fn count_productions_chirho(&self) -> u32 {
        // Simplified: just count based on grammar
        // Real synthesis would have type-directed enumeration
        let base_chirho = 5; // Int, Var, BinOp, If, App
        let var_count_chirho = self.num_vars_chirho;
        let int_range_chirho = 20; // -10 to 10

        base_chirho + var_count_chirho + int_range_chirho + 5 // 5 binop kinds
    }

    /// Generate domain of valid productions at each position
    fn valid_productions_at_depth_chirho(&self, depth_chirho: u32, expected_type_chirho: u32) -> HashSet<u32> {
        let mut valid_chirho = HashSet::new();

        // At max depth, only terminals (Int, Var)
        if depth_chirho >= self.max_depth_chirho {
            // Integers: IDs 0-19
            for i_chirho in 0..20 {
                valid_chirho.insert(i_chirho);
            }
            // Variables: IDs 20-20+num_vars
            for i_chirho in 0..self.num_vars_chirho {
                valid_chirho.insert(20 + i_chirho);
            }
        } else {
            // All productions valid
            for i_chirho in 0..self.count_productions_chirho() {
                valid_chirho.insert(i_chirho);
            }
        }

        // Type filtering would go here
        valid_chirho
    }
}

// ============================================================================
// Benchmark 1: Type Unification with Large Type Space
// ============================================================================

fn bench_type_unification_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("TypeInference/Unification");
    group_chirho.sample_size(50); // Fewer samples for faster benchmarks

    for num_types_chirho in [64, 128, 256, 512, 1000] {
        // Simulate type domain intersection (core of unification)
        // When we unify T1 = T2, we intersect their possible instantiations

        // HashSet baseline
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", num_types_chirho),
            &num_types_chirho,
            |bench_chirho, &n_chirho| {
                // Two type variables, each could be any of n types
                let dom1_chirho: HashSet<u32> = (0..n_chirho).collect();
                let dom2_chirho: HashSet<u32> = (n_chirho/4..n_chirho*3/4).collect();

                bench_chirho.iter(|| {
                    let result_chirho: HashSet<u32> = dom1_chirho
                        .intersection(&dom2_chirho)
                        .copied()
                        .collect();
                    black_box(result_chirho.len())
                })
            },
        );

        // BitVec64 (only for small types)
        if num_types_chirho <= 64 {
            group_chirho.bench_with_input(
                BenchmarkId::new("BitVec64", num_types_chirho),
                &num_types_chirho,
                |bench_chirho, &n_chirho| {
                    let mask_chirho = if n_chirho >= 64 { u64::MAX } else { (1u64 << n_chirho) - 1 };
                    let dom1_chirho = BitVec64Chirho(mask_chirho);
                    let dom2_chirho = BitVec64Chirho(mask_chirho >> (n_chirho / 4));

                    bench_chirho.iter(|| {
                        let result_chirho = dom1_chirho.and_chirho(dom2_chirho);
                        black_box(result_chirho.0.count_ones())
                    })
                },
            );
        }

        // Hierarchical4k
        if num_types_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k", num_types_chirho),
                &num_types_chirho,
                |bench_chirho, &n_chirho| {
                    let dom1_chirho = Hierarchical4kChirho::range_chirho(n_chirho);
                    let dom2_chirho = Hierarchical4kChirho::range_chirho(n_chirho * 3 / 4);

                    bench_chirho.iter(|| {
                        let result_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                        black_box(result_chirho.count_chirho())
                    })
                },
            );
        }

        // Symbolic Range (best for type ranges)
        group_chirho.bench_with_input(
            BenchmarkId::new("SymbolicRange", num_types_chirho),
            &num_types_chirho,
            |bench_chirho, &n_chirho| {
                let dom1_chirho = SymbolicDomainChirho::range_chirho(0, n_chirho as i64);
                let dom2_chirho = SymbolicDomainChirho::range_chirho((n_chirho/4) as i64, (n_chirho*3/4) as i64);

                bench_chirho.iter(|| {
                    let result_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                    black_box(result_chirho.is_empty_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 2: Occurs Check (Prevent Infinite Types)
// ============================================================================

fn bench_occurs_check_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("TypeInference/OccursCheck");
    group_chirho.sample_size(50);

    // Occurs check: does type variable X appear in type T?
    // With domains: does the variable's ID appear in the set of variables in T?

    for num_vars_chirho in [100, 500, 1000, 2000] {
        // Simulate: type T references variables {10, 25, 50, 100, ...}
        // Check if variable X (ID = 50) appears

        // HashSet
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", num_vars_chirho),
            &num_vars_chirho,
            |bench_chirho, &n_chirho| {
                // Variables appearing in type T
                let type_vars_chirho: HashSet<u32> = (0..n_chirho).step_by(10).collect();
                let check_var_chirho = 50u32;

                bench_chirho.iter(|| {
                    black_box(type_vars_chirho.contains(&check_var_chirho))
                })
            },
        );

        // Hierarchical4k contains check
        if num_vars_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k", num_vars_chirho),
                &num_vars_chirho,
                |bench_chirho, &n_chirho| {
                    let mut domain_chirho = Hierarchical4kChirho::empty_chirho();
                    for v_chirho in (0..n_chirho).step_by(10) {
                        let leaf_chirho = (v_chirho / 64) as usize;
                        let bit_chirho = v_chirho % 64;
                        domain_chirho.leaves_chirho[leaf_chirho].0 |= 1 << bit_chirho;
                        domain_chirho.root_chirho.0 |= 1 << leaf_chirho;
                    }
                    let check_var_chirho = 50u32;

                    bench_chirho.iter(|| {
                        black_box(domain_chirho.contains_chirho(check_var_chirho))
                    })
                },
            );
        }
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 3: Synthesis Enumeration (Valid Programs)
// ============================================================================

fn bench_synthesis_enumeration_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Synthesis/Enumeration");
    group_chirho.sample_size(50);

    // Synthesis: enumerate valid AST nodes at each position
    // Domain = production IDs that are valid given type constraints

    for grammar_size_chirho in [50, 100, 200, 500, 1000] {
        // Two sets of valid productions that we need to intersect
        // (e.g., "returns Int" AND "takes Bool argument")

        // HashSet
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", grammar_size_chirho),
            &grammar_size_chirho,
            |bench_chirho, &n_chirho| {
                // Productions returning Int
                let returns_int_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 3 == 0).collect();
                // Productions taking Bool
                let takes_bool_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 5 == 0).collect();

                bench_chirho.iter(|| {
                    let valid_chirho: HashSet<u32> = returns_int_chirho
                        .intersection(&takes_bool_chirho)
                        .copied()
                        .collect();
                    black_box(valid_chirho.len())
                })
            },
        );

        // Hierarchical4k
        if grammar_size_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k", grammar_size_chirho),
                &grammar_size_chirho,
                |bench_chirho, &n_chirho| {
                    let mut returns_int_chirho = Hierarchical4kChirho::empty_chirho();
                    let mut takes_bool_chirho = Hierarchical4kChirho::empty_chirho();

                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 3 == 0) {
                        let leaf_chirho = (x_chirho / 64) as usize;
                        let bit_chirho = x_chirho % 64;
                        returns_int_chirho.leaves_chirho[leaf_chirho].0 |= 1 << bit_chirho;
                        returns_int_chirho.root_chirho.0 |= 1 << leaf_chirho;
                    }
                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 5 == 0) {
                        let leaf_chirho = (x_chirho / 64) as usize;
                        let bit_chirho = x_chirho % 64;
                        takes_bool_chirho.leaves_chirho[leaf_chirho].0 |= 1 << bit_chirho;
                        takes_bool_chirho.root_chirho.0 |= 1 << leaf_chirho;
                    }

                    bench_chirho.iter(|| {
                        let valid_chirho = returns_int_chirho.intersect_chirho(&takes_bool_chirho);
                        black_box(valid_chirho.count_chirho())
                    })
                },
            );
        }

        // Symbolic (modular) - perfect for "every 3rd" and "every 5th"
        group_chirho.bench_with_input(
            BenchmarkId::new("Symbolic_Mod", grammar_size_chirho),
            &grammar_size_chirho,
            |bench_chirho, &n_chirho| {
                // x ≡ 0 (mod 3) AND x ≡ 0 (mod 5) → x ≡ 0 (mod 15)
                let mod3_chirho = SymbolicDomainChirho::modular_chirho(0, 3);
                let mod5_chirho = SymbolicDomainChirho::modular_chirho(0, 5);

                bench_chirho.iter(|| {
                    let combined_chirho = mod3_chirho.intersect_chirho(&mod5_chirho);
                    // Would need to intersect with Range[0, n) to get count
                    black_box(combined_chirho.contains_chirho(0))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 4: Program State Reachability (Verification)
// ============================================================================

fn bench_state_reachability_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Verification/Reachability");
    group_chirho.sample_size(50);

    // Verification: which program states are reachable?
    // Domain = state IDs
    // Transition = state × action → state'

    for num_states_chirho in [100, 500, 1000, 2000] {
        // Simulate: compute reachable states via BFS
        // Then intersect with "bad states" to check safety

        // HashSet (baseline)
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet_intersect", num_states_chirho),
            &num_states_chirho,
            |bench_chirho, &n_chirho| {
                // Reachable states (simulated)
                let reachable_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 2 == 0).collect();
                // Bad states
                let bad_chirho: HashSet<u32> = (n_chirho*3/4..n_chirho).collect();

                bench_chirho.iter(|| {
                    // Safety check: are any bad states reachable?
                    let unsafe_chirho: HashSet<u32> = reachable_chirho
                        .intersection(&bad_chirho)
                        .copied()
                        .collect();
                    black_box(unsafe_chirho.is_empty())
                })
            },
        );

        // Hierarchical4k
        if num_states_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k_intersect", num_states_chirho),
                &num_states_chirho,
                |bench_chirho, &n_chirho| {
                    // Build reachable domain (even states)
                    let mut reachable_chirho = Hierarchical4kChirho::empty_chirho();
                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 2 == 0) {
                        let leaf_chirho = (x_chirho / 64) as usize;
                        let bit_chirho = x_chirho % 64;
                        reachable_chirho.leaves_chirho[leaf_chirho].0 |= 1 << bit_chirho;
                        reachable_chirho.root_chirho.0 |= 1 << leaf_chirho;
                    }

                    // Build bad states domain
                    let mut bad_chirho = Hierarchical4kChirho::empty_chirho();
                    for x_chirho in n_chirho*3/4..n_chirho {
                        let leaf_chirho = (x_chirho / 64) as usize;
                        let bit_chirho = x_chirho % 64;
                        bad_chirho.leaves_chirho[leaf_chirho].0 |= 1 << bit_chirho;
                        bad_chirho.root_chirho.0 |= 1 << leaf_chirho;
                    }

                    bench_chirho.iter(|| {
                        let unsafe_chirho = reachable_chirho.intersect_chirho(&bad_chirho);
                        black_box(unsafe_chirho.is_empty_chirho())
                    })
                },
            );
        }

        // Symbolic ranges (best case for contiguous bad states)
        group_chirho.bench_with_input(
            BenchmarkId::new("SymbolicRange", num_states_chirho),
            &num_states_chirho,
            |bench_chirho, &n_chirho| {
                // Reachable: modular (every 2nd state)
                let reachable_chirho = SymbolicDomainChirho::modular_chirho(0, 2);
                // Bad: range [3n/4, n)
                let bad_chirho = SymbolicDomainChirho::range_chirho((n_chirho*3/4) as i64, (n_chirho-1) as i64);

                bench_chirho.iter(|| {
                    let unsafe_chirho = reachable_chirho.intersect_chirho(&bad_chirho);
                    black_box(unsafe_chirho.is_empty_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 5: Constraint Propagation (Multi-variable)
// ============================================================================

fn bench_constraint_propagation_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("Synthesis/ConstraintProp");
    group_chirho.sample_size(50);

    // Constraint propagation: multiple variables with interconnected constraints
    // Each variable has a domain; constraints narrow domains iteratively

    for num_vars_chirho in [10, 20, 50, 100] {
        let domain_size_chirho = 100u32; // Each variable can be 0-99

        // HashSet baseline
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", num_vars_chirho),
            &num_vars_chirho,
            |bench_chirho, &n_vars_chirho| {
                // Initialize domains
                let mut domains_chirho: Vec<HashSet<u32>> = (0..n_vars_chirho)
                    .map(|_| (0..domain_size_chirho).collect())
                    .collect();

                bench_chirho.iter(|| {
                    // Simulate constraint propagation
                    // Constraint: var[i] != var[i+1] (simplistic)
                    for i_chirho in 0..n_vars_chirho as usize - 1 {
                        // If var[i] is singleton, remove from var[i+1]
                        if domains_chirho[i_chirho].len() == 1 {
                            let val_chirho = *domains_chirho[i_chirho].iter().next().unwrap();
                            domains_chirho[i_chirho + 1].remove(&val_chirho);
                        }
                    }
                    black_box(domains_chirho[0].len())
                })
            },
        );

        // Hierarchical4k
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical4k", num_vars_chirho),
            &num_vars_chirho,
            |bench_chirho, &n_vars_chirho| {
                // Initialize domains
                let mut domains_chirho: Vec<Hierarchical4kChirho> = (0..n_vars_chirho)
                    .map(|_| Hierarchical4kChirho::range_chirho(domain_size_chirho))
                    .collect();

                bench_chirho.iter(|| {
                    // Constraint propagation iteration
                    for i_chirho in 0..n_vars_chirho as usize - 1 {
                        // Intersect consecutive domains (simplified constraint)
                        let new_dom_chirho = domains_chirho[i_chirho].intersect_chirho(&domains_chirho[i_chirho + 1]);
                        domains_chirho[i_chirho + 1] = new_dom_chirho;
                    }
                    black_box(domains_chirho[0].count_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 6: Type Class Resolution (Typeclass Constraints)
// ============================================================================

fn bench_typeclass_resolution_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("TypeInference/Typeclass");
    group_chirho.sample_size(50);

    // Typeclass resolution: find types that satisfy multiple class constraints
    // e.g., "Show a, Eq a, Ord a" → which types implement all three?

    for num_types_chirho in [100, 200, 500, 1000] {
        // Simulate: each typeclass has a set of implementing types

        // HashSet
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet_3class", num_types_chirho),
            &num_types_chirho,
            |bench_chirho, &n_chirho| {
                // Types implementing Show (every 2nd)
                let show_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 2 == 0).collect();
                // Types implementing Eq (every 3rd)
                let eq_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 3 == 0).collect();
                // Types implementing Ord (every 5th)
                let ord_chirho: HashSet<u32> = (0..n_chirho).filter(|x_chirho| x_chirho % 5 == 0).collect();

                bench_chirho.iter(|| {
                    // Find types implementing all three
                    let show_eq_chirho: HashSet<u32> = show_chirho.intersection(&eq_chirho).copied().collect();
                    let all_chirho: HashSet<u32> = show_eq_chirho.intersection(&ord_chirho).copied().collect();
                    black_box(all_chirho.len())
                })
            },
        );

        // Hierarchical4k (3-way intersection)
        if num_types_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k_3class", num_types_chirho),
                &num_types_chirho,
                |bench_chirho, &n_chirho| {
                    let mut show_chirho = Hierarchical4kChirho::empty_chirho();
                    let mut eq_chirho = Hierarchical4kChirho::empty_chirho();
                    let mut ord_chirho = Hierarchical4kChirho::empty_chirho();

                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 2 == 0) {
                        let l_chirho = (x_chirho / 64) as usize;
                        let b_chirho = x_chirho % 64;
                        show_chirho.leaves_chirho[l_chirho].0 |= 1 << b_chirho;
                        show_chirho.root_chirho.0 |= 1 << l_chirho;
                    }
                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 3 == 0) {
                        let l_chirho = (x_chirho / 64) as usize;
                        let b_chirho = x_chirho % 64;
                        eq_chirho.leaves_chirho[l_chirho].0 |= 1 << b_chirho;
                        eq_chirho.root_chirho.0 |= 1 << l_chirho;
                    }
                    for x_chirho in (0..n_chirho).filter(|x_chirho| x_chirho % 5 == 0) {
                        let l_chirho = (x_chirho / 64) as usize;
                        let b_chirho = x_chirho % 64;
                        ord_chirho.leaves_chirho[l_chirho].0 |= 1 << b_chirho;
                        ord_chirho.root_chirho.0 |= 1 << l_chirho;
                    }

                    bench_chirho.iter(|| {
                        let temp_chirho = show_chirho.intersect_chirho(&eq_chirho);
                        let all_chirho = temp_chirho.intersect_chirho(&ord_chirho);
                        black_box(all_chirho.count_chirho())
                    })
                },
            );
        }

        // Symbolic CRT - types divisible by 2, 3, AND 5 = divisible by 30
        group_chirho.bench_with_input(
            BenchmarkId::new("Symbolic_CRT", num_types_chirho),
            &num_types_chirho,
            |bench_chirho, &_n_chirho| {
                let show_chirho = SymbolicDomainChirho::modular_chirho(0, 2);
                let eq_chirho = SymbolicDomainChirho::modular_chirho(0, 3);
                let ord_chirho = SymbolicDomainChirho::modular_chirho(0, 5);

                bench_chirho.iter(|| {
                    let temp_chirho = show_chirho.intersect_chirho(&eq_chirho);
                    let all_chirho = temp_chirho.intersect_chirho(&ord_chirho);
                    // CRT: x ≡ 0 (mod 2) ∩ x ≡ 0 (mod 3) ∩ x ≡ 0 (mod 5)
                    // = x ≡ 0 (mod 30)
                    black_box(all_chirho.contains_chirho(30))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 7: Large Domain Scaling (3-Level Hierarchy)
// ============================================================================

fn bench_large_domain_scaling_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("LargeDomain/Scaling");
    group_chirho.sample_size(30); // Fewer samples for expensive operations

    // Test scaling from 4k to 100k+ values
    for num_values_chirho in [5000, 10000, 20000, 50000, 100000] {
        // HashSet baseline
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", num_values_chirho),
            &num_values_chirho,
            |bench_chirho, &n_chirho| {
                let dom1_chirho: HashSet<u32> = (0..n_chirho).collect();
                let dom2_chirho: HashSet<u32> = (n_chirho/4..n_chirho*3/4).collect();

                bench_chirho.iter(|| {
                    let result_chirho: HashSet<u32> = dom1_chirho
                        .intersection(&dom2_chirho)
                        .copied()
                        .collect();
                    black_box(result_chirho.len())
                })
            },
        );

        // Hierarchical256k (3-level)
        if num_values_chirho <= 262144 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical256k", num_values_chirho),
                &num_values_chirho,
                |bench_chirho, &n_chirho| {
                    let dom1_chirho = Hierarchical256kChirho::range_chirho(n_chirho);
                    let dom2_chirho = Hierarchical256kChirho::range_chirho(n_chirho * 3 / 4);

                    bench_chirho.iter(|| {
                        let result_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                        black_box(result_chirho.is_empty_chirho())
                    })
                },
            );
        }

        // Symbolic Range (constant time regardless of size!)
        group_chirho.bench_with_input(
            BenchmarkId::new("SymbolicRange", num_values_chirho),
            &num_values_chirho,
            |bench_chirho, &n_chirho| {
                let dom1_chirho = SymbolicDomainChirho::range_chirho(0, n_chirho as i64);
                let dom2_chirho = SymbolicDomainChirho::range_chirho((n_chirho/4) as i64, (n_chirho*3/4) as i64);

                bench_chirho.iter(|| {
                    let result_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                    black_box(result_chirho.is_empty_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 8: Sparse vs Dense at Scale
// ============================================================================

fn bench_sparsity_scaling_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("LargeDomain/Sparsity");
    group_chirho.sample_size(30);

    let domain_size_chirho = 50000u32;

    // Test different sparsity levels
    for sparsity_pct_chirho in [1, 5, 10, 25, 50, 100] {
        let num_elements_chirho = domain_size_chirho * sparsity_pct_chirho / 100;

        // HashSet with sparse data
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", format!("{}pct", sparsity_pct_chirho)),
            &sparsity_pct_chirho,
            |bench_chirho, &pct_chirho| {
                // Sparse: every Nth element
                let step_chirho = if pct_chirho > 0 { 100 / pct_chirho } else { 100 };
                let dom1_chirho: HashSet<u32> = (0..domain_size_chirho)
                    .filter(|x_chirho| x_chirho % step_chirho as u32 == 0)
                    .collect();
                let dom2_chirho: HashSet<u32> = (domain_size_chirho/4..domain_size_chirho*3/4)
                    .filter(|x_chirho| x_chirho % step_chirho as u32 == 0)
                    .collect();

                bench_chirho.iter(|| {
                    let result_chirho: HashSet<u32> = dom1_chirho
                        .intersection(&dom2_chirho)
                        .copied()
                        .collect();
                    black_box(result_chirho.len())
                })
            },
        );

        // Hierarchical256k with sparse data
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical256k", format!("{}pct", sparsity_pct_chirho)),
            &sparsity_pct_chirho,
            |bench_chirho, &pct_chirho| {
                let step_chirho = if pct_chirho > 0 { 100 / pct_chirho } else { 100 };

                // Build sparse domains
                let mut dom1_chirho = Hierarchical256kChirho::empty_chirho();
                let mut dom2_chirho = Hierarchical256kChirho::empty_chirho();

                for x_chirho in (0..domain_size_chirho).filter(|x_chirho| x_chirho % step_chirho as u32 == 0) {
                    let idx1_chirho = (x_chirho / 4096) as usize;
                    let idx2_chirho = ((x_chirho % 4096) / 64) as usize;
                    let bit_chirho = x_chirho % 64;

                    if idx1_chirho < 64 && idx2_chirho < 64 {
                        dom1_chirho.leaves_chirho[idx1_chirho][idx2_chirho].0 |= 1 << bit_chirho;
                        dom1_chirho.index_chirho[idx1_chirho].0 |= 1 << idx2_chirho;
                        dom1_chirho.root_chirho.0 |= 1 << idx1_chirho;
                    }
                }

                for x_chirho in (domain_size_chirho/4..domain_size_chirho*3/4).filter(|x_chirho| x_chirho % step_chirho as u32 == 0) {
                    let idx1_chirho = (x_chirho / 4096) as usize;
                    let idx2_chirho = ((x_chirho % 4096) / 64) as usize;
                    let bit_chirho = x_chirho % 64;

                    if idx1_chirho < 64 && idx2_chirho < 64 {
                        dom2_chirho.leaves_chirho[idx1_chirho][idx2_chirho].0 |= 1 << bit_chirho;
                        dom2_chirho.index_chirho[idx1_chirho].0 |= 1 << idx2_chirho;
                        dom2_chirho.root_chirho.0 |= 1 << idx1_chirho;
                    }
                }

                bench_chirho.iter(|| {
                    let result_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                    black_box(result_chirho.is_empty_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_type_unification_chirho,
    bench_occurs_check_chirho,
    bench_synthesis_enumeration_chirho,
    bench_state_reachability_chirho,
    bench_constraint_propagation_chirho,
    bench_typeclass_resolution_chirho,
    bench_large_domain_scaling_chirho,
    bench_sparsity_scaling_chirho,
);

criterion_main!(benches_chirho);
