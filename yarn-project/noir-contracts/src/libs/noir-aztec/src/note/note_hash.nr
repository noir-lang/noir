use dep::std::hash::{pedersen, pedersen_with_separator};
use crate::generators;

fn compute_inner_note_hash(storage_slot: Field, note_hash: Field) -> Field {
    pedersen([
        storage_slot,
        note_hash,
    ])[0]
}

fn compute_siloed_note_hash(contract_address: Field, inner_note_hash: Field) -> Field {
    // TODO: Should include nonce.
    let inputs = [contract_address, inner_note_hash];
    pedersen_with_separator(inputs, generators::OUTER_COMMITMENT)[0]
}