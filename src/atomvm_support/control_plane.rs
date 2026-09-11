//! v26.9.11 portable AtomVM control-plane admission.
//!
//! Composition remains deliberately narrow:
//! runtime readiness and circuit policy may admit an execution attempt, but
//! neither performs execution. `AdmissionToken` is evidence that these two
//! policy gates passed at a caller-supplied tick; it is not process authority.

use super::circuit_breaker::{CircuitBreaker, CircuitRefusal, CircuitState};
use super::runtime_lifecycle::{RuntimeLifecycle, RuntimeState};

/// Chatman ecosystem control-plane release identifier.
pub const ATOMVM_CONTROL_PLANE_VERSION: &str = "26.9.11";

/// Explicit refusal at the composed admission boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPlaneRefusal {
    RuntimeNotReady(RuntimeState),
    Circuit(CircuitRefusal),
}

/// Pure admission evidence from the runtime + circuit policy gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionToken {
    pub admitted_tick: u64,
    pub runtime_state: RuntimeState,
    pub circuit_state: CircuitState,
}

/// Compose existing reusable policy rather than inventing a second lifecycle or
/// circuit implementation.
pub fn admit_execution(
    runtime: &RuntimeLifecycle,
    circuit: &mut CircuitBreaker,
    now_tick: u64,
) -> Result<AdmissionToken, ControlPlaneRefusal> {
    if !runtime.is_ready() {
        return Err(ControlPlaneRefusal::RuntimeNotReady(runtime.state()));
    }

    circuit
        .admit(now_tick)
        .map_err(ControlPlaneRefusal::Circuit)?;

    Ok(AdmissionToken {
        admitted_tick: now_tick,
        runtime_state: runtime.state(),
        circuit_state: circuit.state(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::circuit_breaker::CircuitConfig;

    fn ready_runtime() -> RuntimeLifecycle {
        let mut runtime = RuntimeLifecycle::new();
        runtime.begin_load().unwrap();
        runtime.mark_ready().unwrap();
        runtime
    }

    #[test]
    fn runtime_readiness_is_required_before_circuit_mutation() {
        let runtime = RuntimeLifecycle::new();
        let mut circuit = CircuitBreaker::new(CircuitConfig::new(1, 10).unwrap());
        assert_eq!(
            admit_execution(&runtime, &mut circuit, 1),
            Err(ControlPlaneRefusal::RuntimeNotReady(
                RuntimeState::Uninitialized
            ))
        );
        assert_eq!(circuit.state(), CircuitState::Closed);
    }

    #[test]
    fn open_circuit_refuses_until_reset_window_then_admits_one_probe() {
        let runtime = ready_runtime();
        let mut circuit = CircuitBreaker::new(CircuitConfig::new(1, 10).unwrap());
        circuit.record_failure(5);
        assert_eq!(circuit.state(), CircuitState::Open);

        assert_eq!(
            admit_execution(&runtime, &mut circuit, 14),
            Err(ControlPlaneRefusal::Circuit(CircuitRefusal::Open))
        );

        let token = admit_execution(&runtime, &mut circuit, 15).unwrap();
        assert_eq!(token.runtime_state, RuntimeState::Ready);
        assert_eq!(token.circuit_state, CircuitState::HalfOpen);
        assert_eq!(
            admit_execution(&runtime, &mut circuit, 15),
            Err(ControlPlaneRefusal::Circuit(
                CircuitRefusal::HalfOpenProbeInFlight
            ))
        );
    }

    #[test]
    fn admission_token_never_advances_runtime_to_executing() {
        let runtime = ready_runtime();
        let mut circuit = CircuitBreaker::new(CircuitConfig::new(3, 10).unwrap());
        let token = admit_execution(&runtime, &mut circuit, 4).unwrap();
        assert_eq!(token.runtime_state, RuntimeState::Ready);
        assert_eq!(runtime.state(), RuntimeState::Ready);
    }
}
