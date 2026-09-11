//! Pure AtomVM execution planning and observation classification.
//!
//! This is the portable policy slice of `unrdf/packages/atomvm/process-broker.mjs`.
//! Filesystem checks, process spawning, timeouts, stdout collection, hashing, and
//! termination remain environment-owned. This module can select and validate a
//! plan, classify supplied observations, and manufacture a receipt body; it
//! cannot execute AtomVM.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::evidence::{Digest32, Sealed};
use super::intent::{ConstructedIntent, IntentDraft};
use super::swarm::is_stable_id;

pub const ATOMVM_EXECUTE_OPERATION: &str = "atomvm.execute";

/// Opaque runtime references selected for an admitted target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTargetConfig {
    target_id: String,
    application_ref: String,
    library_refs: Vec<String>,
    expected_marker: String,
    timeout_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeConfigRefusal {
    InvalidTargetId,
    MissingApplicationRef,
    EmptyLibraryRef,
    MissingExpectedMarker,
    InvalidTimeout,
}

impl RuntimeTargetConfig {
    pub fn new(
        target_id: &str,
        application_ref: &str,
        library_refs: Vec<String>,
        expected_marker: &str,
        timeout_ticks: u64,
    ) -> Result<Self, RuntimeConfigRefusal> {
        if !is_stable_id(target_id) {
            return Err(RuntimeConfigRefusal::InvalidTargetId);
        }
        if application_ref.is_empty() {
            return Err(RuntimeConfigRefusal::MissingApplicationRef);
        }
        if library_refs.iter().any(|item| item.is_empty()) {
            return Err(RuntimeConfigRefusal::EmptyLibraryRef);
        }
        if expected_marker.is_empty() {
            return Err(RuntimeConfigRefusal::MissingExpectedMarker);
        }
        if timeout_ticks == 0 {
            return Err(RuntimeConfigRefusal::InvalidTimeout);
        }

        Ok(Self {
            target_id: target_id.to_string(),
            application_ref: application_ref.to_string(),
            library_refs,
            expected_marker: expected_marker.to_string(),
            timeout_ticks,
        })
    }

    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    pub fn application_ref(&self) -> &str {
        &self.application_ref
    }

    pub fn library_refs(&self) -> &[String] {
        &self.library_refs
    }

    pub fn expected_marker(&self) -> &str {
        &self.expected_marker
    }

    pub const fn timeout_ticks(&self) -> u64 {
        self.timeout_ticks
    }
}

/// Fully selected execution information. This is a plan, not proof of execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub intent_digest: Digest32,
    pub target_id: String,
    pub route: Vec<String>,
    pub application_ref: String,
    pub library_refs: Vec<String>,
    pub expected_marker: String,
    pub timeout_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionRefusal {
    IntentDrift,
    OperationNotAdmitted,
    RouteTargetMismatch,
    RuntimeTargetMismatch,
}

/// Stateless selector corresponding to the admission half of AtomVMProcessBroker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExecutionPlanner;

impl ExecutionPlanner {
    /// Verify sealed intent identity, enforce the single admitted operation, and
    /// select an opaque runtime configuration. No filesystem or process action occurs.
    pub fn plan<F>(
        intent: &ConstructedIntent,
        config: &RuntimeTargetConfig,
        digest_intent: F,
    ) -> Result<ExecutionPlan, ExecutionRefusal>
    where
        F: FnOnce(&IntentDraft) -> Digest32,
    {
        if !intent.verify_with(digest_intent) {
            return Err(ExecutionRefusal::IntentDrift);
        }

        let body = intent.body();
        if body.operation.as_str() != ATOMVM_EXECUTE_OPERATION {
            return Err(ExecutionRefusal::OperationNotAdmitted);
        }
        if body.route.last().map(String::as_str) != Some(body.target_id.as_str()) {
            return Err(ExecutionRefusal::RouteTargetMismatch);
        }
        if config.target_id != body.target_id {
            return Err(ExecutionRefusal::RuntimeTargetMismatch);
        }

        Ok(ExecutionPlan {
            intent_digest: intent.digest(),
            target_id: body.target_id.clone(),
            route: body.route.clone(),
            application_ref: config.application_ref.clone(),
            library_refs: config.library_refs.clone(),
            expected_marker: config.expected_marker.clone(),
            timeout_ticks: config.timeout_ticks,
        })
    }
}

/// Environment-observed process termination class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTermination {
    Exited(i32),
    TimedOut,
    SpawnRefused,
}

/// Observation supplied by an external process adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionObservation {
    pub termination: ExecutionTermination,
    pub marker_observed: bool,
    pub stdout_digest: Option<Digest32>,
    pub stderr_digest: Option<Digest32>,
    pub elapsed_ticks: u64,
}

impl ExecutionObservation {
    pub const fn exited(
        exit_code: i32,
        marker_observed: bool,
        stdout_digest: Digest32,
        stderr_digest: Digest32,
        elapsed_ticks: u64,
    ) -> Self {
        Self {
            termination: ExecutionTermination::Exited(exit_code),
            marker_observed,
            stdout_digest: Some(stdout_digest),
            stderr_digest: Some(stderr_digest),
            elapsed_ticks,
        }
    }

    pub const fn timed_out(elapsed_ticks: u64) -> Self {
        Self {
            termination: ExecutionTermination::TimedOut,
            marker_observed: false,
            stdout_digest: None,
            stderr_digest: None,
            elapsed_ticks,
        }
    }

    pub const fn spawn_refused(elapsed_ticks: u64) -> Self {
        Self {
            termination: ExecutionTermination::SpawnRefused,
            marker_observed: false,
            stdout_digest: None,
            stderr_digest: None,
            elapsed_ticks,
        }
    }

    pub const fn classify(&self) -> ExecutionVerdict {
        match self.termination {
            ExecutionTermination::Exited(0) if self.marker_observed => ExecutionVerdict {
                standing: ExecutionStanding::Alive,
                block_reason: None,
            },
            ExecutionTermination::Exited(0) => ExecutionVerdict {
                standing: ExecutionStanding::Blocked,
                block_reason: Some(ExecutionBlockReason::MarkerMissing),
            },
            ExecutionTermination::Exited(_) => ExecutionVerdict {
                standing: ExecutionStanding::Blocked,
                block_reason: Some(ExecutionBlockReason::NonZeroExit),
            },
            ExecutionTermination::TimedOut => ExecutionVerdict {
                standing: ExecutionStanding::Blocked,
                block_reason: Some(ExecutionBlockReason::TimedOut),
            },
            ExecutionTermination::SpawnRefused => ExecutionVerdict {
                standing: ExecutionStanding::Blocked,
                block_reason: Some(ExecutionBlockReason::SpawnRefused),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStanding {
    Alive,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBlockReason {
    NonZeroExit,
    MarkerMissing,
    TimedOut,
    SpawnRefused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionVerdict {
    pub standing: ExecutionStanding,
    pub block_reason: Option<ExecutionBlockReason>,
}

/// Receipt body manufactured from an execution plan and caller-supplied observation.
/// It records observed standing; it does not independently prove process execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptDraft {
    pub intent_digest: Digest32,
    pub target_id: String,
    pub route: Vec<String>,
    pub observation: ExecutionObservation,
    pub verdict: ExecutionVerdict,
    pub completed_tick: u64,
}

pub type ExecutionReceipt = Sealed<ExecutionReceiptDraft>;

impl ExecutionReceiptDraft {
    pub fn from_observation(
        plan: &ExecutionPlan,
        observation: ExecutionObservation,
        completed_tick: u64,
    ) -> Self {
        let verdict = observation.classify();
        Self {
            intent_digest: plan.intent_digest,
            target_id: plan.target_id.clone(),
            route: plan.route.clone(),
            observation,
            verdict,
            completed_tick,
        }
    }

    pub fn seal(self, receipt_digest: Digest32) -> ExecutionReceipt {
        Sealed::new(self, receipt_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::intent::IntentDraft;
    use alloc::vec;

    fn intent(operation: &str, digest: Digest32) -> ConstructedIntent {
        IntentDraft::new(
            "cluster",
            "source",
            "target",
            vec!["source".to_string(), "target".to_string()],
            operation,
            Digest32([2; 32]),
            1,
        )
        .unwrap()
        .seal(digest)
    }

    fn config() -> RuntimeTargetConfig {
        RuntimeTargetConfig::new(
            "target",
            "avm:app",
            vec!["avm:lib".to_string()],
            "atomvm_swarm_alive",
            100,
        )
        .unwrap()
    }

    #[test]
    fn planning_requires_verified_identity_and_admitted_operation() {
        let good_digest = Digest32([3; 32]);
        let sealed = intent(ATOMVM_EXECUTE_OPERATION, good_digest);
        assert_eq!(
            ExecutionPlanner::plan(&sealed, &config(), |_| Digest32([4; 32])),
            Err(ExecutionRefusal::IntentDrift)
        );

        let other = intent("atomvm.observe", good_digest);
        assert_eq!(
            ExecutionPlanner::plan(&other, &config(), |_| good_digest),
            Err(ExecutionRefusal::OperationNotAdmitted)
        );
    }

    #[test]
    fn plan_contains_opaque_refs_without_claiming_files_exist() {
        let digest = Digest32([5; 32]);
        let sealed = intent(ATOMVM_EXECUTE_OPERATION, digest);
        let plan = ExecutionPlanner::plan(&sealed, &config(), |_| digest).unwrap();
        assert_eq!(plan.application_ref, "avm:app");
        assert_eq!(plan.library_refs, vec!["avm:lib".to_string()]);
        assert_eq!(plan.expected_marker, "atomvm_swarm_alive");
    }

    #[test]
    fn observations_have_explicit_block_reasons() {
        let alive = ExecutionObservation::exited(
            0,
            true,
            Digest32([1; 32]),
            Digest32([2; 32]),
            8,
        );
        assert_eq!(alive.classify().standing, ExecutionStanding::Alive);

        let marker_missing = ExecutionObservation::exited(
            0,
            false,
            Digest32([1; 32]),
            Digest32([2; 32]),
            8,
        );
        assert_eq!(
            marker_missing.classify().block_reason,
            Some(ExecutionBlockReason::MarkerMissing)
        );
        assert_eq!(
            ExecutionObservation::timed_out(101).classify().block_reason,
            Some(ExecutionBlockReason::TimedOut)
        );
    }

    #[test]
    fn receipt_sealing_is_independent_from_observation_classification() {
        let digest = Digest32([5; 32]);
        let sealed = intent(ATOMVM_EXECUTE_OPERATION, digest);
        let plan = ExecutionPlanner::plan(&sealed, &config(), |_| digest).unwrap();
        let observation = ExecutionObservation::spawn_refused(1);
        let draft = ExecutionReceiptDraft::from_observation(&plan, observation, 2);
        assert_eq!(draft.verdict.standing, ExecutionStanding::Blocked);
        let receipt = draft.seal(Digest32([7; 32]));
        assert!(receipt.verify_with(|_| Digest32([7; 32])));
    }
}
