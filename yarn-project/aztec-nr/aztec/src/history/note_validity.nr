use crate::{
    context::PrivateContext,
    history::{
        note_inclusion::prove_note_inclusion,
        nullifier_non_inclusion::prove_note_not_nullified,
    },
    note::note_interface::NoteInterface,
};

// A helper function that proves that a note is valid at the given block number
pub fn prove_note_validity<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note_with_header: Note,
    block_number: u32, // The block at which we'll prove that the note exists
    context: &mut PrivateContext
) {
    prove_note_inclusion(note_interface, note_with_header, block_number, *context);
    prove_note_not_nullified(note_interface, note_with_header, block_number, context);
}
