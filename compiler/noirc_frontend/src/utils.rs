use acvm::{AcirField, FieldElement};
use num_bigint::{BigInt, BigUint, Sign};

pub fn field_element_from_big_int(big_int: &BigInt) -> FieldElement {
    // TODO(ary): check if this conversion is okay (the sign is lost, but it's probably fine)
    let big_uint = big_int.magnitude();
    FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be())
}

pub fn field_element_to_big_int(field: &FieldElement) -> BigInt {
    let big_uint = BigUint::from_bytes_be(&field.to_be_bytes());
    BigInt::from_biguint(Sign::Plus, big_uint)
}

pub fn truncate_big_int_to_u128(big_int: &BigInt) -> u128 {
    // TODO(ary): use a more efficient way to do this
    field_element_from_big_int(big_int).to_u128()
}
