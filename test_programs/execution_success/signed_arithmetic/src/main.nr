fn main(mut x: i32, mut y: i32, z: i32) {
    let mut s1: i8 = 5;
    let mut s2: i8 = 8;
    assert(s1 + s2 == 13);
    assert(x + y == 13);

    s2 = -8;
    y = -y;
    assert(s1 + s2 == -3);
    assert(x + y == -3);

    s1 = -15;
    assert(s1 - s2 == -7);
    assert(z - y == -7);

    s1 = -5;
    s2 = 8;
    x = -x;
    y = -y;
    assert(s1 - s2 == -13);
    assert(x - y == -13);

    s2 = -8;
    y = -y;
    assert(s1 * s2 == 40);
    assert(x * y == 40);

    s1 = 1;
    s2 = -8;
    assert(s1 * s2 == -8);
    assert(x / x * y == -8);
}
