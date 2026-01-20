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
    // Create another just for the sake of an alternative dispatch target.
    let h = || &mut 3;
    let y = *g();
    let z =
        // safety: Passing the pair to unconstrained to show that one can be called as well.
        unsafe { go(g) };
    assert_eq(z, *h() - 1);

    (x, y)
}

unconstrained fn go(f: fn() -> &mut Field) -> Field {
    *f()
}
