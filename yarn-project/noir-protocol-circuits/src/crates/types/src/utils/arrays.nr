use crate::abis::new_contract_data::NewContractData;
use dep::std::array;
use crate::utils::bounded_vec::BoundedVec;

// Trait: is_empty
//
// We manually monomorphize the is_empty trait as its a bit simpler 
// and the complexity is localized to this utils package.
//
// The general is_empty trait checks if a data type is is empty,
// and it defines empty for the basic data types as 0.
//
// If a Field is equal to zero, then it is regarded as zero.
// We will go with this definition for now, however it can be problematic 
// if a value can actually be zero. In a future refactor, we can 
// use the optional type for safety. Doing it now would lead to a worse devex
// and would make it harder to sync up with the cpp code.
pub fn is_empty_array<T>(array: [Field; T]) -> bool {
    array.all(|elem| is_empty(elem))
}

fn is_empty(value: Field) -> bool {
    value == 0
}

pub fn is_empty_struct_array<T, N>(array: [T; N], is_empty: fn(T) -> bool) -> bool {
    array.all(|elem| is_empty(elem))
}

// Replace it with `array_to_bounded_vec` when trait is available.
pub fn struct_array_to_bounded_vec<T, N>(
    array: [T; N],
    is_empty_item: fn(T) -> bool,
    empty_item: T
) -> BoundedVec<T, N> {
    let mut len = 0;
    for elem in array {
        if !is_empty_item(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len, empty_value: empty_item }
}

pub fn array_to_bounded_vec<N>(array: [Field; N]) -> BoundedVec<Field, N> {
    let mut len = 0;
    for elem in array {
        if !is_empty(elem) {
            len += 1;
        }
    }

    BoundedVec { storage: array, len, empty_value: 0 }
}

// Routine which validates that all zero values of an array form a contiguous region at the end, i.e.,
// of the form: [*,*,*...,0,0,0,0] where any * is non-zero. Note that a full array of non-zero values is
// valid.
//
// TODO(David): A bit of a nit, we use is_empty, but we are actually doing is_zero.
// TODO: Concretely, are we validating empty or zero regions?
// 
// TODO: We can possibly optimize this by taking advantage of the fact that adding 0 does not 
// TODO: change a running sum.
pub fn validate_array<T>(array: [Field; T]) {
    let array_length = array.len();

    let mut first_zero_pos = array_length;
    let mut last_non_zero_pos = 0;
    // TODO: using is_empty here is a bit wasteful
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

// TODO: Replace it with `validate_array`
pub fn validate_struct_array<T, N>(array: [T; N], is_empty_item: fn(T) -> bool) {
    let array_length = array.len();
    let mut first_zero_pos = array_length;
    let mut last_non_zero_pos = 0;
    for i in 0..array_length {
        let is_empty = is_empty_item(array[i]);
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
pub fn array_length<N>(array: [Field; N]) -> Field {
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

// TODO: Replace it with `array_length`.
pub fn struct_array_length<T, N>(array: [T; N], is_empty_item: fn(T) -> bool) -> Field {
    let mut length = 0;
    let mut end = false;
    for elem in array {
        end |= is_empty_item(elem);
        if !end {
            length += 1;
        }
    }
    length
}

pub fn array_eq<N, S>(array: [Field; N], expected: [Field; S]) -> bool {
    let mut eq = array_length(array) == S;
    for i in 0..S {
        eq = eq & (array[i] == expected[i]);
    }
    eq
}

// TODO: Replace it with `array_eq`. Trait: eq, is_empty
pub fn struct_array_eq<T, N, S>(
    array: [T; N],
    expected: [T; S],
    eq: fn(T, T) -> bool,
    is_empty_item: fn(T) -> bool
) -> bool {
    let length = struct_array_length(array, is_empty_item);
    let mut ret = length == S;
    for i in 0..S {
        ret &= eq(array[i], expected[i]);
    }
    ret
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
