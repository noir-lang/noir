use crate::context::Context;
use crate::note::lifecycle::{create_note, destroy_note};
use crate::note::{
    note_getter::{get_notes, ensure_note_exists},
    note_getter_options::NoteGetterOptions,
    note_interface::NoteInterface,
};
use crate::oracle::create_commitment::create_commitment;

struct Set<Note, N> {
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Set<Note, N> {
    fn new(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Self {
        Set { storage_slot, note_interface }
    }

    fn insert(self, context: Context, mut note: Note) -> Context {
        create_note(context, self.storage_slot, note, self.note_interface)
    }

    fn insert_from_public(self, note: Note) {
        let compute_note_hash = self.note_interface.compute_note_hash;
        let note_hash = compute_note_hash(note);
        create_commitment(note_hash);
    }

    fn has(self, context: Context, note: Note) -> (Context, Note) {
        ensure_note_exists(context, self.storage_slot, self.note_interface, note)
    }

    fn remove(self, context: Context, note: Note) -> Context {
        destroy_note(context, self.storage_slot, note, self.note_interface)
    }

    fn get_notes<S, P>(
        self,
        mut context: Context,
        options: NoteGetterOptions<Note, N, S, P>,
    ) -> (Context, [Note; S]) {
        let storage_slot = self.storage_slot;
        let (new_context, notes) = get_notes(context, storage_slot, self.note_interface, options);
        (new_context, notes)
    }
}
