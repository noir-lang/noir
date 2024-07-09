use dep::protocol_types::{address::AztecAddress, constants::GENERATOR_INDEX__INITIALIZATION_NULLIFIER, hash::pedersen_hash};

use crate::context::{PrivateContext, UnconstrainedContext};
use crate::note::{
    lifecycle::create_note, note_getter::{get_note, view_notes}, note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions, note_emission::NoteEmission
};
use crate::oracle::notes::check_nullifier_exists;
use crate::state_vars::storage::Storage;

// docs:start:struct
struct PrivateImmutable<Note, Context> {
    context: Context,
    storage_slot: Field,
}
// docs:end:struct

impl<T, Context> Storage<T> for PrivateImmutable<T, Context> {}

impl<Note, Context> PrivateImmutable<Note, Context> {
    // docs:start:new
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }
    // docs:end:new

    // The following computation is leaky, in that it doesn't hide the storage slot that has been initialized, nor does it hide the contract address of this contract.
    // When this initialization nullifier is emitted, an observer could do a dictionary or rainbow attack to learn the preimage of this nullifier to deduce the storage slot and contract address.
    // For some applications, leaking the details that a particular state variable of a particular contract has been initialized will be unacceptable.
    // Under such circumstances, such application developers might wish to _not_ use this state variable type.
    // This is especially dangerous for initial assignment to elements of a `Map<AztecAddress, PrivateImmutable>` type (for example), because the storage slot often also identifies an actor. 
    // e.g. the initial assignment to `my_map.at(msg.sender)` will leak: `msg.sender`, the fact that an element of `my_map` was assigned-to for the first time, and the contract_address.
    pub fn compute_initialization_nullifier(self) -> Field {
        pedersen_hash(
            [self.storage_slot],
            GENERATOR_INDEX__INITIALIZATION_NULLIFIER
        )
    }
}

impl<Note> PrivateImmutable<Note, &mut PrivateContext> {
    // docs:start:initialize
    pub fn initialize<N, M>(
        self,
        note: &mut Note
    ) -> NoteEmission<Note> where Note: NoteInterface<N, M> {
        // Nullify the storage slot.
        let nullifier = self.compute_initialization_nullifier();
        self.context.push_nullifier(nullifier, 0);

        create_note(self.context, self.storage_slot, note)
    }
    // docs:end:initialize

    // docs:start:get_note
    pub fn get_note<N, M>(self) -> Note where Note: NoteInterface<N, M> {
        let storage_slot = self.storage_slot;
        get_note(self.context, storage_slot)
    }
    // docs:end:get_note
}

impl<Note> PrivateImmutable<Note, UnconstrainedContext> {
    // docs:start:is_initialized
    unconstrained pub fn is_initialized(self) -> bool {
        let nullifier = self.compute_initialization_nullifier();
        check_nullifier_exists(nullifier)
    }
    // docs:end:is_initialized

    // view_note does not actually use the context, but it calls oracles that are only available in private
    // docs:start:view_note
    unconstrained pub fn view_note<N, M>(self) -> Note  where Note: NoteInterface<N, M> {
        let mut options = NoteViewerOptions::new();
        view_notes(self.storage_slot, options.set_limit(1)).get(0)
    }
    // docs:end:view_note
}
