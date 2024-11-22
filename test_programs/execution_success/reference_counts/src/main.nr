fn main() {
    let mut array = [0, 1, 2];
    assert_refcount(array, 1);

    borrow(array, std::mem::array_refcount(array));
    borrow_mut(&mut array, std::mem::array_refcount(array));
    copy_mut(array, std::mem::array_refcount(array));
}

fn borrow(array: [Field; 3], rc_before_call: u32) {
    assert_refcount(array, rc_before_call);
    println(array[0]);
}

fn borrow_mut(array: &mut [Field; 3], rc_before_call: u32) {
    assert_refcount(*array, rc_before_call + 0); // Issue! This should be rc_before_call + 1
    array[0] = 5;
    println(array[0]);
}

fn copy_mut(mut array: [Field; 3], rc_before_call: u32) {
    assert_refcount(array, rc_before_call + 0); // Issue! This should be rc_before_call + 1
    array[0] = 6;
    println(array[0]);
}

fn assert_refcount(array: [Field; 3], expected: u32) {
    let count = std::mem::array_refcount(array);

    // All refcounts are zero when running this as a constrained program
    if std::runtime::is_unconstrained() {
        if count != expected {
            // Brillig doesn't print the actual & expected arguments on assertion failure
            println(f"actual = {count}, expected = {expected}");
        }
        assert_eq(count, expected);
    } else {
        assert_eq(count, 0);
    }
}
