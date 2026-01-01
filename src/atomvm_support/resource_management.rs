use crate::{
    context::Context,
    term::{Term, TermValue},
};

#[derive(Debug, Clone)]
pub struct Counter {
    pub value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter { value }
    }

    pub fn increment(&mut self) {
        self.value = self.value.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.value = self.value.saturating_sub(1);
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn get(&self) -> i32 {
        self.value
    }
}

pub fn get_counter_value(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let resource_term = &args[0];
    match resource_term.to_value() {
        Ok(TermValue::Resource(res_ref)) => {
            let counter = res_ref.ptr as *mut Counter;
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
        _ => Term(0),
    }
}

pub fn increment_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let resource_term = &args[0];
    match resource_term.to_value() {
        Ok(TermValue::Resource(res_ref)) => {
            let counter = res_ref.ptr as *mut Counter;
            if counter.is_null() {
                return Term(0);
            }

            unsafe {
                (*counter).increment();
            }

            let result = TermValue::int(unsafe { (*counter).get() });
            unsafe {
                let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
                match Term::from_value(result, heap) {
                    Ok(t) => t,
                    Err(_) => Term(0),
                }
            }
        }
        _ => Term(0),
    }
}

pub fn decrement_counter(ctx: *mut Context, args: *const Term, argc: i32) -> Term {
    let ctx = unsafe { &mut *ctx };
    let args = unsafe { core::slice::from_raw_parts(args, argc as usize) };

    if args.len() != 1 {
        return Term(0);
    }

    let resource_term = &args[0];
    match resource_term.to_value() {
        Ok(TermValue::Resource(res_ref)) => {
            let counter = res_ref.ptr as *mut Counter;
            if counter.is_null() {
                return Term(0);
            }

            unsafe {
                (*counter).decrement();
            }

            let result = TermValue::int(unsafe { (*counter).get() });
            unsafe {
                let heap = &mut *(ctx as *mut Context as *mut crate::term::Heap);
                match Term::from_value(result, heap) {
                    Ok(t) => t,
                    Err(_) => Term(0),
                }
            }
        }
        _ => Term(0),
    }
}
