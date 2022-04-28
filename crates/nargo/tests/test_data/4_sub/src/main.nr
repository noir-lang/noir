// Test unsafe integer substraction with underflow: 12 - 2418266113 = 1876701195 modulo 2^32

fn main(mut x : u32, y : u32, z : u32) {
    x = x - y;
    constrain (x) == z;
}
