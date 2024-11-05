fn main(x: u64) {
    let two: u64 = 2;
    let three: u64 = 3;
    // shifts on constant values
    assert(two << 2 == 8);
    assert((two << 3) / 8 == two);
    assert((three >> 1) == 1);
    // shifts on runtime values
    assert(x << 1 == 128);
    assert(x >> 2 == 16);

    regression_2250();

    //regression for 3481
    assert(x << 63 == 0);

    assert_eq((1 as u64) << 32, 0x0100000000);

    //regression for 6201
    let a: i16 = -769;
    assert_eq(a >> 3, -97);
}

fn regression_2250() {
    let a: u1 = 1 >> 1;
    assert(a == 0);

    let b: u32 = 1 >> 32;
    assert(b == 0);
}
