//! Testing utilities and mock implementations for avmnif-rs
//!
//! This module provides centralized testing infrastructure including:
//! - Mock implementations of AtomVM components
//! - Test helpers and utilities
//! - Common test fixtures and data
//!
//! Available for both unit and integration tests.

pub mod mocks;
pub mod helpers;
pub mod fixtures;
pub mod nifs;
pub mod resources;
pub mod tagged;
pub mod ports;

// Re-export everything for convenient imports
pub use mocks::*;
pub use helpers::*;
pub use fixtures::*;