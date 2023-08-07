// Tests a very simple Brillig function.
// 
// The features being tested are cast operations on brillig
fn main()  {
    bool_casts();
    field_casts();
    uint_casts();
    int_casts();
    mixed_casts();
}

unconstrained fn bool_casts() {
    assert(false == 0 as bool);
    assert(true == 1 as bool);
    assert(true == 3 as bool);
}

unconstrained fn field_casts() {
    assert(5 as u8 as Field == 5);
    assert(16 as u4 as Field == 0);
}

unconstrained fn uint_casts() {
    let x: u32 = 100;
    assert(x as u2 == 0);
    assert(x as u4 == 4);
    assert(x as u6 == 36);
    assert(x as u8 == 100);
    assert(x as u64 == 100);
    assert(x as u126 == 100);
}

unconstrained fn int_casts() {
    let x: i32 = 100;
    assert(x as i2 == 0);
    assert(x as i4 == 4);
    assert(x as i6 == -28 as i6);
    assert(x as i8 == 100);
    assert(x as i8 == 100);
    assert(x as i8 == 100);
}


unconstrained fn mixed_casts() {
    assert(100 as u32 as i32 as u32 == 100);
    assert(13 as u4 as i2 as u32 == 1);
    assert(15 as u4 as i2 as u32 == 3);
    assert(1 as u8 as bool == true);
    assert(true as i8 == 1);
}
