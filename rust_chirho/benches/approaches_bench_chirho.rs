//! Benchmarks for Domain Approaches ☧
//!
//! Compares different strategies for handling domains:
//! - Small (BitVec64) vs Paged vs Symbolic
//! - Hardware-accelerated symbolic operations
//! - Hybrid domain partitioning
//!
//! Run with: cargo bench --bench approaches_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;
use minikanren_1bit_chirho::approaches_chirho::{
    PagedDomainChirho,
    SymbolicDomainChirho,
    HybridDomainChirho,
    HwSymbolicDomainChirho,
    Hierarchical4kChirho,
    Hierarchical256kChirho,
    Hierarchical65kChirho,
    Hierarchical262kWideChirho,
    hw_symbolic_chirho::{range_bits_chirho, mod_lut_chirho, range_mod_hw_chirho},
};

// ============================================================================
// Benchmark 1: Domain Intersection at Various Sizes
// ============================================================================

fn bench_intersection_by_size_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("IntersectionChirho");

    // Raw BitVec64 N-way intersections
    group_chirho.bench_function("BitVec64_3way", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0xFFFFFFFF);
        let b_chirho = BitVec64Chirho(0xFF00FF00);
        let c_chirho = BitVec64Chirho(0x0F0F0F0F);
        bench_chirho.iter(|| {
            black_box(a_chirho.and_chirho(b_chirho).and_chirho(c_chirho))
        })
    });

    group_chirho.bench_function("BitVec64_5way", |bench_chirho| {
        let a_chirho = BitVec64Chirho(0xFFFFFFFFFFFFFFFF);
        let b_chirho = BitVec64Chirho(0xFF00FF00FF00FF00);
        let c_chirho = BitVec64Chirho(0x0F0F0F0F0F0F0F0F);
        let d_chirho = BitVec64Chirho(0x3333333333333333);
        let e_chirho = BitVec64Chirho(0x5555555555555555);
        bench_chirho.iter(|| {
            black_box(a_chirho.and_chirho(b_chirho).and_chirho(c_chirho).and_chirho(d_chirho).and_chirho(e_chirho))
        })
    });

    for size_chirho in [16, 32, 64, 128, 256, 1024] {
        // BitVec64 (only for size <= 64)
        if size_chirho <= 64 {
            group_chirho.bench_with_input(
                BenchmarkId::new("BitVec64", size_chirho),
                &size_chirho,
                |bench_chirho, &n_chirho| {
                    let mask_chirho = if n_chirho >= 64 { u64::MAX } else { (1u64 << n_chirho) - 1 };
                    let a_chirho = BitVec64Chirho(mask_chirho);
                    let b_chirho = BitVec64Chirho(mask_chirho >> 1);
                    bench_chirho.iter(|| {
                        black_box(a_chirho.and_chirho(b_chirho))
                    })
                },
            );
        }

        // Paged domain
        group_chirho.bench_with_input(
            BenchmarkId::new("Paged", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = PagedDomainChirho::range_chirho(n_chirho as u64);
                let b_chirho = PagedDomainChirho::range_chirho(n_chirho as u64 / 2);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );

        // Symbolic domain (range intersection)
        group_chirho.bench_with_input(
            BenchmarkId::new("Symbolic_Range", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = SymbolicDomainChirho::range_chirho(0, n_chirho as i64);
                let b_chirho = SymbolicDomainChirho::range_chirho((n_chirho / 4) as i64, (n_chirho * 3 / 4) as i64);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Hierarchical 4K domain (tree of BitVec64s)
    for size_chirho in [100, 500, 1000, 2000, 4000] {
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical4k", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = Hierarchical4kChirho::range_chirho(n_chirho);
                let b_chirho = Hierarchical4kChirho::range_chirho(n_chirho / 2);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Hierarchical 256K domain (64³ = tree of trees of BitVec64s)
    // This shows we scale MASSIVELY beyond 64 bits
    for size_chirho in [1_000, 10_000, 50_000, 100_000, 200_000, 262_000] {
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical256k_64³", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = Hierarchical256kChirho::range_chirho(n_chirho);
                let b_chirho = Hierarchical256kChirho::range_chirho(n_chirho / 2);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Hierarchical 65K domain (256² = 2-level with 256-bit words)
    // Wider/shallower alternative to 64³
    for size_chirho in [1_000, 10_000, 50_000, 65_000] {
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical65k_256²", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = Hierarchical65kChirho::range_chirho(n_chirho);
                let b_chirho = Hierarchical65kChirho::range_chirho(n_chirho / 2);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Hierarchical 262K wide domain (512² = 2-level with 512-bit words)
    // Widest/shallowest - tests AVX-512 potential
    for size_chirho in [1_000, 10_000, 50_000, 100_000, 200_000, 262_000] {
        group_chirho.bench_with_input(
            BenchmarkId::new("Hierarchical262k_512²", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let a_chirho = Hierarchical262kWideChirho::range_chirho(n_chirho);
                let b_chirho = Hierarchical262kWideChirho::range_chirho(n_chirho / 2);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Hierarchical sparse intersection (non-overlapping roots = O(1))
    group_chirho.bench_function("Hierarchical4k_sparse", |bench_chirho| {
        // Domain A: values 0-63 (leaf 0 only)
        let mut a_chirho = Hierarchical4kChirho::empty_chirho();
        a_chirho.leaves_chirho[0] = BitVec64Chirho::ONES_CHIRHO;
        a_chirho.root_chirho = BitVec64Chirho(1);

        // Domain B: values 640-703 (leaf 10 only)
        let mut b_chirho = Hierarchical4kChirho::empty_chirho();
        b_chirho.leaves_chirho[10] = BitVec64Chirho::ONES_CHIRHO;
        b_chirho.root_chirho = BitVec64Chirho(1 << 10);

        bench_chirho.iter(|| {
            // Root AND = 0, so this should be O(1)!
            black_box(a_chirho.intersect_chirho(&b_chirho))
        })
    });

    // Sparse 256² (non-overlapping domains via singleton)
    group_chirho.bench_function("Hierarchical65k_256²_sparse", |bench_chirho| {
        // Domain A: values 0-255 (leaf 0 only)
        let a_chirho = Hierarchical65kChirho::range_chirho(256);
        // Domain B: single value far away (different root region)
        let b_chirho = Hierarchical65kChirho::singleton_chirho(60000);

        bench_chirho.iter(|| {
            black_box(a_chirho.intersect_chirho(&b_chirho))
        })
    });

    // Sparse 512² (non-overlapping domains via singleton)
    group_chirho.bench_function("Hierarchical262k_512²_sparse", |bench_chirho| {
        // Domain A: values 0-511 (leaf 0 only)
        let a_chirho = Hierarchical262kWideChirho::range_chirho(512);
        // Domain B: single value far away (different root region)
        let b_chirho = Hierarchical262kWideChirho::singleton_chirho(200000);

        bench_chirho.iter(|| {
            black_box(a_chirho.intersect_chirho(&b_chirho))
        })
    });

    // Compare all 262K structures at same density (50%)
    group_chirho.bench_function("Compare_262k_64³_dense", |bench_chirho| {
        let a_chirho = Hierarchical256kChirho::range_chirho(131072);
        let b_chirho = Hierarchical256kChirho::range_chirho(131072);
        bench_chirho.iter(|| black_box(a_chirho.intersect_chirho(&b_chirho)))
    });

    group_chirho.bench_function("Compare_262k_512²_dense", |bench_chirho| {
        let a_chirho = Hierarchical262kWideChirho::range_chirho(131072);
        let b_chirho = Hierarchical262kWideChirho::range_chirho(131072);
        bench_chirho.iter(|| black_box(a_chirho.intersect_chirho(&b_chirho)))
    });

    group_chirho.finish();
}

// ============================================================================
// Benchmark 2: Symbolic Operations (CRT, Modular)
// ============================================================================

fn bench_symbolic_ops_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("SymbolicOpsChirho");

    // Chinese Remainder Theorem intersection
    for (m1, m2) in [(3, 5), (7, 11), (13, 17), (97, 101)] {
        group_chirho.bench_with_input(
            BenchmarkId::new("CRT", format!("{}x{}", m1, m2)),
            &(m1, m2),
            |bench_chirho, &(m1_chirho, m2_chirho)| {
                let a_chirho = SymbolicDomainChirho::modular_chirho(1, m1_chirho);
                let b_chirho = SymbolicDomainChirho::modular_chirho(2, m2_chirho);
                bench_chirho.iter(|| {
                    black_box(a_chirho.intersect_chirho(&b_chirho))
                })
            },
        );
    }

    // Single modular containment (not 1000 iterations)
    group_chirho.bench_function("ModContains_single", |bench_chirho| {
        let domain_chirho = SymbolicDomainChirho::modular_chirho(0, 7);
        bench_chirho.iter(|| {
            black_box(domain_chirho.contains_chirho(42))
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Benchmark 3: Hardware-Accelerated Symbolic
// ============================================================================

fn bench_hw_symbolic_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("HwSymbolicChirho");

    // Range materialization
    for (lo, hi) in [(0, 10), (0, 32), (0, 63), (10, 50)] {
        group_chirho.bench_with_input(
            BenchmarkId::new("RangeBits", format!("{}-{}", lo, hi)),
            &(lo, hi),
            |bench_chirho, &(lo_chirho, hi_chirho)| {
                bench_chirho.iter(|| {
                    black_box(range_bits_chirho(lo_chirho, hi_chirho))
                })
            },
        );
    }

    // Modular lookup table
    for modulus_chirho in [2, 3, 5, 7, 10] {
        group_chirho.bench_with_input(
            BenchmarkId::new("ModLUT", modulus_chirho),
            &modulus_chirho,
            |bench_chirho, &m_chirho| {
                bench_chirho.iter(|| {
                    black_box(mod_lut_chirho(0, m_chirho))
                })
            },
        );
    }

    // Combined range + mod (the hardware sweet spot)
    for (range_size, modulus) in [(64, 2), (64, 5), (64, 10), (32, 3)] {
        group_chirho.bench_with_input(
            BenchmarkId::new("RangeMod", format!("{}mod{}", range_size, modulus)),
            &(range_size, modulus),
            |bench_chirho, &(r_chirho, m_chirho)| {
                bench_chirho.iter(|| {
                    black_box(range_mod_hw_chirho(0, r_chirho - 1, 0, m_chirho))
                })
            },
        );
    }

    // Full HwSymbolicDomain with multiple constraints
    group_chirho.bench_function("MultiConstraint", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut domain_chirho = HwSymbolicDomainChirho::new_chirho(64)
                .with_range_chirho(5, 55)
                .with_modular_chirho(0, 5)
                .with_not_equal_chirho(25);
            black_box(domain_chirho.materialize_hw_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Benchmark 4: Hybrid Domain Operations
// ============================================================================

fn bench_hybrid_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("HybridChirho");

    // Small + Small intersection
    group_chirho.bench_function("Small_Small", |bench_chirho| {
        let a_chirho = HybridDomainChirho::Small(BitVec64Chirho(0xFFFF));
        let b_chirho = HybridDomainChirho::Small(BitVec64Chirho(0xFF00));
        bench_chirho.iter(|| {
            black_box(a_chirho.intersect_chirho(&b_chirho))
        })
    });

    // Small + Small + Small intersection (3-way)
    group_chirho.bench_function("Small_Small_Small", |bench_chirho| {
        let a_chirho = HybridDomainChirho::Small(BitVec64Chirho(0xFFFFFFFF));
        let b_chirho = HybridDomainChirho::Small(BitVec64Chirho(0xFF00FF00));
        let c_chirho = HybridDomainChirho::Small(BitVec64Chirho(0x0F0F0F0F));
        bench_chirho.iter(|| {
            let ab_chirho = a_chirho.intersect_chirho(&b_chirho);
            black_box(ab_chirho.intersect_chirho(&c_chirho))
        })
    });

    // Small + Symbolic intersection
    group_chirho.bench_function("Small_Symbolic", |bench_chirho| {
        let small_chirho = HybridDomainChirho::Small(BitVec64Chirho(0xFFFFFFFF));
        let sym_chirho = HybridDomainChirho::Symbolic(
            SymbolicDomainChirho::modular_chirho(0, 3) // Multiples of 3
        );
        bench_chirho.iter(|| {
            black_box(small_chirho.intersect_chirho(&sym_chirho))
        })
    });

    // Large paged intersection
    group_chirho.bench_function("Large_Large", |bench_chirho| {
        let a_chirho = HybridDomainChirho::Large(PagedDomainChirho::range_chirho(1000));
        let b_chirho = HybridDomainChirho::Large(PagedDomainChirho::range_chirho(500));
        bench_chirho.iter(|| {
            black_box(a_chirho.intersect_chirho(&b_chirho))
        })
    });

    // Try-to-small conversion (important for hardware path)
    for size_chirho in [32, 64, 100, 256] {
        group_chirho.bench_with_input(
            BenchmarkId::new("TryToSmall", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let domain_chirho = if n_chirho <= 64 {
                    HybridDomainChirho::Small(BitVec64Chirho((1u64 << n_chirho.min(64)) - 1))
                } else {
                    HybridDomainChirho::Large(PagedDomainChirho::range_chirho(n_chirho as u64))
                };
                bench_chirho.iter(|| {
                    black_box(domain_chirho.try_to_small_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 5: Enumeration / Iteration
// ============================================================================

fn bench_enumeration_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("EnumerationChirho");

    // BitVec64 iteration (baseline)
    group_chirho.bench_function("BitVec64_iter", |bench_chirho| {
        let bits_chirho = BitVec64Chirho(0xAAAAAAAAAAAAAAAA); // Alternating bits
        bench_chirho.iter(|| {
            let mut count_chirho = 0u32;
            let mut b = bits_chirho.0;
            while b != 0 {
                let _ = b.trailing_zeros();
                b &= b - 1;
                count_chirho += 1;
            }
            black_box(count_chirho)
        })
    });

    // Paged domain iteration (smaller sizes to avoid long benchmarks)
    for size_chirho in [32, 64, 128] {
        group_chirho.bench_with_input(
            BenchmarkId::new("Paged_iter", size_chirho),
            &size_chirho,
            |bench_chirho, &n_chirho| {
                let domain_chirho = PagedDomainChirho::range_chirho(n_chirho as u64);
                bench_chirho.iter(|| {
                    black_box(domain_chirho.iter_chirho().count())
                })
            },
        );
    }

    // Symbolic enumeration (when materializable)
    group_chirho.bench_function("Symbolic_enumerate", |bench_chirho| {
        let domain_chirho = SymbolicDomainChirho::range_chirho(0, 63);
        bench_chirho.iter(|| {
            black_box(domain_chirho.try_enumerate_chirho(64))
        })
    });

    // HwSymbolic materialization
    group_chirho.bench_function("HwSymbolic_materialize", |bench_chirho| {
        bench_chirho.iter(|| {
            let mut domain_chirho = HwSymbolicDomainChirho::new_chirho(64)
                .with_range_chirho(0, 63)
                .with_modular_chirho(0, 2);
            black_box(domain_chirho.materialize_hw_chirho())
        })
    });

    group_chirho.finish();
}

// ============================================================================
// Benchmark 6: Real Problem Simulation
// ============================================================================

fn bench_problem_simulation_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("ProblemSimChirho");

    // Simulate Sudoku-like constraint: 9 values, many intersections
    group_chirho.bench_function("Sudoku_like", |bench_chirho| {
        bench_chirho.iter(|| {
            // 81 cells, each starts with domain {1..9}
            let mut domains_chirho = vec![BitVec64Chirho(0b1111111110); 81]; // bits 1-9

            // Simulate constraint propagation (random-ish intersections)
            for i in 0..81 {
                let row = i / 9;
                let col = i % 9;

                // Row constraint
                for j in 0..9 {
                    if j != col {
                        let other = row * 9 + j;
                        domains_chirho[i] = domains_chirho[i].and_chirho(
                            domains_chirho[other].not_chirho().or_chirho(domains_chirho[i])
                        );
                    }
                }
            }

            black_box(domains_chirho[0])
        })
    });

    // Simulate N-Queens-like: column domains with diagonal constraints
    group_chirho.bench_function("NQueens_like", |bench_chirho| {
        bench_chirho.iter(|| {
            let n = 12;
            let mut col_available = BitVec64Chirho((1u64 << n) - 1);
            let mut diag1 = BitVec64Chirho(0);
            let mut diag2 = BitVec64Chirho(0);

            for _row in 0..n {
                let available = col_available
                    .and_chirho(diag1.not_chirho())
                    .and_chirho(diag2.not_chirho());

                if !available.is_zero_chirho() {
                    // Pick first available
                    let chosen = available.lowest_bit_chirho();
                    col_available = col_available.and_chirho(chosen.not_chirho());
                    diag1 = BitVec64Chirho((diag1.0 | chosen.0) << 1);
                    diag2 = BitVec64Chirho((diag2.0 | chosen.0) >> 1);
                }
            }

            black_box(col_available)
        })
    });

    // Large scheduling problem: 256 time slots
    group_chirho.bench_function("Scheduling_256", |bench_chirho| {
        bench_chirho.iter(|| {
            // 20 tasks, each can be in 256 time slots
            let mut domains_chirho: Vec<PagedDomainChirho> = (0..20)
                .map(|_| PagedDomainChirho::range_chirho(255))
                .collect();

            // Constraint: tasks 0 and 1 can't overlap
            let d0 = domains_chirho[0].clone();
            let d1 = domains_chirho[1].clone();
            // Naive: just intersect with shifted versions (simulating "not same time")
            domains_chirho[0] = d0.intersect_chirho(&d1);

            black_box(domains_chirho[0].count_chirho())
        })
    });

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_intersection_by_size_chirho,
    bench_symbolic_ops_chirho,
    bench_hw_symbolic_chirho,
    bench_hybrid_chirho,
    bench_enumeration_chirho,
    bench_problem_simulation_chirho,
);

criterion_main!(benches_chirho);
