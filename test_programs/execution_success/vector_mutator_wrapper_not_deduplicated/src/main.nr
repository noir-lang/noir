// In Brillig, `push_front` writes through its operand's buffer in place when
// the reference count is 1 and there is spare capacity. A function like `wrap`
// that calls a vector mutator on its own parameter must therefore be
// classified as impure: if purity analysis marks it pure-ish, constant folding
// deduplicates the two `producer` calls across the `wrap` call, so `b` aliases
// the buffer `wrap` already corrupted and `b[0]` reads 999 instead of `x`.
//
// The recursion keeps `producer` and `wrap` from being inlined, which is
// required for the calls to still exist when constant folding runs.
unconstrained fn producer(x: Field, n: u32) -> [Field] {
    if n == 0 {
        [x].as_vector()
    } else {
        let v = producer(x, n - 1);
        v.push_back(x)
    }
}

unconstrained fn wrap(v: [Field], d: u32) -> u32 {
    if d == 0 {
        v.push_front(999).len()
    } else {
        wrap(v, d - 1)
    }
}

unconstrained fn compute(x: Field, n: u32) -> (Field, u32) {
    let a = producer(x, n);
    let k = wrap(a, 0);
    let b = producer(x, n);
    (b[0], k)
}

fn main(x: Field, n: u32) -> pub u32 {
    // Safety: the result is checked against `x` below.
    let (first, k) = unsafe { compute(x, n) };
    assert(first == x, "producer must return a fresh vector unaffected by wrap");
    assert(k == n + 2);
    k
}
