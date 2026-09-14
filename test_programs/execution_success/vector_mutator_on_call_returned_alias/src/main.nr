// In Brillig, `push_front` writes through its operand's buffer in place when
// the reference count is 1 and there is spare capacity. `alias` hands its
// parameter straight back, so the vector it returns is the buffer `producer`
// allocated for `a`, and `push_front` on that result writes into `a`. The
// second `producer` call must therefore not be deduplicated against the
// first: if it is, `b` is `a`'s buffer and `b[0]` reads 999 instead of `x`.
//
// The recursion keeps `producer` and `alias` from being inlined, which is
// required for the calls to still exist when constant folding runs.
unconstrained fn producer(x: Field, n: u32) -> [Field] {
    if n == 0 {
        [x].as_vector()
    } else {
        let v = producer(x, n - 1);
        v.push_back(x)
    }
}

unconstrained fn alias(v: [Field], d: u32) -> [Field] {
    if d == 0 {
        v
    } else {
        alias(v, d - 1)
    }
}

unconstrained fn compute(x: Field, n: u32) -> (Field, u32) {
    let a = producer(x, n);
    let k = alias(a, 0).push_front(999).len();
    let b = producer(x, n);
    (b[0], k)
}

fn main(x: Field, n: u32) -> pub u32 {
    // Safety: both results are checked below.
    let (first, k) = unsafe { compute(x, n) };
    assert(first == x, "producer must return a fresh vector unaffected by the push_front");
    assert(k == n + 2);
    k
}
