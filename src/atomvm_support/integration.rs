// Complete integrated system tying together all components
// Demonstrates working end-to-end counter and port system

use crate::atomvm_support::{
    safe_api::SafeCounter,
    message_dispatch::{MessageDispatcher, MessageOp, MessageResponse},
    port_state_machine::PortStateMachine,
    port_drivers::CounterPortData,
    event_system::{EventSystem, EventSource, EventType},
};

/// Complete integrated counter port system
pub struct CounterPortSystem {
    counter: SafeCounter,
    state_machine: PortStateMachine,
    event_system: EventSystem,
}

impl CounterPortSystem {
    pub fn new(initial_value: i32) -> Self {
        CounterPortSystem {
            counter: SafeCounter::new(initial_value),
            state_machine: PortStateMachine::new(),
            event_system: EventSystem::new(),
        }
    }

    /// Process counter operation through the full system
    pub fn process_operation(&mut self, op: MessageOp) -> Result<i32, &'static str> {
        // Record event
        self.event_system.publish_counter_op(op, EventSource::CounterPort);

        // Create port data for state machine
        let mut port_data = CounterPortData::new();
        port_data.counter = self.counter.get();

        // Process through state machine
        let result = self.state_machine.handle_operation(&mut port_data, op)?;

        // Update counter
        match op {
            MessageOp::Inc => {
                self.counter.increment();
            }
            MessageOp::Dec => {
                self.counter.decrement();
            }
            MessageOp::Reset => {
                self.counter.reset();
            }
            MessageOp::Get => {}
            MessageOp::Unknown => {}
        }

        Ok(result)
    }

    /// Get current counter value
    pub fn get_value(&self) -> i32 {
        self.counter.get()
    }

    /// Get operation history
    pub fn operation_count(&self) -> usize {
        self.state_machine.operation_count()
    }

    /// Get events
    pub fn event_count(&self) -> usize {
        self.event_system.event_count()
    }

    /// Execute a sequence of operations
    pub fn execute_sequence(&mut self, ops: &[MessageOp]) -> (i32, usize) {
        let mut success_count = 0;

        for &op in ops {
            if self.process_operation(op).is_ok() {
                success_count += 1;
            }
        }

        (self.counter.get(), success_count)
    }

    /// Get state machine state
    pub fn state_machine_state(&self) -> &'static str {
        match self.state_machine.current_state() {
            crate::atomvm_support::port_state_machine::PortState::Idle => "idle",
            crate::atomvm_support::port_state_machine::PortState::Processing => "processing",
            crate::atomvm_support::port_state_machine::PortState::Error => "error",
        }
    }

    /// Reset system
    pub fn reset(&mut self) {
        self.counter.reset();
        self.state_machine.reset();
        self.event_system.clear_events();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_creation() {
        let sys = CounterPortSystem::new(0);
        assert_eq!(sys.get_value(), 0);
        assert_eq!(sys.operation_count(), 0);
        assert_eq!(sys.event_count(), 0);
    }

    #[test]
    fn test_system_single_operation() {
        let mut sys = CounterPortSystem::new(0);

        let result = sys.process_operation(MessageOp::Inc);
        assert!(result.is_ok());
        assert_eq!(sys.get_value(), 1);
        assert_eq!(sys.operation_count(), 1);
        assert_eq!(sys.event_count(), 1);
    }

    #[test]
    fn test_system_multiple_operations() {
        let mut sys = CounterPortSystem::new(5);

        sys.process_operation(MessageOp::Inc).unwrap();
        sys.process_operation(MessageOp::Inc).unwrap();
        sys.process_operation(MessageOp::Dec).unwrap();

        // 5 + 1 + 1 - 1 = 6
        assert_eq!(sys.get_value(), 6);
        assert_eq!(sys.operation_count(), 3);
        assert_eq!(sys.event_count(), 3);
    }

    #[test]
    fn test_system_sequence() {
        let mut sys = CounterPortSystem::new(10);

        let ops = [
            MessageOp::Inc,
            MessageOp::Inc,
            MessageOp::Dec,
            MessageOp::Get,
        ];

        let (final_value, success_count) = sys.execute_sequence(&ops);

        assert_eq!(final_value, 11);
        assert_eq!(success_count, 4);
    }

    #[test]
    fn test_system_reset() {
        let mut sys = CounterPortSystem::new(0);

        sys.process_operation(MessageOp::Inc).unwrap();
        sys.process_operation(MessageOp::Inc).unwrap();
        assert_eq!(sys.get_value(), 2);

        sys.reset();
        assert_eq!(sys.get_value(), 0);
        assert_eq!(sys.operation_count(), 0);
        assert_eq!(sys.event_count(), 0);
    }

    #[test]
    fn test_system_state_transitions() {
        let mut sys = CounterPortSystem::new(0);

        assert_eq!(sys.state_machine_state(), "idle");

        sys.process_operation(MessageOp::Inc).unwrap();
        assert_eq!(sys.state_machine_state(), "idle");
    }

    #[test]
    fn test_system_boundary_values() {
        let mut sys = CounterPortSystem::new(i32::MAX - 1);

        sys.process_operation(MessageOp::Inc).unwrap();
        assert_eq!(sys.get_value(), i32::MAX);

        // Should saturate
        sys.process_operation(MessageOp::Inc).unwrap();
        assert_eq!(sys.get_value(), i32::MAX);
    }

    #[test]
    fn test_system_error_handling() {
        let mut sys = CounterPortSystem::new(0);

        let result = sys.process_operation(MessageOp::Unknown);
        assert!(result.is_err());
        assert_eq!(sys.state_machine_state(), "error");
    }

    #[test]
    fn test_system_get_operation() {
        let mut sys = CounterPortSystem::new(42);

        let result = sys.process_operation(MessageOp::Get);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_system_complex_workflow() {
        let mut sys = CounterPortSystem::new(0);

        // Simulate real usage pattern
        let ops = [
            MessageOp::Inc,
            MessageOp::Inc,
            MessageOp::Get,
            MessageOp::Dec,
            MessageOp::Inc,
            MessageOp::Inc,
            MessageOp::Get,
            MessageOp::Reset,
            MessageOp::Get,
        ];

        let (final_value, success_count) = sys.execute_sequence(&ops);

        assert_eq!(final_value, 0);
        assert_eq!(success_count, 9);
        assert_eq!(sys.operation_count(), 9);
    }
}
