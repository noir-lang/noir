// `set`, called through a function value, re-points `p` to its own cell,
// so `*x = 6` does not write `a` and the result is 5.
fn set(r: &mut &mut Field) {
    let mut c = 0;
    *r = &mut c;
}

fn g(q: &mut &mut Field, f: fn(&mut &mut Field)) {
    f(q);
}

unconstrained fn h(f: fn(&mut &mut Field)) -> Field {
    let mut a = 0;
    let mut p = &mut a;
    g(&mut p, f);
    let x = p;
    a = 5;
    *x = 6;
    a
}

fn main() {
    // Safety: test
    let v = unsafe { h(set) };
    assert_eq(v, 5);
}
