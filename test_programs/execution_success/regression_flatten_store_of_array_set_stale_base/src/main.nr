// Both conditional writes to `a` start from the same snapshot `old`, so when both run the
// second one overwrites the first and `a` ends up as `[x, 5]`. Iterating over the references
// keeps `a` in memory until flattening.
fn main(x: Field, c0: bool, c: bool, d: bool) -> pub Field {
    let mut a = [x, 2];
    let mut b = [x, 2];
    for r in [&mut a, &mut b] {
        r[1] = 3;
    }
    if c0 {
        a[0] = 7;
    }
    let old = a;
    if c {
        let mut t = old;
        t[0] = 9;
        a = t;
    }
    if d {
        let mut t = old;
        t[1] = 5;
        a = t;
    }
    a[0] * 100 + a[1] + b[0]
}
