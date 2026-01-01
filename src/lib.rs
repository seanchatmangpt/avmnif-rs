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

// AtomVM integration layer
pub mod atomvm_support;

// Testing infrastructure (only compiled for tests)
#[cfg(test)]
pub mod testing;

// Generated code modules (ggen integration)
#[cfg(test)]
pub mod generated;

// Re-export commonly used types - match your existing exports
pub use context::Context;
pub use term::{Term, NifResult};
pub use crate::log::log_info;

// Re-export testing utilities when testing
#[cfg(test)]
pub use testing::*;