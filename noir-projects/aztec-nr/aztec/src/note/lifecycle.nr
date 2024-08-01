use crate::context::{PrivateContext, PublicContext};
use crate::note::{
    note_header::NoteHeader, note_interface::NoteInterface,
    utils::{compute_slotted_note_hash, compute_note_hash_for_consumption}, note_emission::NoteEmission
};
use crate::oracle::notes::{notify_created_note, notify_nullified_note};

pub fn create_note<Note, N, M>(
    context: &mut PrivateContext,
    storage_slot: Field,
    note: &mut Note
) -> NoteEmission<Note> where Note: NoteInterface<N, M> {
    let contract_address = (*context).this_address();
    let note_hash_counter = context.side_effect_counter;

    let header = NoteHeader { contract_address, storage_slot, nonce: 0, note_hash_counter };
    note.set_header(header);
    let slotted_note_hash = compute_slotted_note_hash(*note);

    let serialized_note = Note::serialize_content(*note);
    assert(
        notify_created_note(
        storage_slot,
        Note::get_note_type_id(),
        serialized_note,
        slotted_note_hash,
        note_hash_counter
    )
        == 0
    );

    context.push_note_hash(slotted_note_hash);

    NoteEmission::new(*note)
}

pub fn create_note_hash_from_public<Note, N, M>(
    context: &mut PublicContext,
    storage_slot: Field,
    note: &mut Note
) where Note: NoteInterface<N, M> {
    let contract_address = (*context).this_address();
    // Public note hashes are transient, but have no side effect counters, so we just need note_hash_counter != 0
    let header = NoteHeader { contract_address, storage_slot, nonce: 0, note_hash_counter: 1 };
    note.set_header(header);
    let slotted_note_hash = compute_slotted_note_hash(*note);

    context.push_note_hash(slotted_note_hash);
}

pub fn destroy_note<Note, N, M>(
    context: &mut PrivateContext,
    note: Note
) where Note: NoteInterface<N, M> {
    let (note_hash, nullifier) = note.compute_note_hash_and_nullifier(context);

    let note_hash_counter = note.get_header().note_hash_counter;
    let note_hash_for_consumption = if (note_hash_counter == 0) {
        // Counter is zero, so we're nullifying a settled note and we don't populate the note_hash with real value.
        0
    } else {
        // A non-zero note hash counter implies that we're nullifying a pending note (i.e. one that has not yet been
        // persisted in the trees and is instead in the pending new note hashes array). In such a case we populate its
        // hash with real value to inform the kernel which note we're nullifyng so that it can either squash both
        // the note and the nullifier if it's an inner note hash, or check that the it matches a pending note if it's
        // a siloed note hash.
        note_hash
    };

    let nullifier_counter = context.side_effect_counter;
    assert(notify_nullified_note(nullifier, note_hash_for_consumption, nullifier_counter) == 0);

    context.push_nullifier_for_note_hash(nullifier, note_hash_for_consumption)
}
