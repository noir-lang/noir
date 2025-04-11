use crate::ssa::interpreter::{tests::expect_value, NumericValue, Value};

use super::{executes_with_no_errors, expect_error};

#[test]
fn add() {
    let value = expect_value("
        acir(inline) fn main f0 {
          b0():
            v0 = add i32 2, i32 100
            return v0
        }
    ");
    assert_eq!(value, Value::Numeric(NumericValue::I32(102)));
}

#[test]
fn add_overflow() {
    expect_error("
        acir(inline) fn main f0 {
          b0():
            v0 = add u8 200, u8 100
            return v0
        }
    ");
}

#[test]
fn add_unchecked() {
    executes_with_no_errors("
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_add u8 200, u8 100
            return v0
        }
    ");
}

#[test]
fn sub() {}

#[test]
fn sub_underflow() {}

#[test]
fn sub_unchecked() {}

#[test]
fn mul() {}

#[test]
fn mul_overflow() {}

#[test]
fn mul_unchecked() {}

#[test]
fn div() {}

#[test]
fn div_zero() {}

#[test]
fn r#mod() {}

#[test]
fn mod_zero() {}

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
fn decrement_rc() {}

#[test]
fn if_else() {}

#[test]
fn make_array() {}

#[test]
fn noop() {}
