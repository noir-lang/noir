use crate::{
    context::PrivateContext,
    history::{
    note_inclusion::prove_note_inclusion, note_inclusion::_note_inclusion,
    nullifier_non_inclusion::prove_note_not_nullified, nullifier_non_inclusion::_nullifier_non_inclusion
},
    note::{utils::compute_siloed_nullifier, note_interface::NoteInterface}
};

pub fn prove_note_validity<Note, N>(note: Note, context: &mut PrivateContext) where Note: NoteInterface<N> {
    prove_note_inclusion(note, *context);
    prove_note_not_nullified(note, context);
}

// A helper function that proves that a note is valid at the given block number
pub fn prove_note_validity_at<Note, N>(
    note: Note,
    block_number: u32,
    context: &mut PrivateContext
) where Note: NoteInterface<N> {
    // We are calling the internal functions here because we want to avoid calling get_header_at twice
    let header = context.get_header_at(block_number);
    _note_inclusion(note, header);

    let nullifier = compute_siloed_nullifier(note, context);
    _nullifier_non_inclusion(nullifier, header);
}
