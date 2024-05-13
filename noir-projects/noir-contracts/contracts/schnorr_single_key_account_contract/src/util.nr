use dep::std::{schnorr::verify_signature_slice};
use dep::aztec::prelude::AztecAddress;
use crate::auth_oracle::AuthWitness;

pub fn recover_address(message_hash: Field, witness: AuthWitness) -> AztecAddress {
    let message_bytes = message_hash.to_be_bytes(32);
    // In a single key account contract we re-used ivpk_m as signing key
    let verification = verify_signature_slice(
        witness.keys.ivpk_m.x,
        witness.keys.ivpk_m.y,
        witness.signature,
        message_bytes
    );
    assert(verification == true);

    AztecAddress::compute(witness.keys.hash(), witness.partial_address)
}
