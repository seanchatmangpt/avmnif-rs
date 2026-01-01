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

pub fn handle_echo_message(data: &mut EchoPortData, _message: &Message) -> PortResult {
    data.message_count += 1;
    PortResult::Continue
}

#[derive(Debug, Clone)]
pub struct CounterPortData {
    pub counter: i32,
}

impl CounterPortData {
    pub fn new() -> Self {
        CounterPortData { counter: 0 }
    }

    pub fn increment(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

impl PlatformData for CounterPortData {
    fn cleanup(&mut self) {
    }
}

pub fn handle_counter_message(_data: &mut CounterPortData, _message: &Message) -> PortResult {
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
