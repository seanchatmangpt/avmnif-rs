//! Host and NIF error types

use alloc::string::String;

/// Errors that can occur in the Host
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// Host not yet initialized
    NotInitialized,
    /// Host already initialized
    AlreadyInitialized,
    /// Module not found
    ModuleNotFound,
    /// Invalid bytecode
    InvalidBytecode(String),
    /// NIF call failed
    NifError(String),
    /// IO error
    IoError(String),
    /// Internal error
    Internal(String),
}

impl HostError {
    pub fn message(&self) -> &str {
        match self {
            HostError::NotInitialized => "Host not initialized",
            HostError::AlreadyInitialized => "Host already initialized",
            HostError::ModuleNotFound => "Module not found",
            HostError::InvalidBytecode(_) => "Invalid bytecode",
            HostError::NifError(_) => "NIF error",
            HostError::IoError(_) => "IO error",
            HostError::Internal(_) => "Internal error",
        }
    }
}

/// Result type for Host operations
pub type HostResult<T> = Result<T, HostError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_error_messages() {
        assert_eq!(HostError::NotInitialized.message(), "Host not initialized");
        assert_eq!(HostError::ModuleNotFound.message(), "Module not found");
    }
}
