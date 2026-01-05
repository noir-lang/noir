// This test is exactly the same as `reference_counts_inliner_0` which uses
// arrays rather than vectors.
// This test exists to make sure that our reference counting debug methods match
// between arrays and vectors.
// We could most likely combine the code for these tests (e.g. using generics),
// but it is simpler to debug isolated tests.
// It should only be necessary to have a test at one inliner setting, as we
// are just checking for discrepancies between the array and vector debugging builtin functions.
// The actual functionality of reference counting is tested with the `reference_counts_*` tests.
// We went with testing at an inliner aggressiveness of zero, as this is generally
// the most useful inliner setting for unconstrained functions.
use std::mem::vector_refcount;

fn main(i: u32) {
    let mut vector = #[0, 1, 2];
    assert_refcount(vector, 1, true);

    borrow(vector, vector_refcount(vector));
    borrow_mut(&mut vector, vector_refcount(vector));
    let _ = copy_mut(vector, vector_refcount(vector));

    borrow_mut_two(&mut vector, &mut vector, vector_refcount(vector));

    let mut u32_vector = #[0, 1, 2];
    let rc1 = vector_refcount(vector);
    let rc2 = vector_refcount(u32_vector);
    borrow_mut_two_separate(&mut vector, &mut u32_vector, rc1, rc2);

    regression_7297();

    // Safety: test
    unsafe { regression_10335(i) };
}

fn borrow(vector: [Field], rc_before_call: u32) {
    assert_refcount(vector, rc_before_call, true);
    println(vector[0]);
}

fn borrow_mut(vector: &mut [Field], rc_before_call: u32) {
    assert_refcount(*vector, rc_before_call, true);
    vector[0] = 3;
    println(vector[0]);
}

// Returns a new vector (a copy) to prevent SSA from optimizing away mutations.
fn copy_mut(mut vector: [Field], rc_before_call: u32) -> [Field] {
    assert_refcount(vector, rc_before_call, true);
    vector = #[4, vector[1], vector[2]];
    println(vector[0]);
    vector
}

fn borrow_mut_two(vector1: &mut [Field], vector2: &mut [Field], rc_before_call: u32) {
    assert_refcount(*vector1, rc_before_call, true);
    assert_refcount(*vector2, rc_before_call + 1, true); // should be a copy
    vector1[0] = 5;
    vector2[0] = 6;
    println(vector1[0]); // vector1 & 2 alias, so this should also print 6
    println(vector2[0]);
}

fn borrow_mut_two_separate(
    vector1: &mut [Field],
    vector2: &mut [u32],
    rc_before_call1: u32,
    rc_before_call2: u32,
) {
    assert_refcount(*vector1, rc_before_call1, true);
    assert_refcount(*vector2, rc_before_call2, true);
    vector1[0] = 7;
    vector2[0] = 8;
    println(vector1[0]);
    println(vector2[0]);
}

fn assert_refcount<T>(vector: [T], mut expected: u32, expect_copy: bool) {
    let count = vector_refcount(vector);

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
    let mut vector: [Field] = #[0, 1, 2];

    let refcount_0 = vector_refcount(vector);
    borrow_mut_two(&mut vector, &mut vector, refcount_0);

    let refcount_1 = vector_refcount(vector);
    let vector_2 = copy_mut(vector, refcount_1 + 1);
    let refcount_2 = vector_refcount(vector);

    assert_eq(vector[0], 6, "the original should not be mutated by copy_mut, only borrow_mut_two");
    assert_eq(vector_2[0], 4, "the copy should have the expected content");

    if std::runtime::is_unconstrained() {
        assert(
            refcount_1 != 0,
            "borrow_mut_two should create a fresh vector and not decrease its RC",
        );

        assert_eq(
            refcount_1,
            2,
            "There is 1 clone after `borrow_mut_two` and before `refcount_1` is defined (cloned before vector_refcount call)",
        );
        assert_eq(
            refcount_2,
            refcount_1 + 3,
            "after refcount_1 we clone once in passing vector to copy_mut, once to vector_refcount after, and once within copy_mut",
        );
    }
}

unconstrained fn regression_10335(i: u32) {
    let a: [Field] = #[0, 1, 2];
    let mut b = a;
    b[i] = 3;

    let rc_a = vector_refcount(a);
    let rc_b = vector_refcount(b);

    // The current expectation is that the original array does not get its RC decremented.
    assert_eq(rc_a, 2);
    assert_eq(rc_b, 1);
}
