// Unified system observer combining all diagnostic systems
// Creates observable record of system behavior and state divergence

use alloc::vec::Vec;
use crate::atomvm_support::{
    failure_detection::FailureDetector,
    conflict_detection::ConflictDetector,
    health_monitor::{HealthMonitor, HealthStatus},
    message_dispatch::MessageOp,
};

/// Observation record of a single operation
#[derive(Debug, Clone)]
pub struct Observation {
    pub timestamp: u64,
    pub sequence: u32,
    pub operation: MessageOp,
    pub state_before: i32,
    pub state_after: i32,
    pub execution_time: u64,
    pub failures_detected: usize,
    pub conflicts_detected: usize,
}

/// System observer state
pub enum ObserverState {
    Normal,
    Degraded,
    Recovering,
    Failed,
}

/// Unified system observer
pub struct SystemObserver {
    observations: Vec<Observation>,
    failure_detector: FailureDetector,
    conflict_detector: ConflictDetector,
    health_monitor: HealthMonitor,
    sequence: u32,
    state: ObserverState,
    total_failures: usize,
    total_conflicts: usize,
    observation_enabled: bool,
}

impl SystemObserver {
    pub fn new(initial_value: i32) -> Self {
        SystemObserver {
            observations: Vec::new(),
            failure_detector: FailureDetector::new(initial_value),
            conflict_detector: ConflictDetector::new(),
            health_monitor: HealthMonitor::new(),
            sequence: 0,
            state: ObserverState::Normal,
            total_failures: 0,
            total_conflicts: 0,
            observation_enabled: true,
        }
    }

    /// Record operation observation
    pub fn observe_operation(
        &mut self,
        operation: MessageOp,
        state_before: i32,
        state_after: i32,
        execution_time: u64,
        timestamp: u64,
    ) -> Observation {
        if !self.observation_enabled {
            return Observation {
                timestamp,
                sequence: 0,
                operation,
                state_before,
                state_after,
                execution_time,
                failures_detected: 0,
                conflicts_detected: 0,
            };
        }

        self.sequence += 1;

        // Check for failures
        let failure = self.failure_detector.check_divergence(state_after, timestamp);
        let failure_count = if failure.is_some() { 1 } else { 0 };
        self.total_failures += failure_count;

        // Check for conflicts
        let conflict = self.conflict_detector.check_arithmetic_invariant(operation, state_before, state_after);
        let conflict_count = if conflict.is_some() { 1 } else { 0 };
        self.total_conflicts += conflict_count;

        // Update health
        let success = conflict.is_none() && failure.is_none();
        self.health_monitor.record_operation(success, execution_time);

        // Update state
        self.update_state(timestamp);

        let obs = Observation {
            timestamp,
            sequence: self.sequence,
            operation,
            state_before,
            state_after,
            execution_time,
            failures_detected: failure_count,
            conflicts_detected: conflict_count,
        };

        self.observations.push(obs.clone());
        obs
    }

    /// Update observer state based on diagnostics
    fn update_state(&mut self, current_time: u64) {
        let metrics = self.health_monitor.check_health(current_time);
        let health_status = metrics.status;

        self.state = match health_status {
            HealthStatus::Healthy => ObserverState::Normal,
            HealthStatus::Degraded => ObserverState::Degraded,
            HealthStatus::Critical => {
                if self.failure_detector.failure_count() > 0 {
                    ObserverState::Failed
                } else {
                    ObserverState::Recovering
                }
            }
            HealthStatus::Unknown => ObserverState::Normal,
        };
    }

    /// Get current state
    pub fn current_state(&self) -> &'static str {
        match self.state {
            ObserverState::Normal => "normal",
            ObserverState::Degraded => "degraded",
            ObserverState::Recovering => "recovering",
            ObserverState::Failed => "failed",
        }
    }

    /// Get observation count
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Get health summary
    pub fn health_summary(&mut self, current_time: u64) -> (HealthStatus, u32, u32) {
        let metrics = self.health_monitor.check_health(current_time);
        (metrics.status, metrics.error_rate, self.total_failures as u32)
    }

    /// Get diagnostic summary
    pub fn diagnostic_summary(&self) -> (usize, usize, usize) {
        (
            self.failure_detector.failure_count(),
            self.conflict_detector.conflict_count(),
            self.observations.len(),
        )
    }

    /// Get operations with issues
    pub fn operations_with_issues(&self) -> Vec<&Observation> {
        self.observations
            .iter()
            .filter(|o| o.failures_detected > 0 || o.conflicts_detected > 0)
            .collect()
    }

    /// Get recent operations
    pub fn recent_observations(&self, count: usize) -> Vec<&Observation> {
        self.observations
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Find issue patterns
    pub fn failure_pattern_in_operation(&self, op: MessageOp) -> usize {
        self.observations
            .iter()
            .filter(|o| o.operation == op && o.failures_detected > 0)
            .count()
    }

    /// Enable/disable observation
    pub fn set_observation_enabled(&mut self, enabled: bool) {
        self.observation_enabled = enabled;
    }

    /// Clear observations but keep diagnostics
    pub fn clear_observations(&mut self) {
        self.observations.clear();
    }

    /// Full reset
    pub fn reset(&mut self, initial_value: i32) {
        self.observations.clear();
        self.failure_detector = FailureDetector::new(initial_value);
        self.conflict_detector = ConflictDetector::new();
        self.health_monitor = HealthMonitor::new();
        self.sequence = 0;
        self.total_failures = 0;
        self.total_conflicts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_creation() {
        let observer = SystemObserver::new(0);
        assert_eq!(observer.observation_count(), 0);
        assert_eq!(observer.current_state(), "normal");
    }

    #[test]
    fn test_observe_operation() {
        let mut observer = SystemObserver::new(0);
        let obs = observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);

        assert_eq!(obs.operation, MessageOp::Inc);
        assert_eq!(obs.state_before, 0);
        assert_eq!(obs.state_after, 1);
        assert_eq!(observer.observation_count(), 1);
    }

    #[test]
    fn test_observe_with_conflict() {
        let mut observer = SystemObserver::new(0);
        let obs = observer.observe_operation(MessageOp::Inc, 0, 5, 100, 0);

        assert_eq!(obs.conflicts_detected, 1);
        assert_eq!(observer.total_conflicts, 1);
    }

    #[test]
    fn test_multiple_observations() {
        let mut observer = SystemObserver::new(0);

        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);
        observer.observe_operation(MessageOp::Inc, 1, 2, 105, 1);
        observer.observe_operation(MessageOp::Dec, 2, 1, 95, 2);

        assert_eq!(observer.observation_count(), 3);
    }

    #[test]
    fn test_diagnostic_summary() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);

        let (failures, conflicts, observations) = observer.diagnostic_summary();
        assert_eq!(observations, 1);
    }

    #[test]
    fn test_operations_with_issues() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);
        observer.observe_operation(MessageOp::Inc, 1, 5, 100, 1);
        observer.observe_operation(MessageOp::Dec, 5, 4, 100, 2);

        let issues = observer.operations_with_issues();
        assert!(issues.len() > 0);
    }

    #[test]
    fn test_recent_observations() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);
        observer.observe_operation(MessageOp::Inc, 1, 2, 100, 1);
        observer.observe_operation(MessageOp::Dec, 2, 1, 100, 2);

        let recent = observer.recent_observations(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_failure_pattern() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 5, 100, 0);
        observer.observe_operation(MessageOp::Inc, 5, 10, 100, 1);

        let pattern = observer.failure_pattern_in_operation(MessageOp::Inc);
        assert!(pattern > 0);
    }

    #[test]
    fn test_observation_can_be_disabled() {
        let mut observer = SystemObserver::new(0);
        observer.set_observation_enabled(false);

        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);
        assert_eq!(observer.observation_count(), 0);
    }

    #[test]
    fn test_clear_observations() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 1, 100, 0);
        assert_eq!(observer.observation_count(), 1);

        observer.clear_observations();
        assert_eq!(observer.observation_count(), 0);
    }

    #[test]
    fn test_state_transitions() {
        let mut observer = SystemObserver::new(0);
        assert_eq!(observer.current_state(), "normal");

        observer.observe_operation(MessageOp::Inc, 0, 5, 100, 0);
        observer.observe_operation(MessageOp::Inc, 5, 10, 100, 1);

        // State may transition to degraded after multiple conflicts
    }

    #[test]
    fn test_reset() {
        let mut observer = SystemObserver::new(0);
        observer.observe_operation(MessageOp::Inc, 0, 5, 100, 0);

        assert_eq!(observer.observation_count(), 1);
        observer.reset(0);
        assert_eq!(observer.observation_count(), 0);
        assert_eq!(observer.total_failures, 0);
    }
}
