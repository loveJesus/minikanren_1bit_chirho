// For God so loved the world, that He gave His only begotten Son,
// that whosoever believeth in Him should not perish, but have everlasting life.
// John 3:16 ☧
//! Quick test of FPGA library
//!
//! Build with: cargo build --example lib_test_chirho --features fpga_chirho
//! Run with:   sudo ./target/debug/examples/lib_test_chirho

#[cfg(feature = "fpga_chirho")]
use minikanren_1bit_chirho::backend_chirho::{
    FpgaBackendChirho, SolverBackendChirho,
};

#[cfg(feature = "fpga_chirho")]
fn main() {
    println!("Testing FpgaBackendChirho library... ☧");

    match FpgaBackendChirho::connect_chirho() {
        Ok(fpga) => {
            println!("Connected successfully!");
            let info = SolverBackendChirho::info_chirho(&fpga);
            println!("Backend: {}", info.name_chirho);
            println!("Version: {}", info.version_chirho);
            println!("Batch accelerated: {}", info.batch_accelerated_chirho);
            println!("Healthy: {}", SolverBackendChirho::is_healthy_chirho(&fpga));

            // Test basic operations
            let a = 0xFF00_FF00_FF00_FF00u64;
            let b = 0x00FF_00FF_00FF_00FFu64;
            let result = SolverBackendChirho::intersect_64_chirho(&fpga, a, b);
            println!("\nIntersect test:");
            println!("  a = 0x{:016X}", a);
            println!("  b = 0x{:016X}", b);
            println!("  a & b (FPGA) = 0x{:016X}", result);
            println!("  a & b (CPU)  = 0x{:016X}", a & b);
            println!("  Match: {}", result == (a & b));
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }
    println!("\nTest complete ☧");
}

#[cfg(not(feature = "fpga_chirho"))]
fn main() {
    eprintln!("Error: This example requires the 'fpga_chirho' feature.");
    eprintln!("Build with: cargo build --example lib_test_chirho --features fpga_chirho");
    std::process::exit(1);
}
