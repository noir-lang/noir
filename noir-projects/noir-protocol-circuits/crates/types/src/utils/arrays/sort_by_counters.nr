use crate::{abis::side_effect::Ordered, traits::{Empty, is_empty}};

pub fn order_by_counters_empty_padded_asc<T>(a: T, b: T) -> bool where T: Ordered + Eq + Empty {
    is_empty(b) | (!is_empty(a) & !is_empty(b) & a.counter() < b.counter())
}

pub fn order_by_counters_empty_padded_desc<T>(a: T, b: T) -> bool where T: Ordered + Eq + Empty {
    is_empty(b) | (!is_empty(a) & !is_empty(b) & a.counter() > b.counter())
}

fn sort_by<T, N>(array: [T; N], ordering: fn(T, T) -> bool) -> [T; N] {
    array.sort_via(|a, b| ordering(a, b))
}

pub fn sort_by_counters_asc<T, N>(array: [T; N]) -> [T; N] where T: Ordered + Eq + Empty {
    sort_by(array, order_by_counters_empty_padded_asc)
}

pub fn sort_by_counters_desc<T, N>(array: [T; N]) -> [T; N] where T: Ordered + Eq + Empty {
    sort_by(array, order_by_counters_empty_padded_desc)
}
