// This test is exactly the same as `reference_counts_inliner_0` which uses
// arrays rather than lists.
// This test exists to make sure that our reference counting debug methods match
// between arrays and lists.
// We could most likely combine the code for these tests (e.g. using generics),
// but it is simpler to debug isolated tests.
// It should only be necessary to have a test at one inliner setting, as we
// are just checking for discrepancies between the array and list debugging builtin functions.
// The actual functionality of reference counting is tested with the `reference_counts_*` tests.
// We went with testing at an inliner aggressiveness of zero, as this is generally
// the most useful inliner setting for unconstrained functions.
use std::mem::list_refcount;

fn main(i: u32) {
    let mut list = &[0, 1, 2];
    assert_refcount(list, 1, true);

    borrow(list, list_refcount(list));
    borrow_mut(&mut list, list_refcount(list));
    let _ = copy_mut(list, list_refcount(list));

    borrow_mut_two(&mut list, &mut list, list_refcount(list));

    let mut u32_list = &[0, 1, 2];
    let rc1 = list_refcount(list);
    let rc2 = list_refcount(u32_list);
    borrow_mut_two_separate(&mut list, &mut u32_list, rc1, rc2);

    regression_7297();

    // Safety: test
    unsafe { regression_10335(i) };
}

fn borrow(list: [Field], rc_before_call: u32) {
    assert_refcount(list, rc_before_call, true);
    println(list[0]);
}

fn borrow_mut(list: &mut [Field], rc_before_call: u32) {
    assert_refcount(*list, rc_before_call, true);
    list[0] = 3;
    println(list[0]);
}

// Returns a new list (a copy) to prevent SSA from optimizing away mutations.
fn copy_mut(mut list: [Field], rc_before_call: u32) -> [Field] {
    assert_refcount(list, rc_before_call, true);
    list = &[4, list[1], list[2]];
    println(list[0]);
    list
}

fn borrow_mut_two(list1: &mut [Field], list2: &mut [Field], rc_before_call: u32) {
    assert_refcount(*list1, rc_before_call, true);
    assert_refcount(*list2, rc_before_call + 1, true); // should be a copy
    list1[0] = 5;
    list2[0] = 6;
    println(list1[0]); // list1 & 2 alias, so this should also print 6
    println(list2[0]);
}

fn borrow_mut_two_separate(
    list1: &mut [Field],
    list2: &mut [u32],
    rc_before_call1: u32,
    rc_before_call2: u32,
) {
    assert_refcount(*list1, rc_before_call1, true);
    assert_refcount(*list2, rc_before_call2, true);
    list1[0] = 7;
    list2[0] = 8;
    println(list1[0]);
    println(list2[0]);
}

fn assert_refcount<T>(list: [T], mut expected: u32, expect_copy: bool) {
    let count = list_refcount(list);

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
    let mut list: [Field] = &[0, 1, 2];

    let refcount_0 = list_refcount(list);
    borrow_mut_two(&mut list, &mut list, refcount_0);

    let refcount_1 = list_refcount(list);
    let list_2 = copy_mut(list, refcount_1 + 1);
    let refcount_2 = list_refcount(list);

    assert_eq(list[0], 6, "the original should not be mutated by copy_mut, only borrow_mut_two");
    assert_eq(list_2[0], 4, "the copy should have the expected content");

    if std::runtime::is_unconstrained() {
        assert(
            refcount_1 != 0,
            "borrow_mut_two should create a fresh list and not decrease its RC",
        );

        assert_eq(
            refcount_1,
            2,
            "There is 1 clone after `borrow_mut_two` and before `refcount_1` is defined (cloned before list_refcount call)",
        );
        assert_eq(
            refcount_2,
            refcount_1 + 3,
            "after refcount_1 we clone once in passing list to copy_mut, once to list_refcount after, and once within copy_mut",
        );
    }
}

unconstrained fn regression_10335(i: u32) {
    let a: [Field] = &[0, 1, 2];
    let mut b = a;
    b[i] = 3;

    let rc_a = list_refcount(a);
    let rc_b = list_refcount(b);

    // The current expectation is that the original array does not get its RC decremented.
    assert_eq(rc_a, 2);
    assert_eq(rc_b, 1);
}
