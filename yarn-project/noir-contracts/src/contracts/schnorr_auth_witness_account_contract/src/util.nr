mod auth_oracle;

use dep::std::{schnorr::verify_signature, hash::pedersen_with_separator};
use dep::aztec::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;
use crate::auth_oracle::{AuthWitness};

fn recover_address(
    message_hash: Field,
    witness: AuthWitness,
) -> Field {
    let message_bytes_slice = message_hash.to_be_bytes(32);
    let mut message_bytes: [u8; 32] = [0; 32];
    for i in 0..32 {
      message_bytes[i] = message_bytes_slice[i];
    }
    
    let verification = verify_signature(witness.owner.x, witness.owner.y, witness.signature, message_bytes);
    assert(verification == true);

    let reproduced_address = pedersen_with_separator([witness.owner.x, witness.owner.y, witness.partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)[0];
    reproduced_address
}