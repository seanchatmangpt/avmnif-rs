// High-level safe Rust API wrapper for AtomVM integration
// This module provides ergonomic, type-safe wrappers around the raw FFI functions

use alloc::boxed::Box;
use crate::{
    context::Context,
    term::{Term, TermValue, NifResult, NifError},
};
use crate::atomvm_support::resource_management::Counter;

/// Safe wrapper around Counter resource operations
pub struct SafeCounter {
    counter_ptr: *mut Counter,
}

impl SafeCounter {
    /// Create a new counter with the given initial value
    pub fn new(value: i32) -> Self {
        let counter = Box::new(Counter::new(value));
        let counter_ptr = Box::into_raw(counter);
        SafeCounter { counter_ptr }
    }

    /// Get the current value
    pub fn get(&self) -> i32 {
        if self.counter_ptr.is_null() {
            0
        } else {
            unsafe { (*self.counter_ptr).get() }
        }
    }

    /// Increment the counter and return the new value
    pub fn increment(&mut self) -> i32 {
        if !self.counter_ptr.is_null() {
            unsafe {
                (*self.counter_ptr).increment();
                (*self.counter_ptr).get()
            }
        } else {
            0
        }
    }

    /// Decrement the counter and return the new value
    pub fn decrement(&mut self) -> i32 {
        if !self.counter_ptr.is_null() {
            unsafe {
                (*self.counter_ptr).decrement();
                (*self.counter_ptr).get()
            }
        } else {
            0
        }
    }

    /// Reset the counter to zero
    pub fn reset(&mut self) -> i32 {
        if !self.counter_ptr.is_null() {
            unsafe {
                (*self.counter_ptr).reset();
            }
        }
        0
    }
}

impl Drop for SafeCounter {
    fn drop(&mut self) {
        if !self.counter_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.counter_ptr);
            }
        }
    }
}

/// Safe wrapper around arithmetic operations
pub struct SafeMath;

impl SafeMath {
    /// Add two integers with overflow detection
    pub fn add(a: i32, b: i32) -> Result<i32, NifError> {
        a.checked_add(b).ok_or(NifError::OutOfMemory)
    }

    /// Multiply two integers with overflow detection
    pub fn multiply(x: i32, y: i32) -> Result<i32, NifError> {
        x.checked_mul(y).ok_or(NifError::OutOfMemory)
    }

    /// Sum all integers in a list
    pub fn list_sum(items: &[i32]) -> Result<i32, NifError> {
        items.iter().try_fold(0i32, |acc, &item| {
            acc.checked_add(item).ok_or(NifError::OutOfMemory)
        })
    }

    /// Convert a tuple-like slice to a "list" (just return count)
    pub fn tuple_to_list_count(items: &[i32]) -> usize {
        items.len()
    }
}

/// Type-safe builder for creating Term values
pub struct TermBuilder;

impl TermBuilder {
    /// Create a TermValue integer
    pub fn int(value: i32) -> TermValue {
        TermValue::int(value)
    }

    /// Create an empty list
    pub fn empty_list() -> TermValue {
        TermValue::Nil
    }

    /// Create a list from integers
    pub fn int_list(values: &[i32]) -> TermValue {
        let values_vec: alloc::vec::Vec<_> = values.iter().map(|&v| TermValue::int(v)).collect();
        TermValue::list(values_vec)
    }

    /// Create a tuple from integers
    pub fn int_tuple(values: &[i32]) -> TermValue {
        let values_vec: alloc::vec::Vec<_> = values.iter().map(|&v| TermValue::int(v)).collect();
        TermValue::tuple(values_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_counter_creation() {
        let mut counter = SafeCounter::new(5);
        assert_eq!(counter.get(), 5);
    }

    #[test]
    fn test_safe_counter_increment() {
        let mut counter = SafeCounter::new(0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
    }

    #[test]
    fn test_safe_counter_decrement() {
        let mut counter = SafeCounter::new(5);
        assert_eq!(counter.decrement(), 4);
        assert_eq!(counter.decrement(), 3);
    }

    #[test]
    fn test_safe_math_add() {
        assert_eq!(SafeMath::add(2, 3), Ok(5));
        assert!(SafeMath::add(i32::MAX, 1).is_err());
    }

    #[test]
    fn test_safe_math_multiply() {
        assert_eq!(SafeMath::multiply(3, 4), Ok(12));
        assert!(SafeMath::multiply(i32::MAX, 2).is_err());
    }

    #[test]
    fn test_safe_math_list_sum() {
        assert_eq!(SafeMath::list_sum(&[1, 2, 3, 4, 5]), Ok(15));
        assert_eq!(SafeMath::list_sum(&[]), Ok(0));
    }

    #[test]
    fn test_term_builder() {
        let int_term = TermBuilder::int(42);
        assert_eq!(int_term.as_int(), Some(42));

        let list_term = TermBuilder::int_list(&[1, 2, 3]);
        let list_vec = list_term.list_to_vec();
        assert_eq!(list_vec.len(), 3);
    }
}
