use dep::std::option::Option;
use dep::protocol_types::{
    address::AztecAddress,
    constants::EMPTY_NULLIFIED_COMMITMENT,
};
    
use crate::context::{PrivateContext, Context};
use crate::note::{
    lifecycle::create_note,
    note_getter::{get_note, view_notes},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
};
use crate::oracle::notes::check_nullifier_exists;
use crate::state_vars::singleton::compute_singleton_initialization_nullifier;

// docs:start:struct
struct ImmutableSingleton<Note, N> {
    context: Option<&mut PrivateContext>,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    compute_initialization_nullifier: fn (Field, Option<AztecAddress>) -> Field,
}
// docs:end:struct

impl<Note, N> ImmutableSingleton<Note, N> {
    // docs:start:new
    pub fn new(
        context: Context,
        storage_slot: Field,
        note_interface: NoteInterface<Note, N>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        ImmutableSingleton {
            context: context.private,
            storage_slot,
            note_interface,
            compute_initialization_nullifier: compute_singleton_initialization_nullifier,
        }
    }
    // docs:end:new

    // docs:start:is_initialized
    unconstrained pub fn is_initialized(self, owner: Option<AztecAddress>) -> bool {
        let compute_initialization_nullifier = self.compute_initialization_nullifier;
        let nullifier = compute_initialization_nullifier(self.storage_slot, owner);
        check_nullifier_exists(nullifier)
    }
    // docs:end:is_initialized

    // docs:start:initialize
    pub fn initialize(
        self,
        note: &mut Note, 
        owner: Option<AztecAddress>,
        broadcast: bool,
    ) {
        let context = self.context.unwrap();

        // Nullify the storage slot.
        let compute_initialization_nullifier = self.compute_initialization_nullifier;
        let nullifier = compute_initialization_nullifier(self.storage_slot, owner);
        context.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT);

        create_note(
            context,
            self.storage_slot,
            note,
            self.note_interface,
            broadcast,
        );
    }
    // docs:end:initialize

    // docs:start:get_note
    pub fn get_note(self) -> Note {
        let context = self.context.unwrap();
        let storage_slot = self.storage_slot;
        get_note(context, storage_slot, self.note_interface)
    }
    // docs:end:get_note

    // docs:start:view_note
    unconstrained pub fn view_note(self) -> Note {
        let options = NoteViewerOptions::new().set_limit(1);
        view_notes(self.storage_slot, self.note_interface, options)[0].unwrap()
    }
    // docs:end:view_note
}
