use dep::std;

// Test unsafe integer subtraction with underflow: 12 - 2418266113 = 1876701195 modulo 2^32
fn main(mut x: u32, y: u32, z: u32) {
    x -= y;
    std::assert_eq(x, z);

    // Test constant underflow (regression for #2045)
    let x = -1 as u4;
    std::assert_eq(x, 15);
}
