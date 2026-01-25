//! SyGuS Program Synthesis Benchmarks ☧
//!
//! Compares our domain-pruned enumeration with reference times from CVC5.
//!
//! Run with: cargo bench --bench sygus_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::synthesis_chirho::{
    sygus_chirho::SygusProblemChirho,
    grammar_chirho::EncodedGrammarChirho,
    enumerate_chirho::{SynthesisStateChirho, synthesize_chirho},
};
use minikanren_1bit_chirho::approaches_chirho::Hierarchical4kChirho;

// ============================================================================
// Helper to create test problems
// ============================================================================

fn max2_problem_chirho() -> SygusProblemChirho {
    let content_chirho = r#"
(synth-fun max2 ((x Int) (y Int)) Int)
(constraint (= (max2 0 1) 1))
(constraint (= (max2 1 0) 1))
(constraint (= (max2 3 5) 5))
(constraint (= (max2 5 3) 5))
(constraint (= (max2 2 2) 2))
"#;
    SygusProblemChirho::parse_chirho(content_chirho).unwrap()
}

fn abs_problem_chirho() -> SygusProblemChirho {
    let content_chirho = r#"
(synth-fun abs ((x Int)) Int)
(constraint (= (abs 0) 0))
(constraint (= (abs 1) 1))
(constraint (= (abs -1) 1))
(constraint (= (abs 5) 5))
(constraint (= (abs -5) 5))
"#;
    SygusProblemChirho::parse_chirho(content_chirho).unwrap()
}

// ============================================================================
// Grammar encoding benchmarks
// ============================================================================

fn bench_grammar_encoding_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/GrammarEncode");

    let problem_chirho = max2_problem_chirho();

    group_chirho.bench_function("max2_encode", |bench_chirho| {
        bench_chirho.iter(|| {
            black_box(EncodedGrammarChirho::from_sygus_chirho(&problem_chirho.grammar_chirho))
        })
    });

    let encoded_chirho = EncodedGrammarChirho::from_sygus_chirho(&problem_chirho.grammar_chirho);

    group_chirho.bench_function("max2_domain_lookup", |bench_chirho| {
        bench_chirho.iter(|| {
            // Look up production domains for start symbol
            let domain_chirho = encoded_chirho.domain_for_chirho("Start");
            black_box(domain_chirho.count_chirho())
        })
    });

    group_chirho.bench_function("max2_leaf_domain", |bench_chirho| {
        bench_chirho.iter(|| {
            let domain_chirho = encoded_chirho.leaf_domain_chirho();
            black_box(domain_chirho.count_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Enumeration benchmarks
// ============================================================================

fn bench_enumeration_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/Enumerate");

    let problem_chirho = max2_problem_chirho();

    // Benchmark synthesizing with different max sizes
    for max_size_chirho in [3, 5, 7].iter() {
        group_chirho.bench_with_input(
            BenchmarkId::new("max_size", max_size_chirho),
            max_size_chirho,
            |bench_chirho, &sz_chirho| {
                bench_chirho.iter(|| {
                    black_box(synthesize_chirho(&problem_chirho, sz_chirho))
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Domain pruning benchmarks
// ============================================================================

fn bench_domain_pruning_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/DomainPrune");

    // Simulate constraint propagation with domain intersection
    group_chirho.bench_function("intersect_large", |bench_chirho| {
        let domain1_chirho = Hierarchical4kChirho::range_chirho(1000);
        let domain2_chirho = Hierarchical4kChirho::range_chirho(800);

        bench_chirho.iter(|| {
            black_box(domain1_chirho.intersect_chirho(&domain2_chirho))
        })
    });

    group_chirho.bench_function("union_large", |bench_chirho| {
        let domain1_chirho = Hierarchical4kChirho::range_chirho(500);
        let domain2_chirho = Hierarchical4kChirho::singleton_chirho(700);

        bench_chirho.iter(|| {
            black_box(domain1_chirho.union_chirho(&domain2_chirho))
        })
    });

    group_chirho.bench_function("count_sparse", |bench_chirho| {
        // Sparse domain (every 10th value)
        let mut domain_chirho = Hierarchical4kChirho::empty_chirho();
        for i_chirho in (0..100u32).step_by(10) {
            domain_chirho = domain_chirho.union_chirho(&Hierarchical4kChirho::singleton_chirho(i_chirho));
        }

        bench_chirho.iter(|| {
            black_box(domain_chirho.count_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// End-to-end synthesis benchmarks
// ============================================================================

fn bench_synthesis_e2e_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/E2E");

    group_chirho.bench_function("max2_synth", |bench_chirho| {
        let problem_chirho = max2_problem_chirho();
        bench_chirho.iter(|| {
            black_box(synthesize_chirho(&problem_chirho, 5))
        })
    });

    group_chirho.bench_function("abs_synth", |bench_chirho| {
        let problem_chirho = abs_problem_chirho();
        bench_chirho.iter(|| {
            black_box(synthesize_chirho(&problem_chirho, 5))
        })
    });

    // Reference times from CVC5 on standard SyGuS benchmarks
    // (These would be filled in from actual runs)
    println!("  Note: Reference times from CVC5 would be measured externally");
    println!("  Expected speedup target: 10x on at least 2 problems");

    group_chirho.finish();
}

// ============================================================================
// State propagation benchmarks
// ============================================================================

fn bench_state_propagation_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/StateProp");

    let problem_chirho = max2_problem_chirho();
    let examples_chirho = problem_chirho.io_examples_chirho();

    group_chirho.bench_function("propagate_examples", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut state_chirho = SynthesisStateChirho::from_problem_chirho(&problem_chirho, 5);
            black_box(state_chirho.propagate_examples_chirho(&examples_chirho))
        })
    });

    group_chirho.bench_function("pruning_factor", |bench_chirho| {
        let mut state_chirho = SynthesisStateChirho::from_problem_chirho(&problem_chirho, 5);
        state_chirho.propagate_examples_chirho(&examples_chirho);

        bench_chirho.iter(|| {
            black_box(state_chirho.pruning_factor_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Comparison notes
// ============================================================================

fn bench_comparison_notes_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SyGuS/ComparisonNotes");

    // This benchmark prints comparison methodology
    group_chirho.bench_function("methodology", |bench_chirho| {
        bench_chirho.iter(|| {
            // Placeholder for comparison methodology
            // Actual comparison would:
            // 1. Run CVC5 with: cvc5 --lang sygus2 problem.sl
            // 2. Parse timing from output
            // 3. Compare with our enumeration time

            black_box(42)
        })
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_grammar_encoding_chirho,
    bench_enumeration_chirho,
    bench_domain_pruning_chirho,
    bench_synthesis_e2e_chirho,
    bench_state_propagation_chirho,
    bench_comparison_notes_chirho,
);

criterion_main!(benches_chirho);
