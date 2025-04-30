use std::sync::Arc;

use iter_extended::vecmap;
use noirc_frontend::Shared;

use crate::ssa::{
    interpreter::{
        NumericValue, Value,
        tests::{expect_value, expect_values},
        value::ReferenceValue,
    },
    ir::{
        types::{NumericType, Type},
        value::ValueId,
    },
};

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
    expect_error(
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
    expect_error(
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
    expect_error(
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
    expect_error(
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
            v2 = lt i32 3, i32 -3

            v3 = lt i32 -3, i32 -2
            v4 = lt i32 -3, i32 -3
            v5 = lt i32 -3, i32 -4
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
    assert_eq!(values[1], Value::from_constant(1_u128.into(), NumericType::unsigned(8)));
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
    assert_eq!(values[1], Value::from_constant(7_u128.into(), NumericType::unsigned(8)));
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
    assert_eq!(values[1], Value::from_constant(6_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn shl() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shl u8 3, u32 2
            return v0
        }
    ",
    );
    assert_eq!(value, Value::from_constant(12_u128.into(), NumericType::unsigned(8)));
}

#[test]
#[should_panic]
/// shl should overflow if the rhs is greater than the bit count
fn shl_overflow() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shl u8 3, u32 9
            return v0
        }
    ",
    );
}

#[test]
fn shr() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u8 12, u32 2
            v1 = shr u8 5, u32 1
            v2 = shr u8 5, u32 4
            return v0, v1, v2
        }
    ",
    );
    assert_eq!(values[0], Value::from_constant(3_u128.into(), NumericType::unsigned(8)));
    assert_eq!(values[1], Value::from_constant(2_u128.into(), NumericType::unsigned(8)));
    assert_eq!(values[2], Value::from_constant(0_u128.into(), NumericType::unsigned(8)));
}

#[test]
/// Unlike shl, shr does not error on overflow. It just returns 0. See https://github.com/noir-lang/noir/pull/7509.
fn shr_overflow() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u8 3, u32 9
            return v0
        }
    ",
    );
    assert_eq!(value, Value::from_constant(0_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn cast() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = cast u32 2 as Field
            v1 = cast u32 3 as u8
            v2 = cast i8 -1 as i32
            return v0, v1, v2
        }
    ",
    );
    assert_eq!(values[0], Value::from_constant(2_u128.into(), NumericType::NativeField));
    assert_eq!(values[1], Value::from_constant(3_u128.into(), NumericType::unsigned(8)));
    assert_eq!(values[2], Value::from_constant((-1_i128).into(), NumericType::signed(32)));
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

    let not_constant = !136_u8 as u128;
    assert_eq!(values[2], Value::from_constant(not_constant.into(), NumericType::unsigned(8)));
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
    let constant = 257_u16 as u8 as u128;
    assert_eq!(value, Value::from_constant(constant.into(), NumericType::unsigned(32)));
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
fn constrain_disabled_by_enable_side_effects() {
    executes_with_no_errors(
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

// SSA Parser does not yet parse ConstrainNotEqual
// #[test]
// fn constrain_not_equal() {
//     executes_with_no_errors(
//         "
//         acir(inline) fn main f0 {
//           b0():
//             v0 = eq u8 3, u8 4
//             constrain v0 != u1 1
//             return
//         }
//     ",
//     );
// }
//
// #[test]
// fn constrain_not_equal_disabled_by_enable_side_effects() {
//     executes_with_no_errors(
//         "
//         acir(inline) fn main f0 {
//           b0():
//             enable_side_effects u1 0
//             constrain u1 1 != u1 1
//             return
//         }
//     ",
//     );
// }

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
#[should_panic]
fn range_check_fail() {
    expect_error(
        "
        acir(inline) fn main f0 {
          b0():
            range_check u32 256 to 8 bits
            return
        }
    ",
    );
}

#[test]
fn range_check_disabled_by_enable_side_effects() {
    executes_with_no_errors(
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
    assert_eq!(value, Value::from_constant(16_u32.into(), NumericType::NativeField));
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
    let field_zero = Value::from_constant(0u128.into(), NumericType::NativeField);
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
    assert_eq!(value, Value::from_constant(2_u32.into(), NumericType::NativeField));
}

#[test]
fn array_get_disabled_by_enable_side_effects() {
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
    assert_eq!(value, Value::from_constant(0_u32.into(), NumericType::NativeField));
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

    let v0 = values[0].as_array_or_slice().unwrap();
    let v1 = values[1].as_array_or_slice().unwrap();
    let v2 = values[2].as_array_or_slice().unwrap();

    // acir function, so all rcs are 1
    assert_eq!(*v0.rc.borrow(), 1);
    assert_eq!(*v1.rc.borrow(), 1);
    assert_eq!(*v2.rc.borrow(), 1);

    let one = Value::from_constant(1u32.into(), NumericType::NativeField);
    let two = Value::from_constant(2u32.into(), NumericType::NativeField);
    let four = Value::from_constant(4u32.into(), NumericType::NativeField);
    let five = Value::from_constant(5u32.into(), NumericType::NativeField);

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

    let v0 = values[0].as_array_or_slice().unwrap();
    let v1 = values[1].as_array_or_slice().unwrap();
    let v2 = values[2].as_array_or_slice().unwrap();

    // acir function, so all rcs are 1
    assert_eq!(*v0.rc.borrow(), 1);
    assert_eq!(*v1.rc.borrow(), 1);
    assert_eq!(*v2.rc.borrow(), 1);

    let one = Value::from_constant(1u32.into(), NumericType::NativeField);
    let two = Value::from_constant(2u32.into(), NumericType::NativeField);
    let expected = vec![one, two];

    // No changes are made in case an index is out of bounds
    assert_eq!(*v0.elements.borrow(), expected);
    assert_eq!(*v1.elements.borrow(), expected);
    assert_eq!(*v2.elements.borrow(), expected);
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
    let array = value.as_array_or_slice().unwrap();
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
    let array = value.as_array_or_slice().unwrap();
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
    let array = value.as_array_or_slice().unwrap();
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
    let array = value.as_array_or_slice().unwrap();
    assert_eq!(*array.rc.borrow(), 1);
}

#[test]
fn if_else() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = if u1 1 then u8 2 else (if u1 0) u8 3
            v1 = if u1 0 then u8 2 else (if u1 0) u8 3
            return v0, v1
        }
    ",
    );
    assert_eq!(values[0], Value::from_constant(2_u32.into(), NumericType::unsigned(8)));
    assert_eq!(values[1], Value::from_constant(3_u32.into(), NumericType::unsigned(8)));
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
        Value::from_constant(1u128.into(), NumericType::NativeField),
        Value::from_constant(2u128.into(), NumericType::NativeField),
    ];
    assert_eq!(values[0], Value::array(one_two.clone(), vec![Type::field()]));
    assert_eq!(values[1], Value::slice(one_two, Arc::new(vec![Type::field()])));

    let hello =
        vecmap(b"Hello", |char| Value::from_constant((*char as u32).into(), NumericType::char()));
    assert_eq!(values[2], Value::array(hello.clone(), vec![Type::char()]));
    assert_eq!(values[3], Value::slice(hello, Arc::new(vec![Type::char()])));
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
