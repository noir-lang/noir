// Simple program to test to_radix

use dep::std;

fn main(x : Field) {
    let bits = x.to_le_bits(3);
    std::assert_eq(bits[0], 0);
    std::assert_eq(bits[1], 1);
    std::assert_eq(bits[2], 0);
}
