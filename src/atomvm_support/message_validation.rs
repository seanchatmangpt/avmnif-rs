//! Allocation-free structural validation for generic AtomVM messages.
//!
//! This is the portable subset of `unrdf/packages/atomvm/message-validator.mjs`:
//! RPC calls/results and node-health messages. RDF, SPARQL, tracing, and serialization stay
//! outside avmnif-rs so this crate does not become a second domain-semantic authority.

/// Structural validation refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageValidationError {
    EmptyTarget,
    EmptyModule,
    EmptyFunction,
    MissingRpcError,
    EmptyNodeId,
    ZeroTimestamp,
    ErrorRateOutOfRange,
}

/// Borrowed RPC call envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcCall<'a> {
    pub target: &'a str,
    pub module: &'a str,
    pub function: &'a str,
    /// Argument count only. Argument terms remain owned by the transport/Term layer.
    pub arity: usize,
}

impl<'a> RpcCall<'a> {
    pub const fn validate(self) -> Result<Self, MessageValidationError> {
        if self.target.is_empty() {
            return Err(MessageValidationError::EmptyTarget);
        }
        if self.module.is_empty() {
            return Err(MessageValidationError::EmptyModule);
        }
        if self.function.is_empty() {
            return Err(MessageValidationError::EmptyFunction);
        }
        Ok(self)
    }
}

/// Borrowed RPC result envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcResult<'a> {
    pub ok: bool,
    pub error: Option<&'a str>,
}

impl<'a> RpcResult<'a> {
    pub const fn validate(self) -> Result<Self, MessageValidationError> {
        if !self.ok {
            match self.error {
                Some(error) if !error.is_empty() => {}
                _ => return Err(MessageValidationError::MissingRpcError),
            }
        }
        Ok(self)
    }
}

/// Health status values from the unrdf AtomVM wire contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Optional node health metrics.
///
/// `error_rate_ppm` encodes the source contract's [0,1] error rate without introducing
/// floating-point or NaN semantics into the embedded boundary. 1_000_000 == 100%.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HealthMetrics {
    pub latency_ticks: Option<u64>,
    pub error_rate_ppm: Option<u32>,
    pub queue_depth: Option<u32>,
}

impl HealthMetrics {
    pub const fn validate(self) -> Result<Self, MessageValidationError> {
        if let Some(error_rate_ppm) = self.error_rate_ppm {
            if error_rate_ppm > 1_000_000 {
                return Err(MessageValidationError::ErrorRateOutOfRange);
            }
        }
        Ok(self)
    }
}

/// Borrowed health-check envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthCheck<'a> {
    pub node_id: &'a str,
    /// Caller-defined positive timestamp/tick. Zero is refused, matching the source schema.
    pub timestamp: u64,
    pub status: HealthStatus,
    pub metrics: Option<HealthMetrics>,
}

impl<'a> HealthCheck<'a> {
    pub const fn validate(self) -> Result<Self, MessageValidationError> {
        if self.node_id.is_empty() {
            return Err(MessageValidationError::EmptyNodeId);
        }
        if self.timestamp == 0 {
            return Err(MessageValidationError::ZeroTimestamp);
        }
        if let Some(metrics) = self.metrics {
            match metrics.validate() {
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_call_requires_all_routing_names() {
        assert_eq!(
            RpcCall { target: "", module: "m", function: "f", arity: 0 }.validate(),
            Err(MessageValidationError::EmptyTarget)
        );
        assert_eq!(
            RpcCall { target: "n", module: "", function: "f", arity: 0 }.validate(),
            Err(MessageValidationError::EmptyModule)
        );
        assert_eq!(
            RpcCall { target: "n", module: "m", function: "", arity: 0 }.validate(),
            Err(MessageValidationError::EmptyFunction)
        );
        assert!(RpcCall { target: "n", module: "m", function: "f", arity: 2 }
            .validate()
            .is_ok());
    }

    #[test]
    fn rpc_failure_requires_error_receipt() {
        assert!(RpcResult { ok: true, error: None }.validate().is_ok());
        assert_eq!(
            RpcResult { ok: false, error: None }.validate(),
            Err(MessageValidationError::MissingRpcError)
        );
        assert_eq!(
            RpcResult { ok: false, error: Some("") }.validate(),
            Err(MessageValidationError::MissingRpcError)
        );
        assert!(RpcResult { ok: false, error: Some("timeout") }.validate().is_ok());
    }

    #[test]
    fn health_check_preserves_source_constraints_without_float() {
        let valid = HealthCheck {
            node_id: "node-1",
            timestamp: 1,
            status: HealthStatus::Healthy,
            metrics: Some(HealthMetrics {
                latency_ticks: Some(4),
                error_rate_ppm: Some(250_000),
                queue_depth: Some(3),
            }),
        };
        assert!(valid.validate().is_ok());

        assert_eq!(
            HealthCheck { node_id: "", ..valid }.validate(),
            Err(MessageValidationError::EmptyNodeId)
        );
        assert_eq!(
            HealthCheck { timestamp: 0, ..valid }.validate(),
            Err(MessageValidationError::ZeroTimestamp)
        );
        assert_eq!(
            HealthCheck {
                metrics: Some(HealthMetrics {
                    error_rate_ppm: Some(1_000_001),
                    ..HealthMetrics::default()
                }),
                ..valid
            }
            .validate(),
            Err(MessageValidationError::ErrorRateOutOfRange)
        );
    }
}
