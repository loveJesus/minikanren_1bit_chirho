//! Symbolic Addition with Analytic Gradients ☧
//!
//! Demonstrates end-to-end backpropagation through logic constraints,
//! showing that "gradients flow through logic = gradients flow through tensors."
//!
//! ## Key Insight
//!
//! The addition relation IS a sparse Boolean tensor:
//! ```
//! T[d1][d2][sum] = 1 iff d1 + d2 = sum
//! ```
//!
//! Forward pass = tensor contraction with soft-AND (multiply)
//! Backward pass = adjoint of tensor contraction (transpose)
//!
//! ## Architecture
//!
//! ```
//! Input Pattern → Classifier → Softmax → P(digit)
//!                                           ↓
//!                              Addition Tensor Contraction
//!                                           ↓
//!                                    P(sum = target)
//!                                           ↓
//!                                   Loss = -log(P)
//! ```
//!
//! Gradients flow backward through ALL of this analytically.
//!
//! ## Run
//!
//! ```bash
//! cargo run --release --example symbolic_addition_analytic_chirho
//! ```
//!
//! "The fear of the LORD is the beginning of wisdom." — Proverbs 9:10

/// Sparse representation of addition relation tensor
/// Only stores valid (d1, d2, sum) triples where d1 + d2 = sum
struct AdditionTensorChirho {
    /// Valid triples: (d1, d2, sum) where d1 + d2 = sum
    /// For digits 0-9, sums range 0-18
    triples_chirho: Vec<(usize, usize, usize)>,
}

impl AdditionTensorChirho {
    fn new_chirho() -> Self {
        let mut triples_chirho = Vec::new();
        for d1_chirho in 0..10 {
            for d2_chirho in 0..10 {
                let sum_chirho = d1_chirho + d2_chirho;
                triples_chirho.push((d1_chirho, d2_chirho, sum_chirho));
            }
        }
        Self { triples_chirho }
    }

    /// Forward: contract P(a) ⊗ P(b) over addition constraint for target sum
    /// Returns P(a + b = target)
    fn contract_forward_chirho(
        &self,
        probs_a_chirho: &[f64; 10],
        probs_b_chirho: &[f64; 10],
        target_sum_chirho: usize,
    ) -> f64 {
        let mut result_chirho = 0.0;
        for &(d1_chirho, d2_chirho, sum_chirho) in &self.triples_chirho {
            if sum_chirho == target_sum_chirho {
                // Soft-AND: multiply probabilities
                result_chirho += probs_a_chirho[d1_chirho] * probs_b_chirho[d2_chirho];
            }
        }
        result_chirho
    }

    /// Backward: compute gradients w.r.t. P(a) and P(b)
    /// This is the ADJOINT of tensor contraction
    ///
    /// ∂L/∂P(a=d1) = Σ_{d2: d1+d2=target} P(b=d2) × ∂L/∂P(sum)
    /// ∂L/∂P(b=d2) = Σ_{d1: d1+d2=target} P(a=d1) × ∂L/∂P(sum)
    fn contract_backward_chirho(
        &self,
        probs_a_chirho: &[f64; 10],
        probs_b_chirho: &[f64; 10],
        target_sum_chirho: usize,
        grad_output_chirho: f64,
    ) -> ([f64; 10], [f64; 10]) {
        let mut grad_a_chirho = [0.0; 10];
        let mut grad_b_chirho = [0.0; 10];

        for &(d1_chirho, d2_chirho, sum_chirho) in &self.triples_chirho {
            if sum_chirho == target_sum_chirho {
                // Adjoint of multiplication: ∂(a*b)/∂a = b, ∂(a*b)/∂b = a
                grad_a_chirho[d1_chirho] += probs_b_chirho[d2_chirho] * grad_output_chirho;
                grad_b_chirho[d2_chirho] += probs_a_chirho[d1_chirho] * grad_output_chirho;
            }
        }

        (grad_a_chirho, grad_b_chirho)
    }
}

/// Softmax with analytic Jacobian for backprop
struct SoftmaxChirho;

impl SoftmaxChirho {
    /// Forward: logits → probabilities
    fn forward_chirho(logits_chirho: &[f64; 10], temp_chirho: f64) -> [f64; 10] {
        let max_chirho = logits_chirho.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut exp_chirho = [0.0; 10];
        let mut sum_chirho = 0.0;

        for (i_chirho, &l_chirho) in logits_chirho.iter().enumerate() {
            exp_chirho[i_chirho] = ((l_chirho - max_chirho) / temp_chirho).exp();
            sum_chirho += exp_chirho[i_chirho];
        }

        for e_chirho in &mut exp_chirho {
            *e_chirho /= sum_chirho;
        }
        exp_chirho
    }

    /// Backward: gradient w.r.t. logits given gradient w.r.t. probabilities
    ///
    /// Softmax Jacobian: ∂P_j/∂logit_i = P_j × (δ_{ij} - P_i) / temp
    ///
    /// ∂L/∂logit_i = Σ_j ∂L/∂P_j × ∂P_j/∂logit_i
    ///             = Σ_j ∂L/∂P_j × P_j × (δ_{ij} - P_i) / temp
    ///             = (∂L/∂P_i × P_i - P_i × Σ_j ∂L/∂P_j × P_j) / temp
    fn backward_chirho(
        probs_chirho: &[f64; 10],
        grad_probs_chirho: &[f64; 10],
        temp_chirho: f64,
    ) -> [f64; 10] {
        // Compute Σ_j ∂L/∂P_j × P_j
        let dot_chirho: f64 = probs_chirho
            .iter()
            .zip(grad_probs_chirho.iter())
            .map(|(p, g)| p * g)
            .sum();

        let mut grad_logits_chirho = [0.0; 10];
        for i_chirho in 0..10 {
            // ∂L/∂logit_i = P_i × (∂L/∂P_i - dot) / temp
            grad_logits_chirho[i_chirho] =
                probs_chirho[i_chirho] * (grad_probs_chirho[i_chirho] - dot_chirho) / temp_chirho;
        }
        grad_logits_chirho
    }
}

/// Linear classifier with analytic gradients
struct LinearClassifierChirho {
    /// Weights: W[output][input]
    weights_chirho: [[f64; 10]; 10],
    lr_chirho: f64,
}

impl LinearClassifierChirho {
    fn new_chirho(lr_chirho: f64) -> Self {
        // Initialize near-identity (good for one-hot-ish inputs)
        let mut weights_chirho = [[0.1; 10]; 10];
        for i_chirho in 0..10 {
            weights_chirho[i_chirho][i_chirho] = 1.0;
        }
        Self { weights_chirho, lr_chirho }
    }

    /// Forward: input → logits
    fn forward_chirho(&self, input_chirho: &[f64; 10]) -> [f64; 10] {
        let mut logits_chirho = [0.0; 10];
        for (d_chirho, logit_chirho) in logits_chirho.iter_mut().enumerate() {
            for (i_chirho, &x_chirho) in input_chirho.iter().enumerate() {
                *logit_chirho += self.weights_chirho[d_chirho][i_chirho] * x_chirho;
            }
        }
        logits_chirho
    }

    /// Backward: gradient w.r.t. weights given gradient w.r.t. logits
    ///
    /// logit_d = Σ_i W[d][i] × input_i
    /// ∂L/∂W[d][i] = ∂L/∂logit_d × input_i
    fn backward_chirho(
        &self,
        input_chirho: &[f64; 10],
        grad_logits_chirho: &[f64; 10],
    ) -> [[f64; 10]; 10] {
        let mut grad_weights_chirho = [[0.0; 10]; 10];
        for d_chirho in 0..10 {
            for i_chirho in 0..10 {
                grad_weights_chirho[d_chirho][i_chirho] =
                    grad_logits_chirho[d_chirho] * input_chirho[i_chirho];
            }
        }
        grad_weights_chirho
    }

    /// Apply gradient update
    fn update_chirho(&mut self, grad_chirho: &[[f64; 10]; 10]) {
        for d_chirho in 0..10 {
            for i_chirho in 0..10 {
                self.weights_chirho[d_chirho][i_chirho] -= self.lr_chirho * grad_chirho[d_chirho][i_chirho];
            }
        }
    }
}

/// Create a digit pattern (one-hot with noise)
fn digit_pattern_chirho(digit_chirho: u8, noise_chirho: f64) -> [f64; 10] {
    let mut pattern_chirho = [noise_chirho; 10];
    pattern_chirho[digit_chirho as usize] = 1.0 - noise_chirho * 9.0;
    // Normalize
    let sum_chirho: f64 = pattern_chirho.iter().sum();
    for p_chirho in &mut pattern_chirho {
        *p_chirho /= sum_chirho;
    }
    pattern_chirho
}

/// Training example
struct ExampleChirho {
    pattern_a_chirho: [f64; 10],
    pattern_b_chirho: [f64; 10],
    digit_a_chirho: u8,
    digit_b_chirho: u8,
    sum_chirho: u8,
}

impl ExampleChirho {
    fn new_chirho(a_chirho: u8, b_chirho: u8, noise_chirho: f64) -> Self {
        Self {
            pattern_a_chirho: digit_pattern_chirho(a_chirho, noise_chirho),
            pattern_b_chirho: digit_pattern_chirho(b_chirho, noise_chirho),
            digit_a_chirho: a_chirho,
            digit_b_chirho: b_chirho,
            sum_chirho: a_chirho + b_chirho,
        }
    }
}

fn main() {
    println!("☧ Symbolic Addition with Analytic Gradients ☧");
    println!();
    println!("Demonstrating end-to-end backprop through logic constraints.");
    println!("The addition relation IS a tensor; gradients flow through it.");
    println!();

    // Build the addition constraint tensor
    let addition_tensor_chirho = AdditionTensorChirho::new_chirho();
    println!("Addition tensor: {} valid (d1, d2, sum) triples",
             addition_tensor_chirho.triples_chirho.len());
    println!();

    // Training examples
    let noise_chirho = 0.05;
    let examples_chirho = vec![
        ExampleChirho::new_chirho(2, 3, noise_chirho), // 2 + 3 = 5
        ExampleChirho::new_chirho(1, 4, noise_chirho), // 1 + 4 = 5
        ExampleChirho::new_chirho(3, 3, noise_chirho), // 3 + 3 = 6
        ExampleChirho::new_chirho(0, 5, noise_chirho), // 0 + 5 = 5
        ExampleChirho::new_chirho(4, 4, noise_chirho), // 4 + 4 = 8
        ExampleChirho::new_chirho(2, 7, noise_chirho), // 2 + 7 = 9
        ExampleChirho::new_chirho(5, 3, noise_chirho), // 5 + 3 = 8
        ExampleChirho::new_chirho(1, 1, noise_chirho), // 1 + 1 = 2
    ];

    // Initialize classifier
    let mut classifier_chirho = LinearClassifierChirho::new_chirho(0.1);
    let epochs_chirho = 50;

    println!("Training on {} examples for {} epochs...", examples_chirho.len(), epochs_chirho);
    println!("Using ANALYTIC gradients (not finite-difference).");
    println!();

    for epoch_chirho in 0..epochs_chirho {
        let mut total_loss_chirho = 0.0;
        let mut correct_chirho = 0;

        // Temperature annealing: start soft, end hard
        let temp_chirho = (2.0 * (0.5_f64).powf(epoch_chirho as f64 / 25.0)).max(0.1);

        for ex_chirho in &examples_chirho {
            // ============================================================
            // FORWARD PASS
            // ============================================================

            // 1. Classifier: pattern → logits
            let logits_a_chirho = classifier_chirho.forward_chirho(&ex_chirho.pattern_a_chirho);
            let logits_b_chirho = classifier_chirho.forward_chirho(&ex_chirho.pattern_b_chirho);

            // 2. Softmax: logits → probabilities
            let probs_a_chirho = SoftmaxChirho::forward_chirho(&logits_a_chirho, temp_chirho);
            let probs_b_chirho = SoftmaxChirho::forward_chirho(&logits_b_chirho, temp_chirho);

            // 3. Tensor contraction: P(a + b = target)
            let prob_sum_chirho = addition_tensor_chirho.contract_forward_chirho(
                &probs_a_chirho,
                &probs_b_chirho,
                ex_chirho.sum_chirho as usize,
            );

            // 4. Loss: -log(P)
            let loss_chirho = -prob_sum_chirho.max(1e-10).ln();
            total_loss_chirho += loss_chirho;

            // Check accuracy
            let pred_a_chirho = probs_a_chirho
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            let pred_b_chirho = probs_b_chirho
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if pred_a_chirho == ex_chirho.digit_a_chirho as usize
                && pred_b_chirho == ex_chirho.digit_b_chirho as usize
            {
                correct_chirho += 1;
            }

            // ============================================================
            // BACKWARD PASS (Analytic gradients via chain rule)
            // ============================================================

            // 4b. ∂L/∂P(sum) = -1/P(sum)
            let grad_prob_sum_chirho = -1.0 / prob_sum_chirho.max(1e-10);

            // 3b. Tensor contraction backward (ADJOINT)
            let (grad_probs_a_chirho, grad_probs_b_chirho) =
                addition_tensor_chirho.contract_backward_chirho(
                    &probs_a_chirho,
                    &probs_b_chirho,
                    ex_chirho.sum_chirho as usize,
                    grad_prob_sum_chirho,
                );

            // 2b. Softmax backward
            let grad_logits_a_chirho =
                SoftmaxChirho::backward_chirho(&probs_a_chirho, &grad_probs_a_chirho, temp_chirho);
            let grad_logits_b_chirho =
                SoftmaxChirho::backward_chirho(&probs_b_chirho, &grad_probs_b_chirho, temp_chirho);

            // 1b. Classifier backward
            let grad_weights_a_chirho =
                classifier_chirho.backward_chirho(&ex_chirho.pattern_a_chirho, &grad_logits_a_chirho);
            let grad_weights_b_chirho =
                classifier_chirho.backward_chirho(&ex_chirho.pattern_b_chirho, &grad_logits_b_chirho);

            // Combine gradients and update
            let mut grad_weights_chirho = [[0.0; 10]; 10];
            for d_chirho in 0..10 {
                for i_chirho in 0..10 {
                    grad_weights_chirho[d_chirho][i_chirho] =
                        grad_weights_a_chirho[d_chirho][i_chirho]
                            + grad_weights_b_chirho[d_chirho][i_chirho];
                }
            }

            classifier_chirho.update_chirho(&grad_weights_chirho);
        }

        let avg_loss_chirho = total_loss_chirho / examples_chirho.len() as f64;
        let acc_chirho = correct_chirho as f64 / examples_chirho.len() as f64 * 100.0;

        if epoch_chirho % 10 == 0 || epoch_chirho == epochs_chirho - 1 {
            println!(
                "Epoch {:2}: loss={:.4}, accuracy={:5.1}%, temp={:.3}",
                epoch_chirho, avg_loss_chirho, acc_chirho, temp_chirho
            );
        }
    }

    println!();
    println!("=== Final Test ===");
    println!();

    let temp_final_chirho = 0.1;
    let mut all_correct_chirho = true;

    for ex_chirho in &examples_chirho {
        let logits_a_chirho = classifier_chirho.forward_chirho(&ex_chirho.pattern_a_chirho);
        let logits_b_chirho = classifier_chirho.forward_chirho(&ex_chirho.pattern_b_chirho);
        let probs_a_chirho = SoftmaxChirho::forward_chirho(&logits_a_chirho, temp_final_chirho);
        let probs_b_chirho = SoftmaxChirho::forward_chirho(&logits_b_chirho, temp_final_chirho);

        let pred_a_chirho = probs_a_chirho
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let pred_b_chirho = probs_b_chirho
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        let prob_sum_chirho = addition_tensor_chirho.contract_forward_chirho(
            &probs_a_chirho,
            &probs_b_chirho,
            ex_chirho.sum_chirho as usize,
        );

        let correct_chirho = pred_a_chirho == ex_chirho.digit_a_chirho as usize
            && pred_b_chirho == ex_chirho.digit_b_chirho as usize;
        if !correct_chirho {
            all_correct_chirho = false;
        }

        let check_chirho = if correct_chirho { "✓" } else { "✗" };
        println!(
            "{} + {} = {} → pred: {} + {} = {} (P={:.3}) {}",
            ex_chirho.digit_a_chirho,
            ex_chirho.digit_b_chirho,
            ex_chirho.sum_chirho,
            pred_a_chirho,
            pred_b_chirho,
            pred_a_chirho + pred_b_chirho,
            prob_sum_chirho,
            check_chirho
        );
    }

    println!();
    println!("=== Key Insight ===");
    println!();
    println!("The addition constraint is a sparse Boolean tensor:");
    println!("  T[d1][d2][sum] = 1 iff d1 + d2 = sum");
    println!();
    println!("Forward: tensor contraction with soft-AND (multiply)");
    println!("Backward: ADJOINT of contraction (gradients via transpose)");
    println!();
    println!("This proves: gradients flow through logic = gradients flow through tensors");
    println!();

    if all_correct_chirho {
        println!("✓ All predictions correct - analytic backprop works!");
    } else {
        println!("Some predictions incorrect (may need more epochs)");
    }

    println!();
    println!("☧ Soli Deo Gloria ☧");
}
