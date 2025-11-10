// This test is exactly the same as `reference_counts_inliner_0` which uses
// arrays rather than slices.
// This test exists to make sure that our reference counting debug methods match
// between arrays and slices.
// We could most likely combine the code for these tests (e.g. using generics),
// but it is simpler to debug isolated tests.
// It should only be necessary to have a test at one inliner setting, as we
// are just checking for discrepancies between the array and slice debugging builtin functions.
// The actual functionality of reference counting is tested with the `reference_counts_*` tests.
// We went with testing at an inliner aggressiveness of zero, as this is generally
// the most useful inliner setting for unconstrained functions.
use std::mem::slice_refcount;

fn main(i: u32) {
    let mut slice = &[0, 1, 2];
    assert_refcount(slice, 1, true);

    borrow(slice, slice_refcount(slice));
    borrow_mut(&mut slice, slice_refcount(slice));
    let _ = copy_mut(slice, slice_refcount(slice));

    borrow_mut_two(&mut slice, &mut slice, slice_refcount(slice));

    let mut u32_slice = &[0, 1, 2];
    let rc1 = slice_refcount(slice);
    let rc2 = slice_refcount(u32_slice);
    borrow_mut_two_separate(&mut slice, &mut u32_slice, rc1, rc2);

    regression_7297();

    // Safety: test
    unsafe { regression_10335(i) };
}

fn borrow(slice: [Field], rc_before_call: u32) {
    assert_refcount(slice, rc_before_call, true);
    println(slice[0]);
}

fn borrow_mut(slice: &mut [Field], rc_before_call: u32) {
    assert_refcount(*slice, rc_before_call, true);
    slice[0] = 3;
    println(slice[0]);
}

// Returns a new slice (a copy) to prevent SSA from optimizing away mutations.
fn copy_mut(mut slice: [Field], rc_before_call: u32) -> [Field] {
    assert_refcount(slice, rc_before_call, true);
    slice = &[4, slice[1], slice[2]];
    println(slice[0]);
    slice
}

fn borrow_mut_two(slice1: &mut [Field], slice2: &mut [Field], rc_before_call: u32) {
    assert_refcount(*slice1, rc_before_call, true);
    assert_refcount(*slice2, rc_before_call + 1, true); // should be a copy
    slice1[0] = 5;
    slice2[0] = 6;
    println(slice1[0]); // slice1 & 2 alias, so this should also print 6
    println(slice2[0]);
}

fn borrow_mut_two_separate(
    slice1: &mut [Field],
    slice2: &mut [u32],
    rc_before_call1: u32,
    rc_before_call2: u32,
) {
    assert_refcount(*slice1, rc_before_call1, true);
    assert_refcount(*slice2, rc_before_call2, true);
    slice1[0] = 7;
    slice2[0] = 8;
    println(slice1[0]);
    println(slice2[0]);
}

fn assert_refcount<T>(slice: [T], mut expected: u32, expect_copy: bool) {
    let count = slice_refcount(slice);

    if expect_copy {
        expected += 1;
    }

    if std::runtime::is_unconstrained() {
        if count != expected {
            println(f"actual = {count}, expected = {expected}");
        }
        assert_eq(count, expected);
    } else {
        assert_eq(count, 0);
    }
}

fn regression_7297() {
    let mut slice: [Field] = &[0, 1, 2];

    let refcount_0 = slice_refcount(slice);
    borrow_mut_two(&mut slice, &mut slice, refcount_0);

    let refcount_1 = slice_refcount(slice);
    let slice_2 = copy_mut(slice, refcount_1 + 1);
    let refcount_2 = slice_refcount(slice);

    assert_eq(slice[0], 6, "the original should not be mutated by copy_mut, only borrow_mut_two");
    assert_eq(slice_2[0], 4, "the copy should have the expected content");

    if std::runtime::is_unconstrained() {
        assert(
            refcount_1 != 0,
            "borrow_mut_two should create a fresh slice and not decrease its RC",
        );

        assert_eq(
            refcount_1,
            2,
            "There is 1 clone after `borrow_mut_two` and before `refcount_1` is defined (cloned before slice_refcount call)",
        );
        assert_eq(
            refcount_2,
            refcount_1 + 3,
            "after refcount_1 we clone once in passing slice to copy_mut, once to slice_refcount after, and once within copy_mut",
        );
    }
}

unconstrained fn regression_10335(i: u32) {
    let a: [Field] = &[0, 1, 2];
    let mut b = a;
    b[i] = 3;

    let rc_a = slice_refcount(a);
    let rc_b = slice_refcount(b);

    // The current expectation is that the original array does not get its RC decremented.
    assert_eq(rc_a, 2);
    assert_eq(rc_b, 1);
}
