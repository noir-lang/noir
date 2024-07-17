use crate::{
    traits::{Empty, is_empty},
    utils::arrays::{sort_get_sorted_tuple::{sort_get_sorted_tuple, SortedTuple}}
};

struct SortedResult<T, let N: u32> {
    sorted_array: [T; N],
    sorted_index_hints: [u32; N],
}

pub fn sort_get_sorted_hints<let N: u32, T>(
    values: [T; N],
    ordering: fn(T, T) -> bool
) -> SortedResult<T, N> where T: Eq + Empty {
    let sorted = sort_get_sorted_tuple(
        values,
        |a: T, b: T| is_empty(b) | (!is_empty(a) & !is_empty(b) & ordering(a, b))
    );

    let sorted_array = sorted.map(|t: SortedTuple<T>| t.elem);
    let mut sorted_index_hints = [0; N];
    for i in 0..N {
        if !is_empty(sorted[i].elem) {
            let original_index = sorted[i].original_index;
            sorted_index_hints[original_index] = i;
        }
    }

    SortedResult { sorted_array, sorted_index_hints }
}

#[test]
fn sort_get_sorted_hints_asc_non_padded() {
    let values = [40, 60, 20, 50];
    let res = sort_get_sorted_hints(values, |a: Field, b: Field| a.lt(b));
    assert_eq(res.sorted_array, [20, 40, 50, 60]);
    assert_eq(res.sorted_index_hints, [1, 3, 0, 2]);
}

#[test]
fn sort_get_sorted_hints_desc_non_padded() {
    let values = [40, 20, 60, 50];
    let res = sort_get_sorted_hints(values, |a: Field, b: Field| b.lt(a));
    assert_eq(res.sorted_array, [60, 50, 40, 20]);
    assert_eq(res.sorted_index_hints, [2, 3, 0, 1]);
}

#[test]
fn sort_get_sorted_hints_asc_padded() {
    let values = [40, 60, 20, 50, 0, 0];
    let res = sort_get_sorted_hints(values, |a: Field, b: Field| a.lt(b));
    assert_eq(res.sorted_array, [20, 40, 50, 60, 0, 0]);
    assert_eq(res.sorted_index_hints, [1, 3, 0, 2, 0, 0]);
}

#[test]
fn sort_get_sorted_hints_desc_padded() {
    let values = [40, 20, 60, 50, 0, 0];
    let res = sort_get_sorted_hints(values, |a: Field, b: Field| b.lt(a));
    assert_eq(res.sorted_array, [60, 50, 40, 20, 0, 0]);
    assert_eq(res.sorted_index_hints, [2, 3, 0, 1, 0, 0]);
}
