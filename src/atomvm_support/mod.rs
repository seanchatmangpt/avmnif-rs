//! AtomVM Host Integration
//!
//! Embeds AtomVM as the actor runtime while providing a Rust API surface.
//! Developers write Rust, the system runs on BEAM-ish scheduling.

pub mod host;
pub mod dispatch;
pub mod errors;

pub use host::{Host, VmConfig};
pub use dispatch::NifDispatcher;
pub use errors::{HostError, HostResult};
