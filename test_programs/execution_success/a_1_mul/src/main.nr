// Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
// The circuit should handle properly the growth of the bit size
fn main(mut x: u8, y: u8, z: u8) {
    x *= y;
    x *= x; //144
    x *= x; //20736
    x *= x; //429 981 696
    assert(x == z);
}
