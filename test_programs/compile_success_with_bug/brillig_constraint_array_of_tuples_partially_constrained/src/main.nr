unconstrained fn pairs_helper(x: Field) -> [(Field, Field); 2] {
    [(x, x), (x, x)]
}

fn main(x: Field) -> pub Field {
    // Safety: only the first pair is constrained
    let pairs = unsafe { pairs_helper(x) };
    assert(pairs[0].0 == x);
    assert(pairs[0].1 == x);
    pairs[1].0 + pairs[1].1
}
