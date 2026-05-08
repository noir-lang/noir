// Simple program to test to_radix
fn main(x: Field) {
    let bits: [bool; 3] = x.to_le_bits();
    assert(!bits[0]);
    assert(bits[1]);
    assert(!bits[2]);
}
