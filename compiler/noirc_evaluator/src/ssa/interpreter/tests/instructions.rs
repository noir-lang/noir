use crate::ssa::interpreter::{NumericValue, Value, tests::expect_value};

use super::{executes_with_no_errors, expect_error};

#[test]
fn add() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add i32 2, i32 100
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I32(102)));
}

/// TODO: Replace panic with error
#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn add_overflow() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add u8 200, u8 100
            return v0
        }
    ",
    );
}

#[test]
fn add_unchecked() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_add u8 200, u8 100
            return v0
        }
    ",
    );
}

#[test]
fn sub() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub i32 10101, i32 101
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I32(10000)));
}

/// TODO: Replace panic with error
#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn sub_underflow() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub i8 -120, i8 10
            return v0
        }
    ",
    );
}

#[test]
fn sub_unchecked() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_sub i8 3, i8 10
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I8(-7)));
}

#[test]
fn mul() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul u64 2, u64 100
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::U64(200)));
}

/// TODO: Replace panic with error
#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn mul_overflow() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul u8 128, u8 2
            return v0
        }
    ",
    );
}

#[test]
fn mul_unchecked() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_mul u8 128, u8 2
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::U8(0)));
}

#[test]
fn div() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = div i16 128, i16 2
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I16(64)));
}

/// TODO: Replace panic with error
#[test]
#[should_panic(expected = "Field division by zero")]
fn div_zero() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = div Field 12, Field 0
            return v0
        }
    ",
    );
}

#[test]
fn r#mod() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mod i64 5, i64 3
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I64(2)));
}

/// TODO: Replace panic with error
#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn mod_zero() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mod u8 12, u8 0
            return v0
        }
    ",
    );
}

#[test]
fn eq() {}

#[test]
fn lt() {}

#[test]
fn and() {}

#[test]
fn or() {}

#[test]
fn xor() {}

#[test]
fn shl() {}

#[test]
fn shl_overflow() {}

#[test]
fn shr() {}

#[test]
fn cast() {}

#[test]
fn not() {}

#[test]
fn truncate() {}

#[test]
fn constrain() {}

#[test]
fn constrain_disabled_by_enable_side_effects() {}

#[test]
fn constrain_not_equal() {}

#[test]
fn constrain_not_equal_disabled_by_enable_side_effects() {}

#[test]
fn range_check() {}

#[test]
fn range_check_disabled_by_enable_side_effects() {}

#[test]
fn call() {}

#[test]
fn allocate() {}

#[test]
fn load() {}

#[test]
fn store() {}

#[test]
fn enable_side_effects_if() {}

#[test]
fn array_get() {}

#[test]
fn array_get_disabled_by_enable_side_effects() {}

#[test]
fn array_set() {}

#[test]
fn array_set_disabled_by_enable_side_effects() {}

#[test]
fn increment_rc() {}

#[test]
fn increment_rc_disabled_in_acir() {}

#[test]
fn decrement_rc() {}

#[test]
fn decrement_rc_disabled_in_acir() {}

#[test]
fn if_else() {}

#[test]
fn make_array() {}

// TODO: Add SSA parser support for Noop
// #[test]
// fn noop() {
//     executes_with_no_errors(
//         "
//         acir(inline) fn main f0 {
//           b0():
//             noop
//             noop
//             noop
//             return
//         }
//     ",
//     );
// }
