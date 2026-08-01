use crate::context::Context;
use super::nif_implementations;

pub fn init(_ctx: &mut Context) {
}

crate::nif_collection!(
    avmnif_math,
    init = init,
    nifs = [
        ("add", 2, nif_implementations::nif_add),
        ("multiply", 2, nif_implementations::nif_multiply),
        ("list_sum", 1, nif_implementations::nif_list_sum),
        ("tuple_to_list", 1, nif_implementations::nif_tuple_to_list),
    ]
);
