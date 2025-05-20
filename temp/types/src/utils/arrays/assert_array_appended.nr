use crate::{
    abis::side_effect::{Scoped as ScopedTrait, scoped::Scoped},
    address::aztec_address::AztecAddress,
    traits::{Empty, is_empty},
};

pub fn assert_array_appended<T, let N: u32, let M: u32>(
    dest: [T; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32,
)
where
    T: Empty + Eq,
{
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        // Loop through dest instead of source because we also need to check that dest is appended with empty items.
        should_check |= i == num_prepended_items; // Prepended items have been checked in validate_array_prepended() and can be skipped here.
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i],
                    source[i - num_prepended_items],
                    "source item does not append to dest",
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

// Similar to assert_array_appended, except that the souce items will be appended to dest in reversed order.
pub fn assert_array_appended_reversed<T, let N: u32, let M: u32>(
    dest: [T; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32,
)
where
    T: Empty + Eq,
{
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        should_check |= i == num_prepended_items;
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i],
                    source[items_propagated - i - 1],
                    "source item does not reversed append to dest",
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

// Similar to assert_array_appended, except that the contract address of the dest items will also be checked.
pub fn assert_array_appended_scoped<ST, T, let N: u32, let M: u32>(
    dest: [ST; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32,
    contract_address: AztecAddress,
)
where
    ST: ScopedTrait<T> + Empty + Eq,
    T: Eq,
{
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        should_check |= i == num_prepended_items;
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i].inner(),
                    source[i - num_prepended_items],
                    "source item does not append to dest",
                );
                assert_eq(
                    dest[i].contract_address(),
                    contract_address,
                    "propagated contract address does not match",
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

pub fn assert_array_appended_and_scoped<T, let N: u32, let M: u32>(
    dest: [Scoped<T>; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32,
    contract_address: AztecAddress,
)
where
    T: Eq + Empty,
{
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        should_check |= i == num_prepended_items;
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i].inner,
                    source[i - num_prepended_items],
                    "source item does not append to dest",
                );
                assert_eq(
                    dest[i].contract_address,
                    contract_address,
                    "propagated contract address does not match",
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}
