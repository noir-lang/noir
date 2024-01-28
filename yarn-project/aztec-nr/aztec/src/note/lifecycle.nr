use crate::abi::PublicContextInputs;
use crate::context::{
    PrivateContext,
    PublicContext,
};
use crate::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
};
use crate::oracle::notes::{notify_created_note, notify_nullified_note};

pub fn create_note<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note: &mut Note,
    note_interface: NoteInterface<Note, N>,
    broadcast: bool
) {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    let set_header = note_interface.set_header;
    set_header(note, header);
    // As `is_transient` is true, this will compute the inner note hsah
    let inner_note_hash = compute_note_hash_for_read_or_nullify(note_interface, *note);

    let serialize = note_interface.serialize;
    let serialized_note = serialize(*note);
    assert(notify_created_note(storage_slot, serialized_note, inner_note_hash) == 0);

    context.push_new_note_hash(inner_note_hash);

    if broadcast {
        let broadcast = note_interface.broadcast;
        broadcast(context, storage_slot, *note);
    }
}

pub fn create_note_hash_from_public<Note, N>(
    context: &mut PublicContext,
    storage_slot: Field,
    note: &mut Note,
    note_interface: NoteInterface<Note, N>
) {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    let set_header = note_interface.set_header;
    set_header(note, header);
    let inner_note_hash = compute_note_hash_for_read_or_nullify(note_interface, *note);

    context.push_new_note_hash(inner_note_hash);
}

pub fn destroy_note<Note, N>(
    context: &mut PrivateContext,
    note: Note,
    note_interface: NoteInterface<Note, N>
) {
    let mut nullifier = 0;
    let mut nullified_commitment: Field = 0;
    let compute_nullifier = note_interface.compute_nullifier;
    nullifier = compute_nullifier(note, context);

    // We also need the note commitment corresponding to the "nullifier"
    let get_header = note_interface.get_header;
    let header = get_header(note);
    // `nullified_commitment` is used to inform the kernel which pending commitment
    // the nullifier corresponds to so they can be matched and both squashed/deleted.
    // nonzero nonce implies "persistable" nullifier (nullifies a persistent/in-tree
    // commitment) in which case `nullified_commitment` is not used since the kernel
    // just siloes and forwards the nullifier to its output.
    if (header.is_transient) {
        // TODO(1718): Can we reuse the note commitment computed in `compute_nullifier`?
        nullified_commitment = compute_note_hash_for_read_or_nullify(note_interface, note);
    }
    assert(notify_nullified_note(nullifier, nullified_commitment) == 0);

    context.push_new_nullifier(nullifier, nullified_commitment)
}
