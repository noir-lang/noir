use dep::protocol_types::{
    address::AztecAddress,
    grumpkin_point::GrumpkinPoint,
    grumpkin_private_key::GrumpkinPrivateKey,
};

struct NullifierKeyPair {
    account: AztecAddress,
    public_key: GrumpkinPoint,
    secret_key: GrumpkinPrivateKey,
}

#[oracle(getNullifierKeyPair)]
fn get_nullifier_key_pair_oracle(_account: AztecAddress) -> [Field; 4] {}

unconstrained fn get_nullifier_key_pair_internal(account: AztecAddress) -> NullifierKeyPair {
    let result = get_nullifier_key_pair_oracle(account);
    NullifierKeyPair {
        account,
        public_key: GrumpkinPoint { x: result[0], y: result[1] },
        secret_key: GrumpkinPrivateKey { high: result[2], low: result[3] }
    }
}

pub fn get_nullifier_key_pair(account: AztecAddress) -> NullifierKeyPair {
    get_nullifier_key_pair_internal(account)
}

pub fn get_nullifier_secret_key(account: AztecAddress) -> GrumpkinPrivateKey {
    get_nullifier_key_pair_internal(account).secret_key
}
