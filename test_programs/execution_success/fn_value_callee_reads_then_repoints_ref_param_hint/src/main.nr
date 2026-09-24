// `set` reads the caller's cell out of `*q` before re-pointing `*q` to its own `c`,
// so `*old = 7` writes the caller's `a` and `c` stays 0. The result is 0 + 7 * 10.
fn set(q: &mut &mut Field) -> Field {
    let old = *q;
    let mut c = 0;
    *q = &mut c;
    *old = 7;
    c
}

unconstrained fn h(f: fn(&mut &mut Field) -> Field) -> Field {
    let mut a = 0;
    let mut p = &mut a;
    let r = f(&mut p);
    r + a * 10
}

fn main() {
    // Safety: test
    let v = unsafe { h(set) };
    assert_eq(v, 70);
}
