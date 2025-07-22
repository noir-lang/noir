use std::sync::Arc;

use iter_extended::vecmap;
use noirc_frontend::Shared;

use crate::ssa::{
    interpreter::{
        InterpreterError, NumericValue, Value,
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
    assert_eq!(value, Value::Numeric(NumericValue::U32(102)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I32(102)));
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
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add i8 2, i8 127
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I8(-127)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I8(-127)));
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
    assert_eq!(value, Value::Numeric(NumericValue::U32(10000)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I32(-1)));
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
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = sub i8 136, i8 10  // -120 - 10
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
    ",
    );
    // Expected wrapping sub:
    // i8 can only be -128 to 127
    // -120 - 10 = -130 = 126
    assert!(matches!(value, Value::Numeric(NumericValue::I8(126))));
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
    assert!(matches!(value, Value::Numeric(NumericValue::U8(246))));
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
    assert_eq!(value, Value::Numeric(NumericValue::I8(-7)));
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
    assert_eq!(value, Value::Numeric(NumericValue::U64(200)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I64(200)));
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
    // We return v0 as we simply want the output from the mul operation in this test.
    // However, the valid SSA signed overflow patterns requires that the appropriate
    // casts and truncates follow a signed mul.
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = mul i8 127, i8 2
            v1 = cast v0 as u16
            v2 = truncate v1 to 8 bits, max_bit_size: 16
            v3 = cast v2 as i8
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::I8(-2)));
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
    assert_eq!(value, Value::Numeric(NumericValue::U8(0)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I8(-2)));
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
    assert_eq!(value, Value::Numeric(NumericValue::I64(2)));
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
            v0 = shl i8 3, u8 2
            return v0
        }
    ",
    );
    assert_eq!(value, from_constant(12_u128.into(), NumericType::signed(8)));
}

/// shl does not error on overflow. It just returns zero.
#[test]
fn shl_overflow() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shl u8 3, u8 9
            return v0
        }
    ",
    );
    assert_eq!(value, from_constant(0_u128.into(), NumericType::unsigned(8)));
}

#[test]
fn shr_unsigned() {
    let values = expect_values(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u16 12, u8 2
            v1 = shr u16 5, u8 1
            v2 = shr u16 5, u8 4
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
            v0 = shr i16 65520, u8 2      
            v1 = shr i16 65533, u8 1      
            v2 = shr i16 65528, u8 3 
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
/// shr on unsigned integer does not error on overflow. It just returns 0. See https://github.com/noir-lang/noir/pull/7509.
fn shr_overflow_unsigned() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr u8 3, u8 9
            return v0
        }
    ",
    );
    assert_eq!(value, from_constant(0_u128.into(), NumericType::unsigned(8)));
}

#[test]
/// shr on signed integers does not error on overflow.
/// If the value being shifted is positive we return 0, and -1 if it is negative.
/// See https://github.com/noir-lang/noir/pull/8805.
fn shr_overflow_signed_negative_lhs() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr i8 192, u8 9
            return v0
        }
    ",
    );

    let neg_one = IntegerConstant::Signed { value: -1, bit_size: 8 };
    let (neg_one_constant, typ) = neg_one.into_numeric_constant();
    assert_eq!(value, from_constant(neg_one_constant, typ));
}

#[test]
/// shr on signed integers does not error on overflow.
/// If the value being shifted is positive we return 0, and -1 if it is negative.
/// See https://github.com/noir-lang/noir/pull/8805.
fn shr_overflow_signed_positive_lhs() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = shr i8 1, u8 255
            return v0
        }
    ",
    );

    assert_eq!(value, Value::Numeric(NumericValue::I8(0)));
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

    let not_constant = !136_u8 as u128;
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
    let constant = 257_u16 as u8 as u128;
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
fn constrain_not_equal_not_disabled_by_enable_side_effects() {
    expect_error(
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
        g0 = u32 1779033703
        g1 = u32 3144134277
        g2 = u32 1013904242
        g3 = u32 2773480762
        g4 = u32 1359893119
        g5 = u32 2600822924
        g6 = u32 528734635
        g7 = u32 1541459225
        g8 = make_array [u32 1779033703, u32 3144134277, u32 1013904242, u32 2773480762, u32 1359893119, u32 2600822924, u32 528734635, u32 1541459225] : [u32; 8]
        g9 = u32 64
        g10 = u32 4
        g11 = u32 56
        g12 = u32 16
        g13 = u32 14
        g14 = u64 4294967296
        g15 = u32 256
        g16 = u32 65536
        g17 = u32 16777216

        acir(inline) predicate_pure fn empty_sha f0 {
        b0():
            v18 = make_array [u32 1779033703, u32 3144134277, u32 1013904242, u32 2773480762, u32 1359893119, u32 2600822924, u32 528734635, u32 1541459225] : [u32; 8]     // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:40:41
            v21 = make_array [u32 1, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 16]   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:98:9
            v23 = call f1(v21, u32 1, u32 0) -> [u32; 16]       // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:110:30
            v24 = array_get v23, index u32 0 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:269:16
            v25 = truncate v24 to 24 bits, max_bit_size: 32     // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:285:19
            constrain v25 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:285:19
            v26 = array_get v23, index u32 1 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v26 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v28 = array_get v23, index u32 2 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v28 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v30 = array_get v23, index u32 3 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v30 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v31 = array_get v23, index u32 4 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v31 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v33 = array_get v23, index u32 5 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v33 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v35 = array_get v23, index u32 6 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v35 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v37 = array_get v23, index u32 7 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v37 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v39 = array_get v23, index u32 8 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v39 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v41 = array_get v23, index u32 9 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v41 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v43 = array_get v23, index u32 10 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v43 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v45 = array_get v23, index u32 11 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v45 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v47 = array_get v23, index u32 12 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v47 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v49 = array_get v23, index u32 13 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v49 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            constrain v24 == u32 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:323:19
            v50 = array_get v23, index u32 14 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:504:49
            v51 = cast v50 as u64                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:504:49
            v52 = mul v51, u64 4294967296                       // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:503:29
            v54 = array_get v23, index u32 15 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:504:49
            v55 = cast v54 as u64                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:504:49
            v56 = add v52, v55                                  // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:504:29
            constrain v56 == u64 0                              // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:507:15
            v59 = call sha256_compression(v23, v18) -> [u32; 8] // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:514:13
            v60 = array_get v59, index u32 0 -> u32             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v61 = truncate v60 to 31 bits, max_bit_size: 32     // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v62 = cast v61 as Field                             // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v64 = call to_be_radix(v62, u32 256) -> [u8; 4]     // std/field/mod.nr:175:9
            v65 = array_get v64, index u32 0 -> u8              // std/field/mod.nr:147:25
            v67 = eq v65, u8 127                                // std/field/mod.nr:147:25
            v68 = not v67                                       // std/field/mod.nr:147:25
            v69 = lt v65, u8 127                                // std/field/mod.nr:148:32
            v70 = unchecked_mul v69, v68                        // std/field/mod.nr:148:32
            constrain v70 == v68                                // std/field/mod.nr:148:32
            v71 = array_get v64, index u32 1 -> u8              // std/field/mod.nr:147:25
            v73 = eq v71, u8 255                                // std/field/mod.nr:147:25
            v74 = not v73                                       // std/field/mod.nr:147:25
            v75 = unchecked_mul v67, v74                        // std/field/mod.nr:147:25
            v76 = lt v71, u8 255                                // std/field/mod.nr:148:32
            v77 = unchecked_mul v76, v75                        // std/field/mod.nr:148:32
            constrain v77 == v75                                // std/field/mod.nr:148:32
            v78 = not v75                                       // std/field/mod.nr:148:32
            v79 = unchecked_mul v78, v68                        // std/field/mod.nr:148:32
            v80 = unchecked_add v75, v79                        // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v82 = not v80                                       // std/field/mod.nr:145:25
            v83 = array_get v64, index u32 2 -> u8              // std/field/mod.nr:147:25
            v84 = eq v83, u8 255                                // std/field/mod.nr:147:25
            v85 = not v84                                       // std/field/mod.nr:147:25
            v86 = unchecked_mul v82, v85                        // std/field/mod.nr:147:25
            v87 = lt v83, u8 255                                // std/field/mod.nr:148:32
            v88 = unchecked_mul v87, v86                        // std/field/mod.nr:148:32
            constrain v88 == v86                                // std/field/mod.nr:148:32
            v89 = not v86                                       // std/field/mod.nr:148:32
            v90 = unchecked_mul v89, v80                        // std/field/mod.nr:148:32
            v91 = unchecked_add v86, v90                        // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v92 = not v91                                       // std/field/mod.nr:145:25
            v93 = array_get v64, index u32 3 -> u8              // std/field/mod.nr:147:25
            v94 = eq v93, u8 255                                // std/field/mod.nr:147:25
            v95 = not v94                                       // std/field/mod.nr:147:25
            v96 = unchecked_mul v92, v95                        // std/field/mod.nr:147:25
            v97 = lt v93, u8 255                                // std/field/mod.nr:148:32
            v98 = unchecked_mul v97, v96                        // std/field/mod.nr:148:32
            constrain v98 == v96                                // std/field/mod.nr:148:32
            v99 = not v96                                       // std/field/mod.nr:148:32
            v100 = unchecked_mul v99, v91                       // std/field/mod.nr:148:32
            v101 = unchecked_add v96, v100                      // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v101 == u1 1                              // std/field/mod.nr:153:20
            v102 = array_get v59, index u32 1 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v103 = truncate v102 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v104 = cast v103 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v105 = call to_be_radix(v104, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v106 = array_get v105, index u32 0 -> u8            // std/field/mod.nr:147:25
            v107 = eq v106, u8 127                              // std/field/mod.nr:147:25
            v108 = not v107                                     // std/field/mod.nr:147:25
            v109 = lt v106, u8 127                              // std/field/mod.nr:148:32
            v110 = unchecked_mul v109, v108                     // std/field/mod.nr:148:32
            constrain v110 == v108                              // std/field/mod.nr:148:32
            v111 = array_get v105, index u32 1 -> u8            // std/field/mod.nr:147:25
            v112 = eq v111, u8 255                              // std/field/mod.nr:147:25
            v113 = not v112                                     // std/field/mod.nr:147:25
            v114 = unchecked_mul v107, v113                     // std/field/mod.nr:147:25
            v115 = lt v111, u8 255                              // std/field/mod.nr:148:32
            v116 = unchecked_mul v115, v114                     // std/field/mod.nr:148:32
            constrain v116 == v114                              // std/field/mod.nr:148:32
            v117 = not v114                                     // std/field/mod.nr:148:32
            v118 = unchecked_mul v117, v108                     // std/field/mod.nr:148:32
            v119 = unchecked_add v114, v118                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v120 = not v119                                     // std/field/mod.nr:145:25
            v121 = array_get v105, index u32 2 -> u8            // std/field/mod.nr:147:25
            v122 = eq v121, u8 255                              // std/field/mod.nr:147:25
            v123 = not v122                                     // std/field/mod.nr:147:25
            v124 = unchecked_mul v120, v123                     // std/field/mod.nr:147:25
            v125 = lt v121, u8 255                              // std/field/mod.nr:148:32
            v126 = unchecked_mul v125, v124                     // std/field/mod.nr:148:32
            constrain v126 == v124                              // std/field/mod.nr:148:32
            v127 = not v124                                     // std/field/mod.nr:148:32
            v128 = unchecked_mul v127, v119                     // std/field/mod.nr:148:32
            v129 = unchecked_add v124, v128                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v130 = not v129                                     // std/field/mod.nr:145:25
            v131 = array_get v105, index u32 3 -> u8            // std/field/mod.nr:147:25
            v132 = eq v131, u8 255                              // std/field/mod.nr:147:25
            v133 = not v132                                     // std/field/mod.nr:147:25
            v134 = unchecked_mul v130, v133                     // std/field/mod.nr:147:25
            v135 = lt v131, u8 255                              // std/field/mod.nr:148:32
            v136 = unchecked_mul v135, v134                     // std/field/mod.nr:148:32
            constrain v136 == v134                              // std/field/mod.nr:148:32
            v137 = not v134                                     // std/field/mod.nr:148:32
            v138 = unchecked_mul v137, v129                     // std/field/mod.nr:148:32
            v139 = unchecked_add v134, v138                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v139 == u1 1                              // std/field/mod.nr:153:20
            v140 = array_get v59, index u32 2 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v141 = truncate v140 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v142 = cast v141 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v143 = call to_be_radix(v142, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v144 = array_get v143, index u32 0 -> u8            // std/field/mod.nr:147:25
            v145 = eq v144, u8 127                              // std/field/mod.nr:147:25
            v146 = not v145                                     // std/field/mod.nr:147:25
            v147 = lt v144, u8 127                              // std/field/mod.nr:148:32
            v148 = unchecked_mul v147, v146                     // std/field/mod.nr:148:32
            constrain v148 == v146                              // std/field/mod.nr:148:32
            v149 = array_get v143, index u32 1 -> u8            // std/field/mod.nr:147:25
            v150 = eq v149, u8 255                              // std/field/mod.nr:147:25
            v151 = not v150                                     // std/field/mod.nr:147:25
            v152 = unchecked_mul v145, v151                     // std/field/mod.nr:147:25
            v153 = lt v149, u8 255                              // std/field/mod.nr:148:32
            v154 = unchecked_mul v153, v152                     // std/field/mod.nr:148:32
            constrain v154 == v152                              // std/field/mod.nr:148:32
            v155 = not v152                                     // std/field/mod.nr:148:32
            v156 = unchecked_mul v155, v146                     // std/field/mod.nr:148:32
            v157 = unchecked_add v152, v156                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v158 = not v157                                     // std/field/mod.nr:145:25
            v159 = array_get v143, index u32 2 -> u8            // std/field/mod.nr:147:25
            v160 = eq v159, u8 255                              // std/field/mod.nr:147:25
            v161 = not v160                                     // std/field/mod.nr:147:25
            v162 = unchecked_mul v158, v161                     // std/field/mod.nr:147:25
            v163 = lt v159, u8 255                              // std/field/mod.nr:148:32
            v164 = unchecked_mul v163, v162                     // std/field/mod.nr:148:32
            constrain v164 == v162                              // std/field/mod.nr:148:32
            v165 = not v162                                     // std/field/mod.nr:148:32
            v166 = unchecked_mul v165, v157                     // std/field/mod.nr:148:32
            v167 = unchecked_add v162, v166                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v168 = not v167                                     // std/field/mod.nr:145:25
            v169 = array_get v143, index u32 3 -> u8            // std/field/mod.nr:147:25
            v170 = eq v169, u8 255                              // std/field/mod.nr:147:25
            v171 = not v170                                     // std/field/mod.nr:147:25
            v172 = unchecked_mul v168, v171                     // std/field/mod.nr:147:25
            v173 = lt v169, u8 255                              // std/field/mod.nr:148:32
            v174 = unchecked_mul v173, v172                     // std/field/mod.nr:148:32
            constrain v174 == v172                              // std/field/mod.nr:148:32
            v175 = not v172                                     // std/field/mod.nr:148:32
            v176 = unchecked_mul v175, v167                     // std/field/mod.nr:148:32
            v177 = unchecked_add v172, v176                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v177 == u1 1                              // std/field/mod.nr:153:20
            v178 = array_get v59, index u32 3 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v179 = truncate v178 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v180 = cast v179 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v181 = call to_be_radix(v180, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v182 = array_get v181, index u32 0 -> u8            // std/field/mod.nr:147:25
            v183 = eq v182, u8 127                              // std/field/mod.nr:147:25
            v184 = not v183                                     // std/field/mod.nr:147:25
            v185 = lt v182, u8 127                              // std/field/mod.nr:148:32
            v186 = unchecked_mul v185, v184                     // std/field/mod.nr:148:32
            constrain v186 == v184                              // std/field/mod.nr:148:32
            v187 = array_get v181, index u32 1 -> u8            // std/field/mod.nr:147:25
            v188 = eq v187, u8 255                              // std/field/mod.nr:147:25
            v189 = not v188                                     // std/field/mod.nr:147:25
            v190 = unchecked_mul v183, v189                     // std/field/mod.nr:147:25
            v191 = lt v187, u8 255                              // std/field/mod.nr:148:32
            v192 = unchecked_mul v191, v190                     // std/field/mod.nr:148:32
            constrain v192 == v190                              // std/field/mod.nr:148:32
            v193 = not v190                                     // std/field/mod.nr:148:32
            v194 = unchecked_mul v193, v184                     // std/field/mod.nr:148:32
            v195 = unchecked_add v190, v194                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v196 = not v195                                     // std/field/mod.nr:145:25
            v197 = array_get v181, index u32 2 -> u8            // std/field/mod.nr:147:25
            v198 = eq v197, u8 255                              // std/field/mod.nr:147:25
            v199 = not v198                                     // std/field/mod.nr:147:25
            v200 = unchecked_mul v196, v199                     // std/field/mod.nr:147:25
            v201 = lt v197, u8 255                              // std/field/mod.nr:148:32
            v202 = unchecked_mul v201, v200                     // std/field/mod.nr:148:32
            constrain v202 == v200                              // std/field/mod.nr:148:32
            v203 = not v200                                     // std/field/mod.nr:148:32
            v204 = unchecked_mul v203, v195                     // std/field/mod.nr:148:32
            v205 = unchecked_add v200, v204                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v206 = not v205                                     // std/field/mod.nr:145:25
            v207 = array_get v181, index u32 3 -> u8            // std/field/mod.nr:147:25
            v208 = eq v207, u8 255                              // std/field/mod.nr:147:25
            v209 = not v208                                     // std/field/mod.nr:147:25
            v210 = unchecked_mul v206, v209                     // std/field/mod.nr:147:25
            v211 = lt v207, u8 255                              // std/field/mod.nr:148:32
            v212 = unchecked_mul v211, v210                     // std/field/mod.nr:148:32
            constrain v212 == v210                              // std/field/mod.nr:148:32
            v213 = not v210                                     // std/field/mod.nr:148:32
            v214 = unchecked_mul v213, v205                     // std/field/mod.nr:148:32
            v215 = unchecked_add v210, v214                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v215 == u1 1                              // std/field/mod.nr:153:20
            v216 = array_get v59, index u32 4 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v217 = truncate v216 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v218 = cast v217 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v219 = call to_be_radix(v218, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v220 = array_get v219, index u32 0 -> u8            // std/field/mod.nr:147:25
            v221 = eq v220, u8 127                              // std/field/mod.nr:147:25
            v222 = not v221                                     // std/field/mod.nr:147:25
            v223 = lt v220, u8 127                              // std/field/mod.nr:148:32
            v224 = unchecked_mul v223, v222                     // std/field/mod.nr:148:32
            constrain v224 == v222                              // std/field/mod.nr:148:32
            v225 = array_get v219, index u32 1 -> u8            // std/field/mod.nr:147:25
            v226 = eq v225, u8 255                              // std/field/mod.nr:147:25
            v227 = not v226                                     // std/field/mod.nr:147:25
            v228 = unchecked_mul v221, v227                     // std/field/mod.nr:147:25
            v229 = lt v225, u8 255                              // std/field/mod.nr:148:32
            v230 = unchecked_mul v229, v228                     // std/field/mod.nr:148:32
            constrain v230 == v228                              // std/field/mod.nr:148:32
            v231 = not v228                                     // std/field/mod.nr:148:32
            v232 = unchecked_mul v231, v222                     // std/field/mod.nr:148:32
            v233 = unchecked_add v228, v232                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v234 = not v233                                     // std/field/mod.nr:145:25
            v235 = array_get v219, index u32 2 -> u8            // std/field/mod.nr:147:25
            v236 = eq v235, u8 255                              // std/field/mod.nr:147:25
            v237 = not v236                                     // std/field/mod.nr:147:25
            v238 = unchecked_mul v234, v237                     // std/field/mod.nr:147:25
            v239 = lt v235, u8 255                              // std/field/mod.nr:148:32
            v240 = unchecked_mul v239, v238                     // std/field/mod.nr:148:32
            constrain v240 == v238                              // std/field/mod.nr:148:32
            v241 = not v238                                     // std/field/mod.nr:148:32
            v242 = unchecked_mul v241, v233                     // std/field/mod.nr:148:32
            v243 = unchecked_add v238, v242                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v244 = not v243                                     // std/field/mod.nr:145:25
            v245 = array_get v219, index u32 3 -> u8            // std/field/mod.nr:147:25
            v246 = eq v245, u8 255                              // std/field/mod.nr:147:25
            v247 = not v246                                     // std/field/mod.nr:147:25
            v248 = unchecked_mul v244, v247                     // std/field/mod.nr:147:25
            v249 = lt v245, u8 255                              // std/field/mod.nr:148:32
            v250 = unchecked_mul v249, v248                     // std/field/mod.nr:148:32
            constrain v250 == v248                              // std/field/mod.nr:148:32
            v251 = not v248                                     // std/field/mod.nr:148:32
            v252 = unchecked_mul v251, v243                     // std/field/mod.nr:148:32
            v253 = unchecked_add v248, v252                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v253 == u1 1                              // std/field/mod.nr:153:20
            v254 = array_get v59, index u32 5 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v255 = truncate v254 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v256 = cast v255 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v257 = call to_be_radix(v256, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v258 = array_get v257, index u32 0 -> u8            // std/field/mod.nr:147:25
            v259 = eq v258, u8 127                              // std/field/mod.nr:147:25
            v260 = not v259                                     // std/field/mod.nr:147:25
            v261 = lt v258, u8 127                              // std/field/mod.nr:148:32
            v262 = unchecked_mul v261, v260                     // std/field/mod.nr:148:32
            constrain v262 == v260                              // std/field/mod.nr:148:32
            v263 = array_get v257, index u32 1 -> u8            // std/field/mod.nr:147:25
            v264 = eq v263, u8 255                              // std/field/mod.nr:147:25
            v265 = not v264                                     // std/field/mod.nr:147:25
            v266 = unchecked_mul v259, v265                     // std/field/mod.nr:147:25
            v267 = lt v263, u8 255                              // std/field/mod.nr:148:32
            v268 = unchecked_mul v267, v266                     // std/field/mod.nr:148:32
            constrain v268 == v266                              // std/field/mod.nr:148:32
            v269 = not v266                                     // std/field/mod.nr:148:32
            v270 = unchecked_mul v269, v260                     // std/field/mod.nr:148:32
            v271 = unchecked_add v266, v270                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v272 = not v271                                     // std/field/mod.nr:145:25
            v273 = array_get v257, index u32 2 -> u8            // std/field/mod.nr:147:25
            v274 = eq v273, u8 255                              // std/field/mod.nr:147:25
            v275 = not v274                                     // std/field/mod.nr:147:25
            v276 = unchecked_mul v272, v275                     // std/field/mod.nr:147:25
            v277 = lt v273, u8 255                              // std/field/mod.nr:148:32
            v278 = unchecked_mul v277, v276                     // std/field/mod.nr:148:32
            constrain v278 == v276                              // std/field/mod.nr:148:32
            v279 = not v276                                     // std/field/mod.nr:148:32
            v280 = unchecked_mul v279, v271                     // std/field/mod.nr:148:32
            v281 = unchecked_add v276, v280                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v282 = not v281                                     // std/field/mod.nr:145:25
            v283 = array_get v257, index u32 3 -> u8            // std/field/mod.nr:147:25
            v284 = eq v283, u8 255                              // std/field/mod.nr:147:25
            v285 = not v284                                     // std/field/mod.nr:147:25
            v286 = unchecked_mul v282, v285                     // std/field/mod.nr:147:25
            v287 = lt v283, u8 255                              // std/field/mod.nr:148:32
            v288 = unchecked_mul v287, v286                     // std/field/mod.nr:148:32
            constrain v288 == v286                              // std/field/mod.nr:148:32
            v289 = not v286                                     // std/field/mod.nr:148:32
            v290 = unchecked_mul v289, v281                     // std/field/mod.nr:148:32
            v291 = unchecked_add v286, v290                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v291 == u1 1                              // std/field/mod.nr:153:20
            v292 = array_get v59, index u32 6 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v293 = truncate v292 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v294 = cast v293 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v295 = call to_be_radix(v294, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v296 = array_get v295, index u32 0 -> u8            // std/field/mod.nr:147:25
            v297 = eq v296, u8 127                              // std/field/mod.nr:147:25
            v298 = not v297                                     // std/field/mod.nr:147:25
            v299 = lt v296, u8 127                              // std/field/mod.nr:148:32
            v300 = unchecked_mul v299, v298                     // std/field/mod.nr:148:32
            constrain v300 == v298                              // std/field/mod.nr:148:32
            v301 = array_get v295, index u32 1 -> u8            // std/field/mod.nr:147:25
            v302 = eq v301, u8 255                              // std/field/mod.nr:147:25
            v303 = not v302                                     // std/field/mod.nr:147:25
            v304 = unchecked_mul v297, v303                     // std/field/mod.nr:147:25
            v305 = lt v301, u8 255                              // std/field/mod.nr:148:32
            v306 = unchecked_mul v305, v304                     // std/field/mod.nr:148:32
            constrain v306 == v304                              // std/field/mod.nr:148:32
            v307 = not v304                                     // std/field/mod.nr:148:32
            v308 = unchecked_mul v307, v298                     // std/field/mod.nr:148:32
            v309 = unchecked_add v304, v308                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v310 = not v309                                     // std/field/mod.nr:145:25
            v311 = array_get v295, index u32 2 -> u8            // std/field/mod.nr:147:25
            v312 = eq v311, u8 255                              // std/field/mod.nr:147:25
            v313 = not v312                                     // std/field/mod.nr:147:25
            v314 = unchecked_mul v310, v313                     // std/field/mod.nr:147:25
            v315 = lt v311, u8 255                              // std/field/mod.nr:148:32
            v316 = unchecked_mul v315, v314                     // std/field/mod.nr:148:32
            constrain v316 == v314                              // std/field/mod.nr:148:32
            v317 = not v314                                     // std/field/mod.nr:148:32
            v318 = unchecked_mul v317, v309                     // std/field/mod.nr:148:32
            v319 = unchecked_add v314, v318                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v320 = not v319                                     // std/field/mod.nr:145:25
            v321 = array_get v295, index u32 3 -> u8            // std/field/mod.nr:147:25
            v322 = eq v321, u8 255                              // std/field/mod.nr:147:25
            v323 = not v322                                     // std/field/mod.nr:147:25
            v324 = unchecked_mul v320, v323                     // std/field/mod.nr:147:25
            v325 = lt v321, u8 255                              // std/field/mod.nr:148:32
            v326 = unchecked_mul v325, v324                     // std/field/mod.nr:148:32
            constrain v326 == v324                              // std/field/mod.nr:148:32
            v327 = not v324                                     // std/field/mod.nr:148:32
            v328 = unchecked_mul v327, v319                     // std/field/mod.nr:148:32
            v329 = unchecked_add v324, v328                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v329 == u1 1                              // std/field/mod.nr:153:20
            v330 = array_get v59, index u32 7 -> u32            // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v331 = truncate v330 to 31 bits, max_bit_size: 32   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v332 = cast v331 as Field                           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:518:33
            v333 = call to_be_radix(v332, u32 256) -> [u8; 4]   // std/field/mod.nr:175:9
            v334 = array_get v333, index u32 0 -> u8            // std/field/mod.nr:147:25
            v335 = eq v334, u8 127                              // std/field/mod.nr:147:25
            v336 = not v335                                     // std/field/mod.nr:147:25
            v337 = lt v334, u8 127                              // std/field/mod.nr:148:32
            v338 = unchecked_mul v337, v336                     // std/field/mod.nr:148:32
            constrain v338 == v336                              // std/field/mod.nr:148:32
            v339 = array_get v333, index u32 1 -> u8            // std/field/mod.nr:147:25
            v340 = eq v339, u8 255                              // std/field/mod.nr:147:25
            v341 = not v340                                     // std/field/mod.nr:147:25
            v342 = unchecked_mul v335, v341                     // std/field/mod.nr:147:25
            v343 = lt v339, u8 255                              // std/field/mod.nr:148:32
            v344 = unchecked_mul v343, v342                     // std/field/mod.nr:148:32
            constrain v344 == v342                              // std/field/mod.nr:148:32
            v345 = not v342                                     // std/field/mod.nr:148:32
            v346 = unchecked_mul v345, v336                     // std/field/mod.nr:148:32
            v347 = unchecked_add v342, v346                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v348 = not v347                                     // std/field/mod.nr:145:25
            v349 = array_get v333, index u32 2 -> u8            // std/field/mod.nr:147:25
            v350 = eq v349, u8 255                              // std/field/mod.nr:147:25
            v351 = not v350                                     // std/field/mod.nr:147:25
            v352 = unchecked_mul v348, v351                     // std/field/mod.nr:147:25
            v353 = lt v349, u8 255                              // std/field/mod.nr:148:32
            v354 = unchecked_mul v353, v352                     // std/field/mod.nr:148:32
            constrain v354 == v352                              // std/field/mod.nr:148:32
            v355 = not v352                                     // std/field/mod.nr:148:32
            v356 = unchecked_mul v355, v347                     // std/field/mod.nr:148:32
            v357 = unchecked_add v352, v356                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            v358 = not v357                                     // std/field/mod.nr:145:25
            v359 = array_get v333, index u32 3 -> u8            // std/field/mod.nr:147:25
            v360 = eq v359, u8 255                              // std/field/mod.nr:147:25
            v361 = not v360                                     // std/field/mod.nr:147:25
            v362 = unchecked_mul v358, v361                     // std/field/mod.nr:147:25
            v363 = lt v359, u8 255                              // std/field/mod.nr:148:32
            v364 = unchecked_mul v363, v362                     // std/field/mod.nr:148:32
            constrain v364 == v362                              // std/field/mod.nr:148:32
            v365 = not v362                                     // std/field/mod.nr:148:32
            v366 = unchecked_mul v365, v357                     // std/field/mod.nr:148:32
            v367 = unchecked_add v362, v366                     // std/field/mod.nr:148:32
            enable_side_effects u1 1                            // std/field/mod.nr:147:25
            constrain v367 == u1 1                              // std/field/mod.nr:153:20
            constrain v65 == u8 227                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v71 == u8 176                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v83 == u8 196                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v93 == u8 66                              // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v106 == u8 152                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v111 == u8 252                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v121 == u8 28                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v131 == u8 20                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v144 == u8 154                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v149 == u8 251                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v159 == u8 244                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v169 == u8 200                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v182 == u8 153                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v187 == u8 111                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v197 == u8 185                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v207 == u8 36                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v220 == u8 39                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v225 == u8 174                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v235 == u8 65                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v245 == u8 228                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v258 == u8 100                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v263 == u8 155                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v273 == u8 147                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v283 == u8 76                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v296 == u8 164                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v301 == u8 149                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v311 == u8 153                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v321 == u8 27                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v334 == u8 120                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v339 == u8 82                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v349 == u8 184                            // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            constrain v359 == u8 85                             // /Users/paradox/Desktop/projects/sha256/src/sha256/tests.nr:35:15
            return
        }
        brillig(inline) predicate_pure fn attach_len_to_msg_block f1 {
        b0(v18: [u32; 16], v19: u32, v20: u32):
            v23 = allocate -> &mut [u32; 16]                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            store v18 at v23                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v24 = allocate -> &mut u32                          // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            store v19 at v24                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:292:23
            v25 = truncate v19 to 2 bits, max_bit_size: 32      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:455:18
            v27 = eq v25, u32 0                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:456:8
            jmpif v27 then: b1, else: b2
        b1():
            v45 = load v24 -> u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:465:15
            v46 = div v45, u32 4                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:465:15
            jmp b3(v46)
        b2():
            v28 = div v19, u32 4                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:458:17
            v29 = sub u32 4, v25                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:459:21
            v30 = lt v28, u32 16                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:459:21
            constrain v30 == u1 1, \"Index out of bounds\"        // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:459:21
            v32 = array_get v18, index v28 -> u32               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:460:39
            v33 = truncate v29 to 8 bits, max_bit_size: 32      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:460:53
            v34 = cast v33 as u8                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:460:53
            v36 = mul u8 8, v34                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:414:18
            v37 = shr v32, v36                                  // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:414:9
            v39 = lt v34, u8 4                                  // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:386:12
            jmpif v39 then: b4, else: b5
        b3(v21: u32):
            v47 = lt v21, u32 14                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:465:41
            jmpif v47 then: b6, else: b7
        b4():
            v40 = shl v37, v36                                  // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:389:13
            v42 = lt v36, u8 32                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:389:13
            constrain v42 == u1 1, \"attempt to bit-shift with overflow\" // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:389:13
            jmp b8(v40)
        b5():
            jmp b8(u32 0)
        b6():
            v95 = load v23 -> [u32; 16]                         // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:24
            v96 = lt v21, u32 16                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:24
            constrain v96 == u1 1, \"Index out of bounds\"        // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:24
            v97 = array_set v95, index v21, value u32 0         // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:9
            store v97 at v23                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:9
            v99 = unchecked_add v21, u32 1                      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:466:9
            jmp b3(v99)
        b7():
            v49 = mul u32 8, v20                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:470:15
            v50 = cast v49 as u64                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:471:46
            v52 = shr v50, u8 56                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:434:11
            v53 = truncate v52 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:434:10
            v55 = shr v50, u8 48                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:435:11
            v56 = truncate v55 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:435:10
            v58 = shr v50, u8 40                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:436:11
            v59 = truncate v58 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:436:10
            v60 = shr v50, u8 32                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:437:11
            v61 = truncate v60 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:437:10
            v63 = shr v50, u8 24                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:438:11
            v64 = truncate v63 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:438:10
            v66 = shr v50, u8 16                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:439:11
            v67 = truncate v66 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:439:10
            v68 = shr v50, u8 8                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:440:11
            v69 = truncate v68 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:440:10
            v70 = truncate v50 to 8 bits, max_bit_size: 64      // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:441:10
            v71 = cast v53 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:32
            v72 = shl v71, u8 24                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:31
            v73 = cast v56 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:473:12
            v74 = shl v73, u8 16                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:473:11
            v75 = or v72, v74                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:31
            v76 = cast v59 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:474:12
            v77 = shl v76, u8 8                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:474:11
            v78 = or v75, v77                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:31
            v79 = cast v61 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:475:12
            v80 = or v78, v79                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:31
            v81 = load v23 -> [u32; 16]                         // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:31
            v83 = array_set v81, index u32 15 minus 1, value v80        // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:472:5
            v84 = cast v64 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:36
            v85 = shl v84, u8 24                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:35
            v86 = cast v67 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:478:12
            v87 = shl v86, u8 16                                // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:478:11
            v88 = or v85, v87                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:35
            v89 = cast v69 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:479:12
            v90 = shl v89, u8 8                                 // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:479:11
            v91 = or v88, v90                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:35
            v92 = cast v70 as u32                               // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:480:12
            v93 = or v91, v92                                   // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:35
            v94 = array_set v83, index u32 16 minus 1, value v93        // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:5
            store v94 at v23                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:477:5
            return v94
        b8(v22: u32):
            v43 = array_set v18, index v28, value v22           // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:460:9
            store v43 at v23                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:460:9
            v44 = add v19, v29                                  // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:461:24
            store v44 at v24                                    // /Users/paradox/Desktop/projects/sha256/src/sha256.nr:461:24
            jmp b1()
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
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_get v0, index u32 4 minus 3 -> Field
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
        vec![Value::Numeric(NumericValue::U32(1))],
    );
    assert_eq!(value, from_constant(0_u32.into(), NumericType::NativeField));
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

    let v0 = values[0].as_array_or_slice().unwrap();
    let v1 = values[1].as_array_or_slice().unwrap();
    let v2 = values[2].as_array_or_slice().unwrap();

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
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            v1 = array_set v0, index u32 4 minus 3, value Field 5
            return v0, v1
        }
    ",
    );

    let v0 = values[0].as_array_or_slice().unwrap();
    let v1 = values[1].as_array_or_slice().unwrap();

    // acir function, so all rcs are 1
    assert_eq!(*v0.rc.borrow(), 1);
    assert_eq!(*v1.rc.borrow(), 1);

    let one = from_constant(1u32.into(), NumericType::NativeField);
    let two = from_constant(2u32.into(), NumericType::NativeField);
    let five = from_constant(5u32.into(), NumericType::NativeField);

    // v0 was not mutated
    assert_eq!(*v0.elements.borrow(), vec![one.clone(), two.clone()]);
    // v1 was mutated
    assert_eq!(*v1.elements.borrow(), vec![one, five]);
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
    assert_eq!(values[1], Value::slice(one_two, Arc::new(vec![Type::field()])));

    let hello = vecmap(b"Hello", |char| from_constant((*char as u32).into(), NumericType::char()));
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
