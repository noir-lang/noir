#[test]
fn test_vec_new_good() {
    good();
}

// docs:start:new_example
fn good() -> BoundedVec<Field, 10> {
    // Ok! MaxLen is specified with a type annotation
    let v1: BoundedVec<Field, 3> = BoundedVec::new();
    let v2 = BoundedVec::new();

    // Ok! MaxLen is known from the type of `good`'s return value
    v2
}
// docs:end:new_example

#[test]
fn test_vec_push_pop() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    assert(vec.len() == 0);
    vec.push(2);
    assert(vec.len() == 1);
    vec.push(4);
    assert(vec.len() == 2);
    vec.push(6);
    assert(vec.len() == 3);
    let x = vec.pop();
    assert(x == 6);
    assert(vec.len() == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
}

#[test]
fn test_vec_get_unchecked<N>() {
    let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
    vec.extend_from_array([1, 2, 3, 4]);
    let sum = sum_of_first_three(vec);
    assert_eq(sum, 6);
}

// docs:start:get_unchecked_example
fn sum_of_first_three<let N: u32>(v: BoundedVec<u32, N>) -> u32 {
    // Always ensure the length is larger than the largest
    // index passed to get_unchecked
    assert(v.len() > 2);
    let first = v.get_unchecked(0);
    let second = v.get_unchecked(1);
    let third = v.get_unchecked(2);
    first + second + third
}
// docs:end:get_unchecked_example

#[test(should_fail)]
// docs:start:set_unchecked_example
fn set_unchecked_example() {
    let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
    vec.extend_from_array([1, 2]);

    // Here we're safely writing within the valid range of `vec`
    // `vec` now has the value [42, 2]
    vec.set_unchecked(0, 42);

    // We can then safely read this value back out of `vec`.
    // Notice that we use the checked version of `get` which would prevent reading unsafe values.
    assert_eq(vec.get(0), 42);

    // We've now written past the end of `vec`.
    // As this index is still within the maximum potential length of `v`,
    // it won't cause a constraint failure.
    vec.set_unchecked(2, 42);
    println(vec);

    // This will write past the end of the maximum potential length of `vec`,
    // it will then trigger a constraint failure.
    vec.set_unchecked(5, 42);
    println(vec);
}
// docs:end:set_unchecked_example

#[test(should_fail_with = "push out of bounds")]
fn push_docs_example() {
    // docs:start:bounded-vec-push-example
    let mut v: BoundedVec<Field, 2> = BoundedVec::new();

    v.push(1);
    v.push(2);

    // Panics with failed assertion "push out of bounds"
    v.push(3);
    // docs:end:bounded-vec-push-example
}

#[test]
fn pop_docs_example() {
    // docs:start:bounded-vec-pop-example
    let mut v: BoundedVec<Field, 2> = BoundedVec::new();
    v.push(1);
    v.push(2);

    let two = v.pop();
    let one = v.pop();

    assert(two == 2);
    assert(one == 1);
    // error: cannot pop from an empty vector
    // let _ = v.pop();
    // docs:end:bounded-vec-pop-example
}

#[test]
fn len_docs_example() {
    // docs:start:bounded-vec-len-example
    let mut v: BoundedVec<Field, 4> = BoundedVec::new();
    assert(v.len() == 0);

    v.push(100);
    assert(v.len() == 1);

    v.push(200);
    v.push(300);
    v.push(400);
    assert(v.len() == 4);

    let _ = v.pop();
    let _ = v.pop();
    assert(v.len() == 2);
    // docs:end:bounded-vec-len-example
}

#[test]
fn max_len_docs_example() {
    // docs:start:bounded-vec-max-len-example
    let mut v: BoundedVec<Field, 5> = BoundedVec::new();

    assert(v.max_len() == 5);
    v.push(10);
    assert(v.max_len() == 5);
    // docs:end:bounded-vec-max-len-example
}

#[test]
fn set_len_docs_example() {
    // docs:start:bounded-vec-set-len-example
    let mut v: BoundedVec<Field, 10> = BoundedVec::from_array([1, 2, 3, 4, 5]);

    v.set_unchecked(5, 6);
    v.set_unchecked(6, 7);
    v.set_unchecked(7, 8);
    assert(v.len() == 5);
    v.set_len(9);
    assert(v.len() == 9);
    // docs:end:bounded-vec-set-len-example
}

#[test]
fn storage_docs_example() {
    // docs:start:bounded-vec-storage-example
    let mut v: BoundedVec<Field, 5> = BoundedVec::new();

    assert(v.storage() == [0, 0, 0, 0, 0]);

    v.push(57);
    assert(v.storage() == [57, 0, 0, 0, 0]);
    // docs:end:bounded-vec-storage-example
}

#[test]
fn test_vec_extend_from_array() {
    // docs:start:bounded-vec-extend-from-array-example
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4]);

    assert(vec.len() == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
    // docs:end:bounded-vec-extend-from-array-example
}

#[test]
fn test_vec_extend_from_bounded_vec() {
    // docs:start:bounded-vec-extend-from-bounded-vec-example
    let mut v1: BoundedVec<Field, 5> = BoundedVec::new();
    let mut v2: BoundedVec<Field, 7> = BoundedVec::new();

    v2.extend_from_array([1, 2, 3]);
    v1.extend_from_bounded_vec(v2);

    assert(v1.storage() == [1, 2, 3, 0, 0]);
    assert(v2.storage() == [1, 2, 3, 0, 0, 0, 0]);
    // docs:end:bounded-vec-extend-from-bounded-vec-example
}

#[test(should_fail_with = "extend_from_array out of bounds")]
fn test_vec_extend_from_array_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2, 4, 6]);
}

#[test(should_fail_with = "extend_from_array out of bounds")]
fn test_vec_extend_from_array_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2]);
    assert(vec.len() == 1);
    vec.extend_from_array([4, 6]);
}

#[test(should_fail)]
fn test_vec_get_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2, 4]);
    let _x = vec.get(2);
}

#[test(should_fail)]
fn test_vec_get_not_declared() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2]);
    let _x = vec.get(1);
}

#[test(should_fail)]
fn test_vec_get_uninitialized() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    let _x = vec.get(0);
}

#[test(should_fail_with = "push out of bounds")]
fn test_vec_push_out_of_bound() {
    let mut vec: BoundedVec<Field, 1> = BoundedVec::new();
    vec.push(1);
    vec.push(2);
}

#[test(should_fail_with = "extend_from_bounded_vec out of bounds")]
fn test_vec_extend_from_bounded_vec_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();

    let mut another_vec: BoundedVec<Field, 3> = BoundedVec::new();
    another_vec.extend_from_array([1, 2, 3]);

    vec.extend_from_bounded_vec(another_vec);
}

#[test(should_fail_with = "extend_from_bounded_vec out of bounds")]
fn test_vec_extend_from_bounded_vec_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([1, 2]);

    let mut another_vec: BoundedVec<Field, 1> = BoundedVec::new();
    another_vec.push(3);

    vec.extend_from_bounded_vec(another_vec);
}

#[test]
fn test_vec_any() {
    // docs:start:bounded-vec-any-example
    let mut v: BoundedVec<u32, 3> = BoundedVec::new();
    v.extend_from_array([2, 4, 6]);

    let all_even = !v.any(|elem: u32| elem % 2 != 0);
    assert(all_even);
    // docs:end:bounded-vec-any-example
}

#[test]
fn test_vec_any_not_default() {
    let default_value = 0;
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4]);
    assert(!vec.any(|v| v == default_value));
}
