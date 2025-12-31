//! NIF Dispatcher - handles messages from AtomVM
//!
//! Routes incoming messages to actor handlers and returns results

use alloc::string::String;
use crate::atomvm_support::errors::HostResult;

/// NIF Dispatcher - manages message routing to handlers
pub struct NifDispatcher {
    /// ID counter for actors
    next_actor_id: u32,
}

impl NifDispatcher {
    /// Create new dispatcher
    pub fn new() -> Self {
        NifDispatcher {
            next_actor_id: 1,
        }
    }

    /// Allocate new actor ID
    pub fn allocate_actor_id(&mut self) -> u32 {
        let id = self.next_actor_id;
        self.next_actor_id += 1;
        id
    }

    /// Dispatch message to handler
    /// In real implementation, would route to the appropriate actor handler
    /// based on actor_id
    pub fn dispatch(&self, actor_id: u32, message_data: &[u8]) -> HostResult<Vec<u8>> {
        // TODO: Route to correct handler based on actor_id
        // For now, return success with empty data
        Ok(Vec::new())
    }
}

impl Default for NifDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = NifDispatcher::new();
        assert_eq!(dispatcher.next_actor_id, 1);
    }

    #[test]
    fn test_dispatcher_allocate_actor_id() {
        let mut dispatcher = NifDispatcher::new();

        let id1 = dispatcher.allocate_actor_id();
        let id2 = dispatcher.allocate_actor_id();
        let id3 = dispatcher.allocate_actor_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_dispatcher_dispatch() {
        let dispatcher = NifDispatcher::new();
        let result = dispatcher.dispatch(1, b"test_message");

        assert!(result.is_ok());
    }
}
