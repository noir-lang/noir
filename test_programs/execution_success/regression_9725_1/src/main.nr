unconstrained fn main() {
    let mut b = [true];
    let mut c = [false];
    c = b;
    b[0] = !c[0];
    assert_eq(c[0], true);
}
