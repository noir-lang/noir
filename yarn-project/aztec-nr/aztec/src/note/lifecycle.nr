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
use dep::protocol_types::traits::{Serialize, Deserialize};

pub fn create_note<Note, N>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note: &mut Note,
    broadcast: bool
) where Note: NoteInterface + Serialize<N> + Deserialize<N> {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    // TODO: change this to note.setHeader(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    // As `is_transient` is true, this will compute the inner note hsah
    let inner_note_hash = compute_note_hash_for_read_or_nullify(*note);

    // TODO: Strong typing required because of https://github.com/noir-lang/noir/issues/4088
    let serialized_note: [Field; N] = Note::serialize(*note);
    assert(notify_created_note(storage_slot, serialized_note, inner_note_hash) == 0);

    context.push_new_note_hash(inner_note_hash);

    if broadcast {
        Note::broadcast(*note, context, storage_slot);
    }
}

pub fn create_note_hash_from_public<Note>(context: &mut PublicContext, storage_slot: Field, note: &mut Note) where Note: NoteInterface {
    let contract_address = (*context).this_address();

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, is_transient: true };
    // TODO: change this to note.setHeader(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    let inner_note_hash = compute_note_hash_for_read_or_nullify(*note);

    context.push_new_note_hash(inner_note_hash);
}

pub fn destroy_note<Note>(context: &mut PrivateContext, note: Note) where Note: NoteInterface {
    let mut nullifier = 0;
    let mut nullified_commitment: Field = 0;
    nullifier = note.compute_nullifier(context);

    // We also need the note commitment corresponding to the "nullifier"
    let header = note.get_header();
    // `nullified_commitment` is used to inform the kernel which pending commitment
    // the nullifier corresponds to so they can be matched and both squashed/deleted.
    // nonzero nonce implies "persistable" nullifier (nullifies a persistent/in-tree
    // commitment) in which case `nullified_commitment` is not used since the kernel
    // just siloes and forwards the nullifier to its output.
    if (header.is_transient) {
        // TODO(1718): Can we reuse the note commitment computed in `compute_nullifier`?
        nullified_commitment = compute_note_hash_for_read_or_nullify(note);
    }
    assert(notify_nullified_note(nullifier, nullified_commitment) == 0);

    context.push_new_nullifier(nullifier, nullified_commitment)
}
