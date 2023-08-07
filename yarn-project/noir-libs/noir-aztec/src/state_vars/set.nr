use crate::abi::PublicContextInputs;
use crate::context::Context;
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

    fn insert(self, context: &mut Context, note: &mut Note) {
        create_note(context, self.storage_slot, note, self.note_interface);
    }

    fn insert_from_public(self, inputs: PublicContextInputs, note: &mut Note) {
        create_note_hash_from_public(inputs, self.storage_slot, note, self.note_interface);
    }

    fn assert_contains(self, context: &mut Context, note: &mut Note) {
        ensure_note_exists(context, self.storage_slot, self.note_interface, note);
    }

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): rename to
    // `assert_contains` and replace function above ^ once public kernel injects
    // nonces to note hashes.
    fn assert_contains_note_hash(self, context: &mut Context, note: &mut Note) {
        ensure_note_hash_exists(context, self.storage_slot, self.note_interface, note)
    }

    fn remove(self, context: &mut Context, note: Note) {
        destroy_note(context, self.storage_slot, note, self.note_interface);
    }

    fn get_notes<S, P>(
        self,
        context: &mut Context,
        options: NoteGetterOptions<Note, N, S, P>,
    ) -> [Option<Note>; S] {
        let storage_slot = self.storage_slot;
        let opt_notes = get_notes(context, storage_slot, self.note_interface, options);
        opt_notes
    }
}
