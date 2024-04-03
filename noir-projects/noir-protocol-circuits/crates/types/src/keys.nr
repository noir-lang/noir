use crate::{address::AztecAddress, grumpkin_private_key::GrumpkinPrivateKey, hash::pedersen_hash};

fn field_to_grumpkin_private_key(val: Field) -> GrumpkinPrivateKey {
    let bytes = val.to_be_bytes(32);
    let mut v = 1;
    let mut high = 0;
    let mut low = 0;

    for i in 0..16 {
        high = high + (bytes[15 - i] as Field) * v;
        low = low + (bytes[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    GrumpkinPrivateKey { high, low }
}

pub fn compute_siloed_nullifier_secret_key(secret_key: GrumpkinPrivateKey, contract_address: AztecAddress) -> GrumpkinPrivateKey {
    // TODO: Temporary hack. Should replace it with a secure way to derive the secret key.
    // Match the way keys are derived in circuits.js/src/keys/index.ts
    let hash = pedersen_hash(
        [secret_key.high, secret_key.low, contract_address.to_field()],
        0
    );
    field_to_grumpkin_private_key(hash)
}
