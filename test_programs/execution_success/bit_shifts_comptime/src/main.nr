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

    assert_eq(1_u64 << 32, 0x0100000000);

    //regression for 6201
    let a: i16 = -769;
    assert_eq(a >> 3, -97);

    regression_8310();

    //regression 8791
    assert(-(x as i64) >> 62 == -1);
    assert(a >> 14 == -1);
}

fn regression_2250() {
    let b: u32 = 1 >> 31;
    assert(b == 0);
}

fn regression_8310() {
    let x: i64 = -356710612598522715;
    let b = x >> 62;
    assert(b == -1);
}
