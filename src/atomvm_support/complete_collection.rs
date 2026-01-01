// Complete NIF collection integrating all math, counter, and utility functions
// This module demonstrates how to register multiple NIFs with nif_collection!

use crate::nif_collection;
use crate::atomvm_support::nif_implementations;
use crate::atomvm_support::resource_management;
use crate::atomvm_support::example_module::counter_nifs;

// Module initialization function - called when the module is loaded
fn init(_ctx: &mut crate::Context) {
    // Initialize resources, ports, etc. here
    // This is called with a mutable context reference
}

// Register all math and utility NIFs
nif_collection!(
    avmnif_complete,
    init = init,
    nifs = [
        // Math operations (from nif_implementations)
        ("add", 2, nif_implementations::nif_add),
        ("multiply", 2, nif_implementations::nif_multiply),
        ("list_sum", 1, nif_implementations::nif_list_sum),
        ("tuple_to_list", 1, nif_implementations::nif_tuple_to_list),

        // Counter resource operations (from example_module::counter_nifs)
        ("counter_create", 1, counter_nifs::nif_create_counter),
        ("counter_get", 1, counter_nifs::nif_get_counter),
        ("counter_inc", 1, counter_nifs::nif_increment_counter),
        ("counter_dec", 1, counter_nifs::nif_decrement_counter),
        ("counter_reset", 1, counter_nifs::nif_reset_counter),
    ]
);

// Additional utility functions for the complete module
pub fn get_module_info() -> &'static str {
    "avmnif-rs Complete NIF Collection v0.4.0"
}

pub fn get_nif_count() -> usize {
    9 // Total number of registered NIFs
}
