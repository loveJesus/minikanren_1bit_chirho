//! Symbolic Addition Learning ☧
//!
//! A simplified neurosymbolic benchmark: learn to classify "digit" vectors
//! (simplified MNIST-like inputs) and verify their sum using logic.
//!
//! This demonstrates the MNIST-Addition concept without requiring actual images:
//! - Input: Symbolic "digit" patterns (bit vectors representing digits 0-9)
//! - Neural: Learn to classify patterns to digits (soft classification)
//! - Logic: Verify addition constraint via differentiable miniKanren
//!
//! ## The Task
//!
//! Given pairs of digit patterns (a, b) and target sums c:
//! - Learn classifier weights that map patterns → digits
//! - Use logic constraint: classify(a) + classify(b) = c
//! - Gradients flow through both classifier and addition constraint
//!
//! ## Run
//!
//! ```bash
//! cargo run --example symbolic_addition_chirho
//! ```
//!
//! "Prove all things; hold fast that which is good." — 1 Thessalonians 5:21

use minikanren_1bit_chirho::semiring_chirho::diff_semiring_chirho::{
    soft_eq_chirho, GumbelSoftmaxChirho, DiffProbChirho,
};

/// A symbolic digit pattern (simplified representation)
/// Each digit 0-9 has a unique 10-bit pattern (one-hot plus noise)
#[derive(Debug, Clone, Copy)]
struct DigitPatternChirho {
    bits_chirho: [f64; 10],
}

impl DigitPatternChirho {
    /// Create a noisy one-hot pattern for a digit
    fn from_digit_chirho(d_chirho: u8, noise_chirho: f64) -> Self {
        let mut bits_chirho = [noise_chirho; 10];
        bits_chirho[d_chirho as usize] = 1.0 - noise_chirho * 9.0;
        // Normalize
        let sum_chirho: f64 = bits_chirho.iter().sum();
        for b_chirho in &mut bits_chirho {
            *b_chirho /= sum_chirho;
        }
        Self { bits_chirho }
    }

    /// Create random pattern (noise)
    fn random_chirho(seed_chirho: u64) -> Self {
        let mut bits_chirho = [0.0; 10];
        for (i_chirho, b_chirho) in bits_chirho.iter_mut().enumerate() {
            *b_chirho = ((seed_chirho.wrapping_mul(i_chirho as u64 + 1) % 100) as f64) / 100.0;
        }
        let sum_chirho: f64 = bits_chirho.iter().sum();
        for b_chirho in &mut bits_chirho {
            *b_chirho /= sum_chirho;
        }
        Self { bits_chirho }
    }
}

/// Simple linear classifier: pattern → soft digit prediction
#[derive(Debug, Clone)]
struct DigitClassifierChirho {
    /// Weights: 10x10 matrix (pattern features → digit logits)
    weights_chirho: [[f64; 10]; 10],
    /// Learning rate
    lr_chirho: f64,
}

impl DigitClassifierChirho {
    fn new_chirho(lr_chirho: f64) -> Self {
        // Initialize with identity + small noise (patterns are one-hot-ish)
        let mut weights_chirho = [[0.1; 10]; 10];
        for i_chirho in 0..10 {
            weights_chirho[i_chirho][i_chirho] = 1.0;
        }
        Self { weights_chirho, lr_chirho }
    }

    /// Classify pattern to soft digit probabilities
    fn classify_chirho(&self, pattern_chirho: &DigitPatternChirho, temp_chirho: f64) -> [f64; 10] {
        // Compute logits: W @ pattern
        let mut logits_chirho = [0.0; 10];
        for (d_chirho, logit_chirho) in logits_chirho.iter_mut().enumerate() {
            for (i_chirho, &p_chirho) in pattern_chirho.bits_chirho.iter().enumerate() {
                *logit_chirho += self.weights_chirho[d_chirho][i_chirho] * p_chirho;
            }
        }

        // Softmax with temperature
        let max_logit_chirho = logits_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut exp_chirho = [0.0; 10];
        let mut sum_exp_chirho = 0.0;
        for (i_chirho, &l_chirho) in logits_chirho.iter().enumerate() {
            exp_chirho[i_chirho] = ((l_chirho - max_logit_chirho) / temp_chirho).exp();
            sum_exp_chirho += exp_chirho[i_chirho];
        }
        for e_chirho in &mut exp_chirho {
            *e_chirho /= sum_exp_chirho;
        }
        exp_chirho
    }

    /// Update weights given gradient
    fn update_chirho(&mut self, grad_chirho: &[[f64; 10]; 10]) {
        for d_chirho in 0..10 {
            for i_chirho in 0..10 {
                self.weights_chirho[d_chirho][i_chirho] -= self.lr_chirho * grad_chirho[d_chirho][i_chirho];
            }
        }
    }
}

/// Soft addition constraint: P(classify(a) + classify(b) = c)
fn soft_addition_constraint_chirho(
    probs_a_chirho: &[f64; 10],
    probs_b_chirho: &[f64; 10],
    target_sum_chirho: u8,
) -> f64 {
    // Sum over all (d1, d2) pairs where d1 + d2 = target
    let mut prob_chirho = 0.0;
    for d1_chirho in 0u8..10 {
        for d2_chirho in 0u8..10 {
            if d1_chirho + d2_chirho == target_sum_chirho {
                // Soft AND: multiply probabilities
                prob_chirho += probs_a_chirho[d1_chirho as usize] * probs_b_chirho[d2_chirho as usize];
            }
        }
    }
    prob_chirho
}

/// Training example
#[derive(Debug)]
struct ExampleChirho {
    pattern_a_chirho: DigitPatternChirho,
    pattern_b_chirho: DigitPatternChirho,
    digit_a_chirho: u8,
    digit_b_chirho: u8,
    sum_chirho: u8,
}

impl ExampleChirho {
    fn new_chirho(a_chirho: u8, b_chirho: u8, noise_chirho: f64) -> Self {
        Self {
            pattern_a_chirho: DigitPatternChirho::from_digit_chirho(a_chirho, noise_chirho),
            pattern_b_chirho: DigitPatternChirho::from_digit_chirho(b_chirho, noise_chirho),
            digit_a_chirho: a_chirho,
            digit_b_chirho: b_chirho,
            sum_chirho: a_chirho + b_chirho,
        }
    }
}

fn main() {
    println!("☧ Symbolic Addition Learning ☧");
    println!();
    println!("Demonstrating MNIST-Addition concept with simplified digit patterns.");
    println!();

    // Create training examples
    let noise_chirho = 0.05;
    let examples_chirho: Vec<ExampleChirho> = vec![
        ExampleChirho::new_chirho(2, 3, noise_chirho),  // 2 + 3 = 5
        ExampleChirho::new_chirho(1, 4, noise_chirho),  // 1 + 4 = 5
        ExampleChirho::new_chirho(3, 3, noise_chirho),  // 3 + 3 = 6
        ExampleChirho::new_chirho(0, 5, noise_chirho),  // 0 + 5 = 5
        ExampleChirho::new_chirho(4, 4, noise_chirho),  // 4 + 4 = 8
        ExampleChirho::new_chirho(2, 7, noise_chirho),  // 2 + 7 = 9
        ExampleChirho::new_chirho(5, 3, noise_chirho),  // 5 + 3 = 8
        ExampleChirho::new_chirho(1, 1, noise_chirho),  // 1 + 1 = 2
    ];

    // Initialize classifier
    let mut classifier_chirho = DigitClassifierChirho::new_chirho(0.1);
    let epochs_chirho = 50;
    let mut temp_chirho = 2.0;  // Temperature annealing

    println!("Training on {} examples for {} epochs...", examples_chirho.len(), epochs_chirho);
    println!();

    for epoch_chirho in 0..epochs_chirho {
        let mut total_loss_chirho = 0.0;
        let mut correct_chirho = 0;

        // Temperature annealing
        temp_chirho = 2.0 * (0.5_f64).powf(epoch_chirho as f64 / (epochs_chirho as f64 / 2.0));
        temp_chirho = temp_chirho.max(0.1);

        for ex_chirho in &examples_chirho {
            // Forward pass
            let probs_a_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_a_chirho, temp_chirho);
            let probs_b_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_b_chirho, temp_chirho);

            // Compute addition constraint probability
            let constraint_prob_chirho = soft_addition_constraint_chirho(
                &probs_a_chirho,
                &probs_b_chirho,
                ex_chirho.sum_chirho,
            );

            // Loss = -log(constraint_prob) (cross-entropy)
            let loss_chirho = -constraint_prob_chirho.max(1e-10).ln();
            total_loss_chirho += loss_chirho;

            // Check if predictions are correct (argmax)
            let pred_a_chirho = probs_a_chirho.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            let pred_b_chirho = probs_b_chirho.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if pred_a_chirho == ex_chirho.digit_a_chirho as usize && pred_b_chirho == ex_chirho.digit_b_chirho as usize {
                correct_chirho += 1;
            }

            // Simplified gradient update (numerical)
            let eps_chirho = 1e-4;
            let mut grad_chirho = [[0.0; 10]; 10];

            for d_chirho in 0..10 {
                for i_chirho in 0..10 {
                    // Perturb weight
                    classifier_chirho.weights_chirho[d_chirho][i_chirho] += eps_chirho;

                    // Recompute loss
                    let probs_a_plus_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_a_chirho, temp_chirho);
                    let probs_b_plus_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_b_chirho, temp_chirho);
                    let prob_plus_chirho = soft_addition_constraint_chirho(&probs_a_plus_chirho, &probs_b_plus_chirho, ex_chirho.sum_chirho);
                    let loss_plus_chirho = -prob_plus_chirho.max(1e-10).ln();

                    // Finite difference gradient
                    grad_chirho[d_chirho][i_chirho] = (loss_plus_chirho - loss_chirho) / eps_chirho;

                    // Restore weight
                    classifier_chirho.weights_chirho[d_chirho][i_chirho] -= eps_chirho;
                }
            }

            // Apply gradient
            classifier_chirho.update_chirho(&grad_chirho);
        }

        let avg_loss_chirho = total_loss_chirho / examples_chirho.len() as f64;
        let acc_chirho = correct_chirho as f64 / examples_chirho.len() as f64 * 100.0;

        if epoch_chirho % 10 == 0 || epoch_chirho == epochs_chirho - 1 {
            println!("Epoch {}: loss={:.4}, accuracy={:.1}%, temp={:.3}",
                     epoch_chirho, avg_loss_chirho, acc_chirho, temp_chirho);
        }
    }

    println!();
    println!("=== Final Test ===");
    println!();

    // Test on examples
    let temp_final_chirho = 0.1;
    for ex_chirho in &examples_chirho {
        let probs_a_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_a_chirho, temp_final_chirho);
        let probs_b_chirho = classifier_chirho.classify_chirho(&ex_chirho.pattern_b_chirho, temp_final_chirho);

        let pred_a_chirho = probs_a_chirho.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let pred_b_chirho = probs_b_chirho.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        let constraint_prob_chirho = soft_addition_constraint_chirho(&probs_a_chirho, &probs_b_chirho, ex_chirho.sum_chirho);

        let check_chirho = if pred_a_chirho == ex_chirho.digit_a_chirho as usize
                          && pred_b_chirho == ex_chirho.digit_b_chirho as usize { "✓" } else { "✗" };

        println!("{} + {} = {} → pred: {} + {} (P(constraint)={:.3}) {}",
                 ex_chirho.digit_a_chirho, ex_chirho.digit_b_chirho, ex_chirho.sum_chirho,
                 pred_a_chirho, pred_b_chirho, constraint_prob_chirho, check_chirho);
    }

    println!();
    println!("☧ Soli Deo Gloria ☧");
}
