fn main(c: bool) -> pub (Field, Field) {
    // We are creating lambdas which turn into (constrained, unconstrained) pairs,
    // but the return value is not valid when going from unconstrained to constrained.
    // We expect the unconstrained lambda to be eliminated.
    let f = std::hint::black_box(|| @[1, 2, 3]);
    let v = if c { f() } else { @[4, 5] };
    let x = v[0];

    // TODO(#11268): Trying to choose between lambdas that return references
    // would currently panic in `remove_if_else`, but we can deal with one,
    // because we know we won't be calling the Brillig one from ACIR.
    let g = std::hint::black_box(|| &mut 2);
    let y = *g();

    (x, y)
}
