// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Temperature Annealing ☧
//!
//! Schedules for transitioning from soft (high temperature) to hard (low temperature) logic.

/// Annealing schedule for soft-to-hard transitions
#[derive(Debug, Clone)]
pub struct AnnealingScheduleChirho {
    /// Initial temperature (high = soft)
    pub initial_temp_chirho: f64,
    /// Final temperature (low = hard)
    pub final_temp_chirho: f64,
    /// Total number of steps
    pub total_steps_chirho: usize,
    /// Current step
    pub current_step_chirho: usize,
}

impl AnnealingScheduleChirho {
    /// Create a new annealing schedule
    pub fn new_chirho(initial_temp_chirho: f64, final_temp_chirho: f64, total_steps_chirho: usize) -> Self {
        Self {
            initial_temp_chirho,
            final_temp_chirho,
            total_steps_chirho,
            current_step_chirho: 0,
        }
    }

    /// Create schedule with reasonable defaults
    pub fn default_chirho(total_steps_chirho: usize) -> Self {
        Self::new_chirho(1.0, 0.01, total_steps_chirho)
    }

    /// Get current temperature using exponential decay
    pub fn current_temp_chirho(&self) -> f64 {
        if self.total_steps_chirho == 0 {
            return self.final_temp_chirho;
        }

        let progress_chirho = self.current_step_chirho as f64 / self.total_steps_chirho as f64;
        let ratio_chirho = self.final_temp_chirho / self.initial_temp_chirho;
        self.initial_temp_chirho * ratio_chirho.powf(progress_chirho)
    }

    /// Get temperature at a specific step
    pub fn temp_at_step_chirho(&self, step_chirho: usize) -> f64 {
        if self.total_steps_chirho == 0 {
            return self.final_temp_chirho;
        }

        let progress_chirho = step_chirho as f64 / self.total_steps_chirho as f64;
        let ratio_chirho = self.final_temp_chirho / self.initial_temp_chirho;
        self.initial_temp_chirho * ratio_chirho.powf(progress_chirho)
    }

    /// Advance to next step
    pub fn step_chirho(&mut self) {
        if self.current_step_chirho < self.total_steps_chirho {
            self.current_step_chirho += 1;
        }
    }

    /// Reset to beginning
    pub fn reset_chirho(&mut self) {
        self.current_step_chirho = 0;
    }

    /// Check if annealing is complete
    pub fn is_done_chirho(&self) -> bool {
        self.current_step_chirho >= self.total_steps_chirho
    }

    /// Get progress as fraction [0, 1]
    pub fn progress_chirho(&self) -> f64 {
        if self.total_steps_chirho == 0 {
            return 1.0;
        }
        self.current_step_chirho as f64 / self.total_steps_chirho as f64
    }
}

/// Linear annealing schedule
#[derive(Debug, Clone)]
pub struct LinearAnnealChirho {
    pub initial_temp_chirho: f64,
    pub final_temp_chirho: f64,
    pub total_steps_chirho: usize,
}

impl LinearAnnealChirho {
    pub fn new_chirho(initial_temp_chirho: f64, final_temp_chirho: f64, total_steps_chirho: usize) -> Self {
        Self { initial_temp_chirho, final_temp_chirho, total_steps_chirho }
    }

    pub fn temp_at_step_chirho(&self, step_chirho: usize) -> f64 {
        if self.total_steps_chirho == 0 {
            return self.final_temp_chirho;
        }

        let progress_chirho = (step_chirho as f64 / self.total_steps_chirho as f64).min(1.0);
        self.initial_temp_chirho + (self.final_temp_chirho - self.initial_temp_chirho) * progress_chirho
    }
}

/// Cosine annealing schedule (smoother transitions)
#[derive(Debug, Clone)]
pub struct CosineAnnealChirho {
    pub initial_temp_chirho: f64,
    pub final_temp_chirho: f64,
    pub total_steps_chirho: usize,
}

impl CosineAnnealChirho {
    pub fn new_chirho(initial_temp_chirho: f64, final_temp_chirho: f64, total_steps_chirho: usize) -> Self {
        Self { initial_temp_chirho, final_temp_chirho, total_steps_chirho }
    }

    pub fn temp_at_step_chirho(&self, step_chirho: usize) -> f64 {
        if self.total_steps_chirho == 0 {
            return self.final_temp_chirho;
        }

        let progress_chirho = (step_chirho as f64 / self.total_steps_chirho as f64).min(1.0);
        let cosine_chirho = (1.0 + (std::f64::consts::PI * progress_chirho).cos()) / 2.0;
        self.final_temp_chirho + (self.initial_temp_chirho - self.final_temp_chirho) * cosine_chirho
    }
}

/// Soft equality with temperature
pub fn soft_eq_annealed_chirho(a_chirho: f64, b_chirho: f64, temp_chirho: f64) -> f64 {
    let safe_temp_chirho = temp_chirho.max(1e-10);
    (-(a_chirho - b_chirho).abs() / safe_temp_chirho).exp()
}

/// Soft AND (product) that becomes harder as temp decreases
pub fn soft_and_annealed_chirho(a_chirho: f64, b_chirho: f64, temp_chirho: f64) -> f64 {
    if temp_chirho > 0.5 {
        // Soft: geometric mean
        (a_chirho * b_chirho).sqrt()
    } else {
        // Hard: product
        a_chirho * b_chirho
    }
}

/// Soft OR that becomes harder as temp decreases
pub fn soft_or_annealed_chirho(a_chirho: f64, b_chirho: f64, temp_chirho: f64) -> f64 {
    if temp_chirho > 0.5 {
        // Soft: bounded sum
        (a_chirho + b_chirho).min(1.0)
    } else {
        // Hard: probabilistic OR
        a_chirho + b_chirho - a_chirho * b_chirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_exponential_anneal_chirho() {
        let schedule_chirho = AnnealingScheduleChirho::new_chirho(1.0, 0.01, 100);

        // At step 0, temp should be initial
        assert!((schedule_chirho.temp_at_step_chirho(0) - 1.0).abs() < 1e-6);

        // At final step, temp should be final
        let final_temp_chirho = schedule_chirho.temp_at_step_chirho(100);
        assert!((final_temp_chirho - 0.01).abs() < 1e-6);

        // Midpoint should be geometric mean
        let mid_temp_chirho = schedule_chirho.temp_at_step_chirho(50);
        assert!(mid_temp_chirho < 1.0 && mid_temp_chirho > 0.01);
    }

    #[test]
    fn test_soft_eq_annealed_chirho() {
        // High temp: soft, wide
        let high_temp_chirho = soft_eq_annealed_chirho(0.5, 0.6, 1.0);
        // Low temp: hard, sharp
        let low_temp_chirho = soft_eq_annealed_chirho(0.5, 0.6, 0.01);

        // Low temp should be more discriminating
        assert!(low_temp_chirho < high_temp_chirho);
    }

    #[test]
    fn test_cosine_anneal_chirho() {
        let schedule_chirho = CosineAnnealChirho::new_chirho(1.0, 0.0, 100);

        // Cosine should be smoother at endpoints
        let early_chirho = schedule_chirho.temp_at_step_chirho(5);
        let late_chirho = schedule_chirho.temp_at_step_chirho(95);

        // Early changes slowly (cosine derivative small near 0)
        assert!(early_chirho > 0.9);
        // Late also changes slowly (cosine derivative small near pi)
        assert!(late_chirho < 0.1);
    }
}
