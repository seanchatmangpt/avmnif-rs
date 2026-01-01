// System failure detection and recovery mechanism
// Observes state divergence and manages recovery procedures

use alloc::vec::Vec;
use crate::atomvm_support::health_monitor::HealthStatus;

/// Failure categories with severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FailureKind {
    StateCorruption,
    QueueOverflow,
    DeadlockCondition,
    ResourceExhaustion,
    HealthCritical,
    RecoveryFailed,
}

/// Failure evidence
#[derive(Debug, Clone)]
pub struct Failure {
    pub kind: FailureKind,
    pub timestamp: u64,
    pub sequence: u32,
    pub expected_value: i32,
    pub actual_value: i32,
    pub recoverable: bool,
}

impl Failure {
    pub fn new(kind: FailureKind, timestamp: u64, expected: i32, actual: i32) -> Self {
        Failure {
            kind,
            timestamp,
            sequence: 0,
            expected_value: expected,
            actual_value: actual,
            recoverable: matches!(kind, FailureKind::HealthCritical | FailureKind::QueueOverflow),
        }
    }
}

/// Recovery action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    ClearQueue,
    ResetState,
    ClearHistory,
    ReleaseResources,
    Abort,
}

/// Recovery procedure result
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub action: RecoveryAction,
    pub success: bool,
    pub state_before: i32,
    pub state_after: i32,
}

/// Failure detector observing system state
pub struct FailureDetector {
    failures: Vec<Failure>,
    recoveries: Vec<RecoveryResult>,
    sequence: u32,
    last_known_value: i32,
    expected_value: i32,
    detection_enabled: bool,
    max_failures: usize,
}

impl FailureDetector {
    pub fn new(initial_value: i32) -> Self {
        FailureDetector {
            failures: Vec::new(),
            recoveries: Vec::new(),
            sequence: 0,
            last_known_value: initial_value,
            expected_value: initial_value,
            detection_enabled: true,
            max_failures: 100,
        }
    }

    /// Check for state divergence
    pub fn check_divergence(&mut self, current: i32, timestamp: u64) -> Option<Failure> {
        if !self.detection_enabled {
            return None;
        }

        if current != self.expected_value {
            let failure = Failure::new(
                FailureKind::StateCorruption,
                timestamp,
                self.expected_value,
                current,
            );

            if self.failures.len() < self.max_failures {
                self.failures.push(failure.clone());
            }

            return Some(failure);
        }

        self.last_known_value = current;
        None
    }

    /// Observe operation result
    pub fn observe_operation(&mut self, operation_value: i32, _timestamp: u64) {
        self.sequence += 1;
        self.last_known_value = operation_value;
        self.expected_value = operation_value;
    }

    /// Detect queue overflow
    pub fn detect_queue_overflow(&mut self, queue_size: usize, max_size: usize, timestamp: u64) -> Option<Failure> {
        if queue_size > max_size * 90 / 100 {
            let failure = Failure::new(
                FailureKind::QueueOverflow,
                timestamp,
                max_size as i32,
                queue_size as i32,
            );

            if self.failures.len() < self.max_failures {
                self.failures.push(failure.clone());
            }

            return Some(failure);
        }

        None
    }

    /// Detect critical health
    pub fn detect_health_critical(&mut self, status: HealthStatus, timestamp: u64) -> Option<Failure> {
        if status == HealthStatus::Critical {
            let failure = Failure::new(FailureKind::HealthCritical, timestamp, 0, 0);

            if self.failures.len() < self.max_failures {
                self.failures.push(failure.clone());
            }

            return Some(failure);
        }

        None
    }

    /// Attempt recovery
    pub fn recover(&mut self, failure: &Failure, current_value: i32) -> RecoveryResult {
        let action = match failure.kind {
            FailureKind::QueueOverflow => RecoveryAction::ClearQueue,
            FailureKind::HealthCritical => RecoveryAction::ReleaseResources,
            FailureKind::StateCorruption => RecoveryAction::ResetState,
            _ => RecoveryAction::Abort,
        };

        let success = match action {
            RecoveryAction::ClearQueue => true,
            RecoveryAction::ResetState => true,
            RecoveryAction::ClearHistory => true,
            RecoveryAction::ReleaseResources => true,
            RecoveryAction::Abort => false,
        };

        let state_after = if success { failure.expected_value } else { current_value };

        let result = RecoveryResult {
            action,
            success,
            state_before: current_value,
            state_after,
        };

        self.recoveries.push(result.clone());
        result
    }

    /// Get failure count
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get recovery count
    pub fn recovery_count(&self) -> usize {
        self.recoveries.len()
    }

    /// Get failures by kind
    pub fn failures_by_kind(&self, kind: FailureKind) -> Vec<&Failure> {
        self.failures.iter().filter(|f| f.kind == kind).collect()
    }

    /// Get successful recoveries
    pub fn successful_recoveries(&self) -> Vec<&RecoveryResult> {
        self.recoveries.iter().filter(|r| r.success).collect()
    }

    /// Enable/disable detection
    pub fn set_detection_enabled(&mut self, enabled: bool) {
        self.detection_enabled = enabled;
    }

    /// Reset detector
    pub fn reset(&mut self) {
        self.failures.clear();
        self.recoveries.clear();
        self.sequence = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_creation() {
        let failure = Failure::new(FailureKind::StateCorruption, 0, 10, 5);
        assert_eq!(failure.expected_value, 10);
        assert_eq!(failure.actual_value, 5);
    }

    #[test]
    fn test_detector_creation() {
        let detector = FailureDetector::new(0);
        assert_eq!(detector.failure_count(), 0);
        assert_eq!(detector.expected_value, 0);
    }

    #[test]
    fn test_divergence_detection() {
        let mut detector = FailureDetector::new(5);

        let failure = detector.check_divergence(3, 0);
        assert!(failure.is_some());
        assert_eq!(detector.failure_count(), 1);
    }

    #[test]
    fn test_no_divergence() {
        let mut detector = FailureDetector::new(5);
        detector.expected_value = 5;

        let failure = detector.check_divergence(5, 0);
        assert!(failure.is_none());
    }

    #[test]
    fn test_queue_overflow_detection() {
        let mut detector = FailureDetector::new(0);

        let failure = detector.detect_queue_overflow(950, 1000, 0);
        assert!(failure.is_some());
    }

    #[test]
    fn test_no_queue_overflow() {
        let mut detector = FailureDetector::new(0);

        let failure = detector.detect_queue_overflow(800, 1000, 0);
        assert!(failure.is_none());
    }

    #[test]
    fn test_health_critical_detection() {
        let mut detector = FailureDetector::new(0);

        let failure = detector.detect_health_critical(HealthStatus::Critical, 0);
        assert!(failure.is_some());
        assert_eq!(failure.unwrap().kind, FailureKind::HealthCritical);
    }

    #[test]
    fn test_recovery_queue_overflow() {
        let mut detector = FailureDetector::new(0);
        let failure = Failure::new(FailureKind::QueueOverflow, 0, 0, 950);

        let result = detector.recover(&failure, 5);
        assert!(result.success);
        assert_eq!(result.action, RecoveryAction::ClearQueue);
    }

    #[test]
    fn test_recovery_state_corruption() {
        let mut detector = FailureDetector::new(10);
        let failure = Failure::new(FailureKind::StateCorruption, 0, 10, 5);

        let result = detector.recover(&failure, 5);
        assert!(result.success);
        assert_eq!(result.action, RecoveryAction::ResetState);
    }

    #[test]
    fn test_multiple_failures() {
        let mut detector = FailureDetector::new(0);

        detector.check_divergence(1, 0);
        detector.check_divergence(2, 0);
        detector.check_divergence(3, 0);

        assert_eq!(detector.failure_count(), 3);
    }

    #[test]
    fn test_failures_by_kind() {
        let mut detector = FailureDetector::new(0);

        detector.check_divergence(1, 0); // StateCorruption
        detector.detect_queue_overflow(950, 1000, 0); // QueueOverflow

        let corruptions = detector.failures_by_kind(FailureKind::StateCorruption);
        assert_eq!(corruptions.len(), 1);

        let overflows = detector.failures_by_kind(FailureKind::QueueOverflow);
        assert_eq!(overflows.len(), 1);
    }

    #[test]
    fn test_detection_can_be_disabled() {
        let mut detector = FailureDetector::new(5);
        detector.set_detection_enabled(false);

        let failure = detector.check_divergence(3, 0);
        assert!(failure.is_none());
        assert_eq!(detector.failure_count(), 0);
    }

    #[test]
    fn test_reset() {
        let mut detector = FailureDetector::new(0);
        detector.check_divergence(1, 0);
        detector.detect_queue_overflow(950, 1000, 0);

        assert!(detector.failure_count() > 0);
        detector.reset();
        assert_eq!(detector.failure_count(), 0);
    }

    #[test]
    fn test_recovery_tracking() {
        let mut detector = FailureDetector::new(10);
        let failure1 = Failure::new(FailureKind::QueueOverflow, 0, 0, 950);
        let failure2 = Failure::new(FailureKind::StateCorruption, 0, 10, 5);

        detector.recover(&failure1, 5);
        detector.recover(&failure2, 5);

        assert_eq!(detector.recovery_count(), 2);
        let successful = detector.successful_recoveries();
        assert_eq!(successful.len(), 2);
    }
}
