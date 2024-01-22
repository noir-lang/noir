use crate::oracle::get_public_key::get_public_key;
use dep::protocol_types::{
    address::AztecAddress,
    grumpkin_point::GrumpkinPoint,
};
use dep::std::{
    grumpkin_scalar::GrumpkinScalar,
    grumpkin_scalar_mul::grumpkin_fixed_base,
};

pub fn validate_nullifier_key_against_address(
    address: AztecAddress,
    nullifier_public_key: GrumpkinPoint,
    nullifier_secret_key: GrumpkinScalar
) {
    // TODO: Nullifier public key should be part of the address.
    // Validation of the secret key should happen in the kernel circuit.
    let owner_public_key = get_public_key(address);
    assert(owner_public_key.x == nullifier_public_key.x);
    assert(owner_public_key.y == nullifier_public_key.y);
    let computed_public_key = grumpkin_fixed_base(nullifier_secret_key);
    assert(owner_public_key.x == computed_public_key[0]);
    assert(owner_public_key.y == computed_public_key[1]);
}
