pub mod nif_bindings;
pub mod nif_implementations;
pub mod resource_management;
pub mod port_drivers;
pub mod example_module;
pub mod complete_collection;
pub mod safe_api;
pub mod message_dispatch;
pub mod port_state_machine;
pub mod event_system;
pub mod integration;
pub mod performance;
pub mod command_executor;
pub mod health_monitor;
pub mod operation_scheduler;
pub mod failure_detection;
pub mod conflict_detection;
pub mod system_observer;

// Portable control-plane semantics ported from unrdf/packages/atomvm.
// Keep these namespaced rather than glob-reexported so existing avmnif support
// types remain source-compatible and domain-specific semantics do not collide.
pub mod runtime_lifecycle;
pub mod circuit_breaker;
pub mod supervisor;
pub mod hot_code;
pub mod sla;
pub mod message_validation;

// v26.9.11: split topology, intent identity, execution planning, evidence, and
// composed admission into orthogonal reusable surfaces. None owns actuation.
pub mod evidence;
pub mod swarm;
pub mod intent;
pub mod execution;
pub mod control_plane;

pub use nif_bindings::*;
pub use nif_implementations::*;
pub use resource_management::*;
pub use port_drivers::*;
pub use example_module::*;
pub use complete_collection::*;
pub use safe_api::*;
pub use message_dispatch::*;
pub use port_state_machine::*;
pub use event_system::*;
pub use integration::*;
pub use performance::*;
pub use command_executor::*;
pub use health_monitor::*;
pub use operation_scheduler::*;
pub use failure_detection::*;
pub use conflict_detection::*;
pub use system_observer::*;
