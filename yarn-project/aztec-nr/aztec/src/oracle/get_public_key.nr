use dep::protocol_types::{
    address::{
        AztecAddress,
        PartialAddress,
    },
    grumpkin_point::GrumpkinPoint,
};

#[oracle(getPublicKeyAndPartialAddress)]
fn get_public_key_and_partial_address_oracle(_address: AztecAddress) -> [Field; 3] {}

unconstrained fn get_public_key_and_partial_address_internal(address: AztecAddress) -> [Field; 3] {
    get_public_key_and_partial_address_oracle(address)
}

pub fn get_public_key(address: AztecAddress) -> GrumpkinPoint {
    let result = get_public_key_and_partial_address_internal(address);
    let pub_key = GrumpkinPoint::new(result[0], result[1]);
    let partial_address = PartialAddress::from_field(result[2]);

    let calculated_address = AztecAddress::compute(pub_key, partial_address);
    assert(calculated_address.eq(address));

    pub_key
}
