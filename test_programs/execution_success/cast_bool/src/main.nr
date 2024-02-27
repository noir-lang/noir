fn main(x: field, y: field) {
    let z = x == y;
    let t = z as u8;
    assert(t == 1);
}

