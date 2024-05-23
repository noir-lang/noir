fn foo<N>(_x: [Field; N]) -> [Field; N+1] {
    [0; N+2]
}

fn main() {
    assert(0 == 0);
}
