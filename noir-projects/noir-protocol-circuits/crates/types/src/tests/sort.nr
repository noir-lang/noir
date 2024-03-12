use crate::traits::{Empty, is_empty};

struct SortedTuple<T> {
    value: T,
    original_index: u64,
}

pub fn sort_high_to_low<N, T>(
    values: [T; N],
    is_less_than: fn(T, T) -> bool
) -> [SortedTuple<T>; N] where T: Eq {
    let mut sorted_tuples = [SortedTuple { value: values[0], original_index: 0 }; N];

    for i in 0..N {
        sorted_tuples[i] = SortedTuple {
            value: values[i],
            original_index: i,
        };
    }

    sorted_tuples.sort_via(|a: SortedTuple<T>, b: SortedTuple<T>| (b.value == a.value) | is_less_than(b.value, a.value))
}

struct SortedResult<T, N> {
    sorted_array: [T; N],
    sorted_index_hints: [u64; N],
}

pub fn sort_get_sorted_hints<N, T>(
    values: [T; N],
    ordering: fn(T, T) -> bool
) -> SortedResult<T, N> where T: Eq + Empty {
    let mut tuples = [SortedTuple { value: values[0], original_index: 0 }; N];
    for i in 0..N {
        tuples[i] = SortedTuple {
            value: values[i],
            original_index: i,
        };
    }

    let sorted_tuples = tuples.sort_via(
        |a: SortedTuple<T>, b: SortedTuple<T>| is_empty(b.value) | (!is_empty(a.value) & !is_empty(b.value) & ordering(a.value, b.value))
    );

    let sorted_array = sorted_tuples.map(|t: SortedTuple<T>| t.value);
    let mut sorted_index_hints = [0; N];
    for i in 0..N {
        if !is_empty(sorted_tuples[i].value) {
            let original_index = sorted_tuples[i].original_index;
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
