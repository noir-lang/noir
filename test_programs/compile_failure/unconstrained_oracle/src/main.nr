#[oracle(getNoun)]
unconstrained fn external_fn() -> field {
    100 / 5
}

fn main() {
    let x = anon();
    assert(x * 5 == 100);
}
