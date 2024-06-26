use acvm::{AcirField, FieldElement};
use num_bigint::BigInt;

pub fn field_element_from_big_int(big_int: &BigInt) -> FieldElement {
    // TODO(ary): check if this conversion is okay (the sign is lost, but it's probably fine)
    let big_uint = big_int.magnitude();
    FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be())
}

pub fn truncate_big_int_to_u128(big_int: &BigInt) -> u128 {
    // TODO(ary): use a more efficient way to do this
    field_element_from_big_int(big_int).to_u128()
}
