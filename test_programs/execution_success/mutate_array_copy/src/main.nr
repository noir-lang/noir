unconstrained fn main() {
    let mut a = [1, 2];
    foo(&mut a);
}

unconstrained fn foo(a: &mut [Field; 2]) {
    let a_copy = *a;
    a[0] = 100;
    assert_eq(a_copy, [1, 2]);
    assert_eq(*a, [100, 2]);
}
