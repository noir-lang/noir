use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;
use noirc_frontend::Shared;

use crate::ssa::{
    interpreter::{
        InterpreterError, Value,
        tests::{
            expect_value, expect_value_with_args, expect_values, expect_values_with_args,
            from_constant,
        },
        value::ReferenceValue,
    },
    ir::{
        integer::IntegerConstant,
        types::{NumericType, Type},
        value::ValueId,
    },
};

use super::{executes_with_no_errors, expect_error};

fn make_unfit(value: impl Into<FieldElement>, typ: NumericType) -> Value {
    Value::unfit(value.into(), typ).unwrap()
}

#[test]
fn add_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add u32 2, u32 100
            return v0
        }
    ",
    );
    assert_eq!(value, Value::u32(102));
}

#[test]
fn add_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add i32 2, i32 100
            v1 = truncate v0 to 32 bits, max_bit_size: 33
            return v1
        }
    ",
    );
    assert_eq!(value, Value::i32(102));
}

#[test]
fn add_overflow_unsigned() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add u8 200, u8 100
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn add_overflow_signed() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add i8 2, i8 127
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn add_unchecked_unsigned() {
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
fn add_unchecked_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_add i8 2, i8 127
            return v0
        }
    ",
    );
    assert_ne!(value, Value::i8(-128), "no wrapping");
    assert_eq!(value, make_unfit(129u32, NumericType::signed(8)));
}

#[test]
fn sub_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub u32 10101, u32 101
            return v0
        }
    ",
    );
    assert_eq!(value, Value::u32(10000));
}

#[test]
fn sub_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub i32 10101, i32 10102
            v1 = truncate v0 to 32 bits, max_bit_size: 33
            return v0
        }
    ",
    );
    assert_eq!(value, Value::i32(-1));
}

#[test]
fn sub_underflow_unsigned() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub u8 0, u8 10  // 0 - 10
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn sub_underflow_signed() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub i8 136, i8 10  // -120 - 10
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn sub_unchecked_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_sub u8 0, u8 10  // 0 - 10
            return v0
        }
    ",
    );
    assert_ne!(value, Value::u8(246), "no wrapping");
    assert_eq!(
        value,
        // Note that this is not the same as `Value::i8(-10).convert_to_field()`, because that casts to u8 first.
        make_unfit(FieldElement::zero() - FieldElement::from(10u32), NumericType::unsigned(8))
    );
}

#[test]
fn sub_unchecked_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_sub i8 3, i8 10
            return v0
        }
    ",
    );
    assert_eq!(value, Value::i8(-7));
}

#[test]
fn mul_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul u64 2, u64 100
            return v0
        }
    ",
    );
    assert_eq!(value, Value::u64(200));
}

#[test]
fn mul_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul i64 2, i64 100
            v1 = cast v0 as u128
            v2 = truncate v1 to 64 bits, max_bit_size: 128
            v3 = cast v2 as i64
            return v3
        }
    ",
    );
    assert_eq!(value, Value::i64(200));
}

#[test]
fn mul_overflow_unsigned() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul u8 128, u8 2
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn mul_overflow_signed() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul i8 127, i8 2
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn mul_unchecked_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_mul u8 128, u8 2
            return v0
        }
    ",
    );
    assert_ne!(value, Value::u8(0), "no wrapping");
    assert_eq!(value, make_unfit(256u32, NumericType::unsigned(8)));
}

#[test]
fn mul_unchecked_signed() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = unchecked_mul i8 127, i8 2
            return v0
        }
    ",
    );
    assert_ne!(value, Value::i8(-2), "no wrapping");
    assert_eq!(value, make_unfit(254u32, NumericType::signed(8)));
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
    assert_eq!(value, Value::i16(64));
}

#[test]
fn div_zero() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = div Field 12, Field 0
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::DivisionByZero { .. }));
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
    assert_eq!(value, Value::i64(2));
}

#[test]
fn mod_zero() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mod u8 12, u8 0
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::DivisionByZero { .. }));
}

#[test]
fn eq() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = eq u8 3, u8 4
            return v0
        }
    ",
    );
    assert_eq!(value, Value::bool(false));
}

#[test]
fn lt() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = lt u32 3, u32 7
            v1 = lt i32 3, i32 7
            v2 = lt i32 3, i32 4294967293   // 3 < -3  (false)

            v3 = lt i32 4294967293, i32 4294967294  // -3 < -2
            v4 = lt i32 4294967293, i32 4294967293  // -3 < -3
            v5 = lt i32 4294967293, i32 4294967292  // -3 < -4
            return v0, v1, v2, v3, v4, v5
        }
    ",
    );
    assert_eq!(values[0], Value::bool(true));
    assert_eq!(values[1], Value::bool(true));
    assert_eq!(values[2], Value::bool(false));
    assert_eq!(values[3], Value::bool(true));
    assert_eq!(values[4], Value::bool(false));
    assert_eq!(values[5], Value::bool(false));
}

#[test]
fn and() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = and u1 1, u1 0
            v1 = and u8 3, u8 5
            return v0, v1
        }
    ",
    );
    assert_eq!(values[0], Value::bool(false));
    assert_eq!(values[1], from_constant(1_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn or() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = or u1 1, u1 0
            v1 = or u8 3, u8 5
            return v0, v1
        }
    ",
    );
    assert_eq!(values[0], Value::bool(true));
    assert_eq!(values[1], from_constant(7_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn xor() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = xor u1 1, u1 0
            v1 = xor u8 3, u8 5
            return v0, v1
        }
    ",
    );
    assert_eq!(values[0], Value::bool(true));
    assert_eq!(values[1], from_constant(6_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn shl() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shl i8 3, i8 2
            return v0
        }
    ",
    );
    assert_eq!(value, from_constant(12_u128.into(), NumericType::signed(8)));
}

#[test]
fn shl_overflow() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shl u8 3, u8 9
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn shr_unsigned() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u16 12, u16 2
            v1 = shr u16 5, u16 1
            v2 = shr u16 5, u16 4
            return v0, v1, v2
        }
    ",
    );
    assert_eq!(values[0], from_constant(3_u128.into(), NumericType::unsigned(16)));
    assert_eq!(values[1], from_constant(2_u128.into(), NumericType::unsigned(16)));
    assert_eq!(values[2], from_constant(0_u128.into(), NumericType::unsigned(16)));
}

#[test]
fn shr_signed() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr i16 65520, i16 2
            v1 = shr i16 65533, i16 1
            v2 = shr i16 65528, i16 3
            return v0, v1, v2
        }
    ",
    );

    let neg_four = IntegerConstant::Signed { value: -4, bit_size: 16 };
    let (neg_four_constant, typ) = neg_four.into_numeric_constant();
    assert_eq!(values[0], from_constant(neg_four_constant, typ));
    let neg_two = IntegerConstant::Signed { value: -2, bit_size: 16 };
    let (neg_two_constant, typ) = neg_two.into_numeric_constant();
    assert_eq!(values[1], from_constant(neg_two_constant, typ));
    let neg_one = IntegerConstant::Signed { value: -1, bit_size: 16 };
    let (neg_one_constant, typ) = neg_one.into_numeric_constant();
    assert_eq!(values[2], from_constant(neg_one_constant, typ));
}

#[test]
fn shr_overflow_unsigned() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u8 3, u8 9
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn shr_overflow_signed_negative_lhs() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr i8 192, i8 9
            return v0
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn shr_overflow_signed_negative_rhs() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr i8 1, i8 -3
            return v0
        }
    ",
    );
}

#[test]
fn cast() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = cast u32 2 as Field
            v1 = cast u32 3 as u8
            v2 = cast i8 255 as i32   // -1, remains as 255
            v3 = cast i8 255 as u128  // also zero-extended, remains 255
                                      // casts like this should be sign-extended in Noir
                                      // but we rely on other SSA instructions to manually do this.
            return v0, v1, v2, v3
        }
    ",
    );
    assert_eq!(values[0], from_constant(2_u32.into(), NumericType::NativeField));
    assert_eq!(values[1], from_constant(3_u32.into(), NumericType::unsigned(8)));
    assert_eq!(values[2], from_constant(255_u32.into(), NumericType::signed(32)));
    assert_eq!(values[3], from_constant(255_u32.into(), NumericType::unsigned(128)));
}

#[test]
fn not() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = not u1 0
            v1 = not u1 1
            v2 = not u8 136
            return v0, v1, v2
        }
    ",
    );
    assert_eq!(values[0], Value::bool(true));
    assert_eq!(values[1], Value::bool(false));

    let not_constant = u128::from(!136_u8);
    assert_eq!(values[2], from_constant(not_constant.into(), NumericType::unsigned(8)));
}

#[test]
fn truncate() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = truncate u32 257 to 8 bits, max_bit_size: 9
            return v0
        }
    ",
    );
    let constant = u128::from(257_u16 as u8);
    assert_eq!(value, from_constant(constant.into(), NumericType::unsigned(32)));
}

#[test]
fn constrain() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = eq u8 3, u8 4
            constrain v0 == v0
            constrain v0 == u1 0
            return
        }
    ",
    );
}

#[test]
fn constrain_not_disabled_by_enable_side_effects() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            constrain u1 1 == u1 0
            return
        }
    ",
    );
}

#[test]
fn constrain_not_equal() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = eq u8 3, u8 4
            constrain v0 != u1 1
            return
        }
    ",
    );
}

#[test]
fn constrain_not_equal_is_disabled_by_enable_side_effects() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            constrain u1 1 != u1 1
            return
        }
    ",
    );
}

#[test]
fn range_check() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            range_check u32 1000 to 16 bits
            return
        }
    ",
    );
}

#[test]
fn range_check_fail() {
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            range_check u32 256 to 8 bits
            return
        }
    ",
    );
    assert!(matches!(error, InterpreterError::RangeCheckFailed { .. }));
}

#[test]
fn range_check_not_disabled_by_enable_side_effects() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            range_check u32 256 to 8 bits
            return
        }
    ",
    );
}

#[test]
fn call() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = call f1(Field 4) -> Field
            return v0
        }

        acir(inline) fn square f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
    ",
    );
    assert_eq!(value, from_constant(16_u32.into(), NumericType::NativeField));
}

#[test]
fn allocate() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            return v0
        }
    ",
    );
    let expected = Value::Reference(ReferenceValue {
        original_id: ValueId::test_new(0),
        element: Shared::new(None),
        element_type: Arc::new(Type::field()),
    });
    assert_eq!(value, expected);
}

#[test]
fn load() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            v1 = load v0 -> u1
            return v1
        }
    ",
    );
    assert_eq!(value, Value::bool(true));
}

#[test]
fn store() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            return v0
        }
    ",
    );
    let expected = Value::Reference(ReferenceValue {
        original_id: ValueId::test_new(0),
        element: Shared::new(Some(Value::bool(true))),
        element_type: Arc::new(Type::bool()),
    });
    assert_eq!(value, expected);
}

#[test]
fn enable_side_effects() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v2 = call f1(v1) -> Field
            return v1, v2
        }

        acir(inline) fn foo f1 {
          b0(v0: &mut Field):
            store Field 2 at v0
            return Field 7
        }
    ",
    );
    let field_zero = from_constant(0u128.into(), NumericType::NativeField);
    let expected = Value::Reference(ReferenceValue {
        original_id: ValueId::test_new(1),
        element: Shared::new(Some(field_zero.clone())),
        element_type: Arc::new(Type::field()),
    });
    assert_eq!(values[0], expected);
    assert_eq!(values[1], field_zero);
}

#[test]
fn array_get() {
    let value = expect_value(
        r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_get v0, index u32 1 -> Field
            return v1
        }
    "#,
    );
    assert_eq!(value, from_constant(2_u32.into(), NumericType::NativeField));
}

#[test]
fn array_get_with_offset() {
    let value = expect_value(
        r#"
        brillig(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_get v0, index u32 2 minus 1 -> Field
            return v1
        }
    "#,
    );
    assert_eq!(value, from_constant(2_u32.into(), NumericType::NativeField));
}

#[test]
fn array_get_not_disabled_by_enable_side_effects_if_index_is_known_to_be_safe() {
    let value = expect_value(
        r#"
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_get v0, index u32 1 -> Field
            return v1
        }
    "#,
    );
    assert_eq!(value, from_constant(2_u32.into(), NumericType::NativeField));
}

#[test]
fn array_get_disabled_by_enable_side_effects_if_index_is_not_known_to_be_safe() {
    let value = expect_value_with_args(
        r#"
        acir(inline) fn main f0 {
          b0(v2: u32):
            enable_side_effects u1 0
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_get v0, index v2 -> Field
            return v1
        }
    "#,
        vec![Value::u32(1)],
    );
    // If enable_side_effects is false, array get will retrieve the value at the first compatible index
    assert_eq!(value, from_constant(1_u32.into(), NumericType::NativeField));
}

#[test]
fn array_set() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_set v0, index u32 1, value Field 5
            v2 = array_set mut v0, index u32 0, value Field 4
            return v0, v1, v2
        }
    ",
    );

    let v0 = values[0].as_array_or_vector().unwrap();
    let v1 = values[1].as_array_or_vector().unwrap();
    let v2 = values[2].as_array_or_vector().unwrap();

    // acir function, so all rcs are 1
    assert_eq!(*v0.rc.borrow(), 1);
    assert_eq!(*v1.rc.borrow(), 1);
    assert_eq!(*v2.rc.borrow(), 1);

    let one = from_constant(1u32.into(), NumericType::NativeField);
    let two = from_constant(2u32.into(), NumericType::NativeField);
    let four = from_constant(4u32.into(), NumericType::NativeField);
    let five = from_constant(5u32.into(), NumericType::NativeField);

    // v0 was forcibly mutated via the last `array_set mut`
    assert_eq!(*v0.elements.borrow(), vec![four.clone(), two.clone()]);

    // v1 was not mutated when v2 was created since it is conceptually a different array
    assert_eq!(*v1.elements.borrow(), vec![one, five]);

    assert_eq!(*v2.elements.borrow(), vec![four, two]);
}

#[test]
fn array_set_disabled_by_enable_side_effects() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 0
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_set v0, index u32 1, value Field 5
            v2 = array_set mut v0, index u32 0, value Field 4
            return v0, v1, v2
        }
    ",
    );

    let v0 = values[0].as_array_or_vector().unwrap();
    let v1 = values[1].as_array_or_vector().unwrap();
    let v2 = values[2].as_array_or_vector().unwrap();

    // acir function, so all rcs are 1
    assert_eq!(*v0.rc.borrow(), 1);
    assert_eq!(*v1.rc.borrow(), 1);
    assert_eq!(*v2.rc.borrow(), 1);

    let one = from_constant(1u32.into(), NumericType::NativeField);
    let two = from_constant(2u32.into(), NumericType::NativeField);
    let expected = vec![one, two];

    // No changes are made in case an index is out of bounds
    assert_eq!(*v0.elements.borrow(), expected);
    assert_eq!(*v1.elements.borrow(), expected);
    assert_eq!(*v2.elements.borrow(), expected);
}

#[test]
fn array_set_with_offset() {
    let values = expect_values(
        "
        brillig(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            inc_rc v0
            v1 = array_set v0, index u32 2 minus 1, value Field 5
            return v0, v1
        }
    ",
    );

    let v0 = values[0].as_array_or_vector().unwrap();
    let v1 = values[1].as_array_or_vector().unwrap();

    assert_eq!(*v0.rc.borrow(), 2, "1+1-0; the copy of v1 does not decrease the RC of v0");
    assert_eq!(*v1.rc.borrow(), 1);

    let one = from_constant(1u32.into(), NumericType::NativeField);
    let two = from_constant(2u32.into(), NumericType::NativeField);
    let five = from_constant(5u32.into(), NumericType::NativeField);

    assert_eq!(*v0.elements.borrow(), vec![one.clone(), two], "v0 should not be mutated");
    assert_eq!(*v1.elements.borrow(), vec![one, five], "v1 should be mutated");
}

#[test]
fn increment_rc() {
    let value = expect_value(
        "
        brillig(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            inc_rc v0
            inc_rc v0
            inc_rc v0
            return v0
        }
    ",
    );
    let array = value.as_array_or_vector().unwrap();
    assert_eq!(*array.rc.borrow(), 4);
}

#[test]
fn increment_rc_disabled_in_acir() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            inc_rc v0
            inc_rc v0
            inc_rc v0
            return v0
        }
    ",
    );
    let array = value.as_array_or_vector().unwrap();
    assert_eq!(*array.rc.borrow(), 1);
}

#[test]
fn decrement_rc() {
    let value = expect_value(
        "
        brillig(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            dec_rc v0
            return v0
        }
    ",
    );
    let array = value.as_array_or_vector().unwrap();
    assert_eq!(*array.rc.borrow(), 0);
}

#[test]
fn decrement_rc_disabled_in_acir() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            dec_rc v0
            return v0
        }
    ",
    );
    let array = value.as_array_or_vector().unwrap();
    assert_eq!(*array.rc.borrow(), 1);
}

#[test]
fn if_else() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = if u1 1 then u8 2 else (if u1 0) u8 3
            v1 = if u1 0 then u8 2 else (if u1 1) u8 3
            v2 = if u1 0 then u8 2 else (if u1 0) u8 3
            return v0, v1, v2
        }
    ",
    );
    assert_eq!(values[0], from_constant(2_u32.into(), NumericType::unsigned(8)));
    assert_eq!(values[1], from_constant(3_u32.into(), NumericType::unsigned(8)));
    assert_eq!(values[2], from_constant(0_u32.into(), NumericType::unsigned(8)));
}

#[test]
fn make_array() {
    let values = expect_values(
        r#"
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = make_array [Field 1, Field 2] : [Field]
            v2 = make_array b"Hello"
            v3 = make_array &b"Hello"
            return v0, v1, v2, v3
        }
    "#,
    );
    let one_two = vec![
        from_constant(1u128.into(), NumericType::NativeField),
        from_constant(2u128.into(), NumericType::NativeField),
    ];
    assert_eq!(values[0], Value::array(one_two.clone(), vec![Type::field()]));
    assert_eq!(values[1], Value::vector(one_two, Arc::new(vec![Type::field()])));

    let hello =
        vecmap(b"Hello", |char| from_constant(u32::from(*char).into(), NumericType::char()));
    assert_eq!(values[2], Value::array(hello.clone(), vec![Type::char()]));
    assert_eq!(values[3], Value::vector(hello, Arc::new(vec![Type::char()])));
}

#[test]
fn nop() {
    executes_with_no_errors(
        "
        acir(inline) fn main f0 {
          b0():
            nop
            nop
            nop
            return
        }
    ",
    );
}

// Test that side_effects_enabled state is properly saved and restored across function calls.
// If a callee function disables side effects, this should not affect the caller function
// when the call returns.
#[test]
fn enable_side_effects_not_leaked_across_calls() {
    // If side_effects_enabled state is leaked, the constrain_not_equal would be skipped
    // and the program would succeed even though the values are equal.
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            call f1()
            // After returning from f1, side_effects should be enabled again
            // This constrain should fail because 1 == 1
            constrain u1 1 != u1 1
            return
        }

        // This function disables side effects and doesn't restore them
        acir(inline) fn disable_side_effects f1 {
          b0():
            enable_side_effects u1 0
            return
        }
    ",
    );
    assert!(matches!(error, InterpreterError::ConstrainNeFailed { .. }));
}

// Test that side_effects_enabled state is properly saved and restored across function calls,
// using a checked add instruction that would overflow.
#[test]
fn enable_side_effects_not_leaked_across_calls_2() {
    // If side_effects_enabled state is leaked, the add would return 0 instead of overflowing
    // and the program would succeed.
    let error = expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            call f1()
            // After returning from f1, side_effects should be enabled again
            // This add should overflow because 200 + 100 > 255 (u8 max)
            v0 = add u8 200, u8 100
            return v0
        }

        // This function disables side effects and doesn't restore them
        acir(inline) fn disable_side_effects f1 {
          b0():
            enable_side_effects u1 0
            return
        }
    ",
    );
    assert!(matches!(error, InterpreterError::Overflow { .. }));
}

#[test]
fn test_range_and_xor_bb() {
    let src = "
      acir(inline) fn main f0 {
        b0(v0: Field, v1: Field):
          v2 = call black_box(v0) -> Field
          v3 = call f2(v2,v1) -> Field
          call f3(v3)
          return
      }

      acir(inline) fn test_and_xor f2 {
        b0(v0: Field, v1: Field):
          v2 = truncate v0 to 8 bits, max_bit_size: 254
          v3 = cast v2 as u8
          v4 = truncate v1 to 8 bits, max_bit_size: 254
          v5 = cast v4 as u8
          v8 = and v3, v5
          v9 = xor v3, v5
          v10 = cast v9 as Field
          return v10
      }

      acir(inline) fn test_range f3 {
        b0(v0: Field):
          range_check v0 to 8 bits
          return
      }
      ";
    let values = expect_values_with_args(
        src,
        vec![
            from_constant(1_u128.into(), NumericType::NativeField),
            from_constant(12_u128.into(), NumericType::NativeField),
        ],
    );
    assert!(values.is_empty());
}
