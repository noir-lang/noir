#![cfg(test)]

use std::sync::Arc;

use acvm::{AcirField, FieldElement};

use crate::{
    errors::RuntimeError,
    ssa::{
        interpreter::value::NumericValue,
        ir::types::{NumericType, Type},
    },
};

use super::{Ssa, Value};

mod instructions;

fn executes_with_no_errors(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    assert!(ssa.interpret().is_ok())
}

fn expect_values(src: &str) -> Vec<Value> {
    expect_values_with_args(src, Vec::new())
}

fn expect_value(src: &str) -> Value {
    expect_value_with_args(src, Vec::new())
}

fn expect_error(src: &str) -> RuntimeError {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.interpret().unwrap_err()
}

fn expect_values_with_args(src: &str, args: Vec<Value>) -> Vec<Value> {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.interpret_function(ssa.main_id, args).unwrap()
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

#[test]
fn loads_passed_to_a_call() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut Field
        store Field 0 at v1
        v3 = allocate -> &mut &mut Field
        store v1 at v3
        jmp b1(Field 0)
      b1(v0: Field):
        v4 = eq v0, Field 0
        jmpif v4 then: b3, else: b2
      b2():
        v9 = load v1 -> Field
        v10 = eq v9, Field 2
        constrain v9 == Field 2
        v11 = load v3 -> &mut Field
        call f1(v3)
        v13 = load v3 -> &mut Field
        v14 = load v13 -> Field
        v15 = eq v14, Field 2
        constrain v14 == Field 2
        return v14
      b3():
        v5 = load v3 -> &mut Field
        store Field 2 at v5
        v8 = add v0, Field 1
        jmp b1(v8)
    }
    acir(inline) fn foo f1 {
      b0(v0: &mut Field):
        return
    }  
    ";

    let value = expect_value(src);
    assert_eq!(value, Value::from_constant(2_u128.into(), NumericType::NativeField));
}

#[test]
fn keep_repeat_loads_with_alias_store() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        jmpif v0 then: b2, else: b1
      b1():
        v6 = allocate -> &mut Field
        store Field 1 at v6
        jmp b3(v6, v6, v6)
      b2():
        v4 = allocate -> &mut Field
        store Field 0 at v4
        jmp b3(v4, v4, v4)
      b3(v1: &mut Field, v2: &mut Field, v3: &mut Field):
        v8 = load v1 -> Field
        store Field 2 at v2
        v10 = load v1 -> Field
        store Field 1 at v3
        v11 = load v1 -> Field
        store Field 3 at v3
        v13 = load v1 -> Field
        constrain v8 == Field 0
        constrain v10 == Field 2
        constrain v11 == Field 1
        constrain v13 == Field 3
        return v8, v11
    }
    ";

    let values = expect_values_with_args(src, vec![Value::bool(true)]);
    assert_eq!(values.len(), 2);

    assert_eq!(values[0], Value::from_constant(FieldElement::zero(), NumericType::NativeField));
    assert_eq!(values[1], Value::from_constant(FieldElement::one(), NumericType::NativeField));
}

fn accepts_globals() {
    let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        brillig(inline) predicate_pure fn main f0 {
        b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            constrain v0 == g2
            return
        }
    ";
    executes_with_no_errors(src);
}

#[test]
fn accepts_print() {
    // fn main(x: Field) {
    //     print(x);
    //     println(x);
    // }
    let src = r#"
        brillig(inline) impure fn main f0 {
        b0(v0: Field):
            v12 = make_array b"{\"kind\":\"field\"}"
            call print(u1 0, v0, v12, u1 0)
            inc_rc v12
            call print(u1 1, v0, v12, u1 0)
            return
        }
    "#;
    executes_with_no_errors(src);
}
