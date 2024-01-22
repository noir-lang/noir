use dep::protocol_types::{
    address::AztecAddress,
    grumpkin_point::GrumpkinPoint,
};
use dep::std::grumpkin_scalar::GrumpkinScalar;

struct KeyPair {
    public_key: GrumpkinPoint,
    secret_key: GrumpkinScalar,
}

#[oracle(getNullifierKeyPair)]
fn get_nullifier_key_pair_oracle(_account: AztecAddress) -> [Field; 4] {}

unconstrained fn get_nullifier_key_pair_internal(account: AztecAddress) -> KeyPair {
    let result = get_nullifier_key_pair_oracle(account);
    KeyPair {
        public_key: GrumpkinPoint { x: result[0], y: result[1] },
        secret_key: GrumpkinScalar { high: result[2], low: result[3] }
    }
}

pub fn get_nullifier_key_pair(account: AztecAddress) -> KeyPair {
    get_nullifier_key_pair_internal(account)
}

pub fn get_nullifier_secret_key(account: AztecAddress) -> GrumpkinScalar {
    get_nullifier_key_pair_internal(account).secret_key
}
