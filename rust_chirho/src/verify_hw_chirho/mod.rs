//! Verilator Verification Bridge ☧
//!
//! Co-simulation harness to verify Clash Verilog against Rust reference.
//! Part of P5-00: Bridge of Truth.
//!
//! "Sanctify them through thy truth: thy word is truth." — John 17:17

mod engine_chirho;
mod cmd_chirho;
mod verilator_chirho;

pub use engine_chirho::*;
pub use cmd_chirho::*;
pub use verilator_chirho::*;

#[cfg(test)]
mod proptest_chirho;
