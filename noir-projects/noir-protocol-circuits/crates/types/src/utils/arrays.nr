mod assert_sorted_array;
mod assert_sorted_transformed_value_array;
mod assert_split_sorted_transformed_value_arrays;
mod sort_by_counters;
mod sort_get_order_hints;
mod sort_get_sorted_hints;
mod sort_get_sorted_tuple;
mod sort_get_split_order_hints;

// Re-exports.
use assert_sorted_array::{assert_sorted_array_with_order_hints, assert_sorted_array};
use assert_split_sorted_transformed_value_arrays::{assert_split_sorted_transformed_value_arrays_asc, assert_split_sorted_transformed_value_arrays_desc};
use assert_sorted_transformed_value_array::assert_sorted_transformed_value_array;
use sort_by_counters::{sort_by_counters_asc, sort_by_counters_desc};
use sort_get_order_hints::{OrderHint, sort_get_order_hints_asc, sort_get_order_hints_desc};
use sort_get_sorted_hints::sort_get_sorted_hints;
use sort_get_split_order_hints::{sort_get_split_order_hints_asc, sort_get_split_order_hints_desc, SplitOrderHints};

use crate::traits::{Empty, is_empty};
use crate::abis::side_effect::{Positioned, Ordered};

pub fn array_to_bounded_vec<T, let N: u32>(array: [T; N]) -> BoundedVec<T, N> where T: Empty + Eq {
    let mut len = 0;
    for elem in array {
        if !is_empty(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len }
}

unconstrained fn filter_array_to_bounded_vec_unsafe<T, let N: u32>(
    arr: [T; N],
    should_propagate: [bool; N]
) -> BoundedVec<T, N> {
    let mut vec = BoundedVec::new();
    for i in 0..N {
        if should_propagate[i] {
            vec.push(arr[i]);
        }
    }
    vec
}

pub fn filter_array_to_bounded_vec<T, let N: u32>(arr: [T; N], should_propagate: [bool; N]) -> BoundedVec<T, N> where T: Eq {
    let vec_hint = filter_array_to_bounded_vec_unsafe(arr, should_propagate);
    let mut verifying_index = 0;

    for i in 0..N {
        if should_propagate[i] {
            assert_eq(arr[i], vec_hint.get(verifying_index));
            verifying_index += 1;
        }
    }
    assert_eq(verifying_index, vec_hint.len());

    vec_hint
}

unconstrained pub fn find_index_hint<T, let N: u32>(array: [T; N], find: T) -> u32 where T: Eq {
    let mut index = 0;
    for i in 0..array.len() {
        if array[i] == find {
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

// Deprecated. Use tests/utils/assert_array_eq instead.
pub fn array_eq<T, let N: u32, let S: u32>(array: [T; N], expected: [T; S]) -> bool where T: Empty + Eq {
    let mut eq = array_length(array) == S;

    for i in 0..S {
        eq &= array[i].eq(expected[i]);
    }

    eq
}

pub fn find_index<T, let N: u32, Env>(array: [T; N], find: fn[Env](T) -> bool) -> u32 {
    let mut index = N;
    for i in 0..N {
        if (index == N) & find(array[i]) {
            index = i;
        }
    }
    index
}

pub fn array_cp<T, let N: u32, let S: u32>(array: [T; N]) -> [T; S] where T: Empty {
    let mut result: [T; S] = [T::empty(); S];
    for i in 0..S {
        result[i] = array[i];
    }
    result
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

pub fn check_permutation<T, let N: u32>(
    original_array: [T; N],
    permuted_array: [T; N],
    original_indexes: [u32; N]
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

pub fn assert_deduped_array<T, let N: u32, Env>(
    original_array: [T; N],
    deduped_array: [T; N],
    run_lengths: [u32; N]
) where T: Positioned + Ordered + Empty + Eq {
    /*
    The original_array here needs to be sorted based on the `position` field of the container,
    *and* a secondary sort based on the `counter` field of the container. 

    For example, the storage slot in the case of public data update requests.
    The run_lengths array should contain the length of each run of the original_array.
    The deduped_array should contain the deduplicated array.

    For example, if the original array is writing `(position,value,counter)`s:
    [ (1,1,1), (1,2,4), (2,3,3), (3,4,2), (3,5,5), (3,6,6), (4,7,8), (4,8,9), (5,9,7), (0,0,0), ... padding with zeros ]
    then run_lengths array is:
    [
        2, // run of 1s
        1, // run of 2
        3, // run of 3s
        2, // run of 4s
        1, // run of 5
        0,
        0,
        ... padding with zeros
    ]
    
    then the deduped_array should be:
    [ (1,2,4), (2,3,3), (3,6,6), (4,8,9), (5,9,7), (0,0,0), ... padding with zeros ]
    */

    let deduped_len = validate_array(deduped_array); // This also makes sure that the array is padded with empty items.  
    let run_lengths_len = array_length(run_lengths); // Don't have to be a "validated" array because non zero padded values don't hurt.  
    assert_eq(deduped_len, run_lengths_len, "Deduped array length does not match number of run lengths");

    let mut seen_empty = false;
    // container at the start of the current run
    let mut start_run_index = 0;
    // the index we are collapsing into
    let mut deduped_index = 0;
    // the length of the current run we are collapsing
    let mut run_counter = run_lengths[deduped_index];
    for i in 0..N {
        let current_container = original_array[i];
        if is_empty(current_container) {
            seen_empty = true;
        } else {
            assert(!seen_empty, "Empty values must be padded to the right");
            assert(run_counter > 0, "Invalid run length");
            assert(
                current_container.position().eq(original_array[start_run_index].position()), "The position of the current container must match the start of the run"
            );
            run_counter -= 1;
            if run_counter == 0 {
                assert(
                    deduped_array[deduped_index].eq(current_container), "The container we are collapsing into must match the current container"
                );
                start_run_index = i + 1;
                deduped_index += 1;
                run_counter = run_lengths[deduped_index];
            } else {
                // we're in a run, so this container must have a lower counter.
                // note we don't check for overflow here, as the run_lengths array must be correct.
                assert(
                    current_container.counter() <= original_array[i + 1].counter(), "Containers in a run must be sorted by counter"
                );
            }
        }
    }

    assert_eq(deduped_index, deduped_len, "Final deduped index does not match deduped array length");
}

mod tests {

    use crate::utils::arrays::assert_deduped_array;
    use crate::abis::side_effect::{Positioned, Ordered};
    use crate::traits::{Empty, is_empty};

    struct TestContainer {
        value: Field,
        position: Field,
        counter: u32,
    }

    impl Positioned for TestContainer {
        fn position(self) -> Field {
            self.position
        }
    }

    impl Ordered for TestContainer {
        fn counter(self) -> u32 {
            self.counter
        }
    }

    impl Empty for TestContainer {
        fn empty() -> Self {
            TestContainer { value: 0, position: 0, counter: 0 }
        }
    }

    impl Eq for TestContainer {
        fn eq(self, other: Self) -> bool {
            self.value.eq(other.value) & self.position.eq(other.position) & self.counter.eq(other.counter)
        }
    }

    #[test]
    fn assert_deduped_array_basic_test() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 2 },
            TestContainer { value: 5, position: 3, counter: 5 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 7, position: 4, counter: 8 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 4 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 6, position: 3, counter: 6 },
            TestContainer { value: 8, position: 4, counter: 9 },
            TestContainer { value: 9, position: 5, counter: 7 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_empty_arrays() {
        let original_array = [TestContainer { value: 0, position: 0, counter: 0 }; 12];
        let deduped_array = [TestContainer { value: 0, position: 0, counter: 0 }; 12];
        let run_lengths = [0; 12];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_no_duplicates() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let deduped_array = original_array;
        let run_lengths = [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_single_run_at_end() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 },
            TestContainer { value: 6, position: 6, counter: 7 },
            TestContainer { value: 7, position: 6, counter: 8 },
            TestContainer { value: 8, position: 6, counter: 9 }
        ];
        let deduped_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 2, counter: 2 },
            TestContainer { value: 3, position: 3, counter: 3 },
            TestContainer { value: 4, position: 4, counter: 4 },
            TestContainer { value: 5, position: 5, counter: 5 },
            TestContainer { value: 8, position: 6, counter: 9 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [1, 1, 1, 1, 1, 3, 0, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test]
    fn assert_deduped_array_all_duplicates() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 1, counter: 3 },
            TestContainer { value: 4, position: 1, counter: 4 },
            TestContainer { value: 5, position: 1, counter: 5 },
            TestContainer { value: 6, position: 1, counter: 6 },
            TestContainer { value: 7, position: 1, counter: 7 },
            TestContainer { value: 8, position: 1, counter: 8 },
            TestContainer { value: 9, position: 1, counter: 9 }
        ];
        let deduped_array = [
            TestContainer { value: 9, position: 1, counter: 9 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [9, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with = "Empty values must be padded to the right")]
    fn test_empty_not_padded_right() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 3, position: 2, counter: 3 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 0, position: 0, counter: 0 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 0, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with = "The position of the current container must match the start of the run")]
    fn test_mismatched_position_in_run() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [3, 1, 1, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with = "The container we are collapsing into must match the current container")]
    fn test_mismatched_deduped_value() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 1, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with = "Deduped array length does not match number of run lengths")]
    fn test_run_lengths_not_zero_padded() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 0, position: 0, counter: 0 }
        ];
        let run_lengths = [2, 1, 1, 1]; // Last element should be 0
        assert_deduped_array(original_array, deduped_array, run_lengths);
    }

    #[test(should_fail_with = "Deduped array length does not match number of run lengths")]
    fn test_deduped_padding_not_zero_padded() {
        let original_array = [
            TestContainer { value: 1, position: 1, counter: 1 },
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 }
        ];
        let deduped_array = [
            TestContainer { value: 2, position: 1, counter: 2 },
            TestContainer { value: 3, position: 2, counter: 3 },
            TestContainer { value: 4, position: 3, counter: 4 },
            TestContainer { value: 1, position: 1, counter: 1 }// Last element should be 0
        ];
        let run_lengths = [2, 1, 1, 0];
        assert_deduped_array(original_array, deduped_array, run_lengths);
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
