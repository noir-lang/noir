use crate::oracle::get_public_key::get_public_key;
use crate::types::point::Point;
use dep::protocol_types::address::AztecAddress;

#[oracle(getSecretKey)]
fn get_secret_key_oracle(_owner: Point) -> [Field; dep::std::grumpkin_scalar::GRUMPKIN_SCALAR_SERIALIZED_LEN] {}

unconstrained fn get_secret_key_internal(owner_public_key: Point) -> dep::std::grumpkin_scalar::GrumpkinScalar {
    dep::std::grumpkin_scalar::deserialize_grumpkin_scalar(get_secret_key_oracle(owner_public_key))
}

pub fn get_secret_key(owner: AztecAddress) -> dep::std::grumpkin_scalar::GrumpkinScalar {
    let owner_public_key = get_public_key(owner);
    let secret = get_secret_key_internal(owner_public_key);

    // Constrain the owner - Nullifier secret key is currently just the encryption private key so we can constrain
    // the owner by deriving the public key from the secret key and checking the result.
    let computed_public_key = dep::std::grumpkin_scalar_mul::grumpkin_fixed_base(secret);
    assert(owner_public_key.x == computed_public_key[0]);
    assert(owner_public_key.y == computed_public_key[1]);

    secret
}
