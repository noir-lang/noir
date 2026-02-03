unconstrained fn main() {
    let mut inner = [1, 2];
    foo(inner, &mut inner);
}

unconstrained fn foo(a: [Field; 2], b: &mut [Field; 2]) {
    b[0] = 100;
    assert_eq(a, [1, 2]);
    assert_eq(*b, [100, 2]);
}
