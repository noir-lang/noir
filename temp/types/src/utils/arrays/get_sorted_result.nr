use crate::utils::arrays::get_sorted_tuple::{get_sorted_tuple, SortedTuple};

pub struct SortedResult<T, let N: u32> {
    sorted_array: [T; N],
    sorted_index_hints: [u32; N],
    original_index_hints: [u32; N],
}

pub unconstrained fn get_sorted_result<let N: u32, T>(
    values: [T; N],
    ordering: fn(T, T) -> bool,
) -> SortedResult<T, N> {
    let sorted = get_sorted_tuple(values, ordering);

    let sorted_array = sorted.map(|t: SortedTuple<T>| t.elem);
    let original_index_hints = sorted.map(|t: SortedTuple<T>| t.original_index);
    let mut sorted_index_hints = [0; N];
    for i in 0..N {
        let original_index = sorted[i].original_index;
        sorted_index_hints[original_index] = i;
    }

    SortedResult { sorted_array, sorted_index_hints, original_index_hints }
}

#[test]
unconstrained fn get_sorted_hints_asc_non_padded() {
    let values = [40, 60, 20, 50];
    let res = get_sorted_result(values, |a: u32, b: u32| a < b);
    assert_eq(res.sorted_array, [20, 40, 50, 60]);
    assert_eq(res.sorted_index_hints, [1, 3, 0, 2]);
    assert_eq(res.original_index_hints, [2, 0, 3, 1]);
}

#[test]
unconstrained fn get_sorted_hints_desc_non_padded() {
    let values = [40, 20, 60, 50];
    let res = get_sorted_result(values, |a: u32, b: u32| b < a);
    assert_eq(res.sorted_array, [60, 50, 40, 20]);
    assert_eq(res.sorted_index_hints, [2, 3, 0, 1]);
}

#[test]
unconstrained fn get_sorted_hints_asc_padded() {
    let values = [40, 60, 20, 50, 0, 0];
    let res = get_sorted_result(values, |a: u32, b: u32| (a != 0) & ((b == 0) | (a < b)));
    assert_eq(res.sorted_array, [20, 40, 50, 60, 0, 0]);
    assert_eq(res.sorted_index_hints, [1, 3, 0, 2, 4, 5]);
    assert_eq(res.original_index_hints, [2, 0, 3, 1, 4, 5]);
}

#[test]
unconstrained fn get_sorted_hints_desc_padded() {
    let values = [40, 60, 20, 50, 0, 0];
    let res = get_sorted_result(values, |a: u32, b: u32| (a != 0) & ((b == 0) | (b < a)));
    assert_eq(res.sorted_array, [60, 50, 40, 20, 0, 0]);
    assert_eq(res.sorted_index_hints, [2, 0, 3, 1, 4, 5]);
    assert_eq(res.original_index_hints, [1, 3, 0, 2, 4, 5]);
}
