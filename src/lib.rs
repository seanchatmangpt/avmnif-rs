#![no_std]
extern crate alloc;

// Core modules - keep your existing structure
pub mod atom;
pub mod log;
pub mod term;
pub mod port;
pub mod tagged;
pub mod context;
pub mod resource;
pub mod registry;

// AtomVM host runtime
pub mod atomvm_support;

// Generated code (from ggen)
pub mod generated;

// Testing infrastructure (available for unit and integration tests)
pub mod testing;

// Re-export commonly used types - match your existing exports
pub use context::Context;
pub use term::{Term, NifResult};
pub use crate::log::log_info;

// Re-export testing utilities
pub use testing::*;