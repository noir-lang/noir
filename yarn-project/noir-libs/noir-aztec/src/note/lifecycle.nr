use crate::abi::PublicContextInputs;
use crate::context::Context;
use crate::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_inner_note_hash,
};
use crate::oracle::create_commitment::create_commitment;
use crate::oracle::notes::{notify_created_note, notify_nullified_note};
use crate::constants_gen::EMPTY_NULLIFIED_COMMITMENT;
use crate::types::option::Option;

fn create_note<Note, N>(
    context: &mut Context,
    storage_slot: Field,
    note: &mut Note,
    note_interface: NoteInterface<Note, N>,
) {
    let contract_address = context.inputs.call_context.storage_contract_address;

    let header = NoteHeader { contract_address, storage_slot, nonce: 0 };
    let set_header = note_interface.set_header;
    set_header(note, header);
    let inner_note_hash = compute_inner_note_hash(note_interface, *note);

    let serialise = note_interface.serialise;
    let preimage = serialise(*note);
    assert(notify_created_note(storage_slot, preimage, inner_note_hash) == 0);

    context.push_new_note_hash(inner_note_hash);
}

fn create_note_hash_from_public<Note, N>(
    inputs: PublicContextInputs,
    storage_slot: Field,
    note: &mut Note,
    note_interface: NoteInterface<Note, N>,
) {
    let contract_address = inputs.call_context.storage_contract_address;

    let header = NoteHeader { contract_address, storage_slot, nonce: 0 };
    let set_header = note_interface.set_header;
    set_header(note, header);
    let inner_note_hash = compute_inner_note_hash(note_interface, *note);

    create_commitment(inner_note_hash);
}

fn destroy_note<Note, N>(
    context: &mut Context,
    storage_slot: Field,
    note: Note,
    note_interface: NoteInterface<Note, N>,
) {
    let mut nullifier = 0;
    let mut nullified_commitment: Field = EMPTY_NULLIFIED_COMMITMENT;
    let compute_nullifier = note_interface.compute_nullifier;
    nullifier = compute_nullifier(note);

    let serialise = note_interface.serialise;
    let preimage = serialise(note);

    // We also need the note commitment corresponding to the "nullifier"
    let get_header = note_interface.get_header;
    let header = get_header(note);
    // 0 nonce implies "transient" nullifier (must nullify a commitment in this TX).
    // `nullified_commitment` is used to inform the kernel which pending commitment
    // the nullifier corresponds to so they can be matched and both squashed/deleted.
    // nonzero nonce implies "persistable" nullifier (nullifies a persistent/in-tree
    // commitment) in which case `nullified_commitment` is not used since the kernel
    // just siloes and forwards the nullier to its output.
    if (header.nonce == 0) {
        // TODO(suyash): Can we reuse the note commitment computed in `compute_nullifier`?
        nullified_commitment = compute_inner_note_hash(note_interface, note);
    }
    assert(notify_nullified_note(storage_slot, nullifier, preimage, nullified_commitment) == 0);

    context.push_new_nullifier(nullifier, nullified_commitment)
}