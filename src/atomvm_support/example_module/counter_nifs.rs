// Counter NIF functions for working with Counter resources
// These functions demonstrate resource management with proper FFI signatures

use alloc::boxed::Box;
use crate::{
    context::Context,
    term::{Term, TermValue},
};
use crate::atomvm_support::resource_management::Counter;

/// Create a new counter resource with initial value
pub fn nif_create_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let initial_value = match args[0].to_value() {
        Ok(TermValue::SmallInt(v)) => v,
        _ => return Term(0),
    };

    let counter = Box::new(Counter::new(initial_value));
    let counter_ptr = Box::into_raw(counter);

    let resource = TermValue::Resource(crate::term::ResourceRef {
        type_name: "counter".into(),
        ptr: counter_ptr as *mut core::ffi::c_void,
    });

    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(resource, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Get the current value of a counter
pub fn nif_get_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let counter = match args[0].to_value() {
        Ok(TermValue::Resource(res_ref)) => res_ref.ptr as *mut Counter,
        _ => return Term(0),
    };

    if counter.is_null() {
        return Term(0);
    }

    let value = unsafe { (*counter).get() };
    let result = TermValue::int(value);

    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Increment a counter and return the new value
pub fn nif_increment_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let counter = match args[0].to_value() {
        Ok(TermValue::Resource(res_ref)) => res_ref.ptr as *mut Counter,
        _ => return Term(0),
    };

    if counter.is_null() {
        return Term(0);
    }

    unsafe {
        (*counter).increment();
    }

    let value = unsafe { (*counter).get() };
    let result = TermValue::int(value);

    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Decrement a counter and return the new value
pub fn nif_decrement_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let counter = match args[0].to_value() {
        Ok(TermValue::Resource(res_ref)) => res_ref.ptr as *mut Counter,
        _ => return Term(0),
    };

    if counter.is_null() {
        return Term(0);
    }

    unsafe {
        (*counter).decrement();
    }

    let value = unsafe { (*counter).get() };
    let result = TermValue::int(value);

    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Reset a counter to zero and return the new value
pub fn nif_reset_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let counter = match args[0].to_value() {
        Ok(TermValue::Resource(res_ref)) => res_ref.ptr as *mut Counter,
        _ => return Term(0),
    };

    if counter.is_null() {
        return Term(0);
    }

    unsafe {
        (*counter).reset();
    }

    let result = TermValue::int(0);

    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}
