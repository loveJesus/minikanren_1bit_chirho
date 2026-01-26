//! Training Diagnostics ☧
//!
//! Gradient norm tracking and stability analysis for neurosymbolic validation.
//! Addresses Gemini feedback item G6: prove gradient stability in deep search trees.

/// Gradient statistics for a training step
#[derive(Debug, Clone, Default)]
pub struct GradientStatsChirho {
    /// L2 norm of gradient vector
    pub l2_norm_chirho: f64,
    /// L-infinity norm (max absolute value)
    pub linf_norm_chirho: f64,
    /// Mean gradient value
    pub mean_chirho: f64,
    /// Variance of gradients
    pub variance_chirho: f64,
    /// Number of gradient elements
    pub count_chirho: usize,
    /// Number of zero gradients (potential vanishing)
    pub zero_count_chirho: usize,
    /// Number of very large gradients (potential exploding)
    pub exploding_count_chirho: usize,
}

impl GradientStatsChirho {
    /// Compute statistics from a gradient vector
    pub fn from_gradients_chirho(gradients_chirho: &[f64]) -> Self {
        if gradients_chirho.is_empty() {
            return Self::default();
        }

        let count_chirho = gradients_chirho.len();

        // L2 norm
        let l2_norm_chirho = gradients_chirho.iter()
            .map(|g| g * g)
            .sum::<f64>()
            .sqrt();

        // L-infinity norm
        let linf_norm_chirho = gradients_chirho.iter()
            .map(|g| g.abs())
            .fold(0.0_f64, f64::max);

        // Mean
        let mean_chirho = gradients_chirho.iter().sum::<f64>() / count_chirho as f64;

        // Variance
        let variance_chirho = gradients_chirho.iter()
            .map(|g| (g - mean_chirho).powi(2))
            .sum::<f64>() / count_chirho as f64;

        // Vanishing detection (|g| < 1e-7)
        let zero_count_chirho = gradients_chirho.iter()
            .filter(|g| g.abs() < 1e-7)
            .count();

        // Exploding detection (|g| > 1e3)
        let exploding_count_chirho = gradients_chirho.iter()
            .filter(|g| g.abs() > 1e3)
            .count();

        Self {
            l2_norm_chirho,
            linf_norm_chirho,
            mean_chirho,
            variance_chirho,
            count_chirho,
            zero_count_chirho,
            exploding_count_chirho,
        }
    }

    /// Check if gradients appear stable
    pub fn is_stable_chirho(&self) -> bool {
        // Stable if:
        // - Not all zero (vanishing)
        // - Not exploding (L-inf < 1e3)
        // Note: Zero variance is fine (all gradients equal but non-zero)
        let not_vanishing_chirho = self.zero_count_chirho < self.count_chirho;
        let not_exploding_chirho = self.exploding_count_chirho == 0;

        not_vanishing_chirho && not_exploding_chirho
    }
}

/// Training history tracker
#[derive(Debug, Clone, Default)]
pub struct TrainingHistoryChirho {
    /// Loss at each iteration
    pub losses_chirho: Vec<f64>,
    /// Gradient statistics at each iteration
    pub grad_stats_chirho: Vec<GradientStatsChirho>,
}

impl TrainingHistoryChirho {
    pub fn new_chirho() -> Self {
        Self::default()
    }

    /// Record a training step
    pub fn record_chirho(&mut self, loss_chirho: f64, gradients_chirho: &[f64]) {
        self.losses_chirho.push(loss_chirho);
        self.grad_stats_chirho.push(GradientStatsChirho::from_gradients_chirho(gradients_chirho));
    }

    /// Check if training appears stable throughout
    pub fn is_stable_chirho(&self) -> bool {
        // All steps should be stable
        self.grad_stats_chirho.iter().all(|s| s.is_stable_chirho())
    }

    /// Get summary statistics
    pub fn summary_chirho(&self) -> TrainingSummaryChirho {
        if self.losses_chirho.is_empty() {
            return TrainingSummaryChirho::default();
        }

        let final_loss_chirho = *self.losses_chirho.last().unwrap();
        let initial_loss_chirho = *self.losses_chirho.first().unwrap();
        let loss_improved_chirho = final_loss_chirho < initial_loss_chirho;

        let avg_grad_norm_chirho = self.grad_stats_chirho.iter()
            .map(|s| s.l2_norm_chirho)
            .sum::<f64>() / self.grad_stats_chirho.len() as f64;

        let max_grad_norm_chirho = self.grad_stats_chirho.iter()
            .map(|s| s.l2_norm_chirho)
            .fold(0.0_f64, f64::max);

        let vanishing_steps_chirho = self.grad_stats_chirho.iter()
            .filter(|s| s.zero_count_chirho == s.count_chirho)
            .count();

        let exploding_steps_chirho = self.grad_stats_chirho.iter()
            .filter(|s| s.exploding_count_chirho > 0)
            .count();

        TrainingSummaryChirho {
            iterations_chirho: self.losses_chirho.len(),
            initial_loss_chirho,
            final_loss_chirho,
            loss_improved_chirho,
            avg_grad_norm_chirho,
            max_grad_norm_chirho,
            vanishing_steps_chirho,
            exploding_steps_chirho,
            stable_chirho: self.is_stable_chirho(),
        }
    }

    /// Print training summary
    pub fn print_summary_chirho(&self) {
        let summary_chirho = self.summary_chirho();
        println!("\n=== Training Summary ===");
        println!("Iterations: {}", summary_chirho.iterations_chirho);
        println!("Loss: {:.6} -> {:.6} ({})",
            summary_chirho.initial_loss_chirho,
            summary_chirho.final_loss_chirho,
            if summary_chirho.loss_improved_chirho { "improved" } else { "worsened" }
        );
        println!("Gradient L2 norm: avg={:.6}, max={:.6}",
            summary_chirho.avg_grad_norm_chirho,
            summary_chirho.max_grad_norm_chirho
        );
        println!("Vanishing steps: {} / {}",
            summary_chirho.vanishing_steps_chirho,
            summary_chirho.iterations_chirho
        );
        println!("Exploding steps: {} / {}",
            summary_chirho.exploding_steps_chirho,
            summary_chirho.iterations_chirho
        );
        println!("Overall stability: {}",
            if summary_chirho.stable_chirho { "STABLE" } else { "UNSTABLE" }
        );
    }
}

/// Summary of training run
#[derive(Debug, Clone, Default)]
pub struct TrainingSummaryChirho {
    pub iterations_chirho: usize,
    pub initial_loss_chirho: f64,
    pub final_loss_chirho: f64,
    pub loss_improved_chirho: bool,
    pub avg_grad_norm_chirho: f64,
    pub max_grad_norm_chirho: f64,
    pub vanishing_steps_chirho: usize,
    pub exploding_steps_chirho: usize,
    pub stable_chirho: bool,
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_gradient_stats_chirho() {
        let grads_chirho = vec![0.1, -0.2, 0.3, -0.1, 0.2];
        let stats_chirho = GradientStatsChirho::from_gradients_chirho(&grads_chirho);

        assert!(stats_chirho.l2_norm_chirho > 0.0);
        assert!(stats_chirho.is_stable_chirho());
        assert_eq!(stats_chirho.zero_count_chirho, 0);
        assert_eq!(stats_chirho.exploding_count_chirho, 0);
    }

    #[test]
    fn test_vanishing_detection_chirho() {
        let grads_chirho = vec![1e-10, 1e-11, 1e-12];
        let stats_chirho = GradientStatsChirho::from_gradients_chirho(&grads_chirho);

        assert_eq!(stats_chirho.zero_count_chirho, 3);
        assert!(!stats_chirho.is_stable_chirho());
    }

    #[test]
    fn test_exploding_detection_chirho() {
        let grads_chirho = vec![0.1, 1e5, -1e4];
        let stats_chirho = GradientStatsChirho::from_gradients_chirho(&grads_chirho);

        assert_eq!(stats_chirho.exploding_count_chirho, 2);
        assert!(!stats_chirho.is_stable_chirho());
    }

    #[test]
    fn test_training_history_chirho() {
        let mut history_chirho = TrainingHistoryChirho::new_chirho();

        // Simulate training with decreasing loss and stable gradients
        for i_chirho in 0..10 {
            let loss_chirho = 1.0 - (i_chirho as f64 * 0.08);
            let grads_chirho = vec![0.1 * (1.0 - i_chirho as f64 * 0.05); 5];
            history_chirho.record_chirho(loss_chirho, &grads_chirho);
        }

        let summary_chirho = history_chirho.summary_chirho();
        assert_eq!(summary_chirho.iterations_chirho, 10);
        assert!(summary_chirho.loss_improved_chirho);
        assert!(summary_chirho.stable_chirho);
    }
}
