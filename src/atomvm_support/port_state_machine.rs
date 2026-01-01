// Working port state machine for managing stateful port communication
// Handles transitions between idle, processing, and error states

use crate::atomvm_support::port_drivers::CounterPortData;
use crate::atomvm_support::message_dispatch::{MessageDispatcher, MessageOp};

/// Port communication states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Idle,
    Processing,
    Error,
}

/// State machine for port operations
pub struct PortStateMachine {
    state: PortState,
    dispatcher: MessageDispatcher,
    last_error: Option<&'static str>,
}

impl PortStateMachine {
    pub fn new() -> Self {
        PortStateMachine {
            state: PortState::Idle,
            dispatcher: MessageDispatcher::new(),
            last_error: None,
        }
    }

    /// Current state
    pub fn current_state(&self) -> PortState {
        self.state
    }

    /// Transition to processing state and handle operation
    pub fn handle_operation(
        &mut self,
        port_data: &mut CounterPortData,
        op: MessageOp,
    ) -> Result<i32, &'static str> {
        // Can only process from idle state
        if self.state != PortState::Idle {
            self.state = PortState::Error;
            self.last_error = Some("Invalid state transition");
            return Err("Invalid state transition");
        }

        self.state = PortState::Processing;

        // Dispatch the operation
        let (new_value, success) = self.dispatcher.dispatch_counter(op, port_data.counter);

        if !success {
            self.state = PortState::Error;
            self.last_error = Some("Unknown operation");
            return Err("Unknown operation");
        }

        // Update port data with dispatched result
        port_data.counter = new_value;

        // Return to idle state
        self.state = PortState::Idle;
        Ok(new_value)
    }

    /// Get the last error
    pub fn last_error(&self) -> Option<&'static str> {
        self.last_error
    }

    /// Get operation history
    pub fn operation_count(&self) -> usize {
        self.dispatcher.operation_count()
    }

    /// Clear error state
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Reset machine to initial state
    pub fn reset(&mut self) {
        self.state = PortState::Idle;
        self.dispatcher.clear_log();
        self.last_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let machine = PortStateMachine::new();
        assert_eq!(machine.current_state(), PortState::Idle);
        assert_eq!(machine.operation_count(), 0);
    }

    #[test]
    fn test_state_machine_increment() {
        let mut machine = PortStateMachine::new();
        let mut port_data = CounterPortData::new();

        let result = machine.handle_operation(&mut port_data, MessageOp::Inc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(machine.current_state(), PortState::Idle);
    }

    #[test]
    fn test_state_machine_sequence() {
        let mut machine = PortStateMachine::new();
        let mut port_data = CounterPortData::new();

        machine.handle_operation(&mut port_data, MessageOp::Inc).unwrap();
        machine.handle_operation(&mut port_data, MessageOp::Inc).unwrap();
        machine.handle_operation(&mut port_data, MessageOp::Dec).unwrap();

        assert_eq!(port_data.counter, 1);
        assert_eq!(machine.operation_count(), 3);
    }

    #[test]
    fn test_state_machine_get_operation() {
        let mut machine = PortStateMachine::new();
        let mut port_data = CounterPortData::new();
        port_data.counter = 42;

        let result = machine.handle_operation(&mut port_data, MessageOp::Get);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_state_machine_reset_operation() {
        let mut machine = PortStateMachine::new();
        let mut port_data = CounterPortData::new();
        port_data.counter = 100;

        machine.handle_operation(&mut port_data, MessageOp::Reset).unwrap();
        assert_eq!(port_data.counter, 0);
    }

    #[test]
    fn test_state_machine_error_handling() {
        let mut machine = PortStateMachine::new();
        let mut port_data = CounterPortData::new();

        let result = machine.handle_operation(&mut port_data, MessageOp::Unknown);
        assert!(result.is_err());
        assert_eq!(machine.current_state(), PortState::Error);
        assert!(machine.last_error().is_some());
    }

    #[test]
    fn test_state_machine_reset() {
        let mut machine = PortStateMachine::new();
        machine.state = PortState::Error;
        machine.last_error = Some("test error");

        machine.reset();
        assert_eq!(machine.current_state(), PortState::Idle);
        assert!(machine.last_error().is_none());
        assert_eq!(machine.operation_count(), 0);
    }
}
