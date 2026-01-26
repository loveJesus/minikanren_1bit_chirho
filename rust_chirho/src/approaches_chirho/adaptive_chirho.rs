// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Adaptive Strategy Selection ☧
//!
//! Automatically chooses optimal domain representation based on problem characteristics.
//! Integrates with core miniKanren, not an external solution.
//!
//! # Strategy Selection
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Problem Analysis                             │
//! │                                                                 │
//! │   Domain Size?    Sparsity?     Mode?        Learning?         │
//! │       │              │            │              │              │
//! │       ▼              ▼            ▼              ▼              │
//! │   ┌───────┐     ┌────────┐   ┌────────┐    ┌─────────┐        │
//! │   │ ≤64   │     │ Dense  │   │ Ground │    │ Yes     │        │
//! │   │ ≤4K   │     │ Sparse │   │ Free   │    │ No      │        │
//! │   │ ≤262K │     │ Symbolic│  │ Mixed  │    └────┬────┘        │
//! │   │ ∞     │     └───┬────┘   └───┬────┘         │              │
//! │   └──┬────┘         │            │              │              │
//! │      │              │            │              │              │
//! │      ▼              ▼            ▼              ▼              │
//! │   ┌─────────────────────────────────────────────────────┐     │
//! │   │            Strategy Selection Matrix                 │     │
//! │   │                                                      │     │
//! │   │  BitVec64    : size≤64, dense, no learning          │     │
//! │   │  Hierarchical: size≤262K, any sparsity              │     │
//! │   │  Symbolic    : infinite, range/mod constraints      │     │
//! │   │  Hybrid      : mixed finite+symbolic                │     │
//! │   │  Diff*       : any size with learning=true          │     │
//! │   └─────────────────────────────────────────────────────┘     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Mode-Driven Optimization
//!
//! - **Ground mode**: Use constraint propagation (fast pruning)
//! - **Free mode**: Use lazy enumeration (avoid infinite loops)
//! - **Mixed mode**: Reorder goals to ground variables first

use crate::hardware_chirho::BitVec64Chirho;
use crate::approaches_chirho::{
    Hierarchical4kChirho,
    SymbolicDomainChirho,
    DiffHierarchical4kChirho,
};

/// Problem characteristics for strategy selection
#[derive(Debug, Clone)]
pub struct ProblemAnalysisChirho {
    /// Maximum domain size across all variables
    pub max_domain_size_chirho: u64,
    /// Sparsity: fraction of possible values actually used
    pub sparsity_chirho: f64,
    /// Whether any variable has infinite domain
    pub has_infinite_chirho: bool,
    /// Whether constraints are mostly range/modular (symbolic-friendly)
    pub is_symbolic_friendly_chirho: bool,
    /// Whether learning/gradients are needed
    pub needs_learning_chirho: bool,
    /// Variable groundness: fraction of variables that start ground
    pub groundness_chirho: f64,
}

impl Default for ProblemAnalysisChirho {
    fn default() -> Self {
        Self {
            max_domain_size_chirho: 64,
            sparsity_chirho: 1.0,
            has_infinite_chirho: false,
            is_symbolic_friendly_chirho: false,
            needs_learning_chirho: false,
            groundness_chirho: 0.0,
        }
    }
}

/// Selected execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyChirho {
    /// Use BitVec64 (fastest, ≤64 values)
    BitVec64Chirho,
    /// Use Hierarchical4k (fast, ≤4096 values)
    Hierarchical4kChirho,
    /// Use symbolic constraints (infinite domains)
    SymbolicChirho,
    /// Use differentiable (learning mode)
    DifferentiableChirho,
}

/// Adaptive domain that can switch representations
#[derive(Clone)]
pub enum AdaptiveDomainChirho {
    /// 64-value domain (fastest)
    BitVec64Chirho(BitVec64Chirho),
    /// 4096-value hierarchical domain
    Hierarchical4kChirho(Hierarchical4kChirho),
    /// Symbolic constraints (infinite domains)
    SymbolicChirho(SymbolicDomainChirho),
    /// Differentiable domain (for learning)
    DifferentiableChirho(Box<DiffHierarchical4kChirho>),
}

impl std::fmt::Debug for AdaptiveDomainChirho {
    fn fmt(&self, f_chirho: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BitVec64Chirho(d) => write!(f_chirho, "BitVec64({:016x})", d.0),
            Self::Hierarchical4kChirho(_) => write!(f_chirho, "Hierarchical4k"),
            Self::SymbolicChirho(d) => write!(f_chirho, "Symbolic({:?})", d.constraint_chirho),
            Self::DifferentiableChirho(_) => write!(f_chirho, "Differentiable"),
        }
    }
}

impl AdaptiveDomainChirho {
    /// Get the current strategy
    pub fn strategy_chirho(&self) -> StrategyChirho {
        match self {
            Self::BitVec64Chirho(_) => StrategyChirho::BitVec64Chirho,
            Self::Hierarchical4kChirho(_) => StrategyChirho::Hierarchical4kChirho,
            Self::SymbolicChirho(_) => StrategyChirho::SymbolicChirho,
            Self::DifferentiableChirho(_) => StrategyChirho::DifferentiableChirho,
        }
    }

    /// Check if domain is empty
    pub fn is_empty_chirho(&self) -> bool {
        match self {
            Self::BitVec64Chirho(d) => d.is_zero_chirho(),
            Self::Hierarchical4kChirho(d) => d.is_empty_chirho(),
            Self::SymbolicChirho(d) => d.is_empty_chirho(),
            Self::DifferentiableChirho(d) => d.expected_count_chirho() < 0.5,
        }
    }

    /// Count values in domain (exact or estimate for symbolic/infinite)
    pub fn count_chirho(&self) -> u64 {
        match self {
            Self::BitVec64Chirho(d) => d.popcount_chirho() as u64,
            Self::Hierarchical4kChirho(d) => d.count_chirho() as u64,
            Self::SymbolicChirho(d) => {
                // Try to enumerate up to 64 values
                if let Some(bv_chirho) = d.try_enumerate_chirho(64) {
                    bv_chirho.popcount_chirho() as u64
                } else {
                    u64::MAX // Infinite or too large
                }
            }
            Self::DifferentiableChirho(d) => d.expected_count_chirho() as u64,
        }
    }
}

/// Strategy selector based on problem analysis
pub struct StrategySelectorChirho {
    /// Threshold for switching from BitVec64 to Hierarchical4k
    pub threshold_4k_chirho: u64,
    /// Threshold for switching from Hierarchical4k to Hierarchical256k
    pub threshold_256k_chirho: u64,
    /// Sparsity threshold for preferring symbolic
    pub sparsity_threshold_chirho: f64,
    /// Whether to prefer symbolic for range constraints
    pub prefer_symbolic_ranges_chirho: bool,
}

impl Default for StrategySelectorChirho {
    fn default() -> Self {
        Self {
            threshold_4k_chirho: 64,
            threshold_256k_chirho: 4096,
            sparsity_threshold_chirho: 0.01, // Below 1% density, prefer symbolic
            prefer_symbolic_ranges_chirho: true,
        }
    }
}

impl StrategySelectorChirho {
    /// Select optimal strategy based on problem analysis
    pub fn select_chirho(&self, analysis_chirho: &ProblemAnalysisChirho) -> StrategyChirho {
        // Learning always uses differentiable
        if analysis_chirho.needs_learning_chirho {
            return StrategyChirho::DifferentiableChirho;
        }

        // Infinite domains need symbolic
        if analysis_chirho.has_infinite_chirho {
            return StrategyChirho::SymbolicChirho;
        }

        // Very sparse finite domains can benefit from symbolic
        if analysis_chirho.sparsity_chirho < self.sparsity_threshold_chirho
            && analysis_chirho.is_symbolic_friendly_chirho
        {
            return StrategyChirho::SymbolicChirho;
        }

        // Choose based on domain size
        if analysis_chirho.max_domain_size_chirho <= self.threshold_4k_chirho {
            StrategyChirho::BitVec64Chirho
        } else {
            StrategyChirho::Hierarchical4kChirho
        }
    }

    /// Create domain with selected strategy
    pub fn create_domain_chirho(
        &self,
        strategy_chirho: StrategyChirho,
        size_chirho: u64,
    ) -> AdaptiveDomainChirho {
        match strategy_chirho {
            StrategyChirho::BitVec64Chirho => {
                let n_chirho = size_chirho.min(64) as u32;
                if n_chirho == 64 {
                    AdaptiveDomainChirho::BitVec64Chirho(BitVec64Chirho::ONES_CHIRHO)
                } else {
                    AdaptiveDomainChirho::BitVec64Chirho(BitVec64Chirho((1u64 << n_chirho) - 1))
                }
            }
            StrategyChirho::Hierarchical4kChirho => {
                let n_chirho = size_chirho.min(4096) as u32;
                AdaptiveDomainChirho::Hierarchical4kChirho(Hierarchical4kChirho::range_chirho(n_chirho))
            }
            StrategyChirho::SymbolicChirho => {
                AdaptiveDomainChirho::SymbolicChirho(SymbolicDomainChirho::range_chirho(
                    0,
                    size_chirho as i64 - 1,
                ))
            }
            StrategyChirho::DifferentiableChirho => {
                AdaptiveDomainChirho::DifferentiableChirho(Box::new(
                    DiffHierarchical4kChirho::full_chirho(),
                ))
            }
        }
    }
}

/// Intersect two adaptive domains, potentially switching strategy
pub fn adaptive_intersect_chirho(
    a_chirho: &AdaptiveDomainChirho,
    b_chirho: &AdaptiveDomainChirho,
) -> AdaptiveDomainChirho {
    use AdaptiveDomainChirho::*;

    match (a_chirho, b_chirho) {
        // Same strategy: use native intersection
        (BitVec64Chirho(a), BitVec64Chirho(b)) => {
            BitVec64Chirho(a.and_chirho(*b))
        }
        (Hierarchical4kChirho(a), Hierarchical4kChirho(b)) => {
            Hierarchical4kChirho(a.intersect_chirho(b))
        }
        (SymbolicChirho(a), SymbolicChirho(b)) => {
            SymbolicChirho(a.intersect_chirho(b))
        }
        (DifferentiableChirho(a), DifferentiableChirho(b)) => {
            DifferentiableChirho(Box::new(a.soft_intersect_chirho(b)))
        }

        // Mixed: promote to more general representation
        (BitVec64Chirho(a), Hierarchical4kChirho(b)) |
        (Hierarchical4kChirho(b), BitVec64Chirho(a)) => {
            let a_h4k_chirho = bitvec_to_hierarchical4k_chirho(*a);
            Hierarchical4kChirho(a_h4k_chirho.intersect_chirho(b))
        }

        // Symbolic with hierarchical: materialize if possible
        (SymbolicChirho(a), Hierarchical4kChirho(b)) |
        (Hierarchical4kChirho(b), SymbolicChirho(a)) => {
            // Try to materialize symbolic to hierarchical
            let a_h4k_chirho = symbolic_to_hierarchical4k_chirho(a);
            Hierarchical4kChirho(a_h4k_chirho.intersect_chirho(b))
        }

        // Differentiable with hierarchical: convert to diff
        (DifferentiableChirho(a), Hierarchical4kChirho(b)) |
        (Hierarchical4kChirho(b), DifferentiableChirho(a)) => {
            let b_diff_chirho = DiffHierarchical4kChirho::from_hard_chirho(b);
            DifferentiableChirho(Box::new(a.soft_intersect_chirho(&b_diff_chirho)))
        }

        // BitVec64 with symbolic: materialize to BitVec64 range
        (BitVec64Chirho(a), SymbolicChirho(b)) |
        (SymbolicChirho(b), BitVec64Chirho(a)) => {
            let b_bv_chirho = symbolic_to_bitvec64_chirho(b);
            BitVec64Chirho(a.and_chirho(b_bv_chirho))
        }

        // BitVec64 with differentiable: convert to diff
        (BitVec64Chirho(a), DifferentiableChirho(b)) |
        (DifferentiableChirho(b), BitVec64Chirho(a)) => {
            let a_h4k_chirho = bitvec_to_hierarchical4k_chirho(*a);
            let a_diff_chirho = DiffHierarchical4kChirho::from_hard_chirho(&a_h4k_chirho);
            DifferentiableChirho(Box::new(b.soft_intersect_chirho(&a_diff_chirho)))
        }

        // Symbolic with differentiable: materialize symbolic first
        (SymbolicChirho(a), DifferentiableChirho(b)) |
        (DifferentiableChirho(b), SymbolicChirho(a)) => {
            let a_h4k_chirho = symbolic_to_hierarchical4k_chirho(a);
            let a_diff_chirho = DiffHierarchical4kChirho::from_hard_chirho(&a_h4k_chirho);
            DifferentiableChirho(Box::new(b.soft_intersect_chirho(&a_diff_chirho)))
        }
    }
}

/// Convert BitVec64 to Hierarchical4k
fn bitvec_to_hierarchical4k_chirho(bv_chirho: BitVec64Chirho) -> Hierarchical4kChirho {
    let mut result_chirho = Hierarchical4kChirho::empty_chirho();
    // BitVec64 values go in first leaf
    result_chirho.leaves_chirho[0] = bv_chirho;
    if !bv_chirho.is_zero_chirho() {
        result_chirho.root_chirho = BitVec64Chirho(1); // First leaf active
    }
    result_chirho
}

/// Convert symbolic to Hierarchical4k (materialize)
fn symbolic_to_hierarchical4k_chirho(sym_chirho: &SymbolicDomainChirho) -> Hierarchical4kChirho {
    // Use try_enumerate to get first 64 values as BitVec64
    if let Some(bv_chirho) = sym_chirho.try_enumerate_chirho(64) {
        let mut result_chirho = Hierarchical4kChirho::empty_chirho();
        result_chirho.leaves_chirho[0] = bv_chirho;
        if !bv_chirho.is_zero_chirho() {
            result_chirho.root_chirho = BitVec64Chirho(1);
        }
        result_chirho
    } else {
        // Cannot enumerate, return full domain as approximation
        Hierarchical4kChirho::full_chirho()
    }
}

/// Convert symbolic to BitVec64 (materialize first 64 values)
fn symbolic_to_bitvec64_chirho(sym_chirho: &SymbolicDomainChirho) -> BitVec64Chirho {
    sym_chirho.try_enumerate_chirho(64).unwrap_or(BitVec64Chirho::ONES_CHIRHO)
}

/// Mode for goal execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeChirho {
    /// All arguments ground - direct computation
    GroundChirho,
    /// Some arguments ground - constraint propagation
    MixedChirho,
    /// No arguments ground - enumeration needed
    FreeChirho,
}

/// Analyze goal to determine execution mode
pub fn analyze_mode_chirho(groundness_chirho: &[bool]) -> ModeChirho {
    let ground_count_chirho = groundness_chirho.iter().filter(|&&g| g).count();
    if ground_count_chirho == groundness_chirho.len() {
        ModeChirho::GroundChirho
    } else if ground_count_chirho > 0 {
        ModeChirho::MixedChirho
    } else {
        ModeChirho::FreeChirho
    }
}

/// Reorder goals to process ground-first
pub fn reorder_goals_chirho<T: Clone>(
    goals_chirho: &[T],
    groundness_chirho: &[Vec<bool>],
) -> Vec<(usize, T)> {
    let mut indexed_chirho: Vec<_> = goals_chirho
        .iter()
        .enumerate()
        .zip(groundness_chirho.iter())
        .map(|((i, g), gr)| {
            let ground_count_chirho = gr.iter().filter(|&&b| b).count();
            (i, g.clone(), ground_count_chirho)
        })
        .collect();

    // Sort by groundness (most ground first)
    indexed_chirho.sort_by(|a, b| b.2.cmp(&a.2));

    indexed_chirho.into_iter().map(|(i, g, _)| (i, g)).collect()
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_strategy_selection_small_chirho() {
        let selector_chirho = StrategySelectorChirho::default();
        let analysis_chirho = ProblemAnalysisChirho {
            max_domain_size_chirho: 32,
            ..Default::default()
        };
        assert_eq!(
            selector_chirho.select_chirho(&analysis_chirho),
            StrategyChirho::BitVec64Chirho
        );
    }

    #[test]
    fn test_strategy_selection_medium_chirho() {
        let selector_chirho = StrategySelectorChirho::default();
        let analysis_chirho = ProblemAnalysisChirho {
            max_domain_size_chirho: 1000,
            ..Default::default()
        };
        assert_eq!(
            selector_chirho.select_chirho(&analysis_chirho),
            StrategyChirho::Hierarchical4kChirho
        );
    }

    #[test]
    fn test_strategy_selection_learning_chirho() {
        let selector_chirho = StrategySelectorChirho::default();
        let analysis_chirho = ProblemAnalysisChirho {
            max_domain_size_chirho: 32,
            needs_learning_chirho: true,
            ..Default::default()
        };
        assert_eq!(
            selector_chirho.select_chirho(&analysis_chirho),
            StrategyChirho::DifferentiableChirho
        );
    }

    #[test]
    fn test_strategy_selection_infinite_chirho() {
        let selector_chirho = StrategySelectorChirho::default();
        let analysis_chirho = ProblemAnalysisChirho {
            has_infinite_chirho: true,
            ..Default::default()
        };
        assert_eq!(
            selector_chirho.select_chirho(&analysis_chirho),
            StrategyChirho::SymbolicChirho
        );
    }

    #[test]
    fn test_adaptive_intersect_same_strategy_chirho() {
        let a_chirho = AdaptiveDomainChirho::BitVec64Chirho(BitVec64Chirho(0b1111));
        let b_chirho = AdaptiveDomainChirho::BitVec64Chirho(BitVec64Chirho(0b0011));
        let result_chirho = adaptive_intersect_chirho(&a_chirho, &b_chirho);

        match result_chirho {
            AdaptiveDomainChirho::BitVec64Chirho(d) => assert_eq!(d.0, 0b0011),
            _ => panic!("Expected BitVec64"),
        }
    }

    #[test]
    fn test_mode_analysis_chirho() {
        assert_eq!(analyze_mode_chirho(&[true, true]), ModeChirho::GroundChirho);
        assert_eq!(analyze_mode_chirho(&[true, false]), ModeChirho::MixedChirho);
        assert_eq!(analyze_mode_chirho(&[false, false]), ModeChirho::FreeChirho);
    }

    #[test]
    fn test_goal_reordering_chirho() {
        let goals_chirho = vec!["goal_a", "goal_b", "goal_c"];
        let groundness_chirho = vec![
            vec![false, false], // goal_a: 0 ground
            vec![true, true],   // goal_b: 2 ground
            vec![true, false],  // goal_c: 1 ground
        ];

        let reordered_chirho = reorder_goals_chirho(&goals_chirho, &groundness_chirho);

        // Should be ordered: goal_b (2), goal_c (1), goal_a (0)
        assert_eq!(reordered_chirho[0], (1, "goal_b"));
        assert_eq!(reordered_chirho[1], (2, "goal_c"));
        assert_eq!(reordered_chirho[2], (0, "goal_a"));
    }
}
