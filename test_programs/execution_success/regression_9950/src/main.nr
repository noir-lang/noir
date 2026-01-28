unconstrained fn main() {
    let x = foo(1);
    assert_eq(x, 0);
}

unconstrained fn foo(mut a: Field) -> Field {
    let mut b: &mut Field = (&mut 0xdeadbeef);
    // At this point:
    // a is 0
    // b points to 0xdeadbeef
    for _ in 0..2 {
        a = 0;
        a = *b;
        b = (&mut a);

        // Now b is a pointer to a.
        // In the next iteration, `a = 0` will happen.
        // Because b is a pointer to a, at that point b will be a pointer
        // to a, which points to zero.
        // Then when `a = *b` is executed, a will be zero.
    }
    a
}
