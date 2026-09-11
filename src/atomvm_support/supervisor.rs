//! Pure OTP supervisor restart planning extracted from `unrdf/packages/atomvm`.
//!
//! The source implementation both selects affected children and restarts them. In avmnif-rs
//! those concerns are separated: this module only computes an admitted restart decision.
//! Process stop/start remains an explicit caller-controlled actuation boundary.

use alloc::vec::Vec;
use core::ops::Range;

/// OTP restart strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

/// OTP child restart policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartType {
    Permanent,
    Transient,
    Temporary,
}

/// Why a child exited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitKind {
    Normal,
    Abnormal,
}

/// Child metadata needed for restart selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChildSpec<'a> {
    pub id: &'a str,
    pub restart: RestartType,
}

/// A pure selection of child indices to restart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartPlan {
    pub failed_index: usize,
    pub affected: Range<usize>,
    pub manual: bool,
}

/// Result of observing one child termination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartDecision {
    NoRestart,
    Restart(RestartPlan),
    Shutdown,
}

/// Typed supervisor refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisorRefusal {
    UnknownChild,
    RestartIntensityExceeded,
    InvalidWindow,
}

/// Pure supervisor policy and restart-intensity accounting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisorPlanner {
    strategy: RestartStrategy,
    max_restarts: usize,
    window_ticks: u64,
    restart_ticks: Vec<u64>,
    blocked: bool,
}

impl SupervisorPlanner {
    pub fn new(
        strategy: RestartStrategy,
        max_restarts: usize,
        window_ticks: u64,
    ) -> Result<Self, SupervisorRefusal> {
        if window_ticks == 0 {
            return Err(SupervisorRefusal::InvalidWindow);
        }
        Ok(Self {
            strategy,
            max_restarts,
            window_ticks,
            restart_ticks: Vec::new(),
            blocked: false,
        })
    }

    pub const fn strategy(&self) -> RestartStrategy {
        self.strategy
    }

    pub const fn is_blocked(&self) -> bool {
        self.blocked
    }

    /// Observe a child exit and compute, but do not execute, the restart action.
    pub fn plan_failure(
        &mut self,
        children: &[ChildSpec<'_>],
        failed_index: usize,
        exit: ExitKind,
        now_tick: u64,
    ) -> Result<RestartDecision, SupervisorRefusal> {
        let child = children
            .get(failed_index)
            .ok_or(SupervisorRefusal::UnknownChild)?;

        if !should_restart(child.restart, exit) {
            return Ok(RestartDecision::NoRestart);
        }

        self.admit_restart(now_tick)?;
        Ok(RestartDecision::Restart(RestartPlan {
            failed_index,
            affected: affected_range(self.strategy, children.len(), failed_index),
            manual: false,
        }))
    }

    /// Manual restart mirrors unrdf: it bypasses restart-type and restart-intensity admission.
    pub fn plan_manual_restart(
        &self,
        child_count: usize,
        failed_index: usize,
    ) -> Result<RestartPlan, SupervisorRefusal> {
        if failed_index >= child_count {
            return Err(SupervisorRefusal::UnknownChild);
        }
        Ok(RestartPlan {
            failed_index,
            affected: affected_range(self.strategy, child_count, failed_index),
            manual: true,
        })
    }

    fn admit_restart(&mut self, now_tick: u64) -> Result<(), SupervisorRefusal> {
        if self.blocked {
            return Err(SupervisorRefusal::RestartIntensityExceeded);
        }

        let window = self.window_ticks;
        self.restart_ticks
            .retain(|tick| now_tick.saturating_sub(*tick) <= window);
        self.restart_ticks.push(now_tick);

        // Matches unrdf: maxRestarts=N admits N restarts and blocks the N+1th.
        if self.restart_ticks.len() > self.max_restarts {
            self.blocked = true;
            return Err(SupervisorRefusal::RestartIntensityExceeded);
        }
        Ok(())
    }
}

const fn should_restart(restart: RestartType, exit: ExitKind) -> bool {
    match restart {
        RestartType::Permanent => true,
        RestartType::Temporary => false,
        RestartType::Transient => matches!(exit, ExitKind::Abnormal),
    }
}

const fn affected_range(
    strategy: RestartStrategy,
    child_count: usize,
    failed_index: usize,
) -> Range<usize> {
    match strategy {
        RestartStrategy::OneForOne => failed_index..failed_index + 1,
        RestartStrategy::OneForAll => 0..child_count,
        RestartStrategy::RestForOne => failed_index..child_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHILDREN: [ChildSpec<'static>; 3] = [
        ChildSpec { id: "a", restart: RestartType::Permanent },
        ChildSpec { id: "b", restart: RestartType::Permanent },
        ChildSpec { id: "c", restart: RestartType::Permanent },
    ];

    #[test]
    fn strategies_match_otp_restart_sets() {
        let mut one = SupervisorPlanner::new(RestartStrategy::OneForOne, 3, 100).unwrap();
        let mut all = SupervisorPlanner::new(RestartStrategy::OneForAll, 3, 100).unwrap();
        let mut rest = SupervisorPlanner::new(RestartStrategy::RestForOne, 3, 100).unwrap();

        let range = match one.plan_failure(&CHILDREN, 1, ExitKind::Abnormal, 1).unwrap() {
            RestartDecision::Restart(plan) => plan.affected,
            other => panic!("unexpected decision: {:?}", other),
        };
        assert_eq!(range, 1..2);

        let range = match all.plan_failure(&CHILDREN, 1, ExitKind::Abnormal, 1).unwrap() {
            RestartDecision::Restart(plan) => plan.affected,
            other => panic!("unexpected decision: {:?}", other),
        };
        assert_eq!(range, 0..3);

        let range = match rest.plan_failure(&CHILDREN, 1, ExitKind::Abnormal, 1).unwrap() {
            RestartDecision::Restart(plan) => plan.affected,
            other => panic!("unexpected decision: {:?}", other),
        };
        assert_eq!(range, 1..3);
    }

    #[test]
    fn restart_types_preserve_otp_semantics() {
        let children = [
            ChildSpec { id: "permanent", restart: RestartType::Permanent },
            ChildSpec { id: "transient", restart: RestartType::Transient },
            ChildSpec { id: "temporary", restart: RestartType::Temporary },
        ];
        let mut planner = SupervisorPlanner::new(RestartStrategy::OneForOne, 10, 100).unwrap();
        assert!(matches!(
            planner.plan_failure(&children, 0, ExitKind::Normal, 1).unwrap(),
            RestartDecision::Restart(_)
        ));
        assert_eq!(
            planner.plan_failure(&children, 1, ExitKind::Normal, 2).unwrap(),
            RestartDecision::NoRestart
        );
        assert!(matches!(
            planner.plan_failure(&children, 1, ExitKind::Abnormal, 3).unwrap(),
            RestartDecision::Restart(_)
        ));
        assert_eq!(
            planner.plan_failure(&children, 2, ExitKind::Abnormal, 4).unwrap(),
            RestartDecision::NoRestart
        );
    }

    #[test]
    fn restart_intensity_blocks_n_plus_one() {
        let mut planner = SupervisorPlanner::new(RestartStrategy::OneForOne, 2, 100).unwrap();
        assert!(planner.plan_failure(&CHILDREN, 0, ExitKind::Abnormal, 1).is_ok());
        assert!(planner.plan_failure(&CHILDREN, 0, ExitKind::Abnormal, 2).is_ok());
        assert_eq!(
            planner.plan_failure(&CHILDREN, 0, ExitKind::Abnormal, 3),
            Err(SupervisorRefusal::RestartIntensityExceeded)
        );
        assert!(planner.is_blocked());
    }

    #[test]
    fn manual_restart_is_selection_only_and_bypasses_intensity() {
        let mut planner = SupervisorPlanner::new(RestartStrategy::RestForOne, 0, 100).unwrap();
        assert_eq!(
            planner.plan_failure(&CHILDREN, 0, ExitKind::Abnormal, 1),
            Err(SupervisorRefusal::RestartIntensityExceeded)
        );
        let manual = planner.plan_manual_restart(CHILDREN.len(), 1).unwrap();
        assert_eq!(manual.affected, 1..3);
        assert!(manual.manual);
    }
}
