use dep::std::option::Option;
use crate::abi::PublicContextInputs;
use crate::constants_gen::{MAX_NOTES_PER_PAGE, MAX_READ_REQUESTS_PER_CALL};
use crate::context::{PrivateContext, PublicContext, Context};
use crate::note::{
    lifecycle::{create_note, create_note_hash_from_public, destroy_note},
    note_getter::{get_notes, view_notes},
    note_getter_options::NoteGetterOptions,
    note_header::NoteHeader,
    note_interface::NoteInterface,
    note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_for_read_or_nullify,
};

// docs:start:struct
struct Set<Note, N> {
    context: Context,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
}
// docs:end:struct

impl<Note, N> Set<Note, N> {
    // docs:start:new
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
    // docs:end:new

    // docs:start:insert
    fn insert(self, note: &mut Note) {
        create_note(
            self.context.private.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }
    // docs:end:insert

    // docs:start:insert_from_public
    fn insert_from_public(self, note: &mut Note) {
        create_note_hash_from_public(
            self.context.public.unwrap(),
            self.storage_slot,
            note,
            self.note_interface,
        );
    }
    // docs:end:insert_from_public
    
    // DEPRECATED
    fn assert_contains_and_remove(_self: Self, _note: &mut Note, _nonce: Field) {
        assert(false, "`assert_contains_and_remove` has been deprecated. Please call PXE.addNote() to add a note to the database. Then use Set.get_notes() and Set.remove() in your contract to verify and remove a note.");
    }

    // DEPRECATED
    fn assert_contains_and_remove_publicly_created(_self: Self, _note: &mut Note) {
        assert(false, "`assert_contains_and_remove_publicly_created` has been deprecated. Please call PXE.addNote() to add a note to the database. Then use Set.get_notes() and Set.remove() in your contract to verify and remove a note.");
    }

    // docs:start:remove
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
    // docs:end:remove

    // docs:start:get_notes
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
    // docs:end:get_notes

    // docs:start:view_notes
    unconstrained fn view_notes(
        self,
        options: NoteViewerOptions<Note, N>,
    ) -> [Option<Note>; MAX_NOTES_PER_PAGE] {
        view_notes(self.storage_slot, self.note_interface, options)
    }
    // docs:end:view_notes
}
