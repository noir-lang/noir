use crate::context::{
    PrivateContext,
    PublicContext,
};
use crate::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::{compute_note_hash_for_insertion, compute_note_hash_for_consumption},
};
use crate::oracle::notes::{notify_created_note, notify_nullified_note};

pub fn create_note<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note: &mut Note,
    broadcast: bool
) where Note: NoteInterface<N> {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    // TODO: change this to note.setHeader(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    // As `is_transient` is true, this will compute the inner note hsah
    let inner_note_hash = compute_note_hash_for_insertion(*note);

    // TODO: Strong typing required because of https://github.com/noir-lang/noir/issues/4088
    let serialized_note: [Field; N] = Note::serialize_content(*note);
    assert(notify_created_note(storage_slot, serialized_note, inner_note_hash) == 0);

    context.push_new_note_hash(inner_note_hash);

    if broadcast {
        Note::broadcast(*note, context, storage_slot);
    }
}

pub fn create_note_hash_from_public<Note, N>(context: &mut PublicContext, storage_slot: Field, note: &mut Note) where Note: NoteInterface<N> {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    // TODO: change this to note.setHeader(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    let inner_note_hash = compute_note_hash_for_insertion(*note);

    context.push_new_note_hash(inner_note_hash);
}

pub fn destroy_note<Note, N>(context: &mut PrivateContext, note: Note) where Note: NoteInterface<N> {
    let mut nullifier = 0;
    let mut consumed_note_hash: Field = 0;
    nullifier = note.compute_nullifier(context);

    // We also need the note hash corresponding to the "nullifier"
    let header = note.get_header();
    // `consumed_note_hash` is used to inform the kernel which pending note hash
    // the nullifier corresponds to so they can be matched and both squashed/deleted.
    // nonzero nonce implies "persistable" nullifier (nullifies a persistent/in-tree
    // note hash) in which case `consumed_note_hash` is not used since the kernel
    // just siloes and forwards the nullifier to its output.
    if (header.is_transient) {
        // TODO(1718): Can we reuse the note hash computed in `compute_nullifier`?
        consumed_note_hash = compute_note_hash_for_consumption(note);
    }
    assert(notify_nullified_note(nullifier, consumed_note_hash) == 0);

    context.push_new_nullifier(nullifier, consumed_note_hash)
}
