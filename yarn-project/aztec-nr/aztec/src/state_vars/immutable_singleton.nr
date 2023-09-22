use crate::constants_gen::{EMPTY_NULLIFIED_COMMITMENT, GENERATOR_INDEX__INITIALISATION_NULLIFIER};
use crate::context::{PrivateContext, Context};
use crate::note::{
    lifecycle::create_note,
    note_getter::{get_note, view_notes},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
};
use crate::oracle;
use dep::std::hash::pedersen_with_separator;
use dep::std::option::Option;

struct ImmutableSingleton<Note, N> {
    context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> ImmutableSingleton<Note, N> {
    fn new(
        context: Context,
        storage_slot: Field,
        note_interface: NoteInterface<Note, N>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        ImmutableSingleton {
            context,
            storage_slot,
            note_interface,
        }
    }

    unconstrained fn is_initialized(self) -> bool {
        let nullifier = self.compute_initialisation_nullifier();
        oracle::notes::is_nullifier_emitted(nullifier)
    }

    fn initialize(self, note: &mut Note) {
        // Nullify the storage slot.
        let nullifier = self.compute_initialisation_nullifier();
        self.context.private
            .unwrap()
            .push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT);

        create_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }

    fn compute_initialisation_nullifier(self) -> Field {
        pedersen_with_separator(
            [self.storage_slot],
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
        )[0]
    }

    fn get_note(self) -> Note {
        let storage_slot = self.storage_slot;
        get_note(self.context.private.unwrap(), storage_slot, self.note_interface)
    }

    unconstrained fn view_note(self) -> Note {
        let options = NoteViewerOptions::new().set_limit(1);
        view_notes(self.storage_slot, self.note_interface, options)[0].unwrap()
    }
}
