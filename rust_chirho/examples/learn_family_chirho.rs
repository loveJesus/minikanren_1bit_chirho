//! Learn Family Relations from Grandparent Examples ☧
//!
//! Demonstrates differentiable miniKanren: learning which pairs are in
//! the `parent` relation by supervising on `grandparent = parent ∘ parent`.
//!
//! ## The Problem
//!
//! Given:
//! - A set of candidate parent pairs (some correct, some noise)
//! - Supervision: known grandparent pairs (derived from true parents)
//!
//! Learn: Which candidate pairs are actually parent relationships?
//!
//! ## How It Works
//!
//! 1. Each candidate parent pair has a learnable weight ∈ (0, 1)
//! 2. `grandparent(X, Z)` computed by soft composition: `∃Y: parent(X, Y) ∧ parent(Y, Z)`
//! 3. Loss = difference between predicted and true grandparent probability
//! 4. Gradients flow through soft AND/OR to update parent weights
//!
//! ## Run
//!
//! ```bash
//! cargo run --example learn_family_chirho
//! ```

use minikanren_1bit_chirho::learn_chirho::{
    learnable_chirho::BinaryRelationChirho,
    anneal_chirho::AnnealingScheduleChirho,
};

/// Person IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PersonIdChirho(u8);

impl std::fmt::Display for PersonIdChirho {
    fn fmt(&self, f_chirho: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names_chirho = ["tom", "jane", "mary", "james", "bob", "sue", "noise1", "noise2"];
        write!(f_chirho, "{}", names_chirho.get(self.0 as usize).unwrap_or(&"?"))
    }
}

const TOM: PersonIdChirho = PersonIdChirho(0);
const JANE: PersonIdChirho = PersonIdChirho(1);
const MARY: PersonIdChirho = PersonIdChirho(2);
const JAMES: PersonIdChirho = PersonIdChirho(3);
const BOB: PersonIdChirho = PersonIdChirho(4);
const SUE: PersonIdChirho = PersonIdChirho(5);
const NOISE1: PersonIdChirho = PersonIdChirho(6);
const NOISE2: PersonIdChirho = PersonIdChirho(7);

/// True family tree:
///
/// ```text
///   tom === jane
///       |
///    +--+--+
///    |     |
///  mary  james
///           |
///        +--+--+
///        |     |
///       bob   sue
/// ```
fn true_parent_pairs_chirho() -> Vec<(PersonIdChirho, PersonIdChirho)> {
    vec![
        (TOM, MARY),   // tom is parent of mary
        (JANE, MARY),  // jane is parent of mary
        (TOM, JAMES),  // tom is parent of james
        (JANE, JAMES), // jane is parent of james
        (JAMES, BOB),  // james is parent of bob
        (JAMES, SUE),  // james is parent of sue
    ]
}

/// Candidate pairs (true parents + noise)
fn candidate_parent_pairs_chirho() -> Vec<(PersonIdChirho, PersonIdChirho)> {
    vec![
        // True parents
        (TOM, MARY),
        (JANE, MARY),
        (TOM, JAMES),
        (JANE, JAMES),
        (JAMES, BOB),
        (JAMES, SUE),
        // Noise (incorrect pairs)
        (NOISE1, BOB),   // noise: random person is not bob's parent
        (NOISE2, SUE),   // noise: random person is not sue's parent
        (BOB, TOM),      // noise: bob is not parent of tom (reversed!)
        (MARY, JANE),    // noise: reversed relationship
    ]
}

/// Compute grandparent via soft composition
/// grandparent(X, Z) = ∃Y: parent(X, Y) ∧ parent(Y, Z)
fn soft_grandparent_chirho(
    parent_rel_chirho: &BinaryRelationChirho<PersonIdChirho, PersonIdChirho>,
    x_chirho: PersonIdChirho,
    z_chirho: PersonIdChirho,
) -> f64 {
    let weights_chirho = parent_rel_chirho.weights_chirho();
    let tuples_chirho = &parent_rel_chirho.tuples_chirho;

    // Sum over all possible intermediate Y
    let mut prob_chirho = 0.0;

    for (i_chirho, (p1_chirho, c1_chirho)) in tuples_chirho.iter().enumerate() {
        if *p1_chirho != x_chirho {
            continue; // X must match
        }
        let y_chirho = *c1_chirho; // Intermediate person
        let w1_chirho = weights_chirho[i_chirho];

        for (j_chirho, (p2_chirho, c2_chirho)) in tuples_chirho.iter().enumerate() {
            if *p2_chirho != y_chirho || *c2_chirho != z_chirho {
                continue; // Y and Z must match
            }
            let w2_chirho = weights_chirho[j_chirho];

            // Soft AND: probability both parent pairs exist
            // Using product (probabilistic AND)
            prob_chirho += w1_chirho * w2_chirho;
        }
    }

    // Clamp to [0, 1]
    prob_chirho.min(1.0)
}

/// Training examples: known grandparent pairs
fn grandparent_examples_chirho() -> Vec<((PersonIdChirho, PersonIdChirho), f64)> {
    vec![
        // True grandparents (target = 1.0)
        ((TOM, BOB), 1.0),   // tom is grandparent of bob
        ((TOM, SUE), 1.0),   // tom is grandparent of sue
        ((JANE, BOB), 1.0),  // jane is grandparent of bob
        ((JANE, SUE), 1.0),  // jane is grandparent of sue
        // Non-grandparents (target = 0.0)
        ((MARY, BOB), 0.0),   // mary is not grandparent of bob
        ((JAMES, MARY), 0.0), // james is not grandparent of mary
        ((BOB, TOM), 0.0),    // bob is not grandparent of tom
        ((NOISE1, SUE), 0.0), // noise is not grandparent
    ]
}

fn main() {
    println!("=== Learn Family Relations from Grandparent Examples ☧ ===\n");

    // Initialize learnable parent relation with candidate pairs
    let candidates_chirho = candidate_parent_pairs_chirho();
    let n_candidates_chirho = candidates_chirho.len();
    let mut parent_rel_chirho = BinaryRelationChirho::new_chirho(candidates_chirho.clone(), 0.5);

    println!("Candidate parent pairs ({} total, {} true + {} noise):",
             n_candidates_chirho,
             true_parent_pairs_chirho().len(),
             n_candidates_chirho - true_parent_pairs_chirho().len());
    for (i_chirho, (p_chirho, c_chirho)) in candidates_chirho.iter().enumerate() {
        let is_true_chirho = true_parent_pairs_chirho().contains(&(*p_chirho, *c_chirho));
        println!("  {} ({}, {}) {}",
                 i_chirho, p_chirho, c_chirho,
                 if is_true_chirho { "✓ true" } else { "✗ noise" });
    }
    println!();

    // Training setup
    let examples_chirho = grandparent_examples_chirho();
    let annealer_chirho = AnnealingScheduleChirho::new_chirho(1.0, 0.1, 500);

    println!("Training on {} grandparent examples...\n", examples_chirho.len());

    // Training loop
    let mut total_loss_chirho = 0.0;
    for epoch_chirho in 0..500 {
        let temp_chirho = annealer_chirho.temp_at_step_chirho(epoch_chirho);
        total_loss_chirho = 0.0;

        // Collect gradients to apply later (avoid borrow conflicts)
        let mut grads_chirho = vec![0.0; parent_rel_chirho.tuples_chirho.len()];

        for ((x_chirho, z_chirho), target_chirho) in &examples_chirho {
            // Forward: compute soft grandparent probability
            let pred_chirho = soft_grandparent_chirho(&parent_rel_chirho, *x_chirho, *z_chirho);

            // Loss
            let loss_chirho = (pred_chirho - target_chirho).powi(2);
            total_loss_chirho += loss_chirho;

            // Backward: gradient w.r.t. parent weights
            let grad_pred_chirho = 2.0 * (pred_chirho - target_chirho);
            let weights_chirho = parent_rel_chirho.weights_chirho();
            let tuples_clone_chirho = parent_rel_chirho.tuples_chirho.clone();

            // Compute gradients for each parent pair that could contribute
            for (i_chirho, (p1_chirho, c1_chirho)) in tuples_clone_chirho.iter().enumerate() {
                if *p1_chirho != *x_chirho {
                    continue;
                }
                let y_chirho = *c1_chirho;
                let w1_chirho = weights_chirho[i_chirho];

                for (j_chirho, (p2_chirho, c2_chirho)) in tuples_clone_chirho.iter().enumerate() {
                    if *p2_chirho != y_chirho || *c2_chirho != *z_chirho {
                        continue;
                    }
                    let w2_chirho = weights_chirho[j_chirho];

                    // d(w1 * w2) / d(w1) = w2, scaled by softmax gradient
                    let grad_w1_chirho = grad_pred_chirho * w2_chirho * w1_chirho * (1.0 - w1_chirho) * temp_chirho;
                    let grad_w2_chirho = grad_pred_chirho * w1_chirho * w2_chirho * (1.0 - w2_chirho) * temp_chirho;

                    grads_chirho[i_chirho] += grad_w1_chirho;
                    grads_chirho[j_chirho] += grad_w2_chirho;
                }
            }
        }

        // Apply gradients
        for (i_chirho, grad_chirho) in grads_chirho.iter().enumerate() {
            parent_rel_chirho.accumulate_grad_chirho(i_chirho, *grad_chirho);
        }
        parent_rel_chirho.apply_gradients_chirho();

        // Log progress
        if epoch_chirho % 100 == 0 || epoch_chirho == 499 {
            println!("Epoch {}: loss = {:.4}, temp = {:.3}", epoch_chirho, total_loss_chirho, temp_chirho);
        }
    }

    println!("\n=== Learned Parent Weights ===\n");

    let weights_chirho = parent_rel_chirho.weights_chirho();
    let true_pairs_chirho = true_parent_pairs_chirho();

    // Sort by weight (highest first)
    let mut indexed_chirho: Vec<_> = weights_chirho.iter().enumerate().collect();
    indexed_chirho.sort_by(|(_, a_chirho), (_, b_chirho)| b_chirho.partial_cmp(a_chirho).unwrap());

    println!("Rank | Weight | Pair             | Ground Truth");
    println!("-----|--------|------------------|-------------");
    for (rank_chirho, (i_chirho, weight_chirho)) in indexed_chirho.iter().enumerate() {
        let (p_chirho, c_chirho) = &candidates_chirho[*i_chirho];
        let is_true_chirho = true_pairs_chirho.contains(&(*p_chirho, *c_chirho));
        let truth_chirho = if is_true_chirho { "TRUE" } else { "noise" };
        println!("  {:2} | {:.4}  | ({:6}, {:6}) | {}",
                 rank_chirho + 1, weight_chirho, p_chirho, c_chirho, truth_chirho);
    }

    // Verify that true pairs have higher weights than noise
    let mut true_weights_chirho = Vec::new();
    let mut noise_weights_chirho = Vec::new();

    for (i_chirho, weight_chirho) in weights_chirho.iter().enumerate() {
        let pair_chirho = &candidates_chirho[i_chirho];
        if true_pairs_chirho.contains(pair_chirho) {
            true_weights_chirho.push(*weight_chirho);
        } else {
            noise_weights_chirho.push(*weight_chirho);
        }
    }

    let avg_true_chirho: f64 = true_weights_chirho.iter().sum::<f64>() / true_weights_chirho.len() as f64;
    let avg_noise_chirho: f64 = noise_weights_chirho.iter().sum::<f64>() / noise_weights_chirho.len() as f64;

    println!("\n=== Summary ===");
    println!("Average weight for TRUE pairs:  {:.4}", avg_true_chirho);
    println!("Average weight for NOISE pairs: {:.4}", avg_noise_chirho);
    println!("Separation ratio: {:.2}x", avg_true_chirho / avg_noise_chirho);

    if avg_true_chirho > avg_noise_chirho * 1.5 {
        println!("\n✓ SUCCESS: Model learned to distinguish true parents from noise!");
    } else {
        println!("\n⚠ Learning may need more epochs or tuning.");
    }

    // Verify grandparent predictions
    println!("\n=== Grandparent Predictions ===");
    for ((x_chirho, z_chirho), target_chirho) in &examples_chirho {
        let pred_chirho = soft_grandparent_chirho(&parent_rel_chirho, *x_chirho, *z_chirho);
        let correct_chirho = (pred_chirho > 0.5) == (*target_chirho > 0.5);
        println!("grandparent({}, {}) = {:.3} (target: {:.1}) {}",
                 x_chirho, z_chirho, pred_chirho, target_chirho,
                 if correct_chirho { "✓" } else { "✗" });
    }

    println!("\n☧ Soli Deo Gloria ☧");
}
