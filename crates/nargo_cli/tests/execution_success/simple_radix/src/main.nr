// Simple program to test to_radix

fn main(x : Field) {
    let bits = x.to_le_bits(3);
    assert_eq(bits[0], 0);
    assert_eq(bits[1], 1);
    assert_eq(bits[2], 0);
}
