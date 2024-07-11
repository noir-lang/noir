use dep::protocol_types::{address::AztecAddress, constants::GENERATOR_INDEX__INITIALIZATION_NULLIFIER, hash::pedersen_hash};

use crate::context::{PrivateContext, UnconstrainedContext};
use crate::note::{
    lifecycle::{create_note, destroy_note}, note_getter::{get_note, view_notes},
    note_interface::NoteInterface, note_viewer_options::NoteViewerOptions, note_emission::NoteEmission
};
use crate::oracle::notes::check_nullifier_exists;
use crate::state_vars::storage::Storage;

// docs:start:struct
struct PrivateMutable<Note, Context> {
    context: Context,
    storage_slot: Field
}
// docs:end:struct

mod test;

impl<T, Context> Storage<T> for PrivateMutable<T, Context> {}

impl<Note, Context> PrivateMutable<Note, Context> {
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
    // This is especially dangerous for initial assignment to elements of a `Map<AztecAddress, PrivateMutable>` type (for example), because the storage slot often also identifies an actor. e.g.
    // the initial assignment to `my_map.at(msg.sender)` will leak: `msg.sender`, the fact that an element of `my_map` was assigned-to for the first time, and the contract_address.
    // Note: subsequent nullification of this state variable, via the `replace` method will not be leaky, if the `compute_note_hash_and_nullifier()` method of the underlying note is designed to ensure privacy.
    // For example, if the `compute_note_hash_and_nullifier()` method injects the secret key of a note owner into the computed nullifier's preimage.
    pub fn compute_initialization_nullifier(self) -> Field {
        pedersen_hash(
            [self.storage_slot],
            GENERATOR_INDEX__INITIALIZATION_NULLIFIER
        )
    }
}

impl<Note, N, M> PrivateMutable<Note, &mut PrivateContext> where Note: NoteInterface<N, M> {
    // docs:start:initialize
    pub fn initialize(self, note: &mut Note) -> NoteEmission<Note> {
        // Nullify the storage slot.
        let nullifier = self.compute_initialization_nullifier();
        self.context.push_nullifier(nullifier, 0);

        create_note(self.context, self.storage_slot, note)
    }
    // docs:end:initialize

    // docs:start:replace
    pub fn replace(self, new_note: &mut Note) -> NoteEmission<Note> {
        let prev_note: Note = get_note(self.context, self.storage_slot);

        // Nullify previous note.
        destroy_note(self.context, prev_note);

        // Add replacement note.
        create_note(self.context, self.storage_slot, new_note)
    }
    // docs:end:replace

    pub fn initialize_or_replace(self, note: &mut Note) -> NoteEmission<Note> {
        let is_initialized = check_nullifier_exists(self.compute_initialization_nullifier());

        // check_nullifier_exists() is an unconstrained function - we can constrain a true value by providing an 
        // inclusion proof of the nullifier, but cannot constrain a false value since a non-inclusion proof would only 
        // be valid if done in public.
        // Ultimately, this is not an issue ginen that we'll either:
        //  - initialize the state variable, which would fail if it was already initialized due to the duplicate 
        //    nullifier, or
        //  - replace the current value, which would fail if it was not initialized since we wouldn't be able to produce
        //    an inclusion proof for the current note
        // This means that an honest oracle will assist the prover to produce a valid proof, while a malicious oracle
        // (i.e. one that returns an incorrect value for is_initialized) will simply fail to produce a proof.
        if (!is_initialized) {
            self.initialize(note)
        } else {
            self.replace(note)
        }
    }

    // docs:start:get_note
    pub fn get_note(self) -> NoteEmission<Note> {
        let mut note = get_note(self.context, self.storage_slot);

        // Nullify current note to make sure it's reading the latest note.
        destroy_note(self.context, note);

        // Add the same note again.
        // Because a nonce is added to every note in the kernel, its nullifier will be different.
        create_note(self.context, self.storage_slot, &mut note)
    }
    // docs:end:get_note
}

impl<Note, N, M> PrivateMutable<Note, UnconstrainedContext> where Note: NoteInterface<N, M> {
    unconstrained pub fn is_initialized(self) -> bool {
        let nullifier = self.compute_initialization_nullifier();
        check_nullifier_exists(nullifier)
    }

    // docs:start:view_note
    unconstrained pub fn view_note(self) -> Note {
        let mut options = NoteViewerOptions::new();
        view_notes(self.storage_slot, options.set_limit(1)).get(0)
    }
    // docs:end:view_note
}
