// Regression test for a bug in the ownership pass: when a nested array is
// passed alongside one of its inner arrays (`foo(a, a[1])`), the inner array
// `a[1]` was incorrectly marked as a "last move" and not cloned. The inner
// element aliased the slot inside the cloned outer `a`, so a mutation of `b`
// inside `foo` leaked back through the outer argument.
//
// The same pattern applies when the inner array is reached through a struct
// field (`foo(a, a[1].arr)`).
//
// The `println` calls before each `assert_eq` observe the runtime value of
// `a`, preventing constant folding from masking the aliasing.

struct Entry {
    arr: [bool; 1],
}

#[inline_never]
unconstrained fn mutate_flat(a: [[bool; 1]; 2], mut b: [bool; 1]) {
    b[0] = !b[0];
    println(b);
    println(a);
    assert_eq(a, [[true], [false]]);
    assert_eq(b, [true]);
}

#[inline_never]
unconstrained fn mutate_field(a: [Entry; 2], mut b: [bool; 1]) {
    b[0] = !b[0];
    println(b);
    println(a);
    assert_eq(a[0].arr, [true]);
    assert_eq(a[1].arr, [false]);
    assert_eq(b, [true]);
}

unconstrained fn main() {
    let flat = [[true], [false]];
    mutate_flat(flat, flat[1]);

    let fields = [Entry { arr: [true] }, Entry { arr: [false] }];
    mutate_field(fields, fields[1].arr);
}
