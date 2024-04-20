fn main(x: field, y: pub u32) {
    let bytes = x.to_be_bytes(y);
    assert(bytes[0] == 0);
}
