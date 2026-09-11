//! Portable AtomVM runtime lifecycle extracted from `unrdf/packages/atomvm`.
//!
//! This module preserves the state-machine invariants from the JavaScript
//! runtime without importing browser, Emscripten, filesystem, or wall-clock
//! concerns into the `no_std` NIF layer.

/// AtomVM runtime lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime exists but loading has not started.
    Uninitialized,
    /// Runtime image/module is being loaded.
    Loading,
    /// Runtime is loaded and may begin execution.
    Ready,
    /// Runtime is executing one admitted workload.
    Executing,
    /// The previous lifecycle operation failed.
    Error,
    /// Terminal state. No further lifecycle transitions are admitted.
    Destroyed,
}

/// Lifecycle operations used in refusal evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOperation {
    BeginLoad,
    MarkReady,
    BeginExecution,
    CompleteExecution,
    Fail,
    Destroy,
}

/// A refused lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeTransitionError {
    pub operation: RuntimeOperation,
    pub from: RuntimeState,
}

/// Pure runtime lifecycle controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeLifecycle {
    state: RuntimeState,
}

impl Default for RuntimeLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeLifecycle {
    /// Create an uninitialized runtime lifecycle.
    pub const fn new() -> Self {
        Self {
            state: RuntimeState::Uninitialized,
        }
    }

    /// Current state.
    pub const fn state(&self) -> RuntimeState {
        self.state
    }

    /// True only when new execution may begin.
    pub const fn is_ready(&self) -> bool {
        matches!(self.state, RuntimeState::Ready)
    }

    /// True after load succeeds and while an admitted workload executes.
    pub const fn is_loaded(&self) -> bool {
        matches!(self.state, RuntimeState::Ready | RuntimeState::Executing)
    }

    /// Begin loading. A failed runtime may explicitly retry loading.
    pub fn begin_load(&mut self) -> Result<(), RuntimeTransitionError> {
        match self.state {
            RuntimeState::Uninitialized | RuntimeState::Error => {
                self.state = RuntimeState::Loading;
                Ok(())
            }
            from => Err(RuntimeTransitionError {
                operation: RuntimeOperation::BeginLoad,
                from,
            }),
        }
    }

    /// Admit a successfully loaded runtime into Ready.
    pub fn mark_ready(&mut self) -> Result<(), RuntimeTransitionError> {
        self.transition(
            RuntimeState::Loading,
            RuntimeState::Ready,
            RuntimeOperation::MarkReady,
        )
    }

    /// Begin one execution from Ready.
    pub fn begin_execution(&mut self) -> Result<(), RuntimeTransitionError> {
        self.transition(
            RuntimeState::Ready,
            RuntimeState::Executing,
            RuntimeOperation::BeginExecution,
        )
    }

    /// Return a successful execution to Ready.
    pub fn complete_execution(&mut self) -> Result<(), RuntimeTransitionError> {
        self.transition(
            RuntimeState::Executing,
            RuntimeState::Ready,
            RuntimeOperation::CompleteExecution,
        )
    }

    /// Record failure without manufacturing a recovery transition.
    pub fn fail(&mut self) -> Result<(), RuntimeTransitionError> {
        match self.state {
            RuntimeState::Destroyed => Err(RuntimeTransitionError {
                operation: RuntimeOperation::Fail,
                from: RuntimeState::Destroyed,
            }),
            _ => {
                self.state = RuntimeState::Error;
                Ok(())
            }
        }
    }

    /// Enter the terminal Destroyed state. Destroy is idempotent.
    pub fn destroy(&mut self) -> Result<(), RuntimeTransitionError> {
        self.state = RuntimeState::Destroyed;
        Ok(())
    }

    fn transition(
        &mut self,
        expected: RuntimeState,
        next: RuntimeState,
        operation: RuntimeOperation,
    ) -> Result<(), RuntimeTransitionError> {
        if self.state != expected {
            return Err(RuntimeTransitionError {
                operation,
                from: self.state,
            });
        }
        self.state = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_matches_unrdf_runtime_state_machine() {
        let mut runtime = RuntimeLifecycle::new();
        runtime.begin_load().unwrap();
        runtime.mark_ready().unwrap();
        runtime.begin_execution().unwrap();
        runtime.complete_execution().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Ready);
        assert!(runtime.is_ready());
        assert!(runtime.is_loaded());
    }

    #[test]
    fn execution_before_ready_is_refused() {
        let mut runtime = RuntimeLifecycle::new();
        assert_eq!(
            runtime.begin_execution(),
            Err(RuntimeTransitionError {
                operation: RuntimeOperation::BeginExecution,
                from: RuntimeState::Uninitialized,
            })
        );
    }

    #[test]
    fn failure_can_retry_load_but_destroy_is_terminal() {
        let mut runtime = RuntimeLifecycle::new();
        runtime.fail().unwrap();
        runtime.begin_load().unwrap();
        runtime.mark_ready().unwrap();
        runtime.destroy().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Destroyed);
        assert!(runtime.begin_load().is_err());
        assert!(runtime.begin_execution().is_err());
        assert!(runtime.fail().is_err());
    }
}
