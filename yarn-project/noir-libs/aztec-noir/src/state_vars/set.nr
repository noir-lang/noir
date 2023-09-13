use dep::std::option::Option;
use crate::abi::PublicContextInputs;
use crate::constants_gen::{MAX_NOTES_PER_PAGE, MAX_READ_REQUESTS_PER_CALL};
use crate::context::{PrivateContext, PublicContext, Context};
use crate::note::{
    lifecycle::{create_note, create_note_hash_from_public, destroy_note},
    note_getter::{ensure_note_hash_exists, get_notes, view_notes},
    note_getter_options::NoteGetterOptions,
    note_header::NoteHeader,
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

    fn assert_contains_and_remove(self, note: &mut Note, nonce: Field) {
        // Initialize header of note. Must be done before computing note hashes as it initializes the:
        // - storage slot (used in inner note hash)
        // - the contract address (used in siloed note hash)
        // - and the nonce (used in the unique siloed note hash)
        let context = self.context.private.unwrap();
        let set_header = self.note_interface.set_header;
        let note_header = NoteHeader{
            contract_address: context.this_address(),
            storage_slot: self.storage_slot,
            nonce
        };
        set_header(note, note_header);

        ensure_note_hash_exists(
            context,
            self.note_interface,
            *note,
            false,
        );

        destroy_note(
            context,
            *note,
            self.note_interface,
        );
    }

    // NOTE: this function should ONLY be used for PUBLICLY-CREATED note hashes!
    // WARNING: function will be deprecated/removed eventually once public kernel applies nonces.
    fn assert_contains_and_remove_publicly_created(self, note: &mut Note) {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
        // Should be real nonce (once public kernel applies nonces).
        let nonce = 0;
        let context = self.context.private.unwrap();
        let set_header = self.note_interface.set_header;
        let mut note_header = NoteHeader{
            contract_address: context.this_address(),
            storage_slot: self.storage_slot,
            nonce
        };
        set_header(note, note_header);

        ensure_note_hash_exists(
            context,
            self.note_interface,
            *note,
            true,
        );

        // Set the nonce to nonzero so that the nullifier is treated as persistable
        // (non-transient) and so the private kernel does not attempt to match it to
        // a pending noteHash/commitment and squash them.
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): remove
        // this hack once public kernel injects nonces.
        note_header.nonce = 1;
        set_header(note, note_header);

        destroy_note(
            context,
            *note,
            self.note_interface,
        );
    }

    fn remove(self, note: Note) {
        let context = self.context.private.unwrap();
        let note_hash = compute_note_hash_for_read_or_nullify(self.note_interface, note);
        let has_been_read = context.read_requests.any(|r| r == note_hash);
        assert(has_been_read, "Can only remove a note that has been read from the set.");

        destroy_note(
            context,
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
