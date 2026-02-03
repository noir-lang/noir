unconstrained fn u_shift(a: u128, b: u128) -> Field {
    shift(a, b)
}

fn shift(a: u128, b: u128) -> Field {
    let mut c = 0;

    let rhs = if (b > 127) { 127 } else { b };
    c += (a << rhs) as Field;
    c += (a >> rhs) as Field;
    c += (a << 127) as Field;
    c += (a >> 127) as Field;

    let a = a as u64;
    let b = b as u64;
    let rhs = if (b > 63) { 63 } else { b };
    c += (a << rhs) as Field;
    c += (a >> rhs) as Field;
    c += (a << 63) as Field;
    c += (a >> 63) as Field;

    let a = a as u32;
    let b = b as u32;
    let rhs = if (b > 31) { 31 } else { b };
    c += (a << rhs) as Field;
    c += (a >> rhs) as Field;
    c += (a << 31) as Field;
    c += (a >> 31) as Field;

    let a = a as u16;
    let b = b as u16;
    let rhs = if (b > 15) { 15 } else { b };
    c += (a << rhs) as Field;
    c += (a >> rhs) as Field;
    c += (a << 15) as Field;
    c += (a >> 15) as Field;

    let a = a as u8;
    let b = b as u8;
    let rhs = if (b > 7) { 7 } else { b };
    c += (a << rhs) as Field;
    c += (a >> rhs) as Field;
    c += (a << 7) as Field;
    c += (a >> 7) as Field;

    let a = a as i64;
    let b = b as i64;
    let b = if b <= 0 { 1 } else { b };
    let rhs = if (b > 62) { 62 } else { b };
    c += (a << rhs) as u64 as Field;
    c += (a >> rhs) as u64 as Field;
    c += (a << 62) as u64 as Field;
    c += (a >> 62) as u64 as Field;

    let a = a as i32;
    let b = b as i32;
    let b = if b <= 0 { 1 } else { b };
    let rhs = if (b > 30) { 30 } else { b };
    c += (a << rhs) as u64 as Field;
    c += (a >> rhs) as u64 as Field;
    c += (a << 30) as u64 as Field;
    c += (a >> 30) as u64 as Field;

    let a = a as i16;
    let b = b as i16;
    let b = if b <= 0 { 1 } else { b };
    let rhs = if (b > 14) { 14 } else { b };
    c += (a << rhs) as u64 as Field;
    c += (a >> rhs) as u64 as Field;
    c += (a << 14) as u64 as Field;
    c += (a >> 14) as u64 as Field;

    let a = a as i8;
    let b = b as i8;
    let b = if b <= 0 { 1 } else { b };
    let rhs = if (b > 6) { 6 } else { b };
    c += (a << rhs) as u64 as Field;
    c += (a >> rhs) as u64 as Field;
    c += (a << 6) as u64 as Field;
    c += (a >> 6) as u64 as Field;
    c
}

#[test]
fn test_bit_shifts00() {
    let a = unsafe { u_shift(4, 5) };
    assert_eq(shift(4, 5), a);
    let a = unsafe { u_shift(0, 132) };
    assert_eq(shift(0, 132), a);
    let a = unsafe { u_shift(234234, 42) };
    assert_eq(shift(234234, 42), a);
    let a = unsafe { u_shift(2434, 11) };
    assert_eq(shift(2434, 11), a);
}

#[test]
fn test_signed_shr_i8_rhs_bit_size_minus_one() {
    assert_eq(2_i8 >> 7, 0)
}

#[test]
fn test_signed_shl_i8_rhs_bit_size_minus_one() {
    assert_eq(2_i8 << 7, 0)
}

#[test]
fn test_signed_shr_i32_rhs_bit_size_minus_one() {
    assert_eq(2_i32 >> 31, 0)
}

#[test]
fn test_signed_shl_i32_rhs_bit_size_minus_one() {
    assert_eq(2_i32 << 31, 0)
}

#[test(should_fail)]
fn test_bit_shifts_overflow1() -> i8 {
    2_i8 >> 8
}

#[test(should_fail)]
fn test_bit_shifts_overflow2() -> u8 {
    2_u8 >> 8
}

#[test(should_fail)]
fn test_bit_shifts_overflow3() -> i32 {
    2_i32 >> 32
}

#[test(should_fail)]
fn test_bit_shifts_overflow4() -> u32 {
    2_u32 >> 32
}

#[test(should_fail)]
fn test_bit_shifts_overflow5() -> i32 {
    2_i32 >> -1
}
