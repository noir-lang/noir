#![cfg(test)]

use std::sync::Arc;

use acvm::{AcirField, FieldElement};

use crate::{
    errors::RuntimeError,
    ssa::{interpreter::value::NumericValue, ir::types::Type},
};

use super::{Ssa, Value};

mod instructions;

fn executes_with_no_errors(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    assert!(super::interpret(&ssa).is_ok())
}

fn expect_values(src: &str) -> Vec<Value> {
    expect_values_with_args(src, Vec::new())
}

fn expect_value(src: &str) -> Value {
    expect_value_with_args(src, Vec::new())
}

fn expect_error(src: &str) -> RuntimeError {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret(&ssa).unwrap_err()
}

fn expect_values_with_args(src: &str, args: Vec<Value>) -> Vec<Value> {
    let ssa = Ssa::from_str(src).unwrap();
    super::interpret_function(&ssa, ssa.main_id, args).unwrap()
}

fn expect_value_with_args(src: &str, args: Vec<Value>) -> Value {
    let mut results = expect_values_with_args(src, args);
    assert_eq!(results.len(), 1);
    results.pop().unwrap()
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

#[test]
fn call_function() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = call f1(u32 3) -> u32
            return v1
        }

        acir(inline) fn double f1 {
          b0(v1: u32):
            v2 = mul v1, u32 2
            return v2
        }
    ";
    let actual = expect_value(src);
    assert_eq!(Value::Numeric(NumericValue::U32(6)), actual);
}

#[test]
fn run_flattened_function() {
    let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u1, v1: [[u1; 2]; 3]):
            v2 = not v0
            enable_side_effects v0
            v3 = not v0
            enable_side_effects v0
            v5 = array_get v1, index u32 0 -> [u1; 2]
            v6 = not v0
            v7 = unchecked_mul v0, v6
            enable_side_effects v7
            v8 = array_get v1, index u32 1 -> [u1; 2]
            enable_side_effects v0
            v9 = if v0 then v5 else (if v7) v8
            enable_side_effects v6
            v10 = array_get v1, index u32 2 -> [u1; 2]
            enable_side_effects u1 1
            v12 = if v0 then v5 else (if v6) v10
            return v12
        }";

    let v1_elements = vec![
        Value::array(vec![Value::bool(false), Value::bool(false)], vec![Type::unsigned(1)]),
        Value::array(vec![Value::bool(true), Value::bool(true)], vec![Type::unsigned(1)]),
        Value::array(vec![Value::bool(false), Value::bool(true)], vec![Type::unsigned(1)]),
    ];

    let v1_element_types = vec![Type::Array(Arc::new(vec![Type::unsigned(1)]), 2)];
    let v1 = Value::array(v1_elements, v1_element_types);

    let result = expect_value_with_args(src, vec![Value::bool(true), v1.clone()]);
    assert_eq!(result.to_string(), "rc1 [u1 false, u1 false]");

    let result = expect_value_with_args(src, vec![Value::bool(false), v1]);
    assert_eq!(result.to_string(), "rc1 [u1 false, u1 true]");
}
