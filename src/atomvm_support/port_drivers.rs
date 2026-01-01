use crate::{
    port::{Message, PortResult},
    context::PlatformData,
};
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct EchoPortData {
    pub message_count: i32,
    pub last_message: String,
}

impl EchoPortData {
    pub fn new() -> Self {
        EchoPortData {
            message_count: 0,
            last_message: String::new(),
        }
    }
}

impl PlatformData for EchoPortData {
    fn cleanup(&mut self) {
    }
}

pub fn handle_echo_message(data: &mut EchoPortData, message: &Message) -> PortResult {
    data.message_count += 1;
    // In a real implementation, this would parse the message and echo it back
    // to the sender, incrementing the counter each time.
    PortResult::Continue
}

#[derive(Debug, Clone)]
pub struct CounterPortData {
    pub counter: i32,
    pub last_op: CounterOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterOp {
    Idle,
    Increment,
    Decrement,
    Reset,
}

impl CounterPortData {
    pub fn new() -> Self {
        CounterPortData {
            counter: 0,
            last_op: CounterOp::Idle,
        }
    }

    pub fn increment(&mut self) {
        self.counter = self.counter.saturating_add(1);
        self.last_op = CounterOp::Increment;
    }

    pub fn decrement(&mut self) {
        self.counter = self.counter.saturating_sub(1);
        self.last_op = CounterOp::Decrement;
    }

    pub fn reset(&mut self) {
        self.counter = 0;
        self.last_op = CounterOp::Reset;
    }
}

impl PlatformData for CounterPortData {
    fn cleanup(&mut self) {
        self.reset();
    }
}

pub fn handle_counter_message(data: &mut CounterPortData, _message: &Message) -> PortResult {
    // In a real implementation, this would parse the message to determine
    // which operation to perform (increment, decrement, or reset)
    // and send the result back to the caller.
    PortResult::Continue
}

#[derive(Debug, Clone)]
pub struct BufferPortData {
    pub buffer: alloc::vec::Vec<i32>,
}

impl BufferPortData {
    pub fn new() -> Self {
        BufferPortData {
            buffer: alloc::vec::Vec::new(),
        }
    }

    pub fn push(&mut self, value: i32) {
        let _ = self.buffer.push(value);
    }

    pub fn pop(&mut self) -> Option<i32> {
        self.buffer.pop()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl PlatformData for BufferPortData {
    fn cleanup(&mut self) {
        self.clear();
    }
}

pub fn handle_buffer_message(_data: &mut BufferPortData, _message: &Message) -> PortResult {
    PortResult::Continue
}
