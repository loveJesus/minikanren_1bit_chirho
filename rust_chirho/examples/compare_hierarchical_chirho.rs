// For God so loved the world that He gave His only begotten Son
// that all who believe in Him should not perish but have everlasting life.
//! Compare hierarchical domain structures ☧
//!
//! Benchmarks:
//! - 64³ = 262,144 values (3-level, 64-bit words)
//! - 256² = 65,536 values (2-level, 256-bit words)
//! - 512² = 262,144 values (2-level, 512-bit words)

use minikanren_1bit_chirho::approaches_chirho::{
    Hierarchical4kChirho, Hierarchical16kChirho, Hierarchical256kChirho,
    Hierarchical65kChirho, Hierarchical262kWideChirho,
};
use std::time::Instant;

fn main() {
    println!("Hierarchical Domain Comparison ☧");
    println!("================================\n");

    // Number of intersection operations
    let n_ops_chirho = 10000;

    // Test at different domain densities
    for &fill_ratio_chirho in &[0.1, 0.5, 0.9] {
        println!("Fill ratio: {:.0}%", fill_ratio_chirho * 100.0);
        println!("-----------------");

        // 64² = 4,096 values (baseline)
        {
            let size_chirho = (4096.0 * fill_ratio_chirho) as u32;
            let a_chirho = Hierarchical4kChirho::range_chirho(size_chirho);
            let b_chirho = Hierarchical4kChirho::range_chirho(size_chirho);

            let start_chirho = Instant::now();
            for _ in 0..n_ops_chirho {
                let _ = std::hint::black_box(a_chirho.intersect_chirho(&b_chirho));
            }
            let elapsed_chirho = start_chirho.elapsed();
            let ns_per_op_chirho = elapsed_chirho.as_nanos() as f64 / n_ops_chirho as f64;
            println!("  64² (4K, 2-level):    {:>8.1} ns/op", ns_per_op_chirho);
        }

        // 64 × 256 = 16,384 values
        {
            let size_chirho = (16384.0 * fill_ratio_chirho) as u32;
            let a_chirho = Hierarchical16kChirho::range_chirho(size_chirho);
            let b_chirho = Hierarchical16kChirho::range_chirho(size_chirho);

            let start_chirho = Instant::now();
            for _ in 0..n_ops_chirho {
                let _ = std::hint::black_box(a_chirho.intersect_chirho(&b_chirho));
            }
            let elapsed_chirho = start_chirho.elapsed();
            let ns_per_op_chirho = elapsed_chirho.as_nanos() as f64 / n_ops_chirho as f64;
            println!("  64×256 (16K, 2-level): {:>8.1} ns/op", ns_per_op_chirho);
        }

        // 256² = 65,536 values (2-level, wide)
        {
            let size_chirho = (65536.0 * fill_ratio_chirho) as u32;
            let a_chirho = Hierarchical65kChirho::range_chirho(size_chirho);
            let b_chirho = Hierarchical65kChirho::range_chirho(size_chirho);

            let start_chirho = Instant::now();
            for _ in 0..n_ops_chirho {
                let _ = std::hint::black_box(a_chirho.intersect_chirho(&b_chirho));
            }
            let elapsed_chirho = start_chirho.elapsed();
            let ns_per_op_chirho = elapsed_chirho.as_nanos() as f64 / n_ops_chirho as f64;
            println!("  256² (65K, 2-level):   {:>8.1} ns/op", ns_per_op_chirho);
        }

        // 64³ = 262,144 values (3-level, deep)
        {
            let size_chirho = (262144.0 * fill_ratio_chirho) as u32;
            let a_chirho = Hierarchical256kChirho::range_chirho(size_chirho);
            let b_chirho = Hierarchical256kChirho::range_chirho(size_chirho);

            let start_chirho = Instant::now();
            for _ in 0..n_ops_chirho {
                let _ = std::hint::black_box(a_chirho.intersect_chirho(&b_chirho));
            }
            let elapsed_chirho = start_chirho.elapsed();
            let ns_per_op_chirho = elapsed_chirho.as_nanos() as f64 / n_ops_chirho as f64;
            println!("  64³ (262K, 3-level):   {:>8.1} ns/op", ns_per_op_chirho);
        }

        // 512² = 262,144 values (2-level, wide)
        {
            let size_chirho = (262144.0 * fill_ratio_chirho) as u32;
            let a_chirho = Hierarchical262kWideChirho::range_chirho(size_chirho);
            let b_chirho = Hierarchical262kWideChirho::range_chirho(size_chirho);

            let start_chirho = Instant::now();
            for _ in 0..n_ops_chirho {
                let _ = std::hint::black_box(a_chirho.intersect_chirho(&b_chirho));
            }
            let elapsed_chirho = start_chirho.elapsed();
            let ns_per_op_chirho = elapsed_chirho.as_nanos() as f64 / n_ops_chirho as f64;
            println!("  512² (262K, 2-level):  {:>8.1} ns/op", ns_per_op_chirho);
        }

        println!();
    }

    // Memory comparison
    println!("Memory Usage:");
    println!("-------------");
    println!("  64² (4K):   {} bytes", std::mem::size_of::<Hierarchical4kChirho>());
    println!("  64×256 (16K): {} bytes", std::mem::size_of::<Hierarchical16kChirho>());
    println!("  256² (65K): {} bytes", std::mem::size_of::<Hierarchical65kChirho>());
    println!("  64³ (262K): {} bytes", std::mem::size_of::<Hierarchical256kChirho>());
    println!("  512² (262K): {} bytes", std::mem::size_of::<Hierarchical262kWideChirho>());

    println!("\nConclusion:");
    println!("-----------");
    println!("The 512² (2-level, 512-bit) vs 64³ (3-level, 64-bit) trade-off depends on:");
    println!("  - CPU SIMD support (AVX-512 vs AVX2)");
    println!("  - Domain sparsity (sparse favors deep trees with early exit)");
    println!("  - Cache behavior (wider words may cause more cache misses)");
}
