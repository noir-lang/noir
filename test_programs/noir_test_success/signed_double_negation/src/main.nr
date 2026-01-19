//! We restrict the value of x to not be i<bitsize>::MIN to ensure that `-x` is a valid `i<bitsize>`

global I8_MIN: i8 = -0x80;
global I16_MIN: i16 = -0x8000;
global I32_MIN: i32 = -0x80000000;
global I64_MIN: i64 = -0x8000000000000000;

#[test]
fn double_negate_i8(x: i8) {
    if x != I8_MIN {
        assert_eq(x, -(-x));
    }
}

#[test]
fn double_negate_i16(x: i16) {
    if x != I16_MIN {
        assert_eq(x, -(-x));
    }
}

#[test]
fn double_negate_i32(x: i32) {
    if x != I32_MIN {
        assert_eq(x, -(-x));
    }
}

#[test]
fn double_negate_i64(x: i64) {
    if x != I64_MIN {
        assert_eq(x, -(-x));
    }
}
