// Math NIF functions: add, multiply, list_sum, tuple_to_list
// These are wrappers around the generated math NIFs with proper FFI signatures

use crate::{
    context::Context,
    term::{Term, TermValue, NifError},
};

/// Add two signed integers with overflow detection
pub fn nif_add(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 2 {
        return Term(0);
    }

    let a = match args[0].to_value().and_then(|v| Ok(v.as_int().ok_or(NifError::BadArg)?)) {
        Ok(v) => v,
        Err(_) => return Term(0),
    };

    let b = match args[1].to_value().and_then(|v| Ok(v.as_int().ok_or(NifError::BadArg)?)) {
        Ok(v) => v,
        Err(_) => return Term(0),
    };

    let sum = match a.checked_add(b) {
        Some(s) => s,
        None => return Term(0),
    };

    let result = TermValue::int(sum);
    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Multiply two signed integers with overflow detection
pub fn nif_multiply(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 2 {
        return Term(0);
    }

    let x = match args[0].to_value().and_then(|v| Ok(v.as_int().ok_or(NifError::BadArg)?)) {
        Ok(v) => v,
        Err(_) => return Term(0),
    };

    let y = match args[1].to_value().and_then(|v| Ok(v.as_int().ok_or(NifError::BadArg)?)) {
        Ok(v) => v,
        Err(_) => return Term(0),
    };

    let product = match x.checked_mul(y) {
        Some(p) => p,
        None => return Term(0),
    };

    let result = TermValue::int(product);
    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}

/// Sum all integers in a list
pub fn nif_list_sum(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let list_term = match args[0].to_value() {
        Ok(v) => v,
        Err(_) => return Term(0),
    };

    let list_vec = list_term.list_to_vec();
    let sum = match list_vec.iter().try_fold(0i32, |acc, term| {
        let n = term.as_int().ok_or(NifError::BadArg)?;
        acc.checked_add(n).ok_or(NifError::OutOfMemory)
    }) {
        Ok(s) => s,
        Err(_) => return Term(0),
    };

    let result = TermValue::int(sum);
    unsafe {
        let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
        match Term::from_value(result, heap) {
            Ok(t) => t,
            Err(_) => Term(0),
        }
    }
}
