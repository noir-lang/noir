// Simple program to test to_radix

use dep::std;

fn main(x : Field) {
    let bits = x.to_le_bits(3);
    assert(bits[0] == 0);
    assert(bits[1] == 1);
    assert(bits[2] == 0);
}
