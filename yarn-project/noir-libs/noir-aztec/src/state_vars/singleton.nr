use crate::constants_gen::{EMPTY_NULLIFIED_COMMITMENT, GENERATOR_INDEX__INITIALISATION_NULLIFIER};
use crate::context::{PrivateContext, PublicContext};
use crate::note::{
    lifecycle::{create_note, destroy_note},
    note_getter::{get_note, view_notes},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
};
use crate::oracle;
use dep::std::hash::pedersen_with_separator;
use dep::std::option::Option;

struct Singleton<Note, N> {
    context: Option<&mut PrivateContext>,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Singleton<Note, N> {
    fn new(
        context: Option<&mut PrivateContext>,
        _: Option<&mut PublicContext>,
        storage_slot: Field,
        note_interface: NoteInterface<Note, N>,
    ) -> Self {
        assert(storage_slot != 0); // Storage slot 0 not allowed. Storage slots must start from 1.
        Singleton {
            context,
            storage_slot,
            note_interface,
        }
    }

    unconstrained fn is_initialised(self) -> bool {
        let nullifier = self.compute_initialisation_nullifier();
        oracle::notes::is_nullifier_emitted(nullifier)
    }

    fn initialise(self, note: &mut Note) {
        let context = self.context.unwrap();
        // Nullify the storage slot.
        let nullifier = self.compute_initialisation_nullifier();
        context.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT);

        create_note(context, self.storage_slot, note, self.note_interface);
    }

    fn compute_initialisation_nullifier(self) -> Field {
        pedersen_with_separator(
            [self.storage_slot],
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
        )[0]
    }

    fn replace(self, new_note: &mut Note) {
        let context = self.context.unwrap();
        let prev_note = get_note(context, self.storage_slot, self.note_interface);

        // Nullify previous note.
        destroy_note(context, self.storage_slot, prev_note, self.note_interface);

        // Add replacement note.
        create_note(context, self.storage_slot, new_note, self.note_interface);
    }

    fn get_note(self) -> Note {
        let context = self.context.unwrap();
        let mut note = get_note(context, self.storage_slot, self.note_interface);

        // Nullify current note to make sure it's reading the latest note.
        destroy_note(context, self.storage_slot, note, self.note_interface);

        // Add the same note again.
        // Because a nonce is added to every note in the kernel, its nullifier will be different.
        create_note(context, self.storage_slot, &mut note, self.note_interface);

        note
    }

    unconstrained fn view_note(self) -> Note {
        let options = NoteViewerOptions::new().set_limit(1);
        view_notes(self.storage_slot, self.note_interface, options)[0].unwrap()
    }
}
