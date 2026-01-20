fn main(c: bool) -> pub (Field, Field) {
    // We are creating lambdas which turn into (constrained, unconstrained) pairs,
    // but the return value is not valid when going from unconstrained to constrained.
    // We expect the unconstrained lambda to be eliminated.
    let f = std::hint::black_box(|| @[1, 2, 3]);
    let v = if c { f() } else { @[4, 5] };
    let x = v[0];

    // Trying to choose between lambdas that return references would currently panic
    // in `remove_if_else` (see #11268), but we can deal with one
    let g = std::hint::black_box(|| &mut 2);
    let y = *g();

    // Create a version that only has unconstrained variants,
    // to show that it can be passed to an unconstrained function.
    let h: unconstrained fn() -> &mut Field = || &mut 3;

    // safety: Passing the pair to unconstrained to show that one can be called as well.
    unsafe {
        let n = call_and_deref(g);
        let m = call_and_deref(h);
        assert_eq(n + 1, m);
    };

    (x, y)
}

unconstrained fn call_and_deref(f: fn() -> &mut Field) -> Field {
    *f()
}
