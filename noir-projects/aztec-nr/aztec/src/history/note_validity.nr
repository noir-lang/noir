use crate::{context::PrivateContext, note::note_interface::NoteInterface};

use dep::protocol_types::header::Header;

trait ProveNoteValidity {
    fn prove_note_validity<Note, N, M>(header: Header, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M>;
}

impl ProveNoteValidity for Header {
    fn prove_note_validity<Note, N, M>(self, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M> {
        self.prove_note_inclusion(note);
        self.prove_note_not_nullified(note, context);
    }
}

