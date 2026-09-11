//! `no_std` circuit-breaker state machine ported from `unrdf/packages/atomvm`.
//!
//! Wall-clock access and function execution are intentionally external. Callers provide a
//! monotonic tick, so this module remains deterministic and reusable on embedded AtomVM targets.

/// Circuit state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Invalid circuit configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitConfigError {
    ZeroFailureThreshold,
    ZeroResetAfter,
}

/// Why a call was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitRefusal {
    Open,
    HalfOpenProbeInFlight,
}

/// Configuration expressed in caller-defined monotonic ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitConfig {
    pub failure_threshold: u32,
    pub reset_after_ticks: u64,
}

impl CircuitConfig {
    pub const fn new(
        failure_threshold: u32,
        reset_after_ticks: u64,
    ) -> Result<Self, CircuitConfigError> {
        if failure_threshold == 0 {
            return Err(CircuitConfigError::ZeroFailureThreshold);
        }
        if reset_after_ticks == 0 {
            return Err(CircuitConfigError::ZeroResetAfter);
        }
        Ok(Self {
            failure_threshold,
            reset_after_ticks,
        })
    }
}

/// Deterministic circuit-breaker controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitBreaker {
    config: CircuitConfig,
    state: CircuitState,
    failure_count: u32,
    last_failure_tick: u64,
    half_open_probe_in_flight: bool,
}

impl CircuitBreaker {
    pub const fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_tick: 0,
            half_open_probe_in_flight: false,
        }
    }

    pub const fn state(&self) -> CircuitState {
        self.state
    }

    pub const fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Decide whether one external operation may execute.
    ///
    /// Opening/closing the circuit never executes the operation itself; authority stays with
    /// the caller. When the reset interval expires, exactly one half-open probe is admitted.
    pub fn admit(&mut self, now_tick: u64) -> Result<(), CircuitRefusal> {
        match self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                if now_tick.saturating_sub(self.last_failure_tick)
                    < self.config.reset_after_ticks
                {
                    return Err(CircuitRefusal::Open);
                }
                self.state = CircuitState::HalfOpen;
                self.half_open_probe_in_flight = true;
                Ok(())
            }
            CircuitState::HalfOpen => {
                if self.half_open_probe_in_flight {
                    Err(CircuitRefusal::HalfOpenProbeInFlight)
                } else {
                    self.half_open_probe_in_flight = true;
                    Ok(())
                }
            }
        }
    }

    /// Record success after an admitted external operation.
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.half_open_probe_in_flight = false;
        self.state = CircuitState::Closed;
    }

    /// Record failure after an admitted external operation.
    pub fn record_failure(&mut self, now_tick: u64) {
        self.failure_count = self.failure_count.saturating_add(1);
        self.last_failure_tick = now_tick;
        self.half_open_probe_in_flight = false;

        if self.state == CircuitState::HalfOpen
            || self.failure_count >= self.config.failure_threshold
        {
            self.state = CircuitState::Open;
        }
    }

    /// Explicit administrative close. This mutates only circuit state; it performs no I/O.
    pub fn close(&mut self) {
        self.failure_count = 0;
        self.half_open_probe_in_flight = false;
        self.state = CircuitState::Closed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn breaker() -> CircuitBreaker {
        CircuitBreaker::new(CircuitConfig::new(3, 10).unwrap())
    }

    #[test]
    fn validates_config() {
        assert_eq!(
            CircuitConfig::new(0, 1),
            Err(CircuitConfigError::ZeroFailureThreshold)
        );
        assert_eq!(
            CircuitConfig::new(1, 0),
            Err(CircuitConfigError::ZeroResetAfter)
        );
    }

    #[test]
    fn threshold_opens_and_timeout_admits_one_probe() {
        let mut cb = breaker();
        cb.record_failure(1);
        cb.record_failure(2);
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure(3);
        assert_eq!(cb.state(), CircuitState::Open);
        assert_eq!(cb.admit(12), Err(CircuitRefusal::Open));
        cb.admit(13).unwrap();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert_eq!(
            cb.admit(13),
            Err(CircuitRefusal::HalfOpenProbeInFlight)
        );
    }

    #[test]
    fn half_open_success_closes_and_failure_reopens() {
        let mut cb = breaker();
        for tick in 1..=3 {
            cb.record_failure(tick);
        }
        cb.admit(13).unwrap();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);

        for tick in 20..=22 {
            cb.record_failure(tick);
        }
        cb.admit(32).unwrap();
        cb.record_failure(32);
        assert_eq!(cb.state(), CircuitState::Open);
    }
}
