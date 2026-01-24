//! Fuzz target for unification ☧
//!
//! Tests that unification doesn't panic on arbitrary term pairs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use minikanren_1bit_chirho::{TermStoreChirho, SubstChirho, unify_chirho};

/// Arbitrary term structure for fuzzing
#[derive(Debug, Clone)]
enum FuzzTermChirho {
    VarChirho(u8),
    IntChirho(i64),
    NilChirho,
    ConsChirho(Box<FuzzTermChirho>, Box<FuzzTermChirho>),
}

impl<'a> Arbitrary<'a> for FuzzTermChirho {
    fn arbitrary(u_chirho: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Limit depth to avoid stack overflow
        Self::arbitrary_depth_chirho(u_chirho, 5)
    }
}

impl FuzzTermChirho {
    fn arbitrary_depth_chirho(u_chirho: &mut Unstructured, depth_chirho: usize) -> arbitrary::Result<Self> {
        if depth_chirho == 0 {
            // At max depth, only emit leaves
            let choice_chirho = u_chirho.int_in_range(0..=2)?;
            match choice_chirho {
                0 => Ok(FuzzTermChirho::VarChirho(u_chirho.int_in_range(0..=7)?)),
                1 => Ok(FuzzTermChirho::IntChirho(u_chirho.arbitrary()?)),
                _ => Ok(FuzzTermChirho::NilChirho),
            }
        } else {
            let choice_chirho = u_chirho.int_in_range(0..=3)?;
            match choice_chirho {
                0 => Ok(FuzzTermChirho::VarChirho(u_chirho.int_in_range(0..=7)?)),
                1 => Ok(FuzzTermChirho::IntChirho(u_chirho.arbitrary()?)),
                2 => Ok(FuzzTermChirho::NilChirho),
                _ => Ok(FuzzTermChirho::ConsChirho(
                    Box::new(Self::arbitrary_depth_chirho(u_chirho, depth_chirho - 1)?),
                    Box::new(Self::arbitrary_depth_chirho(u_chirho, depth_chirho - 1)?),
                )),
            }
        }
    }

    fn to_term_chirho(&self, store_chirho: &mut TermStoreChirho, var_map_chirho: &mut [Option<u32>; 8]) -> u32 {
        match self {
            FuzzTermChirho::VarChirho(v_chirho) => {
                if var_map_chirho[*v_chirho as usize].is_none() {
                    let (var_id_chirho, term_id_chirho) = store_chirho.fresh_var_chirho();
                    var_map_chirho[*v_chirho as usize] = Some(term_id_chirho);
                    let _ = var_id_chirho; // suppress warning
                }
                var_map_chirho[*v_chirho as usize].unwrap()
            }
            FuzzTermChirho::IntChirho(n_chirho) => store_chirho.int_chirho(*n_chirho),
            FuzzTermChirho::NilChirho => store_chirho.nil_chirho(),
            FuzzTermChirho::ConsChirho(h_chirho, t_chirho) => {
                let head_id_chirho = h_chirho.to_term_chirho(store_chirho, var_map_chirho);
                let tail_id_chirho = t_chirho.to_term_chirho(store_chirho, var_map_chirho);
                store_chirho.cons_chirho(head_id_chirho, tail_id_chirho)
            }
        }
    }
}

fuzz_target!(|data_chirho: (FuzzTermChirho, FuzzTermChirho)| {
    let (t1_chirho, t2_chirho) = data_chirho;

    let mut store_chirho = TermStoreChirho::new();
    let mut var_map_chirho: [Option<u32>; 8] = [None; 8];

    let term1_chirho = t1_chirho.to_term_chirho(&mut store_chirho, &mut var_map_chirho);
    let term2_chirho = t2_chirho.to_term_chirho(&mut store_chirho, &mut var_map_chirho);

    let mut subst_chirho = SubstChirho::new();

    // This should not panic
    let _result_chirho = unify_chirho(term1_chirho, term2_chirho, &mut subst_chirho, &store_chirho);
});
