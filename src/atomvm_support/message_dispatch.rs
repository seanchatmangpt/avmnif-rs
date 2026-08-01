// Working message dispatch system for port communication
// Routes incoming messages to appropriate handlers based on operation type

use alloc::vec;
use alloc::vec::Vec;
use crate::{
    port::Message,
    term::{Term, TermValue, NifError},
    context::Context,
};

/// Message operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageOp {
    Inc,
    Dec,
    Get,
    Reset,
    Unknown,
}

impl MessageOp {
    /// Parse operation from atom
    pub fn from_atom_name(name: &str) -> Self {
        match name {
            "inc" => MessageOp::Inc,
            "dec" => MessageOp::Dec,
            "get" => MessageOp::Get,
            "reset" => MessageOp::Reset,
            _ => MessageOp::Unknown,
        }
    }
}

/// Message handler result with response
#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub value: i32,
    pub success: bool,
}

impl MessageResponse {
    pub fn ok(value: i32) -> Self {
        MessageResponse { value, success: true }
    }

    pub fn error() -> Self {
        MessageResponse { value: 0, success: false }
    }

    pub fn to_term(&self, ctx: *mut Context) -> Term {
        let tuple = if self.success {
            TermValue::tuple(vec![
                TermValue::Atom(crate::atom::AtomIndex(1)), // ok atom
                TermValue::int(self.value),
            ])
        } else {
            TermValue::tuple(vec![
                TermValue::Atom(crate::atom::AtomIndex(2)), // error atom
            ])
        };

        unsafe {
            let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
            Term::from_value(tuple, heap).unwrap_or(Term(0))
        }
    }
}

/// Message dispatcher for routing and handling
pub struct MessageDispatcher {
    operations: Vec<(MessageOp, i32)>, // Simple operation log
}

impl MessageDispatcher {
    pub fn new() -> Self {
        MessageDispatcher {
            operations: Vec::new(),
        }
    }

    /// Process a counter operation and return result
    pub fn dispatch_counter(&mut self, op: MessageOp, current_value: i32) -> (i32, bool) {
        let (new_value, success) = match op {
            MessageOp::Inc => {
                let v = current_value.saturating_add(1);
                (v, true)
            }
            MessageOp::Dec => {
                let v = current_value.saturating_sub(1);
                (v, true)
            }
            MessageOp::Get => (current_value, true),
            MessageOp::Reset => (0, true),
            MessageOp::Unknown => (current_value, false),
        };

        self.operations.push((op, new_value));
        (new_value, success)
    }

    /// Get operation count
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Clear operation log
    pub fn clear_log(&mut self) {
        self.operations.clear();
    }

    /// Get last operation
    pub fn last_operation(&self) -> Option<(MessageOp, i32)> {
        self.operations.last().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_op_parsing() {
        assert_eq!(MessageOp::from_atom_name("inc"), MessageOp::Inc);
        assert_eq!(MessageOp::from_atom_name("dec"), MessageOp::Dec);
        assert_eq!(MessageOp::from_atom_name("get"), MessageOp::Get);
        assert_eq!(MessageOp::from_atom_name("reset"), MessageOp::Reset);
        assert_eq!(MessageOp::from_atom_name("unknown"), MessageOp::Unknown);
    }

    #[test]
    fn test_message_response() {
        let ok = MessageResponse::ok(42);
        assert!(ok.success);
        assert_eq!(ok.value, 42);

        let err = MessageResponse::error();
        assert!(!err.success);
        assert_eq!(err.value, 0);
    }

    #[test]
    fn test_dispatcher_increment() {
        let mut dispatcher = MessageDispatcher::new();
        let (val, success) = dispatcher.dispatch_counter(MessageOp::Inc, 0);
        assert!(success);
        assert_eq!(val, 1);
        assert_eq!(dispatcher.operation_count(), 1);
    }

    #[test]
    fn test_dispatcher_sequence() {
        let mut dispatcher = MessageDispatcher::new();

        let (v1, _) = dispatcher.dispatch_counter(MessageOp::Inc, 0);
        assert_eq!(v1, 1);

        let (v2, _) = dispatcher.dispatch_counter(MessageOp::Inc, v1);
        assert_eq!(v2, 2);

        let (v3, _) = dispatcher.dispatch_counter(MessageOp::Dec, v2);
        assert_eq!(v3, 1);

        assert_eq!(dispatcher.operation_count(), 3);
    }

    #[test]
    fn test_dispatcher_saturating_arithmetic() {
        let mut dispatcher = MessageDispatcher::new();

        let (val, _) = dispatcher.dispatch_counter(MessageOp::Dec, i32::MIN);
        assert_eq!(val, i32::MIN); // Saturates at min

        let (val, _) = dispatcher.dispatch_counter(MessageOp::Inc, i32::MAX);
        assert_eq!(val, i32::MAX); // Saturates at max
    }
}
