// Tests that 257 truncated to 8 bits is 2. Should fail.
fn main(x: Field, y: u8) {
    let z = x as u8;
    assert(z == y);
}
