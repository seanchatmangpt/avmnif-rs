use crate::{
    context::Context,
    term::Term,
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

    pub fn get(&self) -> i32 {
        self.value
    }
}

pub fn get_counter_value(_ctx: *mut Context, _args: *const Term, _argc: i32) -> Term {
    Term(0)
}

pub fn increment_counter(_ctx: *mut Context, _args: *const Term, _argc: i32) -> Term {
    Term(0)
}

pub fn decrement_counter(_ctx: *mut Context, _args: *const Term, _argc: i32) -> Term {
    Term(0)
}
