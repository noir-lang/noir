use crate::traits::{Empty, is_empty};

fn count_non_empty_elements<T, N>(array: [T; N]) -> u64 where T: Empty + Eq {
    let mut length = 0;
    for elem in array {
        if !is_empty(elem) {
            length += 1;
        }
    }
    length
}

pub fn assert_array_eq<T, N, S>(array: [T; N], expected: [T; S]) where T: Empty + Eq {
    assert_eq(count_non_empty_elements(expected), S, "cannot expect empty element in the result");
    assert_eq(count_non_empty_elements(array), S, "mismatch array lengths");
    for i in 0..S {
        assert_eq(array[i], expected[i], "mismatch array elements");
    }
}

// Swap two items in a BoundedVec.
// Useful when we want to shuffle side effects, which by default are ordered by counters when we add mock data to FixtureBuilder.
pub fn swap_items<T, N>(vec: &mut BoundedVec<T, N>, from_index: u64, to_index: u64) {
    let tmp = vec.storage[from_index];
    vec.storage[from_index] = vec.storage[to_index];
    vec.storage[to_index] = tmp;
}
