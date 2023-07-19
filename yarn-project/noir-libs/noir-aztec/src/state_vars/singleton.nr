use crate::context::Context;
use crate::oracle;
use crate::note::lifecycle::create_note;
use crate::note::lifecycle::destroy_note;
use crate::note::note_getter::get_note;
use crate::note::note_interface::NoteInterface;
use crate::constants_gen::GENERATOR_INDEX__INITIALISATION_NULLIFIER;

struct Singleton<Note, N> {
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Singleton<Note, N> {
    fn new(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Self {
        Singleton { storage_slot, note_interface }
    }

    unconstrained fn is_initialised(self) -> bool {
        let nullifier = self.compute_initialisation_nullifier();
        oracle::notes::is_nullifier_emitted(nullifier)
    }

    fn initialise(self, mut context: Context, note: Note) -> Context {
        // Nullify the storage slot.
        let nullifier = self.compute_initialisation_nullifier();
        context = context.push_new_nullifier(nullifier);

        create_note(context, self.storage_slot, note, self.note_interface)
    }

    fn compute_initialisation_nullifier(self) -> Field {
        let storage_slot = self.storage_slot;
        dep::std::hash::pedersen([
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
            storage_slot,
        ])[0]
    }

    fn replace(self, mut context: Context, new_note: Note) -> (Context, Note) {
        let (new_context, prev_note) = get_note(context, self.storage_slot, self.note_interface);
        context = new_context;

        // Nullify previous note.
        context = destroy_note(context, self.storage_slot, prev_note, self.note_interface);

        // Add replacement note.
        context = create_note(context, self.storage_slot, new_note, self.note_interface);

        (context, prev_note)
    }

    fn get_note(self, mut context: Context, replicate: fn (Note) -> Note) -> (Context, Note) {
        let (new_context, note) = get_note(context, self.storage_slot, self.note_interface);
        context = new_context;

        // Nullify current note.
        context = destroy_note(context, self.storage_slot, note, self.note_interface);

        // Add replica.
        let new_note = replicate(note);
        context = create_note(context, self.storage_slot, new_note, self.note_interface);

        (context, note)
    }
}