use dep::std::option::Option;
use dep::protocol_types::{
    address::AztecAddress,
    constants::{
        GENERATOR_INDEX__INITIALIZATION_NULLIFIER,
    },
    hash::pedersen_hash,
};

use crate::context::{PrivateContext, Context};
use crate::note::{
    lifecycle::create_note,
    note_getter::{get_note, view_notes},
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
};
use crate::oracle::notes::check_nullifier_exists;

// docs:start:struct
struct ImmutableSingleton<Note, N> {
    context: Option<&mut PrivateContext>,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
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
        Self {
            context: context.private,
            storage_slot,
            note_interface,
        }
    }
    // docs:end:new

    // The following computation is leaky, in that it doesn't hide the storage slot that has been initialized, nor does it hide the contract address of this contract.
    // When this initialization nullifier is emitted, an observer could do a dictionary or rainbow attack to learn the preimage of this nullifier to deduce the storage slot and contract address.
    // For some applications, leaking the details that a particular state variable of a particular contract has been initialized will be unacceptable.
    // Under such circumstances, such application developers might wish to _not_ use this state variable type.
    // This is especially dangerous for initial assignment to elements of a `Map<AztecAddress, ImmutableSingleton>` type (for example), because the storage slot often also identifies an actor. 
    // e.g. the initial assignment to `my_map.at(msg.sender)` will leak: `msg.sender`, the fact that an element of `my_map` was assigned-to for the first time, and the contract_address.
    pub fn compute_initialization_nullifier(self) -> Field {
        pedersen_hash([self.storage_slot], GENERATOR_INDEX__INITIALIZATION_NULLIFIER)
    }

    // docs:start:is_initialized
    unconstrained pub fn is_initialized(self) -> bool {
        let nullifier = self.compute_initialization_nullifier();
        check_nullifier_exists(nullifier)
    }
    // docs:end:is_initialized

    // docs:start:initialize
    pub fn initialize(
        self,
        note: &mut Note, 
        broadcast: bool,
    ) {
        let context = self.context.unwrap();

        // Nullify the storage slot.
        let nullifier = self.compute_initialization_nullifier();
        context.push_new_nullifier(nullifier, 0);

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
