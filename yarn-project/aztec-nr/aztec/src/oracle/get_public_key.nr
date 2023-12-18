use crate::types::point::Point;
use crate::address::compute_address;
use dep::protocol_types::address::AztecAddress;

#[oracle(getPublicKeyAndPartialAddress)]
fn get_public_key_and_partial_address_oracle(_address: AztecAddress) -> [Field; 3] {}

unconstrained fn get_public_key_and_partial_address_internal(address: AztecAddress) -> [Field; 3] {
    get_public_key_and_partial_address_oracle(address)
}

pub fn get_public_key(address: AztecAddress) -> Point {
    let result = get_public_key_and_partial_address_internal(address);
    let pub_key_x = result[0];
    let pub_key_y = result[1];
    let partial_address = result[2];

    let calculated_address = compute_address(pub_key_x, pub_key_y, partial_address);
    assert(calculated_address.eq(address));

    Point::new(pub_key_x, pub_key_y)
}
