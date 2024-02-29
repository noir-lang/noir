#[test]
fn test_vec_push_pop() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    assert(vec.len == 0);
    vec.push(2);
    assert(vec.len == 1);
    vec.push(4);
    assert(vec.len == 2);
    vec.push(6);
    assert(vec.len == 3);
    let x = vec.pop();
    assert(x == 6);
    assert(vec.len == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
}

#[test]
fn test_vec_extend_from_array() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4]);
    assert(vec.len == 2);
    assert(vec.get(0) == 2);
    assert(vec.get(1) == 4);
}

#[test(should_fail_with="extend_from_array out of bounds")]
fn test_vec_extend_from_array_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2, 4, 6]);
}

#[test(should_fail_with="extend_from_array out of bounds")]
fn test_vec_extend_from_array_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([2]);
    assert(vec.len == 1);
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

#[test(should_fail_with="push out of bounds")]
fn test_vec_push_out_of_bound() {
    let mut vec: BoundedVec<Field, 1> = BoundedVec::new();
    vec.push(1);
    vec.push(2);
}

#[test(should_fail_with="extend_from_bounded_vec out of bounds")]
fn test_vec_extend_from_bounded_vec_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();

    let mut another_vec: BoundedVec<Field, 3> = BoundedVec::new();
    another_vec.extend_from_array([1, 2, 3]);

    vec.extend_from_bounded_vec(another_vec);
}

#[test(should_fail_with="extend_from_bounded_vec out of bounds")]
fn test_vec_extend_from_bounded_vec_twice_out_of_bound() {
    let mut vec: BoundedVec<Field, 2> = BoundedVec::new();
    vec.extend_from_array([1, 2]);

    let mut another_vec: BoundedVec<Field, 1> = BoundedVec::new();
    another_vec.push(3);

    vec.extend_from_bounded_vec(another_vec);
}

#[test]
fn test_vec_any() {
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4, 6]);
    assert(vec.any(|v| v == 2) == true);
    assert(vec.any(|v| v == 4) == true);
    assert(vec.any(|v| v == 6) == true);
    assert(vec.any(|v| v == 3) == false);
}

#[test]
fn test_vec_any_not_default() {
    let default_value = 0;
    let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
    vec.extend_from_array([2, 4]);
    assert(vec.any(|v| v == default_value) == false);
}
