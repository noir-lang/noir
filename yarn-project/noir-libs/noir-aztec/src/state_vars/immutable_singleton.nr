use crate::context::Context;
use crate::note::lifecycle::create_note;
use crate::note::note_getter::get_note;
use crate::note::note_interface::NoteInterface;
use crate::oracle;
use crate::constants_gen::GENERATOR_INDEX__INITIALISATION_NULLIFIER;
use crate::constants_gen::EMPTY_NULLIFIED_COMMITMENT;

struct ImmutableSingleton<Note, N> {
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> ImmutableSingleton<Note, N> {
    fn new(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Self {
        ImmutableSingleton { storage_slot, note_interface }
    }

    unconstrained fn is_initialised(self) -> bool {
        let nullifier = self.compute_initialisation_nullifier();
        oracle::notes::is_nullifier_emitted(nullifier)
    }

    fn initialise(self, mut context: Context, note: Note) -> Context {
        // Nullify the storage slot.
        let nullifier = self.compute_initialisation_nullifier();
        context = context.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT);

        create_note(context, self.storage_slot, note, self.note_interface)
    }

    fn compute_initialisation_nullifier(self) -> Field {
        let storage_slot = self.storage_slot;
        dep::std::hash::pedersen([
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
            storage_slot,
        ])[0]
    }
    
    fn get_note(self, mut context: Context) -> (Context, Note) {
        let storage_slot = self.storage_slot;
        get_note(context, storage_slot, self.note_interface)
    }
}