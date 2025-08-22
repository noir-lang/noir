fn main(x: u64, y: u64, z: i16, u: i64) {
    // runtime shifts on compile-time known values
    assert(64 << y == 128);
    assert(64 >> y == 32);
    // runtime shifts on runtime values
    assert(x << y == 128);
    assert(x >> y == 32);
    // regression tests for issue #8176, superseded by issue #8791
    assert(u >> (x as i64 - 2) == -1);

    // Bit-shift with signed integers
    let mut a: i8 = y as i8;
    let mut b: i8 = x as i8;
    assert(b << 1 == -128);
    assert(b >> 2 == 16);
    assert(b >> a == 32);
    a = -a;
    assert(a << 6 == -64);
    assert(a << -a == -2);

    assert(x >> (x - 1) == 0);

    assert_eq(z >> 3, -97);

    assert(z >> 14 == -1);
    let rhs = (if x as i16 > 14 { 14 } else { x as i16 });
    assert(z >> rhs == -1);
    assert(-z >> rhs == 0);
}
