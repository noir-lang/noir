use crate::ssa::interpreter::{
    tests::expect_value,
    value::{NumericValue, Value},
};

#[test]
fn to_le_bits() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add Field 0, Field 0
            v1 = call to_le_bits(v0) -> [u1; 2]
            v2 = array_get v1, index u32 0 -> u1
            v3 = not v2
            return v3
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::U1(true)));
}

#[test]
fn to_le_radix() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add Field 0, Field 0
            v1 = call to_le_radix(v0, u32 2) -> [u8; 32]
            v2 = array_get v1, index u32 0 -> u8
            v3 = not v2
            return v3
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::U8(255)));
}

#[test]
fn as_witness() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
        b0():
            v0 = add Field 0, Field 1
            call as_witness(v0)
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::Field(1_u128.into())));
}
