unconstrained fn half_helper(x: Field) -> Field {
    x / 2
}

// The helper leaves constraining the result to its callers.
fn half_unchecked(x: Field) -> Field {
    // Safety: callers must constrain the result
    unsafe { half_helper(x) }
}

fn main(a: Field, b: Field) -> pub Field {
    let p = half_unchecked(a);
    assert_eq(p * 2, a);
    // Same innermost location as `p`, never constrained.
    let q = half_unchecked(b);
    p + q
}
