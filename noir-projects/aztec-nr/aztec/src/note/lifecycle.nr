use dep::protocol_types::grumpkin_point::GrumpkinPoint;
use crate::context::{PrivateContext, PublicContext};
use crate::note::{
    note_header::NoteHeader, note_interface::NoteInterface,
    utils::{compute_note_hash_for_insertion, compute_note_hash_for_consumption}
};
use crate::oracle::notes::{notify_created_note, notify_nullified_note};

pub fn create_note<Note, N, M>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note: &mut Note,
    ovpk_m: GrumpkinPoint,
    ivpk_m: GrumpkinPoint
) where Note: NoteInterface<N, M> {
    let contract_address = (*context).this_address();
    let note_hash_counter = context.side_effect_counter;

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, note_hash_counter };
    // TODO: change this to note.set_header(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    let inner_note_hash = compute_note_hash_for_insertion(*note);

    // TODO: Strong typing required because of https://github.com/noir-lang/noir/issues/4088
    let serialized_note: [Field; N] = Note::serialize_content(*note);
    assert(
        notify_created_note(
        storage_slot,
        Note::get_note_type_id(),
        serialized_note,
        inner_note_hash,
        note_hash_counter
    )
        == 0
    );

    context.push_new_note_hash(inner_note_hash);

    Note::broadcast(*note, context, storage_slot, ovpk_m, ivpk_m);
}

pub fn create_note_hash_from_public<Note, N, M>(
    context: &mut PublicContext,
    storage_slot: Field,
    note: &mut Note
) where Note: NoteInterface<N, M> {
    let contract_address = (*context).this_address();
    // Public note hashes are transient, but have no side effect counters, so we just need note_hash_counter != 0
    let header = NoteHeader { contract_address, storage_slot, nonce: 0, note_hash_counter: 1 };
    // TODO: change this to note.set_header(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    Note::set_header(note, header);
    let inner_note_hash = compute_note_hash_for_insertion(*note);

    context.push_new_note_hash(inner_note_hash);
}

pub fn destroy_note<Note, N, M>(
    context: &mut PrivateContext,
    note: Note
) where Note: NoteInterface<N, M> {
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
    if (header.note_hash_counter != 0) {
        // TODO(1718): Can we reuse the note hash computed in `compute_nullifier`?
        consumed_note_hash = compute_note_hash_for_consumption(note);
    }

    let nullifier_counter = context.side_effect_counter;
    assert(notify_nullified_note(nullifier, consumed_note_hash, nullifier_counter) == 0);

    context.push_new_nullifier(nullifier, consumed_note_hash)
}
