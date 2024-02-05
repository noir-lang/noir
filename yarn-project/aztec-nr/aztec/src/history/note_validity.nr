use crate::{
    context::PrivateContext,
    history::{
        note_inclusion::prove_note_inclusion,
        note_inclusion::prove_note_inclusion_at,
        nullifier_non_inclusion::prove_note_not_nullified,
        nullifier_non_inclusion::prove_note_not_nullified_at,
    },
    note::note_interface::NoteInterface,
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
    prove_note_inclusion_at(note, block_number, *context);
    prove_note_not_nullified_at(note, block_number, context);
}
