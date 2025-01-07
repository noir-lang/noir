use std::mem::array_refcount;

fn main() {
    let mut array = [0, 1, 2];
    assert_refcount(array, 1);

    borrow(array, array_refcount(array));
    borrow_mut(&mut array, array_refcount(array));
    copy_mut(array, array_refcount(array));

    borrow_mut_two(&mut array, &mut array, array_refcount(array));

    let mut u32_array = [0, 1, 2];
    let rc1 = array_refcount(array);
    let rc2 = array_refcount(u32_array);
    borrow_mut_two_separate(&mut array, &mut u32_array, rc1, rc2);
}

fn borrow(array: [Field; 3], rc_before_call: u32) {
    assert_refcount(array, rc_before_call);
    println(array[0]);
}

fn borrow_mut(array: &mut [Field; 3], rc_before_call: u32) {
    // Optimization: inc_rc isn't needed since there is only one array (`array`)
    // of the same type that `array` can be modified through
    assert_refcount(*array, rc_before_call + 0);
    array[0] = 3;
    println(array[0]);
}

fn copy_mut(mut array: [Field; 3], rc_before_call: u32) {
    assert_refcount(array, rc_before_call + 1);
    array[0] = 4;
    println(array[0]);
}

/// Borrow the same array mutably through both parameters, inc_rc is necessary here, although
/// only one is needed to bring the rc from 1 to 2.
fn borrow_mut_two(array1: &mut [Field; 3], array2: &mut [Field; 3], rc_before_call: u32) {
    assert_refcount(*array1, rc_before_call + 1);
    assert_refcount(*array2, rc_before_call + 1);
    array1[0] = 5;
    array2[0] = 6;
    println(array1[0]); // array1 & 2 alias, so this should also print 6
    println(array2[0]);
}

/// Borrow a different array: we should be able to reason that these types cannot be mutably
/// aliased since they're different types so we don't need any inc_rc instructions.
fn borrow_mut_two_separate(
    array1: &mut [Field; 3],
    array2: &mut [u32; 3],
    rc_before_call1: u32,
    rc_before_call2: u32,
) {
    assert_refcount(*array1, rc_before_call1 + 0);
    assert_refcount(*array2, rc_before_call2 + 0);
    array1[0] = 7;
    array2[0] = 8;
    println(array1[0]);
    println(array2[0]);
}

fn assert_refcount<T>(array: [T; 3], expected: u32) {
    let count = array_refcount(array);

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
