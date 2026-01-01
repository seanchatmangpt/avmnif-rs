// Working event system for managing port and NIF events
// Provides an event queue and handler registration

use alloc::vec::Vec;
use crate::atomvm_support::message_dispatch::MessageOp;

/// Event types in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    CounterOperation(MessageOp),
    ResourceCreated,
    ResourceDestroyed,
    PortMessage,
    Error,
}

/// Event with metadata
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub timestamp: u64,
    pub source: EventSource,
}

/// Event source identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    CounterPort,
    EchoPort,
    BufferPort,
    MathNif,
    ResourceNif,
}

impl Event {
    pub fn new(event_type: EventType, source: EventSource) -> Self {
        Event {
            event_type,
            timestamp: 0, // Timestamp is assigned by EventSystem when published
            source,
        }
    }
}

/// Event handler callback type
pub type EventHandler = fn(&Event);

/// Event system managing queue and handlers
pub struct EventSystem {
    events: Vec<Event>,
    handlers: Vec<EventHandler>,
    max_events: usize,
    timestamp_counter: u64,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            events: Vec::new(),
            handlers: Vec::new(),
            max_events: 1000,
            timestamp_counter: 0,
        }
    }

    /// Publish an event
    pub fn publish(&mut self, mut event: Event) {
        self.timestamp_counter += 1;
        event.timestamp = self.timestamp_counter;

        self.events.push(event.clone());

        // Keep bounded queue
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }

        // Notify handlers
        for handler in self.handlers.iter() {
            handler(&event);
        }
    }

    /// Publish counter operation event
    pub fn publish_counter_op(&mut self, op: MessageOp, source: EventSource) {
        let event = Event::new(EventType::CounterOperation(op), source);
        self.publish(event);
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: EventHandler) {
        self.handlers.push(handler);
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get events by source
    pub fn events_by_source(&self, source: EventSource) -> Vec<Event> {
        self.events
            .iter()
            .filter(|e| e.source == source)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub fn events_by_type(&self, event_type: EventType) -> Vec<Event> {
        self.events
            .iter()
            .filter(|e| {
                core::mem::discriminant(&e.event_type) == core::mem::discriminant(&event_type)
            })
            .cloned()
            .collect()
    }

    /// Clear all events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Set maximum events in queue
    pub fn set_max_events(&mut self, max: usize) {
        self.max_events = max;
    }

    /// Get last event
    pub fn last_event(&self) -> Option<&Event> {
        self.events.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_handler(_event: &Event) {
        // Do nothing
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            EventType::CounterOperation(MessageOp::Inc),
            EventSource::CounterPort,
        );
        assert_eq!(event.event_type, EventType::CounterOperation(MessageOp::Inc));
        assert_eq!(event.source, EventSource::CounterPort);
    }

    #[test]
    fn test_event_system_publish() {
        let mut sys = EventSystem::new();
        let event = Event::new(
            EventType::CounterOperation(MessageOp::Inc),
            EventSource::CounterPort,
        );

        sys.publish(event.clone());
        assert_eq!(sys.event_count(), 1);
        assert_eq!(sys.last_event().map(|e| e.timestamp), Some(1));
    }

    #[test]
    fn test_event_system_multiple_publishes() {
        let mut sys = EventSystem::new();

        for i in 0..5 {
            sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);
            assert_eq!(sys.event_count(), i + 1);
        }
    }

    #[test]
    fn test_event_system_by_source() {
        let mut sys = EventSystem::new();

        sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);
        sys.publish_counter_op(MessageOp::Dec, EventSource::EchoPort);
        sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);

        let counter_events = sys.events_by_source(EventSource::CounterPort);
        assert_eq!(counter_events.len(), 2);

        let echo_events = sys.events_by_source(EventSource::EchoPort);
        assert_eq!(echo_events.len(), 1);
    }

    #[test]
    fn test_event_system_handler() {
        let mut sys = EventSystem::new();
        sys.register_handler(dummy_handler);

        sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);
        assert_eq!(sys.event_count(), 1);
    }

    #[test]
    fn test_event_system_max_events() {
        let mut sys = EventSystem::new();
        sys.set_max_events(5);

        for _ in 0..10 {
            sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);
        }

        assert_eq!(sys.event_count(), 5);
    }

    #[test]
    fn test_event_system_clear() {
        let mut sys = EventSystem::new();
        sys.publish_counter_op(MessageOp::Inc, EventSource::CounterPort);
        sys.publish_counter_op(MessageOp::Dec, EventSource::CounterPort);

        assert_eq!(sys.event_count(), 2);
        sys.clear_events();
        assert_eq!(sys.event_count(), 0);
    }
}
