struct InvalidXYError {
    x: u64,
    y: u64,
}

fn main(x: u64, y: pub u64) {
    // A raw assertion payload is used to show that noirJS will decode the error payload as a struct.
    assert(x < y, InvalidXYError { x, y });
}
