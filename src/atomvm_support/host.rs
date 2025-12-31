//! Host runtime: manages AtomVM instance and actor lifecycle

use alloc::string::String;
use alloc::vec::Vec;
use crate::atomvm_support::errors::{HostError, HostResult};

/// Configuration for AtomVM host
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Path to .avm bytecode file
    pub bytecode_path: String,
    /// Initial heap size (bytes)
    pub heap_size: u32,
    /// Number of scheduler threads
    pub num_schedulers: u32,
}

impl VmConfig {
    /// Create default config
    pub fn default_with_bytecode(path: &str) -> Self {
        VmConfig {
            bytecode_path: path.to_string(),
            heap_size: 1024 * 1024,  // 1MB default
            num_schedulers: 1,
        }
    }
}

/// AtomVM Host - manages the virtual machine
pub struct Host {
    config: VmConfig,
    /// Whether host is initialized
    initialized: bool,
    /// Loaded modules
    modules: Vec<String>,
}

impl Host {
    /// Create new host with config
    pub fn new(config: VmConfig) -> Self {
        Host {
            config,
            initialized: false,
            modules: Vec::new(),
        }
    }

    /// Initialize the host
    /// In a real implementation, this would:
    /// - Call atomvm_initialize()
    /// - Set up memory
    /// - Register NIFs
    pub fn initialize(&mut self) -> HostResult<()> {
        if self.initialized {
            return Err(HostError::AlreadyInitialized);
        }

        // TODO: Call into atomvm_initialize with config
        // For now, mark as initialized
        self.initialized = true;
        Ok(())
    }

    /// Load bytecode module
    pub fn load_module(&mut self, path: &str) -> HostResult<()> {
        if !self.initialized {
            return Err(HostError::NotInitialized);
        }

        // TODO: In real implementation:
        // - Read .avm bytecode
        // - Parse and validate
        // - Call atomvm_module_load()
        // - Register with module system

        self.modules.push(path.to_string());
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &VmConfig {
        &self.config
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get loaded modules
    pub fn modules(&self) -> &[String] {
        &self.modules
    }

    /// Shutdown the host
    pub fn shutdown(&mut self) -> HostResult<()> {
        if !self.initialized {
            return Err(HostError::NotInitialized);
        }

        // TODO: Call atomvm_destroy()
        self.initialized = false;
        self.modules.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_creation() {
        let config = VmConfig::default_with_bytecode("test.avm");
        let host = Host::new(config);

        assert!(!host.is_initialized());
        assert_eq!(host.modules().len(), 0);
    }

    #[test]
    fn test_host_initialize() {
        let config = VmConfig::default_with_bytecode("test.avm");
        let mut host = Host::new(config);

        let result = host.initialize();
        assert!(result.is_ok());
        assert!(host.is_initialized());
    }

    #[test]
    fn test_host_cannot_double_init() {
        let config = VmConfig::default_with_bytecode("test.avm");
        let mut host = Host::new(config);

        let _ = host.initialize();
        let result = host.initialize();

        assert!(result.is_err());
    }

    #[test]
    fn test_host_load_module_requires_init() {
        let config = VmConfig::default_with_bytecode("test.avm");
        let mut host = Host::new(config);

        let result = host.load_module("worker.avm");
        assert!(result.is_err());
    }

    #[test]
    fn test_host_load_module_after_init() {
        let config = VmConfig::default_with_bytecode("test.avm");
        let mut host = Host::new(config);

        let _ = host.initialize();
        let result = host.load_module("worker.avm");

        assert!(result.is_ok());
        assert_eq!(host.modules().len(), 1);
    }
}
