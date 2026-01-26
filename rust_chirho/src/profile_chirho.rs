// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! Profiling Infrastructure for Wall-Clock Breakdown ☧
//!
//! Tracks time spent in different phases to prove interning isn't a bottleneck.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut profiler = ProfilerChirho::new_chirho();
//! profiler.start_phase_chirho(PhaseChirho::Intern);
//! // ... do interning ...
//! profiler.end_phase_chirho();
//!
//! profiler.start_phase_chirho(PhaseChirho::Unify);
//! // ... do unification ...
//! profiler.end_phase_chirho();
//!
//! profiler.print_breakdown_chirho();
//! ```

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Execution phases for profiling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseChirho {
    /// Hash-consing / term interning
    Intern,
    /// Unification operations
    Unify,
    /// Walk/dereference operations
    Walk,
    /// Search/branching operations
    Search,
    /// Constraint propagation
    Propagate,
    /// Goal execution
    GoalExec,
    /// Other overhead
    Other,
}

impl PhaseChirho {
    pub fn name_chirho(&self) -> &'static str {
        match self {
            PhaseChirho::Intern => "Intern (hash-cons)",
            PhaseChirho::Unify => "Unify",
            PhaseChirho::Walk => "Walk/deref",
            PhaseChirho::Search => "Search/branch",
            PhaseChirho::Propagate => "Propagate",
            PhaseChirho::GoalExec => "Goal exec",
            PhaseChirho::Other => "Other",
        }
    }
}

/// Phase timing statistics
#[derive(Debug, Clone, Default)]
pub struct PhaseStatsChirho {
    pub total_time_chirho: Duration,
    pub call_count_chirho: u64,
    pub max_time_chirho: Duration,
    pub min_time_chirho: Duration,
}

impl PhaseStatsChirho {
    pub fn new_chirho() -> Self {
        Self {
            total_time_chirho: Duration::ZERO,
            call_count_chirho: 0,
            max_time_chirho: Duration::ZERO,
            min_time_chirho: Duration::MAX,
        }
    }

    pub fn record_chirho(&mut self, duration_chirho: Duration) {
        self.total_time_chirho += duration_chirho;
        self.call_count_chirho += 1;
        self.max_time_chirho = self.max_time_chirho.max(duration_chirho);
        self.min_time_chirho = self.min_time_chirho.min(duration_chirho);
    }

    pub fn avg_time_chirho(&self) -> Duration {
        if self.call_count_chirho == 0 {
            Duration::ZERO
        } else {
            self.total_time_chirho / self.call_count_chirho as u32
        }
    }
}

/// Profiler for tracking execution time breakdown
#[derive(Debug)]
pub struct ProfilerChirho {
    stats_chirho: HashMap<PhaseChirho, PhaseStatsChirho>,
    current_phase_chirho: Option<PhaseChirho>,
    phase_start_chirho: Option<Instant>,
    total_start_chirho: Instant,
    enabled_chirho: bool,
}

impl ProfilerChirho {
    pub fn new_chirho() -> Self {
        Self {
            stats_chirho: HashMap::new(),
            current_phase_chirho: None,
            phase_start_chirho: None,
            total_start_chirho: Instant::now(),
            enabled_chirho: true,
        }
    }

    /// Create a disabled profiler (zero overhead)
    pub fn disabled_chirho() -> Self {
        Self {
            stats_chirho: HashMap::new(),
            current_phase_chirho: None,
            phase_start_chirho: None,
            total_start_chirho: Instant::now(),
            enabled_chirho: false,
        }
    }

    /// Start timing a phase
    #[inline]
    pub fn start_phase_chirho(&mut self, phase_chirho: PhaseChirho) {
        if !self.enabled_chirho {
            return;
        }

        // End previous phase if any
        if self.current_phase_chirho.is_some() {
            self.end_phase_chirho();
        }

        self.current_phase_chirho = Some(phase_chirho);
        self.phase_start_chirho = Some(Instant::now());
    }

    /// End timing the current phase
    #[inline]
    pub fn end_phase_chirho(&mut self) {
        if !self.enabled_chirho {
            return;
        }

        if let (Some(phase_chirho), Some(start_chirho)) =
            (self.current_phase_chirho, self.phase_start_chirho)
        {
            let duration_chirho = start_chirho.elapsed();
            self.stats_chirho
                .entry(phase_chirho)
                .or_insert_with(PhaseStatsChirho::new_chirho)
                .record_chirho(duration_chirho);
        }

        self.current_phase_chirho = None;
        self.phase_start_chirho = None;
    }

    /// Time a closure in a specific phase
    #[inline]
    pub fn time_phase_chirho<F, R>(&mut self, phase_chirho: PhaseChirho, f_chirho: F) -> R
    where
        F: FnOnce() -> R,
    {
        if !self.enabled_chirho {
            return f_chirho();
        }

        self.start_phase_chirho(phase_chirho);
        let result_chirho = f_chirho();
        self.end_phase_chirho();
        result_chirho
    }

    /// Get total elapsed time
    pub fn total_time_chirho(&self) -> Duration {
        self.total_start_chirho.elapsed()
    }

    /// Get stats for a specific phase
    pub fn phase_stats_chirho(&self, phase_chirho: PhaseChirho) -> Option<&PhaseStatsChirho> {
        self.stats_chirho.get(&phase_chirho)
    }

    /// Get breakdown as percentages
    pub fn breakdown_chirho(&self) -> BreakdownChirho {
        let total_chirho = self.total_time_chirho();
        let mut phases_chirho = Vec::new();

        let tracked_total_chirho: Duration = self
            .stats_chirho
            .values()
            .map(|s| s.total_time_chirho)
            .sum();

        for phase_chirho in [
            PhaseChirho::Intern,
            PhaseChirho::Unify,
            PhaseChirho::Walk,
            PhaseChirho::Search,
            PhaseChirho::Propagate,
            PhaseChirho::GoalExec,
            PhaseChirho::Other,
        ] {
            if let Some(stats_chirho) = self.stats_chirho.get(&phase_chirho) {
                let pct_chirho = if total_chirho.as_nanos() > 0 {
                    (stats_chirho.total_time_chirho.as_nanos() as f64
                        / total_chirho.as_nanos() as f64)
                        * 100.0
                } else {
                    0.0
                };
                phases_chirho.push((phase_chirho, stats_chirho.clone(), pct_chirho));
            }
        }

        let untracked_chirho = if total_chirho > tracked_total_chirho {
            total_chirho - tracked_total_chirho
        } else {
            Duration::ZERO
        };

        BreakdownChirho {
            total_time_chirho: total_chirho,
            tracked_time_chirho: tracked_total_chirho,
            untracked_time_chirho: untracked_chirho,
            phases_chirho,
        }
    }

    /// Print formatted breakdown
    pub fn print_breakdown_chirho(&self) {
        let breakdown_chirho = self.breakdown_chirho();

        println!("\n=== Wall-Clock Breakdown ===\n");
        println!(
            "Total time: {:.3}ms",
            breakdown_chirho.total_time_chirho.as_secs_f64() * 1000.0
        );
        println!();

        println!("| Phase                | Time (ms) | Calls    | Avg (ns) | % Total |");
        println!("|----------------------|-----------|----------|----------|---------|");

        for (phase_chirho, stats_chirho, pct_chirho) in &breakdown_chirho.phases_chirho {
            println!(
                "| {:<20} | {:>9.3} | {:>8} | {:>8.1} | {:>6.2}% |",
                phase_chirho.name_chirho(),
                stats_chirho.total_time_chirho.as_secs_f64() * 1000.0,
                stats_chirho.call_count_chirho,
                stats_chirho.avg_time_chirho().as_nanos() as f64,
                pct_chirho
            );
        }

        let untracked_pct_chirho = if breakdown_chirho.total_time_chirho.as_nanos() > 0 {
            (breakdown_chirho.untracked_time_chirho.as_nanos() as f64
                / breakdown_chirho.total_time_chirho.as_nanos() as f64)
                * 100.0
        } else {
            0.0
        };

        if untracked_pct_chirho > 0.1 {
            println!(
                "| {:<20} | {:>9.3} |        - |        - | {:>6.2}% |",
                "Untracked",
                breakdown_chirho.untracked_time_chirho.as_secs_f64() * 1000.0,
                untracked_pct_chirho
            );
        }

        println!();
    }

    /// Reset all stats
    pub fn reset_chirho(&mut self) {
        self.stats_chirho.clear();
        self.current_phase_chirho = None;
        self.phase_start_chirho = None;
        self.total_start_chirho = Instant::now();
    }
}

/// Breakdown result
#[derive(Debug, Clone)]
pub struct BreakdownChirho {
    pub total_time_chirho: Duration,
    pub tracked_time_chirho: Duration,
    pub untracked_time_chirho: Duration,
    pub phases_chirho: Vec<(PhaseChirho, PhaseStatsChirho, f64)>,
}

impl BreakdownChirho {
    /// Get percentage for a specific phase
    pub fn phase_pct_chirho(&self, phase_chirho: PhaseChirho) -> f64 {
        self.phases_chirho
            .iter()
            .find(|(p, _, _)| *p == phase_chirho)
            .map(|(_, _, pct)| *pct)
            .unwrap_or(0.0)
    }

    /// Check if interning is a bottleneck (>20% of total)
    pub fn interning_is_bottleneck_chirho(&self) -> bool {
        self.phase_pct_chirho(PhaseChirho::Intern) > 20.0
    }
}

/// RAII guard for automatic phase timing
pub struct PhaseGuardChirho<'a> {
    profiler_chirho: &'a mut ProfilerChirho,
}

impl<'a> PhaseGuardChirho<'a> {
    pub fn new_chirho(profiler_chirho: &'a mut ProfilerChirho, phase_chirho: PhaseChirho) -> Self {
        profiler_chirho.start_phase_chirho(phase_chirho);
        Self { profiler_chirho }
    }
}

impl<'a> Drop for PhaseGuardChirho<'a> {
    fn drop(&mut self) {
        self.profiler_chirho.end_phase_chirho();
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic_profiling_chirho() {
        let mut profiler_chirho = ProfilerChirho::new_chirho();

        profiler_chirho.start_phase_chirho(PhaseChirho::Intern);
        thread::sleep(Duration::from_micros(100));
        profiler_chirho.end_phase_chirho();

        profiler_chirho.start_phase_chirho(PhaseChirho::Unify);
        thread::sleep(Duration::from_micros(200));
        profiler_chirho.end_phase_chirho();

        let breakdown_chirho = profiler_chirho.breakdown_chirho();
        assert!(breakdown_chirho.phases_chirho.len() >= 2);
    }

    #[test]
    fn test_time_phase_chirho() {
        let mut profiler_chirho = ProfilerChirho::new_chirho();

        let result_chirho = profiler_chirho.time_phase_chirho(PhaseChirho::Intern, || {
            thread::sleep(Duration::from_micros(50));
            42
        });

        assert_eq!(result_chirho, 42);
        let stats_chirho = profiler_chirho.phase_stats_chirho(PhaseChirho::Intern).unwrap();
        assert_eq!(stats_chirho.call_count_chirho, 1);
    }

    #[test]
    fn test_disabled_profiler_chirho() {
        let mut profiler_chirho = ProfilerChirho::disabled_chirho();

        profiler_chirho.start_phase_chirho(PhaseChirho::Intern);
        profiler_chirho.end_phase_chirho();

        let breakdown_chirho = profiler_chirho.breakdown_chirho();
        assert!(breakdown_chirho.phases_chirho.is_empty());
    }

    #[test]
    fn test_breakdown_percentage_chirho() {
        let mut profiler_chirho = ProfilerChirho::new_chirho();

        // Simulate workload
        for _ in 0..10 {
            profiler_chirho.start_phase_chirho(PhaseChirho::Intern);
            thread::sleep(Duration::from_micros(10));
            profiler_chirho.end_phase_chirho();
        }

        for _ in 0..100 {
            profiler_chirho.start_phase_chirho(PhaseChirho::Unify);
            thread::sleep(Duration::from_micros(10));
            profiler_chirho.end_phase_chirho();
        }

        let breakdown_chirho = profiler_chirho.breakdown_chirho();

        // Unify should be ~10x more than Intern
        let intern_pct_chirho = breakdown_chirho.phase_pct_chirho(PhaseChirho::Intern);
        let unify_pct_chirho = breakdown_chirho.phase_pct_chirho(PhaseChirho::Unify);

        assert!(unify_pct_chirho > intern_pct_chirho);
    }
}
