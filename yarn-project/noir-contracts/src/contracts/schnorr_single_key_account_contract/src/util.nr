use dep::protocol_types::address::AztecAddress;
use dep::std::{schnorr::verify_signature};
use dep::aztec::address::compute_address;
use crate::auth_oracle::{AuthWitness};

pub fn recover_address(message_hash: Field, witness: AuthWitness) -> AztecAddress {
    let message_bytes = message_hash.to_be_bytes(32);
    let verification = verify_signature(witness.owner.x,
        witness.owner.y,
        witness.signature,
        message_bytes);
    assert(verification == true);

    compute_address(witness.owner.x, witness.owner.y, witness.partial_address)
}
