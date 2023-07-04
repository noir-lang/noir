use crate::context::Context;
use crate::generators;
use crate::note::lifecycle::create_note;
use crate::note::lifecycle::destroy_note;
use crate::note::note_getter::get_notes;
use crate::note::note_getter_options::NoteGetterOptions;
use crate::note::note_interface::NoteInterface;

struct Set<Note, N> {
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Set<Note, N> {
    fn new(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Self {
        Set { storage_slot, note_interface }
    }

    fn insert(self, context: Context, note: Note) -> Context {
        create_note(context, self.storage_slot, note, self.note_interface)
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
