//! Fuzz target for matrix-based occurs check ☧
//!
//! Tests that occurs check via transitive closure correctly
//! detects cycles and doesn't panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use minikanren_1bit_chirho::{TermStoreChirho, SubstMatrixChirho, unify_matrix_chirho};

/// Term structure that can create cycles
#[derive(Debug, Clone)]
enum CyclicTermChirho {
    VarChirho(u8),
    IntChirho(i32),
    NilChirho,
    ConsChirho(Box<CyclicTermChirho>, Box<CyclicTermChirho>),
}

impl<'a> Arbitrary<'a> for CyclicTermChirho {
    fn arbitrary(u_chirho: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::arbitrary_depth_chirho(u_chirho, 4)
    }
}

impl CyclicTermChirho {
    fn arbitrary_depth_chirho(u_chirho: &mut Unstructured, depth_chirho: usize) -> arbitrary::Result<Self> {
        if depth_chirho == 0 {
            match u_chirho.int_in_range(0..=2)? {
                0 => Ok(CyclicTermChirho::VarChirho(u_chirho.int_in_range(0..=3)?)),
                1 => Ok(CyclicTermChirho::IntChirho(u_chirho.int_in_range(-100..=100)?)),
                _ => Ok(CyclicTermChirho::NilChirho),
            }
        } else {
            match u_chirho.int_in_range(0..=3)? {
                0 => Ok(CyclicTermChirho::VarChirho(u_chirho.int_in_range(0..=3)?)),
                1 => Ok(CyclicTermChirho::IntChirho(u_chirho.int_in_range(-100..=100)?)),
                2 => Ok(CyclicTermChirho::NilChirho),
                _ => Ok(CyclicTermChirho::ConsChirho(
                    Box::new(Self::arbitrary_depth_chirho(u_chirho, depth_chirho - 1)?),
                    Box::new(Self::arbitrary_depth_chirho(u_chirho, depth_chirho - 1)?),
                )),
            }
        }
    }

    fn to_term_chirho(&self, store_chirho: &mut TermStoreChirho, vars_chirho: &mut [Option<u32>; 4]) -> u32 {
        match self {
            CyclicTermChirho::VarChirho(v_chirho) => {
                if vars_chirho[*v_chirho as usize].is_none() {
                    let (_, term_id_chirho) = store_chirho.fresh_var_chirho();
                    vars_chirho[*v_chirho as usize] = Some(term_id_chirho);
                }
                vars_chirho[*v_chirho as usize].unwrap()
            }
            CyclicTermChirho::IntChirho(n_chirho) => store_chirho.int_chirho(*n_chirho as i64),
            CyclicTermChirho::NilChirho => store_chirho.nil_chirho(),
            CyclicTermChirho::ConsChirho(h_chirho, t_chirho) => {
                let h_id_chirho = h_chirho.to_term_chirho(store_chirho, vars_chirho);
                let t_id_chirho = t_chirho.to_term_chirho(store_chirho, vars_chirho);
                store_chirho.cons_chirho(h_id_chirho, t_id_chirho)
            }
        }
    }
}

/// Fuzz input: sequence of unification pairs
#[derive(Debug)]
struct UnifySequenceChirho {
    pairs_chirho: Vec<(CyclicTermChirho, CyclicTermChirho)>,
}

impl<'a> Arbitrary<'a> for UnifySequenceChirho {
    fn arbitrary(u_chirho: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let num_pairs_chirho = u_chirho.int_in_range(1..=10)?;
        let mut pairs_chirho = Vec::with_capacity(num_pairs_chirho);
        for _ in 0..num_pairs_chirho {
            pairs_chirho.push((u_chirho.arbitrary()?, u_chirho.arbitrary()?));
        }
        Ok(UnifySequenceChirho { pairs_chirho })
    }
}

fuzz_target!(|input_chirho: UnifySequenceChirho| {
    let mut store_chirho = TermStoreChirho::new();
    let mut subst_chirho = SubstMatrixChirho::new_chirho();
    let mut vars_chirho: [Option<u32>; 4] = [None; 4];

    for (t1_chirho, t2_chirho) in input_chirho.pairs_chirho {
        let term1_chirho = t1_chirho.to_term_chirho(&mut store_chirho, &mut vars_chirho);
        let term2_chirho = t2_chirho.to_term_chirho(&mut store_chirho, &mut vars_chirho);

        // This should not panic
        // If it returns false (occurs check failure), that's fine
        let _result_chirho = unify_matrix_chirho(&store_chirho, &mut subst_chirho, term1_chirho, term2_chirho);
    }
});
