// Test unsafe integer subtraction with underflow: 12 - 2418266113 = 1876701195 modulo 2^32
fn main(mut x: u32, y: u32, z: u32) {
    // x = x.wrapping_sub(y);
    // dep::std::println(x);
    assert(x == z);

    let mut q: u32 = 12;
    q = q.wrapping_sub(2418266113);
    dep::std::println(q);
    assert(q == z);

    // Test constant underflow (regression for #2045)
    // let x = -1 as u4;
    // assert(x == 15);
}
