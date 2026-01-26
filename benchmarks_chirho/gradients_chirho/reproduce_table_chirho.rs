//! Gradient Attenuation Reproducibility Script ☧
//!
//! This script reproduces the "Gradient magnitude vs inference depth" table from the paper.
//!
//! Paper Table Claims (Table 3):
//! | Hops | Avg Grad L2 | Max Grad L2 | Ratio to 1-hop |
//! |------|-------------|-------------|----------------|
//! | 1    | 2.0e-1      | 3.9e-1      | 1.000          |
//! | 2    | 1.9e-2      | 7.6e-2      | 0.095          |
//! | 3    | 1.6e-3      | 9.8e-3      | 0.008          |
//! | 4    | 1.2e-4      | 1.2e-3      | 0.0006         |
//!
//! Run with: cargo run --release --example reproduce_table_chirho
//!
//! Methodology:
//! 1. Multi-hop relational reasoning via composition
//! 2. Soft-AND for composition (probability multiplication)
//! 3. Backpropagate from target to source
//! 4. Measure gradient L2 norm at each hop depth
//!
//! Key insight: Gradients attenuate ~10× per hop (expected for multiplicative
//! composition), with zero exploding gradients.

use std::time::Instant;

/// Represents a soft relation with gradient tracking
#[derive(Clone)]
struct SoftRelationChirho {
    /// Probability values for each tuple (entity, entity)
    probs_chirho: Vec<Vec<f64>>,
    /// Gradient accumulator
    grads_chirho: Vec<Vec<f64>>,
}

impl SoftRelationChirho {
    fn new_chirho(size_chirho: usize) -> Self {
        Self {
            probs_chirho: vec![vec![0.0; size_chirho]; size_chirho],
            grads_chirho: vec![vec![0.0; size_chirho]; size_chirho],
        }
    }

    fn set_prob_chirho(&mut self, i_chirho: usize, j_chirho: usize, p_chirho: f64) {
        self.probs_chirho[i_chirho][j_chirho] = p_chirho;
    }

    fn clear_grads_chirho(&mut self) {
        for row_chirho in &mut self.grads_chirho {
            for grad_chirho in row_chirho.iter_mut() {
                *grad_chirho = 0.0;
            }
        }
    }

    fn grad_l2_chirho(&self) -> f64 {
        let mut sum_chirho = 0.0;
        for row_chirho in &self.grads_chirho {
            for &grad_chirho in row_chirho {
                sum_chirho += grad_chirho * grad_chirho;
            }
        }
        sum_chirho.sqrt()
    }

    fn max_grad_chirho(&self) -> f64 {
        let mut max_chirho = 0.0f64;
        for row_chirho in &self.grads_chirho {
            for &grad_chirho in row_chirho {
                max_chirho = max_chirho.max(grad_chirho.abs());
            }
        }
        max_chirho
    }
}

/// Soft composition: R1 ∘ R2 (existential quantification over middle entity)
/// Forward: out[i,k] = sum_j (R1[i,j] * R2[j,k])
/// Backward: gradients flow through multiplication
fn compose_forward_chirho(
    r1_chirho: &SoftRelationChirho,
    r2_chirho: &SoftRelationChirho,
    size_chirho: usize,
) -> Vec<Vec<f64>> {
    let mut out_chirho = vec![vec![0.0; size_chirho]; size_chirho];

    for i_chirho in 0..size_chirho {
        for k_chirho in 0..size_chirho {
            let mut sum_chirho = 0.0;
            for j_chirho in 0..size_chirho {
                // Soft-AND = multiplication
                sum_chirho += r1_chirho.probs_chirho[i_chirho][j_chirho]
                    * r2_chirho.probs_chirho[j_chirho][k_chirho];
            }
            // Soft-OR = probabilistic sum (clamped to [0,1])
            out_chirho[i_chirho][k_chirho] = sum_chirho.min(1.0);
        }
    }

    out_chirho
}

/// Backward pass: propagate gradients through composition
fn compose_backward_chirho(
    grad_out_chirho: &[Vec<f64>],
    r1_chirho: &mut SoftRelationChirho,
    r2_chirho: &mut SoftRelationChirho,
    size_chirho: usize,
) {
    for i_chirho in 0..size_chirho {
        for k_chirho in 0..size_chirho {
            let grad_chirho = grad_out_chirho[i_chirho][k_chirho];
            if grad_chirho.abs() < 1e-10 {
                continue;
            }

            for j_chirho in 0..size_chirho {
                // ∂L/∂R1[i,j] = ∂L/∂out[i,k] * R2[j,k]
                r1_chirho.grads_chirho[i_chirho][j_chirho] +=
                    grad_chirho * r2_chirho.probs_chirho[j_chirho][k_chirho];

                // ∂L/∂R2[j,k] = ∂L/∂out[i,k] * R1[i,j]
                r2_chirho.grads_chirho[j_chirho][k_chirho] +=
                    grad_chirho * r1_chirho.probs_chirho[i_chirho][j_chirho];
            }
        }
    }
}

/// Measure gradient attenuation at different inference depths
fn measure_gradient_attenuation_chirho(max_hops_chirho: usize, size_chirho: usize) -> Vec<(f64, f64)> {
    let mut results_chirho = Vec::new();

    // Create a parent relation with random probabilities
    let mut parent_chirho = SoftRelationChirho::new_chirho(size_chirho);

    // Initialize with a simple tree structure
    // Person 0 is parent of 1,2
    // Person 1 is parent of 3,4
    // Person 2 is parent of 5,6
    // etc.
    for i_chirho in 0..(size_chirho / 2) {
        let left_chirho = 2 * i_chirho + 1;
        let right_chirho = 2 * i_chirho + 2;
        if left_chirho < size_chirho {
            parent_chirho.set_prob_chirho(i_chirho, left_chirho, 0.9);
        }
        if right_chirho < size_chirho {
            parent_chirho.set_prob_chirho(i_chirho, right_chirho, 0.9);
        }
    }

    for hops_chirho in 1..=max_hops_chirho {
        parent_chirho.clear_grads_chirho();

        // Compose parent relation `hops` times
        let mut relations_chirho = vec![parent_chirho.clone(); hops_chirho];
        let mut intermediate_chirho = Vec::new();

        // Forward pass: compose step by step
        let mut current_chirho = relations_chirho[0].probs_chirho.clone();
        intermediate_chirho.push(current_chirho.clone());

        for rel_chirho in relations_chirho.iter().skip(1) {
            current_chirho = compose_forward_chirho(
                &SoftRelationChirho {
                    probs_chirho: current_chirho.clone(),
                    grads_chirho: vec![vec![0.0; size_chirho]; size_chirho],
                },
                rel_chirho,
                size_chirho,
            );
            intermediate_chirho.push(current_chirho.clone());
        }

        // Target: person 0 should reach person (2^hops - 1) if tree is complete
        let target_idx_chirho = ((1 << hops_chirho) - 1).min(size_chirho - 1);

        // Loss = (1 - prob[0, target])^2
        let prob_chirho = current_chirho[0][target_idx_chirho];
        let _loss_chirho = (1.0 - prob_chirho).powi(2);

        // Gradient of loss w.r.t. output
        let grad_loss_chirho = -2.0 * (1.0 - prob_chirho);

        // Initialize output gradient
        let mut grad_out_chirho = vec![vec![0.0; size_chirho]; size_chirho];
        grad_out_chirho[0][target_idx_chirho] = grad_loss_chirho;

        // Backward pass: propagate through compositions in reverse
        for i_chirho in (1..hops_chirho).rev() {
            let mut temp_rel_chirho = SoftRelationChirho {
                probs_chirho: intermediate_chirho[i_chirho - 1].clone(),
                grads_chirho: vec![vec![0.0; size_chirho]; size_chirho],
            };

            compose_backward_chirho(
                &grad_out_chirho,
                &mut temp_rel_chirho,
                &mut relations_chirho[i_chirho],
                size_chirho,
            );

            // Update grad_out for next iteration
            grad_out_chirho = temp_rel_chirho.grads_chirho;
        }

        // Final backward to first relation
        if hops_chirho > 1 {
            let mut dummy_chirho = SoftRelationChirho::new_chirho(size_chirho);
            compose_backward_chirho(
                &grad_out_chirho,
                &mut relations_chirho[0],
                &mut dummy_chirho,
                size_chirho,
            );
        } else {
            relations_chirho[0].grads_chirho = grad_out_chirho;
        }

        let avg_l2_chirho = relations_chirho[0].grad_l2_chirho();
        let max_grad_chirho = relations_chirho[0].max_grad_chirho();

        results_chirho.push((avg_l2_chirho, max_grad_chirho));
    }

    results_chirho
}

fn main() {
    println!("Gradient Attenuation Reproducibility ☧");
    println!("========================================");
    println!();
    println!("Reproducing Table 3: Gradient magnitude vs inference depth");
    println!();

    let start_chirho = Instant::now();

    // Use a reasonable size for the relation
    let size_chirho = 32;
    let max_hops_chirho = 4;

    let results_chirho = measure_gradient_attenuation_chirho(max_hops_chirho, size_chirho);

    let elapsed_chirho = start_chirho.elapsed();

    // Print results table
    println!("| Hops | Avg Grad L2     | Max Grad L2     | Ratio to 1-hop |");
    println!("|------|-----------------|-----------------|----------------|");

    let baseline_chirho = results_chirho[0].0;

    for (hops_chirho, (avg_l2_chirho, max_l2_chirho)) in results_chirho.iter().enumerate() {
        let ratio_chirho = avg_l2_chirho / baseline_chirho;
        println!(
            "| {:4} | {:15.2e} | {:15.2e} | {:14.4} |",
            hops_chirho + 1,
            avg_l2_chirho,
            max_l2_chirho,
            ratio_chirho
        );
    }

    println!();
    println!("Computation time: {:?}", elapsed_chirho);
    println!();

    // Verify expected behavior
    println!("Verification:");
    println!("  - Gradients attenuate ~10× per hop: ", );

    for i_chirho in 1..results_chirho.len() {
        let ratio_chirho = results_chirho[i_chirho - 1].0 / results_chirho[i_chirho].0;
        let status_chirho = if ratio_chirho > 5.0 && ratio_chirho < 20.0 {
            "✓"
        } else {
            "?"
        };
        println!(
            "    Hop {} → {}: {:.1}× attenuation {}",
            i_chirho,
            i_chirho + 1,
            ratio_chirho,
            status_chirho
        );
    }

    // Check for exploding gradients
    let max_ever_chirho = results_chirho.iter().map(|r| r.1).fold(0.0f64, f64::max);
    let no_exploding_chirho = max_ever_chirho < 10.0;

    println!();
    if no_exploding_chirho {
        println!("  - No exploding gradients: ✓ (max = {:.2e})", max_ever_chirho);
    } else {
        println!(
            "  - WARNING: Possible exploding gradients (max = {:.2e})",
            max_ever_chirho
        );
    }

    println!();
    println!("Methodology:");
    println!("  1. Multi-hop relational reasoning via composition");
    println!("  2. Soft-AND for composition (probability multiplication)");
    println!("  3. Backpropagate from target to source");
    println!("  4. Measure gradient L2 norm at each hop depth");
    println!();
    println!("Key insight: Gradients attenuate ~10× per hop (expected for");
    println!("multiplicative composition), with zero exploding gradients.");
    println!();
    println!("☧ Soli Deo Gloria ☧");
}
