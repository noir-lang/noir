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
    assert((last_non_zero_pos as u64) <= (first_zero_pos as u64), "invalid array");
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

pub fn array_cp<T, N, S>(array: [T; N]) -> [T; S] where T: Empty {
    let mut result: [T; S] = [T::empty(); S];
    for i in 0..S {
        result[i] = array[i];
    }
    result
}

pub fn array_concat<T, N, S, M>(array1: [T; N], array2: [T; S]) -> [T; M] where T: Empty + Eq {
    let mut result: [T; M] = [T::empty(); M];
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
