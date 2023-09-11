use dep::std::option::Option;
use crate::abi::PublicContextInputs;
use crate::constants_gen::{MAX_NOTES_PER_PAGE, MAX_READ_REQUESTS_PER_CALL};
use crate::context::{PrivateContext, PublicContext, Context};
use crate::note::{
    lifecycle::{create_note, create_note_hash_from_public, destroy_note},
    note_getter::{ensure_note_exists, ensure_note_hash_exists, get_notes, view_notes},
    note_getter_options::NoteGetterOptions,
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_for_read_or_nullify,
};

struct Set<Note, N> {
    context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}

impl<Note, N> Set<Note, N> {
    fn new(
        context: Context,
        storage_slot: Field,
        note_interface: NoteInterface<Note, N>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Set {
            context,
            storage_slot,
            note_interface,
        }
    }

    fn insert(self, note: &mut Note) {
        create_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }

    fn insert_from_public(self, note: &mut Note) {
        create_note_hash_from_public(
            self.context.public.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }

    // TODO(#1386)
    // Should be replaced by `assert_contains_and_remove`.
    fn assert_contains_note_and_remove(self, note: Note) {
        let mut note_with_header = note;
        ensure_note_exists(
            self.context.private.unwrap(),
            self.storage_slot,
            self.note_interface,
            &mut note_with_header,
        );
        destroy_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note_with_header,
            self.note_interface,
        );
    }

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386):
    // replace function above ^ once public kernel injects
    // nonces to note hashes.
    fn assert_contains_and_remove(self, note: Note) {
        let mut note_with_header = note;
        // Modifies note with the header which is necessary for the next step (remove).
        ensure_note_hash_exists(
            self.context.private.unwrap(),
            self.storage_slot,
            self.note_interface,
            &mut note_with_header,
        );

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
        destroy_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note_with_header,
            self.note_interface,
        );
    }

    fn remove(self, note: Note) {
        let note_hash = compute_note_hash_for_read_or_nullify(self.note_interface, note);
        let read_requests = self.context.private.unwrap_unchecked().read_requests;
        let has_been_read = read_requests.any(|r| r == note_hash);
        assert(has_been_read, "Can only remove a note that has been read from the set.");

        destroy_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }

    fn get_notes<FILTER_ARGS>(
        self,
        options: NoteGetterOptions<Note, N, FILTER_ARGS>,
    ) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL] {
        let storage_slot = self.storage_slot;
        let opt_notes = get_notes(
            self.context.private.unwrap(),
            storage_slot,
            self.note_interface,
            options,
        );
        opt_notes
    }

    unconstrained fn view_notes(
        self,
        options: NoteViewerOptions<Note, N>,
    ) -> [Option<Note>; MAX_NOTES_PER_PAGE] {
        view_notes(self.storage_slot, self.note_interface, options)
    }
}
