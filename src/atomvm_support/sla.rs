//! Deterministic SLA accounting ported from the unrdf AtomVM monitoring surface.
//!
//! Callers provide monotonic ticks. No wall clock, tracing provider, allocation, or I/O is
//! required, so the same evidence logic can run on embedded AtomVM targets.

/// Snapshot of SLA measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlaStats {
    pub samples: u64,
    pub violations: u64,
    pub total_ticks: u128,
    pub min_ticks: u64,
    pub max_ticks: u64,
    pub average_ticks: u64,
}

/// One measured operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlaSample {
    pub elapsed_ticks: u64,
    pub violated: bool,
}

/// Fixed-threshold latency tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlaTracker {
    limit_ticks: u64,
    samples: u64,
    violations: u64,
    total_ticks: u128,
    min_ticks: u64,
    max_ticks: u64,
}

impl SlaTracker {
    pub const fn new(limit_ticks: u64) -> Self {
        Self {
            limit_ticks,
            samples: 0,
            violations: 0,
            total_ticks: 0,
            min_ticks: u64::MAX,
            max_ticks: 0,
        }
    }

    pub const fn limit_ticks(&self) -> u64 {
        self.limit_ticks
    }

    /// Record a caller-measured latency.
    pub fn record(&mut self, elapsed_ticks: u64) -> SlaSample {
        self.samples = self.samples.saturating_add(1);
        self.total_ticks = self.total_ticks.saturating_add(elapsed_ticks as u128);
        self.min_ticks = core::cmp::min(self.min_ticks, elapsed_ticks);
        self.max_ticks = core::cmp::max(self.max_ticks, elapsed_ticks);
        let violated = elapsed_ticks > self.limit_ticks;
        if violated {
            self.violations = self.violations.saturating_add(1);
        }
        SlaSample {
            elapsed_ticks,
            violated,
        }
    }

    /// Measure an interval using caller-owned monotonic timestamps.
    pub fn record_interval(&mut self, start_tick: u64, end_tick: u64) -> SlaSample {
        self.record(end_tick.saturating_sub(start_tick))
    }

    pub fn stats(&self) -> SlaStats {
        let average_ticks = if self.samples == 0 {
            0
        } else {
            (self.total_ticks / self.samples as u128).min(u64::MAX as u128) as u64
        };
        SlaStats {
            samples: self.samples,
            violations: self.violations,
            total_ticks: self.total_ticks,
            min_ticks: if self.samples == 0 { 0 } else { self.min_ticks },
            max_ticks: self.max_ticks,
            average_ticks,
        }
    }

    pub fn reset(&mut self) {
        self.samples = 0;
        self.violations = 0;
        self.total_ticks = 0;
        self.min_ticks = u64::MAX;
        self.max_ticks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_roundtrip_statistics_without_wall_clock() {
        let mut tracker = SlaTracker::new(10);
        assert!(!tracker.record_interval(100, 108).violated);
        assert!(tracker.record_interval(200, 215).violated);
        assert!(!tracker.record(7).violated);

        assert_eq!(
            tracker.stats(),
            SlaStats {
                samples: 3,
                violations: 1,
                total_ticks: 30,
                min_ticks: 7,
                max_ticks: 15,
                average_ticks: 10,
            }
        );
    }

    #[test]
    fn empty_stats_are_zero_and_reset_is_replayable() {
        let mut tracker = SlaTracker::new(5);
        assert_eq!(tracker.stats().min_ticks, 0);
        tracker.record(9);
        tracker.reset();
        assert_eq!(tracker.stats().samples, 0);
        assert_eq!(tracker.stats().violations, 0);
    }
}
