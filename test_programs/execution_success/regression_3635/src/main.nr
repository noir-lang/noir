// Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
// The circuit should handle properly the growth of the bit size
use dep::std;

fn main() {
    let x: u8 = 0x61;
    let y: u8 = "a".as_bytes()[0];
    assert_eq(x, y);
    assert_eq(x >> 1, y >> 1);
}
