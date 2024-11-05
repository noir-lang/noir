fn main(x: u64, y: u8, z: i16) {
    // runtime shifts on compile-time known values
    assert(64 << y == 128);
    assert(64 >> y == 32);
    // runtime shifts on runtime values
    assert(x << y == 128);
    assert(x >> y == 32);

    // Bit-shift with signed integers
    let mut a: i8 = y as i8;
    let mut b: i8 = x as i8;
    assert(b << 1 == -128);
    assert(b >> 2 == 16);
    assert(b >> y == 32);
    a = -a;
    assert(a << 7 == -128);
    assert(a << y == -2);

    assert(x >> (x as u8) == 0);

    assert_eq(z >> 3, -97);
}
