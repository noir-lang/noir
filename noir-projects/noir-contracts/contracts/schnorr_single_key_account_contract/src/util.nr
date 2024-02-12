use dep::aztec::protocol_types::address::{AztecAddress, PublicKeysHash};
use dep::std::{schnorr::verify_signature};
use crate::auth_oracle::{AuthWitness};

pub fn recover_address(message_hash: Field, witness: AuthWitness) -> AztecAddress {
    let message_bytes = message_hash.to_be_bytes(32);
    let verification = verify_signature(
        witness.owner.x,
        witness.owner.y,
        witness.signature,
        message_bytes
    );
    assert(verification == true);

    AztecAddress::compute(
        PublicKeysHash::compute(witness.owner),
        witness.partial_address
    )
}
