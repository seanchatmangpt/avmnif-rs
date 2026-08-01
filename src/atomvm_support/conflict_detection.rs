// Conflict detection between intended and actual system behavior
// Exposes contradictions and invariant violations

use alloc::vec::Vec;
use crate::atomvm_support::message_dispatch::MessageOp;

/// Conflict types representing broken invariants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    SequenceViolation,
    ArithmeticInvariant,
    OrderingViolation,
    ConsistencyViolation,
    TimingViolation,
}

/// Conflict evidence
#[derive(Debug, Clone)]
pub struct Conflict {
    pub kind: ConflictKind,
    pub sequence: u32,
    pub operation: MessageOp,
    pub expected_result: i32,
    pub actual_result: i32,
    pub violation_detail: &'static str,
}

/// Conflict tracker
pub struct ConflictDetector {
    conflicts: Vec<Conflict>,
    sequence: u32,
    last_operation: Option<MessageOp>,
    last_value: i32,
    invariant_checks_enabled: bool,
}

impl ConflictDetector {
    pub fn new() -> Self {
        ConflictDetector {
            conflicts: Vec::new(),
            sequence: 0,
            last_operation: None,
            last_value: 0,
            invariant_checks_enabled: true,
        }
    }

    /// Check arithmetic invariant: operations produce deterministic results
    pub fn check_arithmetic_invariant(
        &mut self,
        op: MessageOp,
        current_value: i32,
        result: i32,
    ) -> Option<Conflict> {
        if !self.invariant_checks_enabled {
            return None;
        }

        self.sequence += 1;

        let expected = match op {
            MessageOp::Inc => current_value.saturating_add(1),
            MessageOp::Dec => current_value.saturating_sub(1),
            MessageOp::Get => current_value,
            MessageOp::Reset => 0,
            MessageOp::Unknown => return None,
        };

        if result != expected {
            let conflict = Conflict {
                kind: ConflictKind::ArithmeticInvariant,
                sequence: self.sequence,
                operation: op,
                expected_result: expected,
                actual_result: result,
                violation_detail: "Operation produced unexpected value",
            };
            self.conflicts.push(conflict.clone());
            return Some(conflict);
        }

        self.last_operation = Some(op);
        self.last_value = result;
        None
    }

    /// Check ordering: operations on same value must be consistent
    pub fn check_ordering_invariant(
        &mut self,
        op1: MessageOp,
        _op2: MessageOp,
        combined_result: i32,
        sequential_result: i32,
    ) -> Option<Conflict> {
        if !self.invariant_checks_enabled {
            return None;
        }

        if combined_result != sequential_result {
            self.sequence += 1;
            let conflict = Conflict {
                kind: ConflictKind::OrderingViolation,
                sequence: self.sequence,
                operation: op1,
                expected_result: sequential_result,
                actual_result: combined_result,
                violation_detail: "Operation ordering produces different results",
            };
            self.conflicts.push(conflict.clone());
            return Some(conflict);
        }

        None
    }

    /// Check consistency: repeated operations are idempotent where applicable
    pub fn check_idempotence(
        &mut self,
        op: MessageOp,
        _initial: i32,
        once: i32,
        twice: i32,
    ) -> Option<Conflict> {
        if !self.invariant_checks_enabled {
            return None;
        }

        let should_be_idempotent = matches!(op, MessageOp::Get | MessageOp::Reset);

        if should_be_idempotent && once != twice {
            self.sequence += 1;
            let conflict = Conflict {
                kind: ConflictKind::ConsistencyViolation,
                sequence: self.sequence,
                operation: op,
                expected_result: once,
                actual_result: twice,
                violation_detail: "Idempotent operation produced different results on repetition",
            };
            self.conflicts.push(conflict.clone());
            return Some(conflict);
        }

        None
    }

    /// Check sequence consistency
    pub fn check_sequence_consistency(
        &mut self,
        operations: &[MessageOp],
        expected_sequences: &[i32],
        actual_sequences: &[i32],
    ) -> Option<Conflict> {
        if !self.invariant_checks_enabled {
            return None;
        }

        for (i, (expected, actual)) in expected_sequences.iter().zip(actual_sequences.iter()).enumerate() {
            if expected != actual {
                self.sequence += 1;
                let conflict = Conflict {
                    kind: ConflictKind::SequenceViolation,
                    sequence: self.sequence,
                    operation: operations[i],
                    expected_result: *expected,
                    actual_result: *actual,
                    violation_detail: "Operation sequence produced inconsistent state",
                };
                self.conflicts.push(conflict.clone());
                return Some(conflict);
            }
        }

        None
    }

    /// Check timing: operations should complete within bounds
    pub fn check_timing_bound(
        &mut self,
        op: MessageOp,
        actual_time: u64,
        max_time: u64,
    ) -> Option<Conflict> {
        if !self.invariant_checks_enabled {
            return None;
        }

        if actual_time > max_time {
            self.sequence += 1;
            let conflict = Conflict {
                kind: ConflictKind::TimingViolation,
                sequence: self.sequence,
                operation: op,
                expected_result: max_time as i32,
                actual_result: actual_time as i32,
                violation_detail: "Operation exceeded timing bound",
            };
            self.conflicts.push(conflict.clone());
            return Some(conflict);
        }

        None
    }

    /// Get conflict count
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Get conflicts by kind
    pub fn conflicts_by_kind(&self, kind: ConflictKind) -> Vec<&Conflict> {
        self.conflicts.iter().filter(|c| c.kind == kind).collect()
    }

    /// Get most recent conflict
    pub fn latest_conflict(&self) -> Option<&Conflict> {
        self.conflicts.last()
    }

    /// Enable/disable checking
    pub fn set_checking_enabled(&mut self, enabled: bool) {
        self.invariant_checks_enabled = enabled;
    }

    /// Reset detector
    pub fn reset(&mut self) {
        self.conflicts.clear();
        self.sequence = 0;
        self.last_operation = None;
    }

    /// Get all conflicts with operation
    pub fn conflicts_with_operation(&self, op: MessageOp) -> Vec<&Conflict> {
        self.conflicts.iter().filter(|c| c.operation == op).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_invariant_inc() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_arithmetic_invariant(MessageOp::Inc, 5, 6);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_arithmetic_invariant_violation() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        assert!(conflict.is_some());
        assert_eq!(detector.conflict_count(), 1);
    }

    #[test]
    fn test_arithmetic_invariant_dec() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_arithmetic_invariant(MessageOp::Dec, 5, 4);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_arithmetic_invariant_get() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_arithmetic_invariant(MessageOp::Get, 5, 5);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_arithmetic_invariant_reset() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_arithmetic_invariant(MessageOp::Reset, 100, 0);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_ordering_violation() {
        let mut detector = ConflictDetector::new();
        // Inc then Dec should equal the original value
        let conflict = detector.check_ordering_invariant(
            MessageOp::Inc,
            MessageOp::Dec,
            6, // Combined result
            5, // Sequential result
        );
        assert!(conflict.is_some());
    }

    #[test]
    fn test_ordering_consistent() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_ordering_invariant(
            MessageOp::Inc,
            MessageOp::Dec,
            5,
            5,
        );
        assert!(conflict.is_none());
    }

    #[test]
    fn test_idempotence_get() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_idempotence(MessageOp::Get, 5, 5, 5);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_idempotence_reset() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_idempotence(MessageOp::Reset, 100, 0, 0);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_idempotence_violation() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_idempotence(MessageOp::Get, 5, 5, 6);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_sequence_consistency() {
        let mut detector = ConflictDetector::new();
        let ops = [MessageOp::Inc, MessageOp::Inc, MessageOp::Dec];
        let expected = [1, 2, 1];
        let actual = [1, 2, 1];

        let conflict = detector.check_sequence_consistency(&ops, &expected, &actual);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_sequence_violation() {
        let mut detector = ConflictDetector::new();
        let ops = [MessageOp::Inc, MessageOp::Inc, MessageOp::Dec];
        let expected = [1, 2, 1];
        let actual = [1, 2, 2];

        let conflict = detector.check_sequence_consistency(&ops, &expected, &actual);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_timing_violation() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_timing_bound(MessageOp::Inc, 200, 100);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_timing_within_bound() {
        let mut detector = ConflictDetector::new();
        let conflict = detector.check_timing_bound(MessageOp::Inc, 50, 100);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_conflicts_by_kind() {
        let mut detector = ConflictDetector::new();
        detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        detector.check_timing_bound(MessageOp::Dec, 200, 100);

        let arithmetic_conflicts = detector.conflicts_by_kind(ConflictKind::ArithmeticInvariant);
        assert_eq!(arithmetic_conflicts.len(), 1);

        let timing_conflicts = detector.conflicts_by_kind(ConflictKind::TimingViolation);
        assert_eq!(timing_conflicts.len(), 1);
    }

    #[test]
    fn test_detection_can_be_disabled() {
        let mut detector = ConflictDetector::new();
        detector.set_checking_enabled(false);

        let conflict = detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        assert!(conflict.is_none());
        assert_eq!(detector.conflict_count(), 0);
    }

    #[test]
    fn test_reset() {
        let mut detector = ConflictDetector::new();
        detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        assert_eq!(detector.conflict_count(), 1);

        detector.reset();
        assert_eq!(detector.conflict_count(), 0);
    }

    #[test]
    fn test_latest_conflict() {
        let mut detector = ConflictDetector::new();
        detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        detector.check_timing_bound(MessageOp::Dec, 200, 100);

        let latest = detector.latest_conflict();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().kind, ConflictKind::TimingViolation);
    }

    #[test]
    fn test_conflicts_with_operation() {
        let mut detector = ConflictDetector::new();
        detector.check_arithmetic_invariant(MessageOp::Inc, 5, 10);
        detector.check_arithmetic_invariant(MessageOp::Inc, 5, 11);
        detector.check_arithmetic_invariant(MessageOp::Dec, 5, 6);

        let inc_conflicts = detector.conflicts_with_operation(MessageOp::Inc);
        assert_eq!(inc_conflicts.len(), 2);

        let dec_conflicts = detector.conflicts_with_operation(MessageOp::Dec);
        assert_eq!(dec_conflicts.len(), 1);
    }
}
