use crate::hash::pedersen_hash;
use crate::constants_gen::{GENERATOR_INDEX__UNIQUE_COMMITMENT, GENERATOR_INDEX__SILOED_COMMITMENT};

pub fn compute_inner_hash(storage_slot: Field, note_hash: Field) -> Field {
    // TODO(#1205) Do we need a generator index here?
    pedersen_hash([storage_slot, note_hash],0)
}

pub fn compute_siloed_hash(contract_address: Field, inner_note_hash: Field) -> Field {
    let inputs = [contract_address, inner_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__SILOED_COMMITMENT)
}

pub fn compute_unique_hash(nonce: Field, siloed_note_hash: Field) -> Field {
    let inputs = [nonce, siloed_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__UNIQUE_COMMITMENT)
}
