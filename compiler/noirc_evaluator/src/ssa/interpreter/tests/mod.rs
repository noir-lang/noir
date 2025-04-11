#![cfg(test)]

use acvm::{AcirField, FieldElement};

use crate::{errors::RuntimeError, ssa::interpreter::value::NumericValue};

use super::{Ssa, Value};

mod instructions;

fn executes_with_no_errors(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    assert!(super::interpret(&ssa).is_ok())
}

fn expect_values(src: &str) -> Vec<Value> {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret(&ssa).unwrap()
}

fn expect_value(src: &str) -> Value {
    let mut results = expect_values(src);
    assert_eq!(results.len(), 1);
    results.pop().unwrap()
}

fn expect_error(src: &str) -> RuntimeError {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret(&ssa).unwrap_err()
}

#[test]
fn empty_program() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return
        }
    ";
    executes_with_no_errors(src);
}

#[test]
fn return_all_numeric_constant_types() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return Field 0, u1 1, u8 2, u16 3, u32 4, u64 5, u128 6, i8 -1, i16 -2, i32 -3, i64 -4
        }
    ";
    let returns = expect_values(src);
    assert_eq!(returns.len(), 11);

    assert_eq!(returns[0], Value::Numeric(NumericValue::Field(FieldElement::zero())));
    assert_eq!(returns[1], Value::Numeric(NumericValue::U1(true)));
    assert_eq!(returns[2], Value::Numeric(NumericValue::U8(2)));
    assert_eq!(returns[3], Value::Numeric(NumericValue::U16(3)));
    assert_eq!(returns[4], Value::Numeric(NumericValue::U32(4)));
    assert_eq!(returns[5], Value::Numeric(NumericValue::U64(5)));
    assert_eq!(returns[6], Value::Numeric(NumericValue::U128(6)));
    assert_eq!(returns[7], Value::Numeric(NumericValue::I8(-1)));
    assert_eq!(returns[8], Value::Numeric(NumericValue::I16(-2)));
    assert_eq!(returns[9], Value::Numeric(NumericValue::I32(-3)));
    assert_eq!(returns[10], Value::Numeric(NumericValue::I64(-4)));
}
