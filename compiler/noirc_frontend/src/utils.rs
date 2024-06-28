use acvm::{AcirField, FieldElement};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Signed;

pub fn big_int_to_field_element(big_int: &BigInt) -> FieldElement {
    let big_uint = big_int.magnitude();
    let field_element = FieldElement::from_be_bytes_reduce(&big_uint.to_bytes_be());
    if big_int.is_negative() {
        -field_element
    } else {
        field_element
    }
}

pub fn field_element_to_big_int(field: &FieldElement) -> BigInt {
    let big_uint = BigUint::from_bytes_be(&field.to_be_bytes());
    BigInt::from_biguint(Sign::Plus, big_uint)
}

pub fn truncate_big_int_to_u128(big_int: &BigInt) -> u128 {
    // TODO: use a more efficient way to do this
    big_int_to_field_element(big_int).to_u128()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_int_to_field_element() {
        let big_int = BigInt::from(1);
        let field_element = big_int_to_field_element(&big_int);
        assert_eq!(field_element.to_i128(), 1);

        let big_int = BigInt::from(-1);
        let field_element = big_int_to_field_element(&big_int);
        assert_eq!(field_element.to_i128(), -1);
    }
}
