use dep::std::array;
use dep::std::cmp::Eq;
use crate::traits::{Empty, is_empty};

pub fn array_to_bounded_vec<T, N>(array: [T; N]) -> BoundedVec<T, N> where T: Empty + Eq {
    let mut len = 0;
    for elem in array {
        if !is_empty(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len }
}

// Routine which validates that all zero values of an array form a contiguous region at the end, i.e.,
// of the form: [*,*,*...,0,0,0,0] where any * is non-zero. Note that a full array of non-zero values is
// valid.
pub fn validate_array<T, N>(array: [T; N]) where T: Empty + Eq {
    let array_length = array.len();

    let mut first_zero_pos = array_length;
    let mut last_non_zero_pos = 0;

    for i in 0..array_length {
        let is_empty = is_empty(array[i]);
        if !is_empty {
            last_non_zero_pos = i;
        } else if is_empty & (first_zero_pos == array_length) {
            first_zero_pos = i;
        }
    }
    assert(last_non_zero_pos <= first_zero_pos, "invalid array");
}

// Helper method to determine the number of non-zero/empty elements in a validated array (ie, validate_array(array)
// should be true).
pub fn array_length<T, N>(array: [T; N]) -> u64 where T: Empty + Eq {
    let mut length = 0;
    let mut end = false;
    for elem in array {
        end |= is_empty(elem);
        if !end {
            length += 1;
        }
    }
    length
}

pub fn array_eq<T, N, S>(array: [T; N], expected: [T; S]) -> bool where T: Empty + Eq {
    let mut eq = array_length(array) == S;

    for i in 0..S {
        eq &= array[i].eq(expected[i]);
    }

    eq
}

pub fn find_index<T, N, Env>(array: [T; N], find: fn[Env](T) -> bool) -> u64 {
    let mut index = N;
    for i in 0..N {
        if (index == N) & find(array[i]) {
            index = i;
        }
    }
    index
}

pub fn array_cp<T, N, S>(array: [T; N]) -> [T; S] where T: Empty {
    let mut result: [T; S] = [T::empty(); S];
    for i in 0..S {
        result[i] = array[i];
    }
    result
}

pub fn array_merge<T, N>(array1: [T; N], array2: [T; N]) -> [T; N] where T: Empty + Eq {
    let mut result: [T; N] = [T::empty(); N];
    let mut i = 0;
    for elem in array1 {
        if !is_empty(elem) {
            result[i] = elem;
            i += 1;
        }
    }
    for elem in array2 {
        if !is_empty(elem) {
            result[i] = elem;
            i += 1;
        }
    }
    result
}

pub fn check_permutation<T, N>(
    original_array: [T; N],
    permuted_array: [T; N],
    original_indexes: [u64; N]
) where T: Eq + Empty {
    let mut seen_value = [false; N];
    for i in 0..N {
        let index = original_indexes[i];
        let original_value = original_array[index];
        assert(permuted_array[i].eq(original_value), "Invalid index");
        assert(!seen_value[index], "Duplicated index");
        seen_value[index] = true;
    }
}

pub fn assert_sorted_array<T, N, Env>(
    original_array: [T; N],
    sorted_array: [T; N],
    sorted_indexes: [u64; N],
    ordering: fn[Env](T, T) -> bool
) where T: Eq + Empty {
    let mut seen_empty = false;
    for i in 0..N {
        let original_value = original_array[i];
        if is_empty(original_value) {
            seen_empty = true;
            assert(is_empty(sorted_array[i]), "Empty values must not be mixed with sorted values");
        } else {
            assert(!seen_empty, "Empty values must be padded to the right");

            let index = sorted_indexes[i];
            assert(sorted_array[index].eq(original_value), "Invalid index");
            if i != 0 {
                assert(ordering(sorted_array[i - 1], sorted_array[i]), "Values not sorted");
            }
        }
    }
}

#[test]
fn smoke_validate_array() {
    let valid_array = [];
    validate_array(valid_array);

    let valid_array = [0];
    validate_array(valid_array);

    let valid_array = [1, 2, 3];
    validate_array(valid_array);

    let valid_array = [1, 2, 3, 0];
    validate_array(valid_array);

    let valid_array = [1, 2, 3, 0, 0];
    validate_array(valid_array);
}

#[test(should_fail_with = "invalid array")]
fn smoke_validate_array_invalid() {
    let invalid_array = [0, 1];
    validate_array(invalid_array);
}

#[test(should_fail_with = "invalid array")]
fn smoke_validate_array_invalid_case2() {
    let invalid_array = [0, 0, 0, 0, 1];
    validate_array(invalid_array);
}

#[test]
fn test_empty_array_length() {
    assert_eq(array_length([]), 0);
    assert_eq(array_length([0]), 0);
    assert_eq(array_length([0, 0, 0]), 0);
}

#[test]
fn test_array_length() {
    assert_eq(array_length([123]), 1);
    assert_eq(array_length([123, 456]), 2);
    assert_eq(array_length([123, 456, 0]), 2);
    assert_eq(array_length([123, 0, 456]), 1);
    assert_eq(array_length([0, 123, 0, 456]), 0);
}

#[test]
fn find_index_greater_than_min() {
    let values = [10, 20, 30, 40];
    let min = 22;
    let index = find_index(values, |v: Field| min.lt(v));
    assert_eq(index, 2);
}

#[test]
fn find_index_not_found() {
    let values = [10, 20, 30, 40];
    let min = 100;
    let index = find_index(values, |v: Field| min.lt(v));
    assert_eq(index, 4);
}

#[test]
fn check_permutation_basic_test() {
    let original_array = [1, 2, 3];
    let permuted_array = [3, 1, 2];
    let indexes = [2, 0, 1];
    check_permutation(original_array, permuted_array, indexes);
}

#[test(should_fail_with = "Duplicated index")]
fn check_permutation_duplicated_index() {
    let original_array = [0, 1, 0];
    let permuted_array = [1, 0, 0];
    let indexes = [1, 0, 0];
    check_permutation(original_array, permuted_array, indexes);
}

#[test(should_fail_with = "Invalid index")]
fn check_permutation_invalid_index() {
    let original_array = [0, 1, 2];
    let permuted_array = [1, 0, 0];
    let indexes = [1, 0, 2];
    check_permutation(original_array, permuted_array, indexes);
}

#[test]
fn assert_sorted_array_asc() {
    let original = [30, 20, 90, 50, 0, 0];
    let sorted = [20, 30, 50, 90, 0, 0];
    let indexes = [1, 0, 3, 2, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test]
fn assert_sorted_array_desc() {
    let original = [30, 20, 90, 50, 0, 0];
    let sorted = [90, 50, 30, 20, 0, 0];
    let indexes = [2, 3, 0, 1, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| b.lt(a));
}

#[test]
fn assert_sorted_array_all_empty() {
    let original = [0, 0, 0, 0, 0, 0];
    let sorted = [0, 0, 0, 0, 0, 0];
    let indexes = [0, 0, 0, 0, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test(should_fail_with="Values not sorted")]
fn assert_sorted_array_failed_ordering() {
    let original = [30, 20, 90, 50, 0, 0];
    let sorted = [20, 30, 90, 50, 0, 0];
    let indexes = [1, 0, 2, 3, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test(should_fail_with="Values not sorted")]
fn assert_sorted_array_failed_misplaced_sorted() {
    let original = [30, 20, 90, 50, 0, 0];
    let sorted = [20, 30, 50, 0, 0, 90];
    let indexes = [1, 0, 5, 2, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test(should_fail_with="Invalid index")]
fn assert_sorted_array_failed_wrong_index() {
    let original = [30, 20, 90, 50, 0, 0];
    let sorted = [20, 30, 50, 90, 0, 0];
    let indexes = [1, 1, 2, 3, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test(should_fail_with="Empty values must be padded to the right")]
fn assert_sorted_array_failed_not_padded() {
    let original = [30, 20, 90, 0, 50, 0];
    let sorted = [20, 30, 90, 0, 0, 0];
    let indexes = [1, 0, 2, 0, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}

#[test(should_fail_with="Empty values must not be mixed with sorted values")]
fn assert_sorted_array_failed_mixed_empty() {
    let original = [30, 20, 90, 0, 0, 0];
    let sorted = [20, 30, 90, 0, 0, 10];
    let indexes = [1, 0, 2, 0, 0, 0];
    assert_sorted_array(original, sorted, indexes, |a: Field, b: Field| a.lt(b));
}
