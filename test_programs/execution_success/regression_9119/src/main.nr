unconstrained fn main(c: bool) -> pub bool {
    let b = if c {
        [(&mut true, 1)]
    } else {
        [(&mut true, 1)]
    };
    *b[0].0
}
