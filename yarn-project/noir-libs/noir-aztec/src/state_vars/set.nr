use crate::abi::PublicContextInputs;
use crate::context::{
    PrivateContext,
    PublicContext,
};
use crate::note::lifecycle::{create_note, create_note_hash_from_public, destroy_note};
use crate::note::{
    note_getter::{get_notes, ensure_note_exists, ensure_note_hash_exists},
    note_getter_options::NoteGetterOptions,
    note_interface::NoteInterface,
    utils::compute_inner_note_hash,
};
use crate::types::option::Option;

struct Set<Note, N> {
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Set<Note, N> {
    fn new(storage_slot: Field, note_interface: NoteInterface<Note, N>) -> Self {
        Set { storage_slot, note_interface }
    }

    fn insert(self, context: &mut PrivateContext, note: &mut Note) {
        create_note(context, self.storage_slot, note, self.note_interface);
    }

    fn insert_from_public(self, context: PublicContext, note: &mut Note) {
        create_note_hash_from_public(context, self.storage_slot, note, self.note_interface);
    }

    // TODO(#1386)
    // Should be replaced by `assert_contains_and_remove`.
    fn assert_contains_note_and_remove(self, context: &mut PrivateContext, note: Note) {
        let mut note_with_header = note;
        ensure_note_exists(context, self.storage_slot, self.note_interface, &mut note_with_header);
        self.remove(context, note_with_header);
    }

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386):
    // replace function above ^ once public kernel injects
    // nonces to note hashes.
    fn assert_contains_and_remove(self, context: &mut PrivateContext, note: Note) {
        let mut note_with_header = note;
        // Modifies note with the header which is necessary for the next step (remove).
        ensure_note_hash_exists(context, self.storage_slot, self.note_interface, &mut note_with_header);

        let get_header = self.note_interface.get_header;
        let set_header = self.note_interface.set_header;
        let mut header = get_header(note);
        // Set the nonce to nonzero so that the nullifier is treated as persistable
        // (non-transient) and so the private kernel does not attempt to match it to
        // a pending noteHash/commitment and squash them.
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): remove
        // this hack once public kernel injects nonces.
        header.nonce = 1;
        set_header(&mut note_with_header, header);

        self.remove(context, note_with_header);
    }

    fn remove(self, context: &mut PrivateContext, note: Note) {
        destroy_note(context, self.storage_slot, note, self.note_interface);
    }

    fn get_notes<S, P>(
        self,
        context: &mut PrivateContext,
        options: NoteGetterOptions<Note, N, S, P>,
    ) -> [Option<Note>; S] {
        let storage_slot = self.storage_slot;
        let opt_notes = get_notes(context, storage_slot, self.note_interface, options);
        opt_notes
    }
}
