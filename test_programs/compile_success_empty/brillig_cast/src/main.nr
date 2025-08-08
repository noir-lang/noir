// Tests a very simple Brillig function.
//
// The features being tested are cast operations on brillig
fn main() {
    // Safety: testing context
    unsafe {
        field_casts();
        uint_casts();
        int_casts();
        mixed_casts();
    }
}

unconstrained fn field_casts() {
    assert(5 as u8 as Field == 5);
    assert(256 as u8 as Field == 0);
}

unconstrained fn uint_casts() {
    let x: u32 = 300;
    assert(x as u8 == 44);
    assert(x as u32 == 300);
    assert(x as u64 == 300);
}

unconstrained fn int_casts() {
    let x: i32 = 456;
    assert(x as i8 == -(56 as i8));
    assert(x as i64 == 456);
}

unconstrained fn mixed_casts() {
    assert(100 as u32 as i32 as u32 == 100);
    assert(257 as u8 as u32 == 1);
    assert(true as i8 == 1);
}
