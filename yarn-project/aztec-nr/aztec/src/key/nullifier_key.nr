use dep::protocol_types::{
    address::AztecAddress,
    grumpkin_point::GrumpkinPoint,
};

pub fn validate_nullifier_key_against_address(_address: AztecAddress, _nullifier_public_key: GrumpkinPoint) {
    // TODO: Nullifier public key should be part of the address.
}
