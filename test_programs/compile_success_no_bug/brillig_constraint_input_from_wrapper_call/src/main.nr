unconstrained fn halve_helper(x: Field) -> Field {
    x / 2
}

fn halve(x: Field) -> Field {
    // Safety: constrained below
    let y = unsafe { halve_helper(x) };
    assert(y * 2 == x);
    y
}

unconstrained fn third_helper(x: Field) -> Field {
    x / 3
}

fn main(a: Field, b: Field) -> pub Field {
    let p = halve(a);
    let q = halve(b);
    // Safety: constrained below
    let r = unsafe { third_helper(q) };
    assert(r * 3 == q);
    p + r
}
