// `alias` hands its array parameter back, so `c[0] = 9` in `mutator` writes
// into the buffer of `a` in place. `b` is built from the same elements as `a`
// and must not be deduplicated against it: if it is, `b[0]` reads 9 instead
// of `x`.
//
// The recursion keeps `mutator` and `alias` from being inlined, which is
// required for the calls to still exist when constant folding runs.
unconstrained fn main(x: Field, i: u32) -> pub Field {
    let a = [x, x];
    let m = mutator(a, 1);
    let b = [x, x];
    assert(b[i] == x, "b must be a fresh array unaffected by the write");
    b[i] + m
}

unconstrained fn mutator(a: [Field; 2], n: u32) -> Field {
    if n == 0 {
        let mut c = alias(a, 1);
        c[0] = 9;
        c[1]
    } else {
        mutator(a, n - 1)
    }
}

unconstrained fn alias(a: [Field; 2], n: u32) -> [Field; 2] {
    if n == 0 {
        a
    } else {
        alias(a, n - 1)
    }
}
