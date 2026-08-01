// Complete working example module demonstrating:
// - NIF function registration with nif_collection!
// - Resource management with Counter resources
// - Port drivers with message handling
// - Integrated FFI layer for AtomVM

pub mod math_nifs;
pub mod counter_nifs;
pub mod echo_port;

// Example module initialization
pub fn init_example_module() {
    // In a real AtomVM module, this would be called during
    // module initialization to set up resources, ports, etc.
    // This is a placeholder for module-level setup.
}
