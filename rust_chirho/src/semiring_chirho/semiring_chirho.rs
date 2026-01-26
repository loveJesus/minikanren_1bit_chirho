// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Semiring Abstraction ☧
//!
//! Generalize Boolean logic to other semirings:
//! - Boolean: OR for +, AND for * (standard logic)
//! - Probabilistic: + for +, * for * (probabilities)
//! - Tropical: min for +, + for * (shortest paths)
//! - Counting: + for +, * for * over naturals (count solutions)

use std::ops::{Add, Mul};
use std::fmt::Debug;

/// A semiring with additive identity (zero) and multiplicative identity (one)
pub trait SemiringChirho: Clone + Debug + PartialEq + Add<Output = Self> + Mul<Output = Self> {
    /// Additive identity (failure/zero probability/infinity)
    fn zero_chirho() -> Self;

    /// Multiplicative identity (success/probability 1/zero cost)
    fn one_chirho() -> Self;

    /// Check if this is the zero element
    fn is_zero_chirho(&self) -> bool {
        *self == Self::zero_chirho()
    }
}

/// Boolean semiring: OR for +, AND for *
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoolSemiringChirho(pub bool);

impl Add for BoolSemiringChirho {
    type Output = Self;
    fn add(self, rhs_chirho: Self) -> Self {
        BoolSemiringChirho(self.0 || rhs_chirho.0)
    }
}

impl Mul for BoolSemiringChirho {
    type Output = Self;
    fn mul(self, rhs_chirho: Self) -> Self {
        BoolSemiringChirho(self.0 && rhs_chirho.0)
    }
}

impl SemiringChirho for BoolSemiringChirho {
    fn zero_chirho() -> Self {
        BoolSemiringChirho(false)
    }
    fn one_chirho() -> Self {
        BoolSemiringChirho(true)
    }
}

/// Probabilistic semiring: + for +, * for *
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ProbSemiringChirho(pub f64);

impl Add for ProbSemiringChirho {
    type Output = Self;
    fn add(self, rhs_chirho: Self) -> Self {
        // Clamp to [0, 1] for stability
        ProbSemiringChirho((self.0 + rhs_chirho.0).min(1.0))
    }
}

impl Mul for ProbSemiringChirho {
    type Output = Self;
    fn mul(self, rhs_chirho: Self) -> Self {
        ProbSemiringChirho(self.0 * rhs_chirho.0)
    }
}

impl SemiringChirho for ProbSemiringChirho {
    fn zero_chirho() -> Self {
        ProbSemiringChirho(0.0)
    }
    fn one_chirho() -> Self {
        ProbSemiringChirho(1.0)
    }
}

/// Tropical semiring (min-plus): min for +, + for *
/// Good for shortest path / Viterbi
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TropicalSemiringChirho(pub f64);

impl TropicalSemiringChirho {
    pub const INFINITY_CHIRHO: f64 = f64::INFINITY;
}

impl Add for TropicalSemiringChirho {
    type Output = Self;
    fn add(self, rhs_chirho: Self) -> Self {
        TropicalSemiringChirho(self.0.min(rhs_chirho.0))
    }
}

impl Mul for TropicalSemiringChirho {
    type Output = Self;
    fn mul(self, rhs_chirho: Self) -> Self {
        TropicalSemiringChirho(self.0 + rhs_chirho.0)
    }
}

impl SemiringChirho for TropicalSemiringChirho {
    fn zero_chirho() -> Self {
        TropicalSemiringChirho(Self::INFINITY_CHIRHO) // Additive identity for min
    }
    fn one_chirho() -> Self {
        TropicalSemiringChirho(0.0) // Multiplicative identity for +
    }
}

/// Counting semiring: count number of derivations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CountSemiringChirho(pub u64);

impl Add for CountSemiringChirho {
    type Output = Self;
    fn add(self, rhs_chirho: Self) -> Self {
        CountSemiringChirho(self.0.saturating_add(rhs_chirho.0))
    }
}

impl Mul for CountSemiringChirho {
    type Output = Self;
    fn mul(self, rhs_chirho: Self) -> Self {
        CountSemiringChirho(self.0.saturating_mul(rhs_chirho.0))
    }
}

impl SemiringChirho for CountSemiringChirho {
    fn zero_chirho() -> Self {
        CountSemiringChirho(0)
    }
    fn one_chirho() -> Self {
        CountSemiringChirho(1)
    }
}

/// Log semiring: numerically stable probabilities
/// Stores log(p), uses log-sum-exp for addition
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LogSemiringChirho(pub f64);

impl LogSemiringChirho {
    pub const NEG_INFINITY_CHIRHO: f64 = f64::NEG_INFINITY;

    /// Log-sum-exp: log(exp(a) + exp(b)) with numerical stability
    fn log_sum_exp_chirho(a_chirho: f64, b_chirho: f64) -> f64 {
        if a_chirho == Self::NEG_INFINITY_CHIRHO {
            return b_chirho;
        }
        if b_chirho == Self::NEG_INFINITY_CHIRHO {
            return a_chirho;
        }
        let max_chirho = a_chirho.max(b_chirho);
        max_chirho + ((a_chirho - max_chirho).exp() + (b_chirho - max_chirho).exp()).ln()
    }
}

impl Add for LogSemiringChirho {
    type Output = Self;
    fn add(self, rhs_chirho: Self) -> Self {
        LogSemiringChirho(Self::log_sum_exp_chirho(self.0, rhs_chirho.0))
    }
}

impl Mul for LogSemiringChirho {
    type Output = Self;
    fn mul(self, rhs_chirho: Self) -> Self {
        LogSemiringChirho(self.0 + rhs_chirho.0)
    }
}

impl SemiringChirho for LogSemiringChirho {
    fn zero_chirho() -> Self {
        LogSemiringChirho(Self::NEG_INFINITY_CHIRHO)
    }
    fn one_chirho() -> Self {
        LogSemiringChirho(0.0) // log(1) = 0
    }
}

/// Weighted bit matrix: sparse matrix with semiring values
#[derive(Debug, Clone)]
pub struct WeightedMatrixChirho<S: SemiringChirho> {
    entries_chirho: std::collections::HashMap<(u32, u32), S>,
    num_rows_chirho: u32,
    num_cols_chirho: u32,
}

impl<S: SemiringChirho> WeightedMatrixChirho<S> {
    pub fn new(rows_chirho: u32, cols_chirho: u32) -> Self {
        Self {
            entries_chirho: std::collections::HashMap::new(),
            num_rows_chirho: rows_chirho,
            num_cols_chirho: cols_chirho,
        }
    }

    pub fn set_chirho(&mut self, row_chirho: u32, col_chirho: u32, val_chirho: S) {
        if !val_chirho.is_zero_chirho() {
            self.entries_chirho.insert((row_chirho, col_chirho), val_chirho);
            self.num_rows_chirho = self.num_rows_chirho.max(row_chirho + 1);
            self.num_cols_chirho = self.num_cols_chirho.max(col_chirho + 1);
        }
    }

    pub fn get_chirho(&self, row_chirho: u32, col_chirho: u32) -> S {
        self.entries_chirho
            .get(&(row_chirho, col_chirho))
            .cloned()
            .unwrap_or_else(S::zero_chirho)
    }

    /// Matrix multiplication with semiring operations
    pub fn matmul_chirho(&self, other_chirho: &WeightedMatrixChirho<S>) -> WeightedMatrixChirho<S> {
        let mut result_chirho = WeightedMatrixChirho::new(self.num_rows_chirho, other_chirho.num_cols_chirho);

        // Group entries by row for self
        let mut self_by_row_chirho: std::collections::HashMap<u32, Vec<(u32, S)>> =
            std::collections::HashMap::new();
        for ((r_chirho, c_chirho), v_chirho) in &self.entries_chirho {
            self_by_row_chirho
                .entry(*r_chirho)
                .or_default()
                .push((*c_chirho, v_chirho.clone()));
        }

        // Group entries by column for other
        let mut other_by_col_chirho: std::collections::HashMap<u32, Vec<(u32, S)>> =
            std::collections::HashMap::new();
        for ((r_chirho, c_chirho), v_chirho) in &other_chirho.entries_chirho {
            other_by_col_chirho
                .entry(*c_chirho)
                .or_default()
                .push((*r_chirho, v_chirho.clone()));
        }

        // Compute result
        for (row_chirho, row_entries_chirho) in &self_by_row_chirho {
            for (col_chirho, col_entries_chirho) in &other_by_col_chirho {
                let mut sum_chirho = S::zero_chirho();

                for (k1_chirho, v1_chirho) in row_entries_chirho {
                    for (k2_chirho, v2_chirho) in col_entries_chirho {
                        if k1_chirho == k2_chirho {
                            sum_chirho = sum_chirho + (v1_chirho.clone() * v2_chirho.clone());
                        }
                    }
                }

                if !sum_chirho.is_zero_chirho() {
                    result_chirho.set_chirho(*row_chirho, *col_chirho, sum_chirho);
                }
            }
        }

        result_chirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bool_semiring_chirho() {
        let t_chirho = BoolSemiringChirho(true);
        let f_chirho = BoolSemiringChirho(false);

        assert_eq!(t_chirho + f_chirho, t_chirho); // OR
        assert_eq!(t_chirho * f_chirho, f_chirho); // AND
        assert_eq!(f_chirho + f_chirho, f_chirho);
        assert_eq!(t_chirho * t_chirho, t_chirho);
    }

    #[test]
    fn test_prob_semiring_chirho() {
        let p1_chirho = ProbSemiringChirho(0.5);
        let p2_chirho = ProbSemiringChirho(0.3);

        let product_chirho = p1_chirho * p2_chirho;
        assert!((product_chirho.0 - 0.15).abs() < 1e-10);
    }

    #[test]
    fn test_tropical_semiring_chirho() {
        let a_chirho = TropicalSemiringChirho(3.0);
        let b_chirho = TropicalSemiringChirho(5.0);

        assert_eq!((a_chirho + b_chirho).0, 3.0); // min
        assert_eq!((a_chirho * b_chirho).0, 8.0); // +
    }

    #[test]
    fn test_count_semiring_chirho() {
        let a_chirho = CountSemiringChirho(3);
        let b_chirho = CountSemiringChirho(4);

        assert_eq!((a_chirho + b_chirho).0, 7);
        assert_eq!((a_chirho * b_chirho).0, 12);
    }

    #[test]
    fn test_weighted_matmul_chirho() {
        let mut a_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(2, 2);
        let mut b_chirho = WeightedMatrixChirho::<CountSemiringChirho>::new(2, 2);

        a_chirho.set_chirho(0, 0, CountSemiringChirho(1));
        a_chirho.set_chirho(0, 1, CountSemiringChirho(2));
        b_chirho.set_chirho(0, 0, CountSemiringChirho(1));
        b_chirho.set_chirho(1, 0, CountSemiringChirho(1));

        let c_chirho = a_chirho.matmul_chirho(&b_chirho);

        // c[0,0] = a[0,0]*b[0,0] + a[0,1]*b[1,0] = 1*1 + 2*1 = 3
        assert_eq!(c_chirho.get_chirho(0, 0).0, 3);
    }
}
