// `alias` hands its array parameter straight back, so in Brillig the array it
// returns is the buffer `producer` allocated for `a`. `a` is passed to
// `write_through_alias` as its last use, with a reference count of 1, so the
// `w[i] = 99` write lands in that buffer in place. The second `producer` call
// must therefore not be deduplicated against the first: if it is, `b` is
// `a`'s buffer and `b[0]` reads 99 instead of 1.
#[inline_never]
unconstrained fn producer() -> [Field; 3] {
    [1, 2, 3]
}

#[inline_never]
unconstrained fn alias(v: [Field; 3]) -> [Field; 3] {
    v
}

#[inline_never]
unconstrained fn write_through_alias(v: [Field; 3], i: u32) -> Field {
    let mut w = alias(v);
    w[i] = 99;
    w[0]
}

unconstrained fn compute(i: u32) -> (Field, Field) {
    let a = producer();
    let x = write_through_alias(a, i);
    let b = producer();
    (b[0], x)
}

fn main(i: u32) -> pub Field {
    // Safety: both results are checked below.
    let (first, x) = unsafe { compute(i) };
    assert(first == 1, "producer must return a fresh array unaffected by the write");
    assert((i == 0) | (x == 1));
    first
}
