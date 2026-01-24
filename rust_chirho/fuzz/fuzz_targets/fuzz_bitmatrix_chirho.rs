//! Fuzz target for bit matrices ☧
//!
//! Tests that bit matrix operations don't panic and maintain invariants.

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use minikanren_1bit_chirho::BitMatrix64Chirho;

/// Operations on bit matrices
#[derive(Debug, Clone)]
enum MatrixOpChirho {
    SetChirho(u8, u8),
    ClearChirho(u8, u8),
    AndChirho,
    OrChirho,
    MatmulChirho,
    TransitiveClosureChirho,
}

#[derive(Debug)]
struct FuzzInputChirho {
    size_chirho: u8,
    ops_chirho: Vec<MatrixOpChirho>,
}

impl<'a> Arbitrary<'a> for FuzzInputChirho {
    fn arbitrary(u_chirho: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let size_chirho = u_chirho.int_in_range(1..=32)?;
        let num_ops_chirho = u_chirho.int_in_range(0..=20)?;

        let mut ops_chirho = Vec::with_capacity(num_ops_chirho);
        for _ in 0..num_ops_chirho {
            let op_chirho = match u_chirho.int_in_range(0..=5)? {
                0 => MatrixOpChirho::SetChirho(
                    u_chirho.int_in_range(0..=size_chirho - 1)?,
                    u_chirho.int_in_range(0..=size_chirho - 1)?,
                ),
                1 => MatrixOpChirho::ClearChirho(
                    u_chirho.int_in_range(0..=size_chirho - 1)?,
                    u_chirho.int_in_range(0..=size_chirho - 1)?,
                ),
                2 => MatrixOpChirho::AndChirho,
                3 => MatrixOpChirho::OrChirho,
                4 => MatrixOpChirho::MatmulChirho,
                _ => MatrixOpChirho::TransitiveClosureChirho,
            };
            ops_chirho.push(op_chirho);
        }

        Ok(FuzzInputChirho { size_chirho, ops_chirho })
    }
}

fuzz_target!(|input_chirho: FuzzInputChirho| {
    let n_chirho = input_chirho.size_chirho;
    let mut m1_chirho = BitMatrix64Chirho::new_chirho(n_chirho, n_chirho);
    let mut m2_chirho = BitMatrix64Chirho::new_chirho(n_chirho, n_chirho);

    for op_chirho in input_chirho.ops_chirho {
        match op_chirho {
            MatrixOpChirho::SetChirho(r_chirho, c_chirho) => {
                m1_chirho.set_chirho(r_chirho, c_chirho);
            }
            MatrixOpChirho::ClearChirho(r_chirho, c_chirho) => {
                m1_chirho.clear_chirho(r_chirho, c_chirho);
            }
            MatrixOpChirho::AndChirho => {
                let _result_chirho = m1_chirho.and_chirho(&m2_chirho);
            }
            MatrixOpChirho::OrChirho => {
                let _result_chirho = m1_chirho.or_chirho(&m2_chirho);
            }
            MatrixOpChirho::MatmulChirho => {
                let _result_chirho = m1_chirho.matmul_chirho(&m2_chirho);
            }
            MatrixOpChirho::TransitiveClosureChirho => {
                let tc_chirho = m1_chirho.transitive_closure_chirho();
                // Invariant: TC contains original
                for r_chirho in 0..n_chirho {
                    for c_chirho in 0..n_chirho {
                        if m1_chirho.get_chirho(r_chirho, c_chirho) {
                            assert!(tc_chirho.get_chirho(r_chirho, c_chirho));
                        }
                    }
                }
            }
        }
        // Swap matrices sometimes
        std::mem::swap(&mut m1_chirho, &mut m2_chirho);
    }
});
