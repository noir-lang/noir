use dep::std::hash::{pedersen, pedersen_with_separator};
use crate::constants_gen::{GENERATOR_INDEX__UNIQUE_COMMITMENT, GENERATOR_INDEX__SILOED_COMMITMENT};

fn compute_inner_hash(storage_slot: Field, note_hash: Field) -> Field {
    // TODO(#1205) Do we need a generator index here?
    pedersen([storage_slot, note_hash])[0]
}

fn compute_siloed_hash(contract_address: Field, inner_note_hash: Field) -> Field {
    let inputs = [contract_address, inner_note_hash];
    pedersen_with_separator(inputs, GENERATOR_INDEX__SILOED_COMMITMENT)[0]
}

fn compute_unique_hash(nonce: Field, siloed_note_hash: Field) -> Field {
    let inputs = [nonce, siloed_note_hash];
    pedersen_with_separator(inputs, GENERATOR_INDEX__UNIQUE_COMMITMENT)[0]
}
