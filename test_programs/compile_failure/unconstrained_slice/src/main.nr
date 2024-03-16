unconstrained fn uncon_slice() -> [Field] {
    let mut slice: [Field] = [0; 2];

    let mut new_slice = slice.push_back(6);
    new_slice
}

fn main() {
    let e = uncon_slice();
    e
}
