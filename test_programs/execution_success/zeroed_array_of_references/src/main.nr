use std::mem::zeroed;

// Each slot of a zeroed array of references must own its own allocation, just like
// the fields of a zeroed tuple do. Writing through one slot must not be observable
// through the others.
fn main() {
    let arr: [&mut Field; 3] = zeroed();
    *arr[1] = 7;
    assert_eq(*arr[0], 0);
    assert_eq(*arr[1], 7);
    assert_eq(*arr[2], 0);

    // The reference can also be nested under a tuple inside the array.
    let nested: [(&mut Field,); 3] = zeroed();
    *nested[1].0 = 42;
    assert_eq(*nested[0].0, 0);
    assert_eq(*nested[1].0, 42);
    assert_eq(*nested[2].0, 0);
}
