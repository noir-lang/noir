use dep::protocol_types::{constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, abis::read_request::ReadRequest};
use crate::context::{PrivateContext, PublicContext, UnconstrainedContext};
use crate::note::{
    constants::MAX_NOTES_PER_PAGE, lifecycle::{create_note, create_note_hash_from_public, destroy_note},
    note_getter::{get_notes, view_notes}, note_getter_options::NoteGetterOptions,
    note_header::NoteHeader, note_interface::NoteInterface, note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_for_read_request, note_emission::NoteEmission
};
use crate::state_vars::storage::Storage;

// docs:start:struct
struct PrivateSet<Note, Context> {
    context: Context,
    storage_slot: Field,
}
// docs:end:struct

impl<T, Context> Storage<T> for PrivateSet<T, Context> {}

impl<Note, Context> PrivateSet<Note, Context> {
    // docs:start:new
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PrivateSet { context, storage_slot }
    }
    // docs:end:new
}

impl<Note, let N: u32, let M: u32> PrivateSet<Note, &mut PublicContext> where Note: NoteInterface<N, M> {
    // docs:start:insert_from_public
    pub fn insert_from_public(self, note: &mut Note) {
        create_note_hash_from_public(self.context, self.storage_slot, note);
    }
    // docs:end:insert_from_public
}

impl<Note, let N: u32, let M: u32> PrivateSet<Note, &mut PrivateContext> where Note: NoteInterface<N, M> + Eq {
    // docs:start:insert
    pub fn insert(self, note: &mut Note) -> NoteEmission<Note> {
        create_note(self.context, self.storage_slot, note)
    }
    // docs:end:insert

    // docs:start:remove
    pub fn remove(self, note: Note) {
        let note_hash = compute_note_hash_for_read_request(note);
        let has_been_read = self.context.note_hash_read_requests.any(|r: ReadRequest| r.value == note_hash);
        assert(has_been_read, "Can only remove a note that has been read from the set.");

        destroy_note(self.context, note);
    }
    // docs:end:remove

    // docs:start:get_notes
    pub fn get_notes<FILTER_ARGS>(
        self,
        options: NoteGetterOptions<Note, N, M, FILTER_ARGS>
    ) -> BoundedVec<Note, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL> {
        get_notes(self.context, self.storage_slot, options)
    }
    // docs:end:get_notes
}

impl<Note, let N: u32, let M: u32> PrivateSet<Note, UnconstrainedContext> where Note: NoteInterface<N, M> {
    // docs:start:view_notes
    unconstrained pub fn view_notes(
        self,
        options: NoteViewerOptions<Note, N, M>
    ) -> BoundedVec<Note, MAX_NOTES_PER_PAGE> {
        view_notes(self.storage_slot, options)
    }
    // docs:end:view_notes
}
