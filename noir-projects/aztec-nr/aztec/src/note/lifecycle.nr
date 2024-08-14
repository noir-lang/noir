use crate::context::{PrivateContext, PublicContext};
use crate::note::{
    note_header::NoteHeader, note_interface::NoteInterface,
    utils::{compute_note_hash_for_read_request, compute_note_hash_for_nullify_internal},
    note_emission::NoteEmission
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
    // TODO(#7771): inject compute_note_hash(...) func to notes with macros.
    let note_hash = note.compute_note_hiding_point().x;

    let serialized_note = Note::serialize_content(*note);
    assert(
        notify_created_note(
        storage_slot,
        Note::get_note_type_id(),
        serialized_note,
        note_hash,
        note_hash_counter
    )
        == 0
    );

    context.push_note_hash(note_hash);

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
    // TODO(#7771): inject compute_note_hash(...) func to notes with macros.
    let note_hash = note.compute_note_hiding_point().x;

    context.push_note_hash(note_hash);
}

// Note: This function is currently totally unused.
pub fn destroy_note<Note, N, M>(
    context: &mut PrivateContext,
    note: Note
) where Note: NoteInterface<N, M> {
    let note_hash_for_read_request = compute_note_hash_for_read_request(note);

    destroy_note_unsafe(context, note, note_hash_for_read_request)
}

pub fn destroy_note_unsafe<Note, N, M>(
    context: &mut PrivateContext,
    note: Note,
    note_hash_for_read_request: Field
) where Note: NoteInterface<N, M> {
    let note_hash_for_nullify = compute_note_hash_for_nullify_internal(note, note_hash_for_read_request);
    let nullifier = note.compute_nullifier(context, note_hash_for_nullify);

    let note_hash_counter = note.get_header().note_hash_counter;
    let notification_note_hash = if (note_hash_counter == 0) {
        // Counter is zero, so we're nullifying a settled note and we don't populate the note_hash with real value.
        0
    } else {
        // A non-zero note hash counter implies that we're nullifying a pending note (i.e. one that has not yet been
        // persisted in the trees and is instead in the pending new note hashes array). In such a case we populate its
        // hash with real value to inform the kernel which note we're nullifyng so that it can either squash both
        // the note and the nullifier if it's an inner note hash, or check that the it matches a pending note if it's
        // a siloed note hash.
        note_hash_for_nullify
    };

    let nullifier_counter = context.side_effect_counter;
    assert(notify_nullified_note(nullifier, notification_note_hash, nullifier_counter) == 0);

    context.push_nullifier_for_note_hash(nullifier, notification_note_hash)
}

