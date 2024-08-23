mod assert_combined_deduped_array;
mod assert_combined_sorted_transformed_value_array;
mod assert_exposed_sorted_transformed_value_array;
mod assert_sorted_array;
mod assert_sorted_transformed_value_array;
mod assert_split_sorted_transformed_value_arrays;
mod assert_split_transformed_value_arrays;
mod get_sorted_hints;
mod get_sorted_tuple;
mod sort_by;
mod sort_by_counter;

// Re-exports.
use assert_combined_deduped_array::{
    assert_combined_deduped_array, dedupe_array::dedupe_array,
    get_deduped_hints::{DedupedHints, get_deduped_hints},
    sort_by_position_then_counter::sort_by_position_then_counter
};
use assert_combined_sorted_transformed_value_array::{
    assert_combined_sorted_transformed_value_array_asc,
    get_combined_order_hints::{CombinedOrderHint, get_combined_order_hints_asc}
};
use assert_exposed_sorted_transformed_value_array::{
    assert_exposed_sorted_transformed_value_array,
    get_order_hints::{get_order_hints_asc, get_order_hints_desc, OrderHint}
};
use assert_sorted_array::assert_sorted_array;
use assert_split_sorted_transformed_value_arrays::{
    assert_split_sorted_transformed_value_arrays_asc, assert_split_sorted_transformed_value_arrays_desc,
    get_split_order_hints::{get_split_order_hints_asc, get_split_order_hints_desc, SplitOrderHints}
};
use assert_sorted_transformed_value_array::{assert_sorted_transformed_value_array, assert_sorted_transformed_value_array_capped_size};
use assert_split_transformed_value_arrays::assert_split_transformed_value_arrays;
use get_sorted_hints::get_sorted_hints;
use sort_by_counter::{sort_by_counter_asc, sort_by_counter_desc};

use crate::traits::{Empty, is_empty};

pub fn array_to_bounded_vec<T, let N: u32>(array: [T; N]) -> BoundedVec<T, N> where T: Empty + Eq {
    let mut len = 0;
    for elem in array {
        if !is_empty(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len }
}

unconstrained pub fn find_index_hint<T, let N: u32, Env>(array: [T; N], find: fn[Env](T) -> bool) -> u32 {
    let mut index = N;
    for i in 0..N {
        if (index == N) & find(array[i]) {
            index = i;
        }
    }
    index
}

// Routine which validates that all zero values of an array form a contiguous region at the end, i.e.,
// of the form: [*,*,*...,0,0,0,0] where any * is non-zero. Note that a full array of non-zero values is
// valid.
pub fn validate_array<T, let N: u32>(array: [T; N]) -> u32 where T: Empty + Eq {
    let mut seen_empty = false;
    let mut length = 0;
    for i in 0..N {
        if is_empty(array[i]) {
            seen_empty = true;
        } else {
            assert(seen_empty == false, "invalid array");
            length += 1;
        }
    }
    length
}

unconstrained fn count_non_empty_elements<T, let N: u32>(array: [T; N]) -> u32 where T: Empty + Eq {
    let mut length = 0;
    let mut seen_empty = false;
    for elem in array {
        if is_empty(elem) {
            seen_empty = true;
        } else if !seen_empty {
            length += 1;
        }
    }
    length
}

// Helper function to count the number of non-empty elements in a validated array.
// Important: Only use it for validated arrays: validate_array(array) should be true.
pub fn array_length<T, let N: u32>(array: [T; N]) -> u32 where T: Empty + Eq {
    let length = count_non_empty_elements(array);
    if length != 0 {
        assert(!is_empty(array[length - 1]));
    }
    if length != N {
        assert(is_empty(array[length]));
    }
    length
}

pub fn array_concat<T, let N: u32, let M: u32, let S: u32>(array1: [T; N], array2: [T; M]) -> [T; S] {
    assert_eq(N + M, S, "combined array length does not match return array length");
    let mut result = [array1[0]; S];
    for i in 1..N {
        result[i] = array1[i];
    }
    for i in 0..M {
        result[i + N] = array2[i];
    }
    result
}

pub fn array_merge<T, let N: u32>(array1: [T; N], array2: [T; N]) -> [T; N] where T: Empty + Eq {
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

pub fn check_permutation<T, let N: u32>(original_array: [T; N], permuted_array: [T; N], original_indexes: [u32; N]) where T: Eq + Empty {
    let mut seen_value = [false; N];
    for i in 0..N {
        let index = original_indexes[i];
        let original_value = original_array[index];
        assert(permuted_array[i].eq(original_value), "Invalid index");
        assert(!seen_value[index], "Duplicated index");
        seen_value[index] = true;
    }
}

#[test]
fn smoke_validate_array() {
    let valid_array: [Field; 0] = [];
    assert(validate_array(valid_array) == 0);

    let valid_array = [0];
    assert(validate_array(valid_array) == 0);

    let valid_array = [3];
    assert(validate_array(valid_array) == 1);

    let valid_array = [1, 2, 3];
    assert(validate_array(valid_array) == 3);

    let valid_array = [1, 2, 3, 0];
    assert(validate_array(valid_array) == 3);

    let valid_array = [1, 2, 3, 0, 0];
    assert(validate_array(valid_array) == 3);
}

#[test(should_fail_with = "invalid array")]
fn smoke_validate_array_invalid_case0() {
    let invalid_array = [0, 1];
    let _ = validate_array(invalid_array);
}

#[test(should_fail_with = "invalid array")]
fn smoke_validate_array_invalid_case1() {
    let invalid_array = [1, 0, 0, 1, 0];
    let _ = validate_array(invalid_array);
}

#[test(should_fail_with = "invalid array")]
fn smoke_validate_array_invalid_case2() {
    let invalid_array = [0, 0, 0, 0, 1];
    let _ = validate_array(invalid_array);
}

#[test]
fn test_empty_array_length() {
    assert_eq(array_length([0]), 0);
    assert_eq(array_length([0, 0, 0]), 0);
}

#[test]
fn test_array_length() {
    assert_eq(array_length([123]), 1);
    assert_eq(array_length([123, 0, 0]), 1);
    assert_eq(array_length([123, 456]), 2);
    assert_eq(array_length([123, 456, 0]), 2);
}

#[test]
fn test_array_length_invalid_arrays() {
    // Result can be misleading (but correct) for invalid arrays.
    assert_eq(array_length([0, 0, 123]), 0);
    assert_eq(array_length([0, 123, 0]), 0);
    assert_eq(array_length([0, 123, 456]), 0);
    assert_eq(array_length([123, 0, 456]), 1);
}

#[test]
fn find_index_greater_than_min() {
    let values = [10, 20, 30, 40];
    let min = 22;
    let index = find_index_hint(values, |v: Field| min.lt(v));
    assert_eq(index, 2);
}

#[test]
fn find_index_not_found() {
    let values = [10, 20, 30, 40];
    let min = 100;
    let index = find_index_hint(values, |v: Field| min.lt(v));
    assert_eq(index, 4);
}

#[test]
fn test_array_concat() {
    let array0 = [1, 2, 3];
    let array1 = [4, 5];
    let concated = array_concat(array0, array1);
    assert_eq(concated, [1, 2, 3, 4, 5]);
}

#[test(should_fail_with="combined array length does not match return array length")]
fn array_concat_fails_inconsistent_lengths() {
    let array0 = [1, 2, 3];
    let array1 = [4, 5];
    let _concated: [Field; 4] = array_concat(array0, array1);
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
