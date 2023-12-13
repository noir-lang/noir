use dep::protocol_types::{
    address::AztecAddress,
    constants::GENERATOR_INDEX__CONTRACT_ADDRESS,
    hash::pedersen_hash,
};

pub fn compute_address(pub_key_x: Field, pub_key_y: Field, partial_address: Field) -> AztecAddress {
    AztecAddress::from_field(
        pedersen_hash(
            [pub_key_x, pub_key_y, partial_address],
            GENERATOR_INDEX__CONTRACT_ADDRESS
        )
    )
}
