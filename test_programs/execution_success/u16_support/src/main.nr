fn main(x: u16) {
    test_u16(x);
    unsafe {
        //@safety: testing context
        test_u16_unconstrained(x);
    }
}

unconstrained fn test_u16_unconstrained(x: u16) {
    test_u16(x)
}

fn test_u16(x: u16) {
    let t1: u16 = 1234;
    let t2: u16 = 4321;
    let t = t1 + t2;

    let t4 = t - t2;
    assert(t4 == t1);

    let mut small_int = x as u16;
    let shift = small_int << (x as u8);
    assert(shift == 8);
    assert(shift >> (x as u8) == small_int);
    assert(shift >> 15 == 0);
    assert(shift << 15 == 0);
}
