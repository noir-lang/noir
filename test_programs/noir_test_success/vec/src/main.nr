use std::collections::vec::Vec;

fn main() { }

#[test]
fn set_updates_values_properly() {
    let mut vec = Vec { slice: &[0, 0, 0, 0, 0] };

    vec.set(0, 42);
    assert_eq(vec.slice, &[42, 0, 0, 0, 0]);

    vec.set(1, 43);
    assert_eq(vec.slice, &[42, 43, 0, 0, 0]);

    vec.set(2, 44);
    assert_eq(vec.slice, &[42, 43, 44, 0, 0]);

    vec.set(1, 10);
    assert_eq(vec.slice, &[42, 10, 44, 0, 0]);

    vec.set(0, 0);
    assert_eq(vec.slice, &[0, 10, 44, 0, 0]);
}

#[test(should_fail)]
fn panics_when_writing_elements_past_end_of_vec() {
    let mut vec = Vec::new();
    vec.set(0, 42);

    // Need to use println to avoid DIE removing the write operation.
    crate::println(vec.get(0));
}
