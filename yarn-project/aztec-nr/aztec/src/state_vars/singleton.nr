use dep::std::option::Option;
use crate::constants_gen::{EMPTY_NULLIFIED_COMMITMENT, GENERATOR_INDEX__INITIALISATION_NULLIFIER};
use crate::context::{PrivateContext, PublicContext, Context};
use crate::note::{
    lifecycle::{create_note, destroy_note},
    note_getter::{get_note, view_notes},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
};
use crate::oracle::{
    get_secret_key::get_secret_key,
    notes::check_nullifier_exists,
};
use dep::std::hash::pedersen_with_separator;

fn compute_singleton_initialization_nullifier(storage_slot: Field, owner: Option<Field>) -> Field {
    if owner.is_some() {
        let secret = get_secret_key(owner.unwrap_unchecked());
        pedersen_with_separator(
            [storage_slot, secret.low, secret.high],
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
        )[0]
    } else {
        pedersen_with_separator(
            [storage_slot],
            GENERATOR_INDEX__INITIALISATION_NULLIFIER,
        )[0]
    }
}

// docs:start:struct
struct Singleton<Note, N> {
    context: Option<&mut PrivateContext>,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    compute_initialization_nullifier: fn (Field, Option<Field>) -> Field,
}
// docs:end:struct

impl<Note, N> Singleton<Note, N> {
    // docs:start:new
    fn new(
        context: Context,
        storage_slot: Field,
        note_interface: NoteInterface<Note, N>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Singleton {
            context: context.private,
            storage_slot,
            note_interface,
            compute_initialization_nullifier: compute_singleton_initialization_nullifier,
        }
    }
    // docs:end:new

    // docs:start:is_initialized
    unconstrained fn is_initialized(self, owner: Option<Field>) -> bool {
        let compute_initialization_nullifier = self.compute_initialization_nullifier;
        let nullifier = compute_initialization_nullifier(self.storage_slot, owner);
        check_nullifier_exists(nullifier)
    }
    // docs:end:is_initialized

    // docs:start:initialize
    fn initialize(self, note: &mut Note, owner: Option<Field>) {
        let context = self.context.unwrap();

        // Nullify the storage slot.
        let compute_initialization_nullifier = self.compute_initialization_nullifier;
        let nullifier = compute_initialization_nullifier(self.storage_slot, owner);
        context.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT);

        create_note(context, self.storage_slot, note, self.note_interface);
    }
    // docs:end:initialize

    // docs:start:replace
    fn replace(self, new_note: &mut Note) {
        let context = self.context.unwrap();
        let prev_note = get_note(context, self.storage_slot, self.note_interface);

        // Nullify previous note.
        destroy_note(context, prev_note, self.note_interface);

        // Add replacement note.
        create_note(context, self.storage_slot, new_note, self.note_interface);
    }
    // docs:end:replace

    // docs:start:get_note
    fn get_note(self) -> Note {
        let context = self.context.unwrap();
        let mut note = get_note(context, self.storage_slot, self.note_interface);

        // Nullify current note to make sure it's reading the latest note.
        destroy_note(context, note, self.note_interface);

        // Add the same note again.
        // Because a nonce is added to every note in the kernel, its nullifier will be different.
        create_note(context, self.storage_slot, &mut note, self.note_interface);

        note
    }
    // docs:end:get_note

    unconstrained fn view_note(self) -> Note {
        let options = NoteViewerOptions::new().set_limit(1);
        view_notes(self.storage_slot, self.note_interface, options)[0].unwrap()
    }
}
