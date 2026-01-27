//! Benchmark: Heap-based vs Hardware Optics ☧

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;
use minikanren_1bit_chirho::optics_hw_chirho::{
    DomainHwChirho, PrismHwChirho, LensHwChirho, StateHwChirho,
    PartitionedDomainChirho, traverse_all_chirho,
};

// ============================================================================
// Hardware Optics Benchmarks
// ============================================================================

fn bench_domain_intersect_hw_chirho(c: &mut Criterion) {
    let d1_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xAAAA_AAAA_AAAA_AAAA) };
    let d2_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0x5555_5555_5555_5555) };

    c.bench_function("hw_domain_intersect", |b| {
        b.iter(|| {
            black_box(d1_chirho.intersect_chirho(&d2_chirho))
        })
    });
}

fn bench_domain_union_hw_chirho(c: &mut Criterion) {
    let d1_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xAAAA_AAAA_AAAA_AAAA) };
    let d2_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0x5555_5555_5555_5555) };

    c.bench_function("hw_domain_union", |b| {
        b.iter(|| {
            black_box(d1_chirho.union_chirho(&d2_chirho))
        })
    });
}

fn bench_prism_preview_hw_chirho(c: &mut Criterion) {
    let prism_chirho = PrismHwChirho::new_chirho(BitVec64Chirho(0xFF00_FF00_FF00_FF00));
    let domain_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xFFFF_FFFF_FFFF_FFFF) };

    c.bench_function("hw_prism_preview", |b| {
        b.iter(|| {
            black_box(prism_chirho.preview_chirho(&domain_chirho))
        })
    });
}

fn bench_lens_get_set_hw_chirho(c: &mut Criterion) {
    let state_chirho = StateHwChirho::<16>::new_all_full_chirho();
    let lens_chirho = LensHwChirho::new_chirho(7);
    let new_domain_chirho = DomainHwChirho::singleton_chirho(42);

    c.bench_function("hw_lens_get", |b| {
        b.iter(|| {
            black_box(lens_chirho.get_chirho(&state_chirho))
        })
    });

    c.bench_function("hw_lens_set", |b| {
        b.iter(|| {
            black_box(lens_chirho.set_chirho(state_chirho, new_domain_chirho))
        })
    });
}

fn bench_traverse_all_hw_chirho(c: &mut Criterion) {
    let state_chirho = StateHwChirho::<16>::new_all_full_chirho();
    let mask_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xFF) };

    c.bench_function("hw_traverse_all_16", |b| {
        b.iter(|| {
            black_box(traverse_all_chirho(state_chirho, |d| d.intersect_chirho(&mask_chirho)))
        })
    });
}

fn bench_partitioned_domain_chirho(c: &mut Criterion) {
    let d1_chirho = PartitionedDomainChirho::singleton_chirho(100);
    let d2_chirho = PartitionedDomainChirho::singleton_chirho(200);

    c.bench_function("hw_partitioned_intersect", |b| {
        b.iter(|| {
            black_box(d1_chirho.intersect_chirho(&d2_chirho))
        })
    });

    c.bench_function("hw_partitioned_union", |b| {
        b.iter(|| {
            black_box(d1_chirho.union_chirho(&d2_chirho))
        })
    });
}

// ============================================================================
// Scaling benchmarks
// ============================================================================

fn bench_mass_intersect_chirho(c: &mut Criterion) {
    let mut group = c.benchmark_group("mass_intersect_chirho");

    for n in [10, 100, 1000, 10000].iter() {
        // Hardware: intersect n domains
        group.bench_with_input(BenchmarkId::new("hw", n), n, |b, &n| {
            let domains_chirho: Vec<_> = (0..n)
                .map(|i| DomainHwChirho { bits_chirho: BitVec64Chirho(i as u64 | 0xFF) })
                .collect();

            b.iter(|| {
                let mut acc_chirho = DomainHwChirho::full_chirho();
                for d in &domains_chirho {
                    acc_chirho = acc_chirho.intersect_chirho(d);
                }
                black_box(acc_chirho)
            })
        });

        // Heap-based: simulate with HashSet (what optics_chirho.rs would do)
        group.bench_with_input(BenchmarkId::new("heap_hashset", n), n, |b, &n| {
            use std::collections::HashSet;
            let domains_chirho: Vec<HashSet<u32>> = (0..n)
                .map(|i| (0..64u32).filter(|x| (i as u64 | 0xFF) & (1u64 << x) != 0).collect())
                .collect();

            b.iter(|| {
                let mut acc_chirho: HashSet<u32> = (0..64).collect();
                for d in &domains_chirho {
                    acc_chirho = acc_chirho.intersection(d).copied().collect();
                }
                black_box(acc_chirho)
            })
        });
    }

    group.finish();
}

fn bench_popcount_chirho(c: &mut Criterion) {
    let domain_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0xDEAD_BEEF_CAFE_BABE) };

    c.bench_function("hw_popcount", |b| {
        b.iter(|| {
            black_box(domain_chirho.popcount_chirho())
        })
    });
}

fn bench_singleton_check_chirho(c: &mut Criterion) {
    let singleton_chirho = DomainHwChirho::singleton_chirho(42);
    let non_singleton_chirho = DomainHwChirho { bits_chirho: BitVec64Chirho(0b1111) };

    c.bench_function("hw_is_singleton_yes", |b| {
        b.iter(|| {
            black_box(singleton_chirho.is_singleton_chirho())
        })
    });

    c.bench_function("hw_is_singleton_no", |b| {
        b.iter(|| {
            black_box(non_singleton_chirho.is_singleton_chirho())
        })
    });
}

criterion_group!(
    benches_chirho,
    bench_domain_intersect_hw_chirho,
    bench_domain_union_hw_chirho,
    bench_prism_preview_hw_chirho,
    bench_lens_get_set_hw_chirho,
    bench_traverse_all_hw_chirho,
    bench_partitioned_domain_chirho,
    bench_mass_intersect_chirho,
    bench_popcount_chirho,
    bench_singleton_check_chirho,
);

criterion_main!(benches_chirho);
